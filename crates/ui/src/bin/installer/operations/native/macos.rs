use super::super::journal::sibling;
use super::super::{InstallRequest, platform};
use k580_ui::install_mode::InstallMode;
use k580_ui::{macos_bundle, macos_launch_services};
use std::path::{Path, PathBuf};

pub(super) struct MacIntegration {
    pub(super) touched: bool,
    bundle: Option<PathBuf>,
    original_bundle: bool,
    handlers: Option<macos_launch_services::DefaultHandlers>,
    recovery: Option<PathBuf>,
}

impl MacIntegration {
    pub(super) fn capture(
        request: &InstallRequest,
        gui: &Path,
        system_files: &mut Vec<PathBuf>,
        association_files: &mut Vec<PathBuf>,
    ) -> Result<Self, String> {
        let mut bundle_files = Vec::new();
        let bundle = if request.mode == InstallMode::System || request.associate_program_files {
            let bundle = if request.mode == InstallMode::Portable {
                request.install_dir.join(macos_bundle::APP_BUNDLE_NAME)
            } else {
                macos_bundle::applications_dir()?.join(macos_bundle::APP_BUNDLE_NAME)
            };
            macos_bundle::validate_launcher_bundle(&bundle, gui)?;
            for name in [
                "Contents/Info.plist",
                "Contents/Resources/KR580.icns",
                "Contents/Resources/KR580Document.icns",
                "Contents/MacOS/kr580",
            ] {
                bundle_files.push(bundle.join(name));
            }
            let current = bundle.join("Contents/MacOS/kr580");
            for name in ["KR580", "kr580-launcher"] {
                let legacy = bundle.join("Contents/MacOS").join(name);
                if legacy.is_file()
                    && (std::fs::canonicalize(&legacy).ok() != std::fs::canonicalize(&current).ok())
                {
                    bundle_files.push(legacy);
                }
            }
            Some(bundle)
        } else {
            None
        };
        if request.mode == InstallMode::System {
            system_files.extend(bundle_files.iter().cloned());
        }
        if request.associate_program_files {
            association_files.extend(bundle_files);
        }
        if request.mode == InstallMode::System && request.create_desktop_shortcut {
            system_files.push(platform::unix::desktop_dir().join("KR580.command"));
        }
        let original_bundle = bundle.as_ref().is_some_and(|path| path.is_dir());
        let handlers = request
            .associate_program_files
            .then(macos_launch_services::DefaultHandlers::capture);
        Ok(Self {
            touched: false,
            bundle,
            original_bundle,
            handlers,
            recovery: None,
        })
    }

    pub(super) fn register(&mut self, root: &Path, gui: &Path) -> Result<(), String> {
        let handlers = self
            .handlers
            .as_mut()
            .ok_or_else(|| "association snapshot is missing".to_owned())?;
        let path = sibling(&root.join("handlers.json"), "handler-recovery")?;
        self.recovery = Some(path.clone());
        let bytes = serde_json::to_vec_pretty(&handlers.originals().collect::<Vec<_>>())
            .map_err(|error| format!("serialize handler backup: {error}"))?;
        std::fs::write(&path, bytes)
            .map_err(|error| format!("write handler recovery {}: {error}", path.display()))?;
        let bundle = self
            .bundle
            .as_ref()
            .ok_or_else(|| "association bundle is missing".to_owned())?;
        self.touched = true;
        macos_bundle::write_launcher_bundle(bundle, gui)?;
        macos_launch_services::register_bundle(bundle)?;
        handlers.apply()
    }

    pub(super) fn rollback(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        if self.touched
            && self.original_bundle
            && let Some(bundle) = &self.bundle
            && let Err(error) = macos_launch_services::register_bundle(bundle)
        {
            errors.push(error);
        }
        if let Some(handlers) = &mut self.handlers
            && let Err(error) = handlers.rollback()
        {
            errors.push(error);
        }
        if errors.is_empty() {
            self.commit();
            Ok(())
        } else {
            Err(format!(
                "{}; handler recovery: {}",
                errors.join("; "),
                self.recovery
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "not created".into())
            ))
        }
    }

    pub(super) fn commit(&mut self) {
        if let Some(path) = self.recovery.take()
            && let Err(error) = std::fs::remove_file(&path)
        {
            tracing::warn!(path = %path.display(), %error, "handler backup retained");
        }
    }
}
