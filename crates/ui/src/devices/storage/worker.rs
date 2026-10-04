use super::{StorageCommand, mpsc};
use crate::devices::DeviceError;
use tokio::io::{AsyncWriteExt, BufWriter};

type Output = BufWriter<tokio::fs::File>;

pub(super) async fn write_file(
    file: std::fs::File,
    mut commands: mpsc::Receiver<StorageCommand>,
    errors: mpsc::Sender<DeviceError>,
) {
    let result = async {
        let mut file = Some(BufWriter::new(tokio::fs::File::from_std(file)));
        let mut batch = Vec::with_capacity(4096);
        let mut bytes = Vec::with_capacity(4096);
        while commands.recv_many(&mut batch, 4096).await != 0 {
            for command in batch.drain(..) {
                match command {
                    StorageCommand::Write(byte) => bytes.push(byte),
                    StorageCommand::Flush => flush(&mut file, &mut bytes).await?,
                    StorageCommand::Attach(next) => {
                        flush(&mut file, &mut bytes).await?;
                        file = Some(BufWriter::new(tokio::fs::File::from_std(next)));
                    }
                    StorageCommand::Detach => {
                        flush(&mut file, &mut bytes).await?;
                        file = None;
                    }
                }
            }
            flush(&mut file, &mut bytes).await?;
        }
        flush(&mut file, &mut bytes).await
    }
    .await;
    if let Err(error) = result {
        let _ = errors.try_send(DeviceError::from(error));
    }
}

async fn flush(file: &mut Option<Output>, bytes: &mut Vec<u8>) -> std::io::Result<()> {
    if let Some(file) = file {
        file.write_all(bytes).await?;
        bytes.clear();
        file.flush().await?;
    } else if !bytes.is_empty() {
        return Err(std::io::Error::other("storage write has no attached file"));
    }
    Ok(())
}
