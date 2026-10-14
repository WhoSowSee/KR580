use super::{Asset, PreparedUpdate, Release, TransferProgress};
use ring::digest::{Context, SHA256};
use std::sync::{Arc, atomic::Ordering};
use tokio::io::AsyncWriteExt;

pub(crate) async fn download(
    release: Release,
    progress: Arc<TransferProgress>,
) -> Result<PreparedUpdate, String> {
    let asset = release
        .asset
        .as_ref()
        .ok_or("no update package for this installation")?;
    let temporary = tempfile::Builder::new()
        .prefix("kr580-update-")
        .tempdir()
        .map_err(|error| error.to_string())?;
    let archive = temporary.path().join(&asset.name);
    let response = super::client()?
        .get(&asset.url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    receive(response, &archive, asset, &progress).await?;
    progress.preparing.store(true, Ordering::Relaxed);
    let version = release.version;
    tokio::task::spawn_blocking(move || {
        let prepared = super::install::prepare(&archive, temporary.path(), version)?;
        let _ = std::fs::remove_file(&archive);
        let _ = temporary.keep();
        Ok(prepared)
    })
    .await
    .map_err(|error| error.to_string())?
}

async fn receive(
    mut response: reqwest::Response,
    archive: &std::path::Path,
    asset: &Asset,
    progress: &TransferProgress,
) -> Result<(), String> {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&archive)
        .await
        .map_err(|error| error.to_string())?;
    let mut digest = Context::new(&SHA256);
    let mut received = 0u64;
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        received = received
            .checked_add(chunk.len() as u64)
            .ok_or("download size overflow")?;
        if received > asset.size {
            return Err("download exceeds release size".into());
        }
        file.write_all(&chunk)
            .await
            .map_err(|error| error.to_string())?;
        digest.update(&chunk);
        progress.received.store(received, Ordering::Relaxed);
    }
    file.sync_all().await.map_err(|error| error.to_string())?;
    drop(file);
    let hash = digest
        .finish()
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let actual = format!("sha256:{hash}");
    if received != asset.size
        || !asset
            .digest
            .as_ref()
            .is_some_and(|expected| expected.eq_ignore_ascii_case(&actual))
    {
        return Err("download size or SHA-256 digest does not match the release".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
