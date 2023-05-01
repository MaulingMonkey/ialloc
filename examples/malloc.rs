#![cfg_attr(feature = "nightly", feature(allocator_api  ))] // Allocator
#![cfg_attr(feature = "nightly", feature(const_option   ))] // const unwrap()
#![cfg_attr(feature = "nightly", feature(int_roundings  ))] // next_multiple_of

#[cfg(feature = "nightly")] include!("nightly/malloc.rs");
include!("nightly/stable-main-stub.rs");
