use super::AppErrorKind;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ErrorData {
    pub(super) kind: AppErrorKind,
    message: Arc<str>,
    source: Arc<dyn Error + Send + Sync>,
}

impl ErrorData {
    pub(super) fn capture(kind: AppErrorKind, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            kind,
            message: source.to_string().into(),
            source: Arc::new(source),
        }
    }
}

impl fmt::Display for ErrorData {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ErrorData {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl PartialEq for ErrorData {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.message == other.message
    }
}

impl Eq for ErrorData {}

impl From<String> for ErrorData {
    fn from(message: String) -> Self {
        Self::capture(AppErrorKind::Generic, MessageFailure(message))
    }
}

impl From<&str> for ErrorData {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct MessageFailure(String);
