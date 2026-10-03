use super::{AppSnapshot, Emulator};

impl Emulator {
    pub(crate) fn poll_devices(
        &mut self,
        published_network: &mut (u64, u64),
    ) -> Option<AppSnapshot> {
        let floppy_changed = self.bus.floppy.poll();
        let hdd_changed = self.bus.hdd.poll();
        let printer_changed = self.bus.printer.poll();
        let network_revision = self.bus.network.revision();
        if !floppy_changed
            && !hdd_changed
            && !printer_changed
            && network_revision == *published_network
        {
            return None;
        }
        *published_network = network_revision;
        Some(self.snapshot())
    }
}

#[cfg(test)]
mod tests;
