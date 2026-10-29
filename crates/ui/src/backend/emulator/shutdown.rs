use super::{AppError, Emulator};

impl Emulator {
    pub(in crate::backend) fn shutdown_devices(&mut self) -> Result<(), AppError> {
        let (floppy, hdd, printer) = self.io_runtime.block_on(async {
            tokio::join!(
                self.bus.floppy.shutdown(),
                self.bus.hdd.shutdown(),
                self.bus.printer.shutdown(),
            )
        });
        floppy?;
        hdd?;
        printer?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
