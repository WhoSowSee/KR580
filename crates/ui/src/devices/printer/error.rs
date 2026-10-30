use std::error::Error;
use std::fmt;

/// Classification of native printer failures without parsing display text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrinterErrorKind {
    UnsupportedPlatform,
    NativeApi,
    InvalidData,
}

/// An extended error code returned by the common print dialog.
#[derive(Debug, thiserror::Error)]
#[error("printer setup failed: 0x{code:04X}")]
pub struct PrinterDialogError {
    code: u32,
}

impl PrinterDialogError {
    /// Returns the unmodified `CommDlgExtendedError` code.
    pub fn code(&self) -> u32 {
        self.code
    }
}

/// A printer failure with display context and its original cause when available.
#[derive(Debug)]
pub struct PrinterError {
    kind: PrinterErrorKind,
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl PrinterError {
    /// Returns the failure category without interpreting localized message text.
    pub fn kind(&self) -> PrinterErrorKind {
        self.kind
    }

    #[cfg(any(windows, test))]
    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self {
            kind: PrinterErrorKind::InvalidData,
            message: message.into(),
            source: None,
        }
    }

    #[cfg(any(windows, test))]
    pub(super) fn invalid_source(
        message: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind: PrinterErrorKind::InvalidData,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    #[cfg(windows)]
    pub(super) fn native(
        message: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind: PrinterErrorKind::NativeApi,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    #[cfg(windows)]
    pub(super) fn native_message(message: impl Into<String>) -> Self {
        Self {
            kind: PrinterErrorKind::NativeApi,
            message: message.into(),
            source: None,
        }
    }

    #[cfg(windows)]
    pub(super) fn dialog(code: u32) -> Self {
        let source = PrinterDialogError { code };
        Self::native(source.to_string(), source)
    }

    #[cfg(not(windows))]
    pub(super) fn unsupported(message: impl Into<String>) -> Self {
        Self {
            kind: PrinterErrorKind::UnsupportedPlatform,
            message: message.into(),
            source: None,
        }
    }
}

impl fmt::Display for PrinterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for PrinterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &dyn Error)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::{PrinterDialogError, PrinterError, PrinterErrorKind};
    use std::error::Error;

    #[test]
    fn native_causes_retain_their_original_type_and_code() {
        let cause = std::io::Error::from_raw_os_error(5);
        let error = PrinterError::native("Win32 failure", cause);
        assert_eq!(error.kind(), PrinterErrorKind::NativeApi);
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .raw_os_error(),
            Some(5)
        );
        let code = windows::core::HRESULT(0x80004005u32 as i32);
        let error = PrinterError::native("COM failure", windows::core::Error::from_hresult(code));
        assert_eq!(error.to_string(), "COM failure");
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<windows::core::Error>()
                .unwrap()
                .code(),
            code
        );
        let error = PrinterError::dialog(5);
        assert_eq!(error.to_string(), "printer setup failed: 0x0005");
        let source = error.source().unwrap();
        assert_eq!(
            source.downcast_ref::<PrinterDialogError>().unwrap().code(),
            5
        );
    }
}
