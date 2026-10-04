use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use winreg::enums::REG_EXPAND_SZ;
use winreg::types::ToRegValue;

static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn rollback_restores_raw_type_and_preserves_foreign_values_and_subkeys() {
    let fixture = Fixture::new();
    let key = root_key(InstallScope::User)
        .create_subkey(&fixture.key)
        .unwrap()
        .0;
    let mut original = "%USERPROFILE%\\bin".to_reg_value();
    original.vtype = REG_EXPAND_SZ;
    key.set_raw_value("Path", &original).unwrap();
    let mut journal = RegistryJournal::default();
    journal
        .capture_values(
            InstallScope::User,
            vec![
                (
                    fixture.key.clone(),
                    "Path".into(),
                    Some("new path".to_reg_value()),
                ),
                (
                    fixture.key.clone(),
                    "Created".into(),
                    Some(1u32.to_reg_value()),
                ),
            ],
            InstallStep::Path,
        )
        .unwrap();
    journal
        .before_change(&fixture.directory, InstallStep::Path)
        .unwrap();
    key.set_value("Path", &"new path").unwrap();
    key.set_value("Created", &1u32).unwrap();
    journal.observe().unwrap();
    key.set_value("Foreign", &"keep").unwrap();
    key.create_subkey("ForeignChild")
        .unwrap()
        .0
        .set_value("OwnedByOther", &42u32)
        .unwrap();
    journal.rollback().unwrap();
    assert!(key.get_raw_value("Path").unwrap() == original);
    assert!(key.get_raw_value("Created").is_err());
    assert_eq!(key.get_value::<String, _>("Foreign").unwrap(), "keep");
    assert_eq!(
        key.open_subkey("ForeignChild")
            .unwrap()
            .get_value::<u32, _>("OwnedByOther")
            .unwrap(),
        42
    );
}

#[test]
fn conflict_keeps_foreign_value_and_recovery_json() {
    let fixture = Fixture::new();
    let key = root_key(InstallScope::User)
        .create_subkey(&fixture.key)
        .unwrap()
        .0;
    key.set_value("Path", &"old").unwrap();
    let mut journal = RegistryJournal::default();
    journal
        .capture_values(
            InstallScope::User,
            vec![(
                fixture.key.clone(),
                "Path".into(),
                Some("installer".to_reg_value()),
            )],
            InstallStep::Path,
        )
        .unwrap();
    journal
        .before_change(&fixture.directory, InstallStep::Path)
        .unwrap();
    key.set_value("Path", &"installer").unwrap();
    journal.observe().unwrap();
    key.set_value("Path", &"foreign").unwrap();
    assert!(
        journal
            .rollback()
            .unwrap_err()
            .contains("registry rollback conflict")
    );
    assert_eq!(key.get_value::<String, _>("Path").unwrap(), "foreign");
    assert!(journal.recovery.as_ref().unwrap().is_file());
}

struct Fixture {
    key: String,
    directory: PathBuf,
}

#[test]
fn unrelated_phase_changes_are_not_recorded_as_installer_writes() {
    let fixture = Fixture::new();
    let key = root_key(InstallScope::User)
        .create_subkey(&fixture.key)
        .unwrap()
        .0;
    key.set_value("Path", &"old path").unwrap();
    key.set_value("Association", &"old association").unwrap();
    let mut journal = RegistryJournal::default();
    journal
        .capture_values(
            InstallScope::User,
            vec![(
                fixture.key.clone(),
                "Path".into(),
                Some("installer".to_reg_value()),
            )],
            InstallStep::Path,
        )
        .unwrap();
    journal
        .capture_values(
            InstallScope::User,
            vec![(
                fixture.key.clone(),
                "Association".into(),
                Some("new association".to_reg_value()),
            )],
            InstallStep::Association,
        )
        .unwrap();
    journal
        .before_change(&fixture.directory, InstallStep::Path)
        .unwrap();
    key.set_value("Path", &"installer").unwrap();
    key.set_value("Association", &"foreign").unwrap();
    assert!(journal.observe().is_err());
    journal.rollback().unwrap();
    assert_eq!(key.get_value::<String, _>("Path").unwrap(), "old path");
    assert_eq!(
        key.get_value::<String, _>("Association").unwrap(),
        "foreign"
    );
}
impl Fixture {
    fn new() -> Self {
        let nonce = NEXT.fetch_add(1, Ordering::Relaxed);
        let key = format!(
            "Software\\KR580\\Tests\\InstallerRollback-{}-{nonce}",
            std::process::id()
        );
        let directory = std::env::temp_dir().join(format!(
            "kr580-registry-rollback-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        Self { key, directory }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(
            self.key
                .starts_with("Software\\KR580\\Tests\\InstallerRollback-")
        );
        let _ = root_key(InstallScope::User).delete_subkey_all(&self.key);
        assert!(
            std::fs::canonicalize(&self.directory)
                .unwrap()
                .starts_with(std::fs::canonicalize(std::env::temp_dir()).unwrap())
        );
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
