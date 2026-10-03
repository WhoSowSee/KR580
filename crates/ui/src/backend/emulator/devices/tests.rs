use super::Emulator;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

struct CountingAllocator;

// SAFETY: Every allocation and deallocation delegates unchanged arguments to System.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
        // SAFETY: The allocator caller guarantees a valid Layout.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The allocator caller supplies the live System allocation and its original Layout.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn unchanged_polls_allocate_nothing_with_long_device_buffers() {
    let mut emulator = Emulator::default();
    emulator.bus.floppy.set_debug_buffer(true);
    for value in (0..100_000).map(|value| value as u8) {
        emulator.bus.monitor.output_byte(value);
        emulator.bus.printer.output_byte(value);
        emulator.bus.floppy.write_byte(value).unwrap();
    }
    let mut revision = emulator.bus.network.revision();
    COUNTING.set(true);
    for _ in 0..2_000 {
        assert!(emulator.poll_devices(&mut revision).is_none());
    }
    COUNTING.set(false);
    assert_eq!(ALLOCATIONS.get(), 0);
}
