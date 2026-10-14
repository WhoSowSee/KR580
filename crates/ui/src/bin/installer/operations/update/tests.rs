use super::{ParentProcess, PendingFiles};
use crate::installer::operations::binary_name;
use crate::installer::operations::source::SourceBinary;
use crate::installer::operations::source::SourceBundle;
use k580_ui::install_mode::{InstallManifest, InstallMode, InstallScope};
use std::path::Path;

fn installation(root: &Path) {
    std::fs::create_dir_all(root.join("app")).unwrap();
    std::fs::create_dir_all(root.join("bin")).unwrap();
    for (directory, name) in [("app", "kr580"), ("bin", "kr"), ("app", "uninstaller")] {
        std::fs::write(root.join(directory).join(binary_name(name)), b"old").unwrap();
    }
    k580_ui::install_mode::write_manifest(
        root,
        &InstallManifest::new(InstallMode::Portable, InstallScope::User),
    )
    .unwrap();
    std::fs::create_dir(root.join("data")).unwrap();
    std::fs::write(root.join("data/settings.json"), b"user settings").unwrap();
}

fn source() -> SourceBundle {
    SourceBundle {
        kr: SourceBinary::Embedded(b"new launcher"),
        kr580: SourceBinary::Embedded(b"new emulator"),
        uninstaller: SourceBinary::Embedded(b"new uninstaller"),
    }
}

#[test]
fn prepared_update_does_not_replace_files_until_applied() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    installation(root);
    let manifest = std::fs::read(root.join("install.json")).unwrap();
    let pending = PendingFiles::prepare(root, source()).unwrap();
    let mut command = std::process::Command::new(if cfg!(windows) { "cmd" } else { "cat" });
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.args(["/C", "more"]).creation_flags(0x08000000);
    }
    let mut parent = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let pid = parent.id();
    let (started, ready) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let process = ParentProcess::open(pid).unwrap();
        started.send(()).unwrap();
        process.wait().unwrap();
        let mut pending = pending;
        pending.apply().unwrap();
    });
    ready
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert!(!worker.is_finished());
    assert_eq!(
        std::fs::read(root.join("app").join(binary_name("kr580"))).unwrap(),
        b"old"
    );
    drop(parent.stdin.take());
    parent.wait().unwrap();
    worker.join().unwrap();
    assert_eq!(
        std::fs::read(root.join("app").join(binary_name("kr580"))).unwrap(),
        b"new emulator"
    );
    assert_eq!(
        std::fs::read(root.join("data/settings.json")).unwrap(),
        b"user settings"
    );
    assert_eq!(std::fs::read(root.join("install.json")).unwrap(), manifest);
}

#[test]
fn failed_replacement_restores_preceding_files() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    installation(root);
    let mut pending = PendingFiles::prepare(root, source()).unwrap();
    std::fs::remove_file(&pending.staged[1].0).unwrap();
    assert!(pending.apply().is_err());
    for (directory, name) in [("app", "kr580"), ("bin", "kr"), ("app", "uninstaller")] {
        assert_eq!(
            std::fs::read(root.join(directory).join(binary_name(name))).unwrap(),
            b"old"
        );
    }
    assert_eq!(
        std::fs::read(root.join("data/settings.json")).unwrap(),
        b"user settings"
    );
}
