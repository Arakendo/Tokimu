//! Allocation observation for full A/B/C stereo-camera construction.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

use tokimu_math_study::workloads::{
    baseline_stereo_camera_workload, owned_stereo_camera_workload,
    provider_backed_stereo_camera_workload,
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

fn observe(run: fn(u32) -> f32, iterations: u32) -> (usize, f32) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    let checksum = run(iterations);
    (ALLOCATIONS.load(Ordering::Relaxed), checksum)
}

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|value| value.parse::<u32>())
        .transpose()
        .expect("iteration count must be an unsigned integer")
        .unwrap_or(100_000);

    let (baseline_allocations, baseline) = observe(baseline_stereo_camera_workload, iterations);
    let (provider_backed_allocations, provider_backed) =
        observe(provider_backed_stereo_camera_workload, iterations);
    let (owned_allocations, owned) = observe(owned_stereo_camera_workload, iterations);
    assert!((baseline - provider_backed).abs() <= 1.0e-3);
    assert!((baseline - owned).abs() <= 1.0e-3);
    assert_eq!(
        baseline_allocations, 0,
        "baseline camera workload allocated"
    );
    assert_eq!(
        provider_backed_allocations, 0,
        "B camera workload allocated"
    );
    assert_eq!(owned_allocations, 0, "C camera workload allocated");

    println!("iterations={iterations}");
    println!("baseline_allocations={baseline_allocations}");
    println!("provider_backed_allocations={provider_backed_allocations}");
    println!("owned_allocations={owned_allocations}");
    println!("checksum={baseline}");
}
