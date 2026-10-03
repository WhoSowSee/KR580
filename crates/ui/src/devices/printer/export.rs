use super::{PrintCompletion, native};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

pub(super) async fn write_spools(
    path: PathBuf,
    mut spools: mpsc::Receiver<Vec<u8>>,
    completion: mpsc::Sender<PrintCompletion>,
) {
    while let Some(bytes) = spools.recv().await {
        let result = async {
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await?;
            file.write_all(&bytes).await?;
            file.flush().await
        }
        .await;
        let result =
            result.map_err(|error: std::io::Error| native::PrintFailure::Failed(error.to_string()));
        if completion.send(PrintCompletion { result }).await.is_err() {
            break;
        }
    }
}
