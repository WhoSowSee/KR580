use super::{StorageCommand, mpsc};
use crate::devices::DeviceError;
use tokio::io::{AsyncWriteExt, BufWriter};

pub(super) async fn write_file(
    file: std::fs::File,
    mut commands: mpsc::Receiver<StorageCommand>,
    errors: mpsc::Sender<DeviceError>,
) {
    let result = async {
        let mut file = BufWriter::new(tokio::fs::File::from_std(file));
        let mut batch = Vec::with_capacity(4096);
        let mut bytes = Vec::with_capacity(4096);
        while commands.recv_many(&mut batch, 4096).await != 0 {
            for command in batch.drain(..) {
                match command {
                    StorageCommand::Write(byte) => bytes.push(byte),
                    StorageCommand::Flush => {
                        file.write_all(&bytes).await?;
                        bytes.clear();
                        file.flush().await?;
                    }
                }
            }
            file.write_all(&bytes).await?;
            bytes.clear();
            file.flush().await?;
        }
        file.flush().await
    }
    .await;
    if let Err(error) = result {
        let _ = errors.try_send(DeviceError::from(error));
    }
}
