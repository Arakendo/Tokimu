//! Test-only allocation observation for ordinary Alternative C value work.
//!
//! The `unsafe` allocator forwarding is isolated to this host-only harness;
//! the candidate source remains safe Rust and has no allocation API. Run this
//! integration test by itself so unrelated test-runner activity cannot be
//! counted during the measured interval.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use tokimu_math_study_owned_subset::{Mat4, Vec3};

struct TrackingAllocator;

static TRACKING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACKING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if TRACKING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if TRACKING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[test]
fn ordinary_owned_value_operations_allocate_zero_times() {
    let transform = Mat4::from_translation(Vec3::new(4.0, -2.0, 1.0))
        * Mat4::from_rotation_y(0.7)
        * Mat4::from_scale(Vec3::new(1.5, 0.5, 2.0));
    let inverse = transform.try_inverse().expect("conditioned transform");
    let vectors = [
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(-4.0, 5.0, -6.0),
        Vec3::new(7.0, -8.0, 9.0),
    ];

    ALLOCATIONS.store(0, Ordering::Relaxed);
    TRACKING.store(true, Ordering::Relaxed);
    let mut checksum = 0.0;
    for _ in 0..512 {
        for vector in vectors {
            let normalized = vector.try_normalize().expect("finite nonzero vector");
            let transformed = transform.transform_point3(normalized);
            let restored = inverse.transform_point3(transformed);
            checksum += restored.dot(normalized) + transform.to_cols_array()[0];
        }
    }
    TRACKING.store(false, Ordering::Relaxed);

    assert!(checksum.is_finite());
    assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), 0);
}
