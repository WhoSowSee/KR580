use super::super::source::SourceBinary;
use super::*;
use k580_ui::install_mode::InstallScope;
use std::fs;

struct IsolatedIntegration {
    paths: Vec<PathBuf>,
}
impl Integration for IsolatedIntegration {
    fn files(&self, step: InstallStep) -> Vec<PathBuf> {
        let index = match step {
            InstallStep::Path => 0,
            InstallStep::System => 1,
            InstallStep::Association => 2,
            _ => return Vec::new(),
        };
        vec![self.paths[index].clone()]
    }
    fn add_path(&mut self, _: &InstallRequest, _: &Path) -> Result<bool, String> {
        fs::write(&self.paths[0], b"new PATH").map_err(|e| e.to_string())?;
        Ok(true)
    }
    fn install_system(&mut self, _: &InstallRequest, _: &Path, _: &Path) -> Result<bool, String> {
        fs::write(&self.paths[1], b"new shortcut").map_err(|e| e.to_string())?;
        Ok(true)
    }
    fn associate(&mut self, _: &InstallRequest, _: &Path) -> Result<(), String> {
        fs::write(&self.paths[2], b"new association").map_err(|e| e.to_string())
    }
    fn retire_legacy_association(&mut self, _: &Path, _: &InstallManifest) -> Result<(), String> {
        Ok(())
    }
    fn observe(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn rollback(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn commit(&mut self) {}
}

#[test]
fn every_late_failure_restores_previous_installation_and_external_files() {
    for fail_at in [
        InstallStep::Gui,
        InstallStep::Launcher,
        InstallStep::Uninstaller,
        InstallStep::Legacy,
        InstallStep::Path,
        InstallStep::System,
        InstallStep::Association,
        InstallStep::Manifest,
    ] {
        let root = temporary_root();
        let install = root.join("install");
        fs::create_dir_all(install.join("app")).unwrap();
        fs::create_dir_all(install.join("bin")).unwrap();
        fs::create_dir_all(install.join("data")).unwrap();
        let owned = [
            install.join("app").join(binary_name("kr580")),
            install.join("bin").join(binary_name("kr")),
            install.join("app").join(binary_name("uninstaller")),
            install.join("app").join(binary_name("k580")),
        ];
        for path in &owned {
            fs::write(path, b"old binary").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o750)).unwrap();
            }
        }
        let user_data = install.join("data/settings.json");
        fs::write(&user_data, b"user data").unwrap();
        let manifest = InstallManifest::new(InstallMode::Portable, InstallScope::User);
        k580_ui::install_mode::write_manifest(&install, &manifest).unwrap();
        let original_manifest = fs::read(install.join(MANIFEST_FILENAME)).unwrap();
        let external = [
            root.join("profile"),
            root.join("shortcut"),
            root.join("association"),
        ];
        for path in &external {
            fs::write(path, b"foreign original").unwrap();
        }
        let mut integration = IsolatedIntegration {
            paths: external.to_vec(),
        };
        let result = run(
            request(install.clone()),
            source(),
            &mut integration,
            |step| {
                if step == fail_at {
                    Err(format!("injected {step:?}"))
                } else {
                    Ok(())
                }
            },
        );
        assert!(result.unwrap_err().starts_with("injected"), "{fail_at:?}");
        for path in &owned {
            assert_eq!(fs::read(path).unwrap(), b"old binary", "{fail_at:?}");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o750
                );
            }
        }
        for path in &external {
            assert_eq!(fs::read(path).unwrap(), b"foreign original", "{fail_at:?}");
        }
        assert_eq!(fs::read(user_data).unwrap(), b"user data");
        assert_eq!(
            fs::read(install.join(MANIFEST_FILENAME)).unwrap(),
            original_manifest
        );
        assert!(!install.join(".kr580-install.lock").exists());
        remove_root(root);
    }
}

