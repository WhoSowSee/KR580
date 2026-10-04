use super::{binary_name, platform};
use std::path::{Path, PathBuf};

mod payload {
    include!(concat!(env!("OUT_DIR"), "/installer_payload.rs"));
}

pub(super) struct SourceBundle {
    pub(super) kr: SourceBinary,
    pub(super) kr580: SourceBinary,
    pub(super) uninstaller: SourceBinary,
}

pub(super) enum SourceBinary {
    Embedded(&'static [u8]),
    File(PathBuf),
}

impl SourceBundle {
    pub(super) fn discover() -> Result<Self, String> {
        if let Some(bundle) = Self::embedded()? {
            return Ok(bundle);
        }
        Ok(Self {
            kr: SourceBinary::File(find_source_binary("kr")?),
            kr580: SourceBinary::File(find_source_binary("kr580")?),
            uninstaller: SourceBinary::File(find_uninstaller_source()?),
        })
    }

    fn embedded() -> Result<Option<Self>, String> {
        match (
            payload::EMBEDDED_KR,
            payload::EMBEDDED_KR580,
            payload::EMBEDDED_UNINSTALLER,
        ) {
            (Some(kr), Some(kr580), Some(uninstaller)) => Ok(Some(Self {
                kr: SourceBinary::Embedded(kr),
                kr580: SourceBinary::Embedded(kr580),
                uninstaller: SourceBinary::Embedded(uninstaller),
            })),
            (None, None, None) => Ok(None),
            _ => Err("embedded installer payload is incomplete".to_owned()),
        }
    }
}

fn find_uninstaller_source() -> Result<PathBuf, String> {
    find_source_binary("k580-uninstaller")
        .or_else(|_| std::env::current_exe().map_err(|e| format!("current exe: {e}")))
}

fn find_source_binary(name: &str) -> Result<PathBuf, String> {
    let current = std::env::current_exe().map_err(|e| format!("current exe: {e}"))?;
    let binary = binary_name(name);
    let mut candidates = Vec::new();

    if let Some(dir) = current.parent() {
        candidates.push(dir.join(binary.clone()));
        if let Some(root) = dir.parent() {
            candidates.push(root.join("bin").join(binary.clone()));
            candidates.push(root.join("app").join(binary.clone()));
        }
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    candidates.push(
        manifest_dir
            .join("..")
            .join("..")
            .join("target")
            .join(profile)
            .join(binary),
    );

    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| format!("{name} binary not found"))
}

pub(super) fn copy_executable(source: &SourceBinary, destination: &Path) -> Result<(), String> {
    match source {
        SourceBinary::Embedded(bytes) => {
            std::fs::write(destination, bytes)
                .map_err(|e| format!("write {}: {e}", destination.display()))?;
        }
        SourceBinary::File(path) => {
            std::fs::copy(path, destination)
                .map_err(|e| format!("copy {}: {e}", path.display()))?;
        }
    }
    platform::make_executable(destination)
}
