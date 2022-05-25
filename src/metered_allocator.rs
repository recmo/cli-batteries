#![cfg(feature = "metered_allocator")]

use core::sync::atomic::{AtomicBool, Ordering};
use once_cell::sync::Lazy;
use prometheus::{
    exponential_buckets, register_histogram, register_int_counter, Histogram, IntCounter,
};
use std::alloc::{GlobalAlloc, Layout};

pub use std::alloc::System as StdAlloc;

static ALLOCATED: Lazy<IntCounter> =
    Lazy::new(|| register_int_counter!("mem_alloc", "Cumulative memory allocated.").unwrap());
static FREED: Lazy<IntCounter> =
    Lazy::new(|| register_int_counter!("mem_free", "Cumulative memory freed.").unwrap());
static SIZE: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "mem_alloc_size",
        "Distribution of allocation sizes.",
        exponential_buckets(16.0, 4.0, 10).unwrap()
    )
    .unwrap()
});

pub struct MeteredAllocator<T: GlobalAlloc> {
    inner:    T,
    metering: AtomicBool,
}

impl<T: GlobalAlloc> MeteredAllocator<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            metering: AtomicBool::new(false),
        }
    }

    pub fn start_metering(&self) {
        if self.metering.load(Ordering::Acquire) {
            return;
        }
        Lazy::force(&ALLOCATED);
        Lazy::force(&SIZE);
        Lazy::force(&FREED);
        self.metering.store(true, Ordering::Release);
    }

    fn count_alloc(&self, size: usize) {
        // Avoid re-entrancy here when metrics are first initialized.
        if self.metering.load(Ordering::Acquire) {
            ALLOCATED.inc_by(size as u64);
            #[allow(clippy::cast_precision_loss)]
            SIZE.observe(size as f64);
        }
    }

    fn count_dealloc(&self, size: usize) {
        if self.metering.load(Ordering::Acquire) {
            FREED.inc_by(size as u64);
        }
    }
}

// GlobalAlloc is an unsafe trait for allocators
#[allow(unsafe_code)]
unsafe impl<T: GlobalAlloc> GlobalAlloc for MeteredAllocator<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.count_alloc(layout.size());
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.count_dealloc(layout.size());
        self.inner.dealloc(ptr, layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.count_alloc(layout.size());
        self.inner.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = layout.size();
        if new_size >= old_size {
            self.count_alloc(new_size - old_size);
        } else {
            self.count_dealloc(old_size - new_size);
        }
        self.inner.realloc(ptr, layout, new_size)
    }
}