#[test]
fn successful_commit_promotes_staged_payloads_and_publishes_only_final_manifest() {
    let root = temporary_root();
    let install = root.join("install");
    fs::create_dir_all(install.join("app")).unwrap();
    let unmanaged_legacy = install.join("app").join(binary_name("k580"));
    fs::write(&unmanaged_legacy, b"foreign legacy").unwrap();
    let mut integration = IsolatedIntegration {
        paths: ["profile", "shortcut", "association"]
            .map(|name| root.join(name))
            .to_vec(),
    };
    let report = run(
        request(install.clone()),
        source(),
        &mut integration,
        |step| {
            if step != InstallStep::Manifest {
                assert!(!install.join(MANIFEST_FILENAME).exists());
            }
            Ok(())
        },
    )
    .unwrap();
    assert!(report.path_changed && report.system_integrated && report.file_association_created);
    assert_eq!(fs::read(&report.kr580_path).unwrap(), b"new gui");
    assert_eq!(fs::read(unmanaged_legacy).unwrap(), b"foreign legacy");
    assert_eq!(
        fs::read(install.join("bin").join(binary_name("kr"))).unwrap(),
        b"new launcher"
    );
    assert_eq!(
        read_manifest(&install).unwrap(),
        InstallManifest::new(InstallMode::System, InstallScope::User).with_file_association(true)
    );
    remove_root(root);
}

#[test]
fn rollback_preserves_foreign_changes_and_retains_recovery_backup() {
    let root = temporary_root();
    let path = root.join("shortcut");
    fs::write(&path, b"before").unwrap();
    let mut journal = FileJournal::default();
    journal
        .change(std::slice::from_ref(&path), || {
            fs::write(&path, b"installer").map_err(|e| e.to_string())
        })
        .unwrap();
    fs::write(&path, b"foreign after install").unwrap();
    let error = journal.rollback().unwrap_err();
    assert!(error.contains("rollback conflict"));
    assert_eq!(fs::read(&path).unwrap(), b"foreign after install");
    assert!(fs::read_dir(&root).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".kr580-backup-")
    }));
    remove_root(root);
}

#[test]
fn preflight_refuses_unmanaged_payload_and_fresh_rollback_removes_only_new_files() {
    for conflict in [false, true] {
        let root = temporary_root();
        let install = root.join("install");
        let gui = install.join("app").join(binary_name("kr580"));
        if conflict {
            fs::create_dir_all(gui.parent().unwrap()).unwrap();
            fs::write(&gui, b"foreign payload").unwrap();
        }
        let foreign = root.join("user-data");
        fs::write(&foreign, b"keep").unwrap();
        let mut integration = IsolatedIntegration {
            paths: ["profile", "shortcut", "association"]
                .map(|name| root.join(name))
                .to_vec(),
        };
        let result = run(
            request(install.clone()),
            source(),
            &mut integration,
            |step| {
                if step == InstallStep::Manifest {
                    Err("late failure".into())
                } else {
                    Ok(())
                }
            },
        );
        assert!(result.is_err());
        if conflict {
            assert_eq!(fs::read(gui).unwrap(), b"foreign payload");
        } else {
            assert!(!install.exists());
        }
        assert_eq!(fs::read(foreign).unwrap(), b"keep");
        for path in integration.paths {
            assert!(!path.exists());
        }
        remove_root(root);
    }
}

#[test]
fn unrelated_file_change_between_phases_survives_installation_rollback() {
    let root = temporary_root();
    let install = root.join("install");
    let paths = ["profile", "shortcut", "association"].map(|name| root.join(name));
    for path in &paths {
        fs::write(path, b"before").unwrap();
    }
    let mut integration = IsolatedIntegration {
        paths: paths.to_vec(),
    };
    let result = run(request(install), source(), &mut integration, |step| {
        if step == InstallStep::Path {
            fs::write(&paths[2], b"foreign").unwrap();
        }
        Ok(())
    });
    assert!(
        result
            .unwrap_err()
            .contains("file changed before installation step")
    );
    assert_eq!(fs::read(&paths[0]).unwrap(), b"before");
    assert_eq!(fs::read(&paths[1]).unwrap(), b"before");
    assert_eq!(fs::read(&paths[2]).unwrap(), b"foreign");
    remove_root(root);
}

fn source() -> SourceBundle {
    SourceBundle {
        kr580: SourceBinary::Embedded(b"new gui"),
        kr: SourceBinary::Embedded(b"new launcher"),
        uninstaller: SourceBinary::Embedded(b"new uninstaller"),
    }
}
fn request(install_dir: PathBuf) -> InstallRequest {
    InstallRequest {
        mode: InstallMode::System,
        scope: InstallScope::User,
        install_dir,
        add_to_path: true,
        create_desktop_shortcut: true,
        associate_program_files: true,
    }
}
fn temporary_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "kr580-install-rollback-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    root
}
fn remove_root(root: PathBuf) {
    assert!(
        fs::canonicalize(&root)
            .unwrap()
            .starts_with(fs::canonicalize(std::env::temp_dir()).unwrap())
    );
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("kr580-install-rollback-")
    );
    fs::remove_dir_all(root).unwrap();
}
