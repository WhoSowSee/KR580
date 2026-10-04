use super::super::journal::sibling;
use super::super::transaction::InstallStep;
use super::super::{InstallRequest, platform};
use k580_ui::install_mode::InstallScope;
use std::path::{Path, PathBuf};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE};
use winreg::types::ToRegValue;
use winreg::{RegKey, RegValue};

struct SavedValue {
    scope: InstallScope,
    phase: InstallStep,
    subkey: String,
    name: String,
    original: Option<RegValue>,
    expected: Option<RegValue>,
    observation_failed: bool,
    planned: Option<RegValue>,
}

#[derive(Default)]
pub(super) struct RegistryJournal {
    values: Vec<SavedValue>,
    recovery: Option<PathBuf>,
    active_phase: Option<InstallStep>,
}

impl RegistryJournal {
    pub(super) fn capture(
        request: &InstallRequest,
        gui: &Path,
        legacy_scope: Option<InstallScope>,
    ) -> Result<Self, String> {
        let mut journal = Self::default();
        let targets = platform::windows::planned_registry_values(request)?;
        let (path, system): (Vec<_>, Vec<_>) = targets
            .into_iter()
            .map(|(key, name, value)| (key, name, Some(value)))
            .partition(|(_, name, _)| name == "Path");
        journal.capture_values(request.scope, path, InstallStep::Path)?;
        journal.capture_values(request.scope, system, InstallStep::System)?;
        if request.associate_program_files {
            let targets = k580_ui::file_assoc::registry_values_for_executable(gui)?
                .into_iter()
                .map(|value| (value.subkey, value.name, Some(value.value.to_reg_value())))
                .collect();
            journal.capture_values(request.scope, targets, InstallStep::Association)?;
        } else if let Some(scope) = legacy_scope {
            let targets = k580_ui::file_assoc::registry_values_for_executable(gui)?
                .into_iter()
                .map(|value| (value.subkey, value.name, None))
                .collect();
            journal.capture_values(scope, targets, InstallStep::Association)?;
        }
        Ok(journal)
    }

    fn capture_values(
        &mut self,
        scope: InstallScope,
        targets: Vec<(String, String, Option<RegValue>)>,
        phase: InstallStep,
    ) -> Result<(), String> {
        for (subkey, name, planned) in targets {
            let original = read(scope, &subkey, &name)?;
            let expected = copy_value(&original);
            self.values.push(SavedValue {
                scope,
                phase,
                subkey,
                name,
                original,
                expected,
                observation_failed: false,
                planned,
            });
        }
        Ok(())
    }

    pub(super) fn before_change(&mut self, root: &Path, phase: InstallStep) -> Result<(), String> {
        for saved in &self.values {
            if read(saved.scope, &saved.subkey, &saved.name)? != saved.expected {
                return Err(format!(
                    "registry value changed before installation step: {}\\{}",
                    saved.subkey, saved.name
                ));
            }
        }
        self.active_phase = Some(phase);
        if self.recovery.is_none() && !self.values.is_empty() {
            let bytes = serde_json::to_vec_pretty(&self.values.iter().map(|saved| serde_json::json!({
                "scope": saved.scope,
                "subkey": saved.subkey,
                "name": saved.name,
                "original": saved.original.as_ref().map(|value| serde_json::json!({ "bytes": value.bytes, "type": value.vtype.clone() as u32 })),
            })).collect::<Vec<_>>()).map_err(|error| format!("serialize registry backup: {error}"))?;
            let path = sibling(&root.join("registry.json"), "registry-recovery")?;
            self.recovery = Some(path.clone());
            std::fs::write(&path, bytes)
                .map_err(|error| format!("write registry recovery {}: {error}", path.display()))?;
        }
        Ok(())
    }

    pub(super) fn observe(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        for saved in &mut self.values {
            match read(saved.scope, &saved.subkey, &saved.name) {
                Ok(value)
                    if Some(saved.phase) == self.active_phase
                        && (value == saved.planned || value == saved.expected) =>
                {
                    saved.expected = value
                }
                Ok(value) if value != saved.expected => {
                    if Some(saved.phase) == self.active_phase {
                        saved.observation_failed = true;
                    }
                    errors.push(format!(
                        "foreign registry change retained: {}\\{}",
                        saved.subkey, saved.name
                    ));
                }
                Ok(_) => {}
                Err(error) => {
                    if Some(saved.phase) == self.active_phase {
                        saved.observation_failed = true;
                    }
                    errors.push(error);
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    pub(super) fn rollback(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        for saved in self.values.iter().rev() {
            if saved.original == saved.expected && !saved.observation_failed {
                continue;
            }
            let result = (|| {
                if saved.observation_failed
                    || read(saved.scope, &saved.subkey, &saved.name)? != saved.expected
                {
                    return Err(format!(
                        "registry rollback conflict: {}\\{}",
                        saved.subkey, saved.name
                    ));
                }
                let key = root_key(saved.scope)
                    .create_subkey_with_flags(&saved.subkey, KEY_QUERY_VALUE | KEY_SET_VALUE)
                    .map_err(|error| format!("restore registry key {}: {error}", saved.subkey))?
                    .0;
                match &saved.original {
                    Some(value) => key.set_raw_value(&saved.name, value),
                    None => key.delete_value(&saved.name),
                }
                .map_err(|error| {
                    format!(
                        "restore registry value {}\\{}: {error}",
                        saved.subkey, saved.name
                    )
                })
            })();
            if let Err(error) = result {
                errors.push(error);
            }
        }
        if errors.is_empty() {
            self.commit();
            Ok(())
        } else {
            Err(format!(
                "{}; registry recovery: {}",
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
            tracing::warn!(path = %path.display(), %error, "registry backup retained");
        }
    }

    pub(super) fn changed(&self) -> bool {
        self.values
            .iter()
            .any(|value| value.original != value.expected || value.observation_failed)
    }
}

fn root_key(scope: InstallScope) -> RegKey {
    RegKey::predef(match scope {
        InstallScope::User => HKEY_CURRENT_USER,
        InstallScope::Machine => HKEY_LOCAL_MACHINE,
    })
}

fn read(scope: InstallScope, subkey: &str, name: &str) -> Result<Option<RegValue>, String> {
    let key = match root_key(scope).open_subkey_with_flags(subkey, KEY_QUERY_VALUE) {
        Ok(key) => key,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("read registry key {subkey}: {error}")),
    };
    match key.get_raw_value(name) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("read registry value {subkey}\\{name}: {error}")),
    }
}

fn copy_value(value: &Option<RegValue>) -> Option<RegValue> {
    value.as_ref().map(|value| RegValue {
        bytes: value.bytes.clone(),
        vtype: value.vtype.clone(),
    })
}

#[cfg(test)]
mod tests;
