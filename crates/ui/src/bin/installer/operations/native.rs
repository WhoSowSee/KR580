#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(any(windows, target_os = "linux"))]
use super::read_manifest;
use super::transaction::{InstallStep, Integration};
use super::{InstallRequest, platform};
use k580_ui::install_mode::InstallManifest;
#[cfg(any(windows, target_os = "linux"))]
use k580_ui::install_mode::InstallMode;
use std::path::{Path, PathBuf};

pub(super) struct NativeIntegration {
    path_files: Vec<PathBuf>,
    system_files: Vec<PathBuf>,
    association_files: Vec<PathBuf>,
    #[cfg(target_os = "linux")]
    services_changed: bool,
    #[cfg(windows)]
    registry: windows::RegistryJournal,
    #[cfg(target_os = "macos")]
    macos: macos::MacIntegration,
}

impl NativeIntegration {
    pub(super) fn capture(request: &InstallRequest) -> Result<Self, String> {
        #[cfg(any(windows, target_os = "macos"))]
        let gui = request
            .install_dir
            .join("app")
            .join(super::binary_name("kr580"));
        #[cfg(any(windows, target_os = "linux"))]
        let legacy_scope = if request
            .install_dir
            .join(k580_ui::install_mode::MANIFEST_FILENAME)
            .is_file()
        {
            let previous = read_manifest(&request.install_dir)?;
            (previous.file_association
                && !request.associate_program_files
                && request
                    .install_dir
                    .join("app")
                    .join(super::binary_name("k580"))
                    .is_file())
            .then_some(previous.scope)
        } else {
            None
        };
        let mut path_files = Vec::new();
        let mut system_files = Vec::new();
        let mut association_files = Vec::new();
        #[cfg(windows)]
        let registry = windows::RegistryJournal::capture(request, &gui, legacy_scope)?;
        #[cfg(windows)]
        if request.mode == InstallMode::System {
            system_files.extend(platform::windows::rollback_files(
                request.scope,
                request.create_desktop_shortcut,
            ));
        }
        #[cfg(unix)]
        if request.add_to_path {
            path_files.push(platform::unix::profile_path());
        }
        #[cfg(target_os = "linux")]
        {
            if request.mode == InstallMode::System {
                system_files
                    .push(k580_ui::desktop_entry::data_home()?.join("applications/kr580.desktop"));
                if request.create_desktop_shortcut {
                    system_files.push(platform::unix::desktop_dir().join("KR580.desktop"));
                }
            }
            if request.associate_program_files || legacy_scope.is_some() {
                association_files.extend(k580_ui::file_assoc::registration_paths()?);
            }
        }
        #[cfg(target_os = "macos")]
        let macos = macos::MacIntegration::capture(
            request,
            &gui,
            &mut system_files,
            &mut association_files,
        )?;
        for files in [&mut path_files, &mut system_files, &mut association_files] {
            files.sort();
            files.dedup();
        }
        Ok(Self {
            path_files,
            system_files,
            association_files,
            #[cfg(target_os = "linux")]
            services_changed: false,
            #[cfg(windows)]
            registry,
            #[cfg(target_os = "macos")]
            macos,
        })
    }

    fn before_change(&mut self, request: &InstallRequest, step: InstallStep) -> Result<(), String> {
        #[cfg(windows)]
        self.registry.before_change(&request.install_dir, step)?;
        #[cfg(not(windows))]
        let _ = (request, step);
        Ok(())
    }
}

impl Integration for NativeIntegration {
    fn files(&self, step: InstallStep) -> Vec<PathBuf> {
        match step {
            InstallStep::Path => self.path_files.clone(),
            InstallStep::System => self.system_files.clone(),
            InstallStep::Association => self.association_files.clone(),
            _ => Vec::new(),
        }
    }

    fn add_path(&mut self, request: &InstallRequest, bin: &Path) -> Result<bool, String> {
        self.before_change(request, InstallStep::Path)?;
        platform::add_to_path(bin, request.scope)
    }

    fn install_system(
        &mut self,
        request: &InstallRequest,
        gui: &Path,
        uninstaller: &Path,
    ) -> Result<bool, String> {
        self.before_change(request, InstallStep::System)?;
        #[cfg(target_os = "macos")]
        {
            self.macos.touched = true;
        }
        #[cfg(target_os = "linux")]
        {
            self.services_changed = true;
        }
        #[cfg(not(windows))]
        let _ = uninstaller;
        platform::install_system_integration(&platform::SystemIntegrationRequest {
            #[cfg(windows)]
            scope: request.scope,
            #[cfg(windows)]
            install_dir: &request.install_dir,
            kr580_path: gui,
            #[cfg(windows)]
            uninstaller_path: uninstaller,
            create_desktop_shortcut: request.create_desktop_shortcut,
        })
        .map(|report| report.desktop_shortcut_created)
    }

    fn associate(&mut self, request: &InstallRequest, gui: &Path) -> Result<(), String> {
        self.before_change(request, InstallStep::Association)?;
        #[cfg(target_os = "linux")]
        {
            self.services_changed = true;
        }
        #[cfg(target_os = "macos")]
        {
            self.macos.register(&request.install_dir, gui)
        }
        #[cfg(not(target_os = "macos"))]
        {
            k580_ui::file_assoc::register_for_executable(gui, request.scope)
        }
    }

    fn retire_legacy_association(
        &mut self,
        legacy: &Path,
        manifest: &InstallManifest,
    ) -> Result<(), String> {
        #[cfg(windows)]
        self.registry.before_change(
            legacy
                .parent()
                .and_then(Path::parent)
                .ok_or_else(|| "legacy binary has no install root".to_owned())?,
            InstallStep::Association,
        )?;
        #[cfg(not(target_os = "macos"))]
        {
            #[cfg(target_os = "linux")]
            {
                self.services_changed = true;
            }
            k580_ui::file_assoc::unregister_for_executable(legacy, manifest.scope)
        }
        #[cfg(target_os = "macos")]
        {
            let _ = (legacy, manifest);
            Ok(())
        }
    }

    fn observe(&mut self) -> Result<(), String> {
        #[cfg(windows)]
        {
            self.registry.observe()
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    fn rollback(&mut self) -> Result<(), String> {
        #[cfg(windows)]
        {
            let changed = self.registry.changed();
            let result = self.registry.rollback();
            if changed {
                platform::windows::notify_restored_environment();
                k580_ui::file_assoc::refresh_shell();
            }
            result
        }
        #[cfg(target_os = "linux")]
        {
            if self.services_changed {
                let data = k580_ui::desktop_entry::data_home()?;
                if data.join("mime").is_dir() {
                    k580_ui::desktop_entry::update_mime_database(&data.join("mime"))?;
                }
                if data.join("applications").is_dir() {
                    k580_ui::desktop_entry::update_desktop_database(&data.join("applications"))?;
                }
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            self.macos.rollback()
        }
    }

    fn commit(&mut self) {
        #[cfg(windows)]
        self.registry.commit();
        #[cfg(target_os = "macos")]
        self.macos.commit();
    }
}
