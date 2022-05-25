#[cfg(feature = "metered_allocator")]
use crate::MeteredAllocator;

#[cfg(feature = "mimalloc")]
pub use ::mimalloc::MiMalloc;

#[cfg(all(not(feature = "mimalloc"), feature = "metered_allocator"))]
use std::alloc::System;

#[cfg(all(not(feature = "mimalloc"), feature = "metered_allocator"))]
#[global_allocator]
pub static ALLOCATOR: MeteredAllocator<System> = MeteredAllocator::new(System);

#[cfg(all(feature = "mimalloc", not(feature = "metered_allocator")))]
#[global_allocator]
pub static ALLOCATOR: MiMalloc = MiMalloc;

#[cfg(all(feature = "mimalloc", feature = "metered_allocator"))]
#[global_allocator]
pub static ALLOCATOR: MeteredAllocator<MiMalloc> = MeteredAllocator::new(MiMalloc);

pub fn start_metering() {
    #[cfg(feature = "metered_allocator")]
    {
        ALLOCATOR.start_metering();
    }
}
