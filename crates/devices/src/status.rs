use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceStatus {
    Ready,
    NotReady,
    Busy,
    NoData,
    Connected,
    Listening,
    Disconnected,
    Error(String),
}

impl DeviceStatus {
    pub fn code(&self) -> u8 {
        match self {
            Self::Ready | Self::Connected | Self::Listening => 0,
            Self::NoData => 1,
            Self::NotReady => 2,
            Self::Busy => 3,
            Self::Disconnected => 4,
            Self::Error(_) => 0xFF,
        }
    }
}
