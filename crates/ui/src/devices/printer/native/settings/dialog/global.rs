use super::last_os_error;
use windows_sys::Win32::Foundation::HGLOBAL;
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

pub(super) struct LockedGlobal {
    handle: HGLOBAL,
    ptr: *const u8,
    len: usize,
}

impl LockedGlobal {
    pub(super) fn lock(handle: HGLOBAL) -> Result<Self, String> {
        // SAFETY: The private caller supplies a global block owned by the live PRINTDLGW.
        let len = unsafe { GlobalSize(handle) };
        if len == 0 || len > isize::MAX as usize {
            return Err("printer dialog memory size is invalid".to_owned());
        }
        // SAFETY: The live dialog owns handle; the successful lock is paired with Drop's unlock.
        let ptr = unsafe { GlobalLock(handle) }.cast::<u8>();
        if ptr.is_null() {
            return Err(last_os_error("GlobalLock"));
        }
        Ok(Self { handle, ptr, len })
    }

    pub(super) fn bytes(&self) -> &[u8] {
        // SAFETY: PrintDlgW filled the owned block and GlobalLock pins all len bytes until Drop.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for LockedGlobal {
    fn drop(&mut self) {
        // SAFETY: This object owns one successful lock and the dialog still owns its memory.
        unsafe { GlobalUnlock(self.handle) };
    }
}
