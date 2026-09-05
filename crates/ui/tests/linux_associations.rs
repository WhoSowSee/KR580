#![cfg(target_os = "linux")]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("kr580-assoc-test-{}-{nonce}", std::process::id()));
        for path in ["tools", "config", "data", "home", "app"] {
            std::fs::create_dir_all(root.join(path)).unwrap();
        }
        let fixture = Self { root };
        fixture.script("xdg-mime", r#"#!/bin/sh
file="$XDG_CONFIG_HOME/mimeapps.list"
if [ "$1" = query ]; then
  [ ! -f "$file" ] || sed -n 's/^application\/x-kr580=\([^;]*\).*/\1/p' "$file"
  exit 0
fi
printf '[Default Applications]\napplication/x-kr580=%s;\ntext/plain=editor.desktop;\n' "$2" > "$file"
if [ "$FAIL_AT" = default ]; then exit 1; fi
"#);
        fixture.script(
            "update-mime-database",
            "#!/bin/sh\n[ \"$FAIL_AT\" != mime ]\n",
        );
        fixture.script(
            "update-desktop-database",
            "#!/bin/sh\n[ \"$FAIL_AT\" != desktop ]\n",
        );
        fixture
    }

    fn script(&self, name: &str, contents: &str) {
        let path = self.root.join("tools").join(name);
        std::fs::write(&path, contents).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn run(&self, flag: &str, failure: &str, owner: &str) -> Output {
        let executable = self.root.join("app").join(owner);
        std::fs::write(&executable, b"fixture").unwrap();
        Command::new(env!("CARGO_BIN_EXE_kr"))
            .arg(flag)
            .env_remove("SNAP")
            .env_remove("SNAP_USER_COMMON")
            .env("KR580_GUI_EXECUTABLE", executable)
            .env("HOME", self.root.join("home"))
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("XDG_DATA_HOME", self.root.join("data"))
            .env("XDG_DATA_DIRS", self.root.join("empty"))
            .env("XDG_CONFIG_DIRS", self.root.join("empty"))
            .env("XDG_CURRENT_DESKTOP", "")
            .env("FAIL_AT", failure)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.root.join("tools").display()),
            )
            .output()
            .unwrap()
    }

    fn set_default(&self, handler: &str) {
        std::fs::write(self.root.join("config/mimeapps.list"), format!(
            "[Default Applications]\napplication/x-kr580={handler};\ntext/plain=editor.desktop;\n"
        )).unwrap();
    }

    fn read(&self, path: &str) -> Vec<u8> {
        std::fs::read(self.root.join(path)).unwrap()
    }
    fn exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn succeeded(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn registration_removal_restores_previous_default_and_preserves_foreign_owner() {
    let fixture = Fixture::new();
    fixture.set_default("previous.desktop");
    succeeded(fixture.run("-r", "", "kr580"));
    succeeded(fixture.run("-r", "", "kr580"));
    let handler = fixture.read("data/applications/kr580-file-handler.desktop");
    succeeded(fixture.run("-u", "", "other"));
    assert_eq!(
        fixture.read("data/applications/kr580-file-handler.desktop"),
        handler
    );
    succeeded(fixture.run("-u", "", "kr580"));
    assert!(!fixture.exists("data/applications/kr580-file-handler.desktop"));
    assert!(!fixture.exists("data/kr580/file-association.json"));
    assert!(
        String::from_utf8(fixture.read("config/mimeapps.list"))
            .unwrap()
            .contains("previous.desktop")
    );
}

#[test]
fn changed_external_default_is_preserved_on_removal() {
    let fixture = Fixture::new();
    fixture.set_default("previous.desktop");
    succeeded(fixture.run("-r", "", "kr580"));
    fixture.set_default("user-choice.desktop");
    let expected = fixture.read("config/mimeapps.list");
    succeeded(fixture.run("-u", "", "kr580"));
    assert_eq!(fixture.read("config/mimeapps.list"), expected);
}

#[test]
fn late_registration_failure_rolls_back_new_and_existing_metadata() {
    for failure in ["mime", "desktop", "default"] {
        let fixture = Fixture::new();
        fixture.set_default("previous.desktop");
        let original = fixture.read("config/mimeapps.list");
        assert!(!fixture.run("-r", failure, "kr580").status.success());
        assert_eq!(fixture.read("config/mimeapps.list"), original);
        assert!(!fixture.exists("data/applications/kr580-file-handler.desktop"));
        assert!(!fixture.exists("data/kr580/file-association.json"));
        succeeded(fixture.run("-r", "", "kr580"));
        let before = fixture.read("data/applications/kr580-file-handler.desktop");
        let state = fixture.read("data/kr580/file-association.json");
        let defaults = fixture.read("config/mimeapps.list");
        assert!(!fixture.run("-r", failure, "other").status.success());
        assert_eq!(
            fixture.read("data/applications/kr580-file-handler.desktop"),
            before
        );
        assert_eq!(fixture.read("data/kr580/file-association.json"), state);
        assert_eq!(fixture.read("config/mimeapps.list"), defaults);
    }
}

#[test]
fn failed_default_restoration_rolls_back_removed_files() {
    let fixture = Fixture::new();
    fixture.set_default("previous.desktop");
    succeeded(fixture.run("-r", "", "kr580"));
    let before = fixture.read("data/applications/kr580-file-handler.desktop");
    let defaults = fixture.read("config/mimeapps.list");
    assert!(!fixture.run("-u", "default", "kr580").status.success());
    assert_eq!(
        fixture.read("data/applications/kr580-file-handler.desktop"),
        before
    );
    assert_eq!(fixture.read("config/mimeapps.list"), defaults);
    assert!(fixture.exists("data/kr580/file-association.json"));
}

#[test]
fn no_previous_default_leaves_no_stale_entries() {
    let fixture = Fixture::new();
    succeeded(fixture.run("-r", "", "kr580"));
    succeeded(fixture.run("-u", "", "kr580"));
    let defaults = String::from_utf8(fixture.read("config/mimeapps.list")).unwrap();
    assert!(!defaults.contains("kr580-file-handler.desktop"));
    assert!(defaults.contains("text/plain=editor.desktop;"));
}

#[test]
fn invalid_executable_does_not_create_registration_files() {
    let fixture = Fixture::new();
    assert!(!fixture.run("-r", "", "tab\tname").status.success());
    assert!(!fixture.exists("data/applications/kr580-file-handler.desktop"));
    assert!(!fixture.exists("data/mime/packages/application-x-kr580.xml"));
}
