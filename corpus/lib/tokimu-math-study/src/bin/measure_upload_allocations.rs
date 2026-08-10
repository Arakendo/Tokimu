//! Allocation evidence for the representative B/C provider upload boundaries.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

use tokimu_math_study::{migration_b, migration_c};

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|argument| argument.parse::<u32>())
        .transpose()
        .expect("iteration count must be an unsigned integer")
        .unwrap_or(1_000_000);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    let b_checksum = migration_b::provider_upload_workload(iterations);
    let b_allocations = ALLOCATIONS.load(Ordering::Relaxed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    let c_checksum = migration_c::provider_upload_workload(iterations);
    let c_allocations = ALLOCATIONS.load(Ordering::Relaxed);

    assert_eq!(b_checksum, c_checksum);
    assert_eq!(b_allocations, 0, "provider-backed upload path allocated");
    assert_eq!(c_allocations, 0, "owned upload path allocated");

    println!("iterations={iterations}");
    println!("provider_backed_upload_allocations={b_allocations}");
    println!("owned_upload_allocations={c_allocations}");
    println!("checksum={b_checksum}");
}
