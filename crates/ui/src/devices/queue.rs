use super::DeviceError;
use tokio::sync::mpsc::{Sender, error::TrySendError};

pub(super) fn enqueue<T>(sender: &Sender<T>, value: T) -> Result<(), DeviceError> {
    sender.try_send(value).map_err(|error| match error {
        TrySendError::Full(_) => DeviceError::Busy,
        TrySendError::Closed(_) => DeviceError::Disconnected,
    })
}
