use k580_core::{Cpu8080State, NullBus};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

struct CountingAllocator;

// SAFETY: Allocation and deallocation delegate unchanged layouts and pointers to System.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCATIONS.try_with(|count| {
            if let Some(value) = count.get() {
                count.set(Some(value + 1));
            }
        });
        // SAFETY: The caller provides the valid allocation layout required by System.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The caller provides a live allocation and its original layout.
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn instruction_metadata_executes_a_loop_without_heap_allocations() {
    let mut cpu = Cpu8080State::default();
    cpu.set_memory_block(0, &[0x04, 0xC3, 0x00, 0x00]).unwrap();
    let mut bus = NullBus::default();
    ALLOCATIONS.with(|count| count.set(Some(0)));
    for _ in 0..20_000 {
        cpu.step_instruction_metadata(&mut bus).unwrap();
    }
    let allocations = ALLOCATIONS.with(|count| count.replace(None).unwrap());
    assert_eq!(allocations, 0);
    assert_eq!(cpu.registers.b, 16);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.cycle_count, 150_000);
}
