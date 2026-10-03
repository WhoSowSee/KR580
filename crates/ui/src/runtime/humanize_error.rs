use crate::backend::AppError;
use crate::i18n::{Key, Lang};
use k580_ui::backend::error::AppErrorKind;

pub(crate) fn humanize(error: &AppError, lang: Lang) -> String {
    let key = match error.kind() {
        AppErrorKind::ProgramExtension => Key::ErrNotA580File,
        AppErrorKind::SubprogramExtension => Key::ErrNotAKrsFile,
        AppErrorKind::EmptyFile => Key::ErrFileEmpty,
        AppErrorKind::ProgramSize => Key::ErrWrong580Size,
        AppErrorKind::SettingsVersion => Key::ErrSettingsNewerVersion,
        AppErrorKind::SettingsJson => Key::ErrSettingsCorrupt,
        AppErrorKind::ImportFormat => Key::ErrCannotReadFileFormat,
        AppErrorKind::ExportFormat => Key::ErrCannotWriteTable,
        AppErrorKind::ReadFile => Key::ErrCannotReadFile,
        AppErrorKind::WriteFile => Key::ErrCannotWriteFile,
        AppErrorKind::NotFound => Key::ErrFileNotFound,
        AppErrorKind::PermissionDenied => Key::ErrPermissionDenied,
        AppErrorKind::AlreadyExists => Key::ErrFileAlreadyExists,
        AppErrorKind::DiskFull => Key::ErrDiskFull,
        AppErrorKind::Io => Key::ErrIoGeneric,
        AppErrorKind::AddressRange => Key::ErrAddressOutOfRange,
        AppErrorKind::UndocumentedOpcode => Key::ErrUndocumentedOpcode,
        AppErrorKind::DeviceNotReady => Key::ErrDeviceNotReady,
        AppErrorKind::DeviceBusy => Key::ErrDeviceBusy,
        AppErrorKind::Internal => Key::ErrInternal,
        AppErrorKind::Generic => return format!("{} ({error})", lang.t(Key::ErrGenericFailed)),
    };
    lang.t(key).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_io_kind_wins_over_os_number_and_message_wording() {
        for language in [Lang::Ru, Lang::En] {
            for (kind, expected) in [
                (
                    std::io::ErrorKind::PermissionDenied,
                    Key::ErrPermissionDenied,
                ),
                (std::io::ErrorKind::Other, Key::ErrIoGeneric),
                (std::io::ErrorKind::StorageFull, Key::ErrDiskFull),
            ] {
                let error = std::io::Error::new(kind, "access denied, no space left (os error 5)");
                assert_eq!(humanize(&error.into(), language), language.t(expected));
            }
        }
    }
}
