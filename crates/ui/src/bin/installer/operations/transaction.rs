use super::journal::{FileJournal, absolute, sibling};
use super::source::{SourceBundle, copy_executable};
use super::{InstallReport, InstallRequest, binary_name, read_manifest};
use k580_ui::install_mode::{InstallManifest, InstallMode, MANIFEST_FILENAME};
use std::path::{Path, PathBuf};

pub(super) trait Integration {
    fn files(&self, step: InstallStep) -> Vec<PathBuf>;
    fn add_path(&mut self, request: &InstallRequest, bin: &Path) -> Result<bool, String>;
    fn install_system(
        &mut self,
        request: &InstallRequest,
        gui: &Path,
        uninstaller: &Path,
    ) -> Result<bool, String>;
    fn associate(&mut self, request: &InstallRequest, gui: &Path) -> Result<(), String>;
    fn retire_legacy_association(
        &mut self,
        legacy: &Path,
        manifest: &InstallManifest,
    ) -> Result<(), String>;
    fn observe(&mut self) -> Result<(), String>;
    fn rollback(&mut self) -> Result<(), String>;
    fn commit(&mut self);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstallStep {
    Gui,
    Launcher,
    Uninstaller,
    Legacy,
    Path,
    System,
    Association,
    Manifest,
}

pub(super) fn run(
    mut request: InstallRequest,
    source: SourceBundle,
    integration: &mut impl Integration,
    mut checkpoint: impl FnMut(InstallStep) -> Result<(), String>,
) -> Result<InstallReport, String> {
    request.install_dir = absolute(&request.install_dir)?;
    let app = request.install_dir.join("app");
    let bin = request.install_dir.join("bin");
    let gui = app.join(binary_name("kr580"));
    let launcher = bin.join(binary_name("kr"));
    let uninstaller = app.join(binary_name("uninstaller"));
    let legacy = app.join(binary_name("k580"));
    let manifest_path = request.install_dir.join(MANIFEST_FILENAME);
    let previous = preflight(
        &request,
        &[
            gui.clone(),
            launcher.clone(),
            uninstaller.clone(),
            legacy.clone(),
            manifest_path.clone(),
        ],
    )?;
    let mut files = FileJournal::default();
    let targets = [
        gui.clone(),
        launcher.clone(),
        uninstaller.clone(),
        manifest_path.clone(),
    ];
    files.prepare(&targets)?;
    if previous.is_some() && legacy.is_file() {
        files.prepare(std::slice::from_ref(&legacy))?;
    }
    let external = [
        InstallStep::Path,
        InstallStep::System,
        InstallStep::Association,
    ]
    .into_iter()
    .flat_map(|step| integration.files(step))
    .collect::<Vec<_>>();
    files.prepare(&external)?;
    files.create_directory(&app)?;
    files.create_directory(&bin)?;
    if request.mode == InstallMode::Portable {
        files.create_directory(&request.install_dir.join("data"))?;
    }
    let lease = Lease::acquire(&request.install_dir)?;
    let mut staged = Staged::default();
    let result = (|| {
        for (source, target, step) in [
            (&source.kr580, &gui, InstallStep::Gui),
            (&source.kr, &launcher, InstallStep::Launcher),
            (&source.uninstaller, &uninstaller, InstallStep::Uninstaller),
        ] {
            let path = sibling(target, "stage")?;
            staged.0.push(path.clone());
            copy_executable(source, &path)?;
            staged.1.push((path, target.clone(), step));
        }
        for (path, target, step) in &staged.1 {
            files.change(std::slice::from_ref(target), || {
                std::fs::rename(path, target)
                    .map_err(|error| format!("install {}: {error}", target.display()))
            })?;
            checkpoint(*step)?;
        }
        if let Some(previous) = &previous
            && legacy.is_file()
        {
            if !request.associate_program_files && previous.file_association {
                let operation = files.change(&integration.files(InstallStep::Association), || {
                    integration.retire_legacy_association(&legacy, previous)
                });
                combine_observation(operation, integration.observe())?;
            }
            files.change(std::slice::from_ref(&legacy), || {
                std::fs::remove_file(&legacy)
                    .map_err(|error| format!("remove legacy binary: {error}"))
            })?;
        }
        checkpoint(InstallStep::Legacy)?;
        let path_changed = if request.add_to_path {
            let operation = files.change(&integration.files(InstallStep::Path), || {
                integration.add_path(&request, &bin)
            });
            combine_observation(operation, integration.observe())?
        } else {
            false
        };
        checkpoint(InstallStep::Path)?;
        let desktop_shortcut_created = if request.mode == InstallMode::System {
            let operation = files.change(&integration.files(InstallStep::System), || {
                integration.install_system(&request, &gui, &uninstaller)
            });
            combine_observation(operation, integration.observe())?
        } else {
            false
        };
        checkpoint(InstallStep::System)?;
        if request.associate_program_files {
            let operation = files.change(&integration.files(InstallStep::Association), || {
                integration.associate(&request, &gui)
            });
            combine_observation(operation, integration.observe())?;
        }
        checkpoint(InstallStep::Association)?;
        let manifest = InstallManifest::new(request.mode, request.scope)
            .with_file_association(request.associate_program_files);
        files.change(std::slice::from_ref(&manifest_path), || {
            k580_ui::install_mode::write_manifest(&request.install_dir, &manifest)
        })?;
        checkpoint(InstallStep::Manifest)?;
        Ok(InstallReport {
            mode: request.mode,
            install_dir: request.install_dir.clone(),
            kr580_path: gui,
            path_changed,
            system_integrated: request.mode == InstallMode::System,
            desktop_shortcut_created,
            file_association_created: request.associate_program_files,
        })
    })();
    drop(staged);
    drop(lease);
    match result {
        Ok(report) => {
            files.commit();
            integration.commit();
            Ok(report)
        }
        Err(error) => {
            let file_rollback = files.rollback();
            let external_rollback = integration.rollback();
            let rollback_errors = [file_rollback, external_rollback]
                .into_iter()
                .filter_map(Result::err)
                .collect::<Vec<_>>();
            if rollback_errors.is_empty() {
                files.clean_directories();
                Err(error)
            } else {
                Err(format!(
                    "{error}; rollback failed: {}",
                    rollback_errors.join("; ")
                ))
            }
        }
    }
}

fn preflight(
    request: &InstallRequest,
    targets: &[PathBuf],
) -> Result<Option<InstallManifest>, String> {
    for path in std::iter::once(&request.install_dir)
        .chain(targets.iter())
        .chain(
            [
                request.install_dir.join("app"),
                request.install_dir.join("bin"),
                request.install_dir.join("data"),
            ]
            .iter(),
        )
    {
        if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_symlink()) {
            return Err(format!(
                "installation path is a symbolic link: {}",
                path.display()
            ));
        }
    }
    let previous = match std::fs::symlink_metadata(request.install_dir.join(MANIFEST_FILENAME)) {
        Ok(_) => Some(read_manifest(&request.install_dir)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(format!("inspect installation manifest: {error}")),
    };
    if previous
        .as_ref()
        .is_some_and(|manifest| manifest.manifest_version != 1)
    {
        return Err("unsupported installation manifest version".into());
    }
    if previous.is_none() && targets.iter().take(3).any(|path| path.exists()) {
        return Err("unmanaged files occupy the installation binary paths".into());
    }
    Ok(previous)
}

fn combine_observation<T>(
    operation: Result<T, String>,
    observation: Result<(), String>,
) -> Result<T, String> {
    match (operation, observation) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(observation)) => Err(format!(
            "{error}; record integration changes: {observation}"
        )),
    }
}

struct Lease {
    file: Option<std::fs::File>,
    path: PathBuf,
}
impl Lease {
    fn acquire(root: &Path) -> Result<Self, String> {
        let path = root.join(".kr580-install.lock");
        let file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|error| format!("installation lock {}: {error}", path.display()))?;
        Ok(Self {
            file: Some(file),
            path,
        })
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.file.take();
        let _ = std::fs::remove_file(&self.path);
    }
}

#[derive(Default)]
struct Staged(Vec<PathBuf>, Vec<(PathBuf, PathBuf, InstallStep)>);
impl Drop for Staged {
    fn drop(&mut self) {
        for path in &self.0 {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests;
