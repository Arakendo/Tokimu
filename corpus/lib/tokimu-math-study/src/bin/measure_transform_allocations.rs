//! Allocation evidence for the shared A/B transform workload.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

use tokimu_math_study::workloads::{
    baseline_transform_workload, owned_transform_workload, provider_backed_transform_workload,
};

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
    let baseline_checksum = baseline_transform_workload(iterations);
    let baseline_allocations = ALLOCATIONS.load(Ordering::Relaxed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    let candidate_checksum = provider_backed_transform_workload(iterations);
    let candidate_allocations = ALLOCATIONS.load(Ordering::Relaxed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    let owned_checksum = owned_transform_workload(iterations);
    let owned_allocations = ALLOCATIONS.load(Ordering::Relaxed);

    assert_eq!(baseline_checksum, candidate_checksum);
    assert_eq!(baseline_checksum, owned_checksum);
    assert_eq!(baseline_allocations, 0, "baseline workload allocated");
    assert_eq!(
        candidate_allocations, 0,
        "provider-backed workload allocated"
    );
    assert_eq!(owned_allocations, 0, "owned workload allocated");

    println!("iterations={iterations}");
    println!("baseline_allocations={baseline_allocations}");
    println!("provider_backed_allocations={candidate_allocations}");
    println!("owned_allocations={owned_allocations}");
    println!("checksum={baseline_checksum:?}");
}
