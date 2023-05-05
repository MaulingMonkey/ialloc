#![cfg_attr(feature = "nightly", feature(allocator_api  ))] // Allocator

#[cfg(feature = "nightly")] include!("nightly/malloc.rs");
include!("nightly/stable-main-stub.rs");
