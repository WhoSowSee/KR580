#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppErrorKind {
    ProgramExtension,
    SubprogramExtension,
    EmptyFile,
    ProgramSize,
    SettingsVersion,
    SettingsJson,
    ImportFormat,
    ExportFormat,
    NotFound,
    PermissionDenied,
    AlreadyExists,
    DiskFull,
    ReadFile,
    WriteFile,
    Io,
    AddressRange,
    UndocumentedOpcode,
    DeviceNotReady,
    Internal,
    Generic,
}

impl AppErrorKind {
    pub(super) fn io(error: &std::io::Error, fallback: Self) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            std::io::ErrorKind::AlreadyExists => Self::AlreadyExists,
            std::io::ErrorKind::StorageFull => Self::DiskFull,
            _ => fallback,
        }
    }

    pub(super) fn port(error: &k580_core::PortError) -> Self {
        match error {
            k580_core::PortError::NotReady => Self::DeviceNotReady,
            k580_core::PortError::PathNotFound(_) => Self::NotFound,
            k580_core::PortError::PermissionDenied(_) => Self::PermissionDenied,
            k580_core::PortError::Io(_) => Self::Io,
            _ => Self::Generic,
        }
    }
}
