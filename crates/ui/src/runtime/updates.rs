mod download;
mod install;
mod release;

pub(crate) use install::PreparedUpdate;
pub(crate) use release::{Asset, Release, check};

use std::sync::Once;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static RUSTLS_PROVIDER: Once = Once::new();

#[derive(Default, Debug)]
pub(crate) struct TransferProgress {
    received: AtomicU64,
    preparing: AtomicBool,
}

impl TransferProgress {
    pub(crate) fn preparing(&self) -> bool {
        self.preparing.load(Ordering::Relaxed)
    }

    pub(crate) fn fraction(&self, total: u64) -> f32 {
        (self.received.load(Ordering::Relaxed) as f64 / total.max(1) as f64).clamp(0.0, 1.0) as f32
    }
}

pub(crate) use download::download;

pub(crate) fn ensure_tls_provider() {
    RUSTLS_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn client() -> Result<reqwest::Client, String> {
    ensure_tls_provider();
    reqwest::Client::builder()
        .user_agent(concat!("KR580/", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .connect_timeout(std::time::Duration::from_secs(15))
        .read_timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| error.to_string())
}
