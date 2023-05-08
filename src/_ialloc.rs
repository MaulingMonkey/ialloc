#![cfg_attr(allocator_api = "unstable", feature(allocator_api))]
#![doc = include_str!("../Readme.md")]
#![doc = include_str!("../doc/features.md")]
#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(any(feature = "alloc", doc, test))] extern crate alloc;
#[cfg(any(feature = "std",   doc, test))] extern crate std;

type AllocNN    = core::ptr::NonNull<core::mem::MaybeUninit<u8>>;
type AllocNN0   = core::ptr::NonNull<u8>;


#[doc(hidden)] pub mod _impls;

pub use align::alignment::*;
#[doc(hidden)] pub use align::constants::*;
mod align {
    pub mod alignment;
    pub mod constants;
}

#[path = "allocator/_allocator.rs"      ] pub mod allocator;
#[path = "boxed/_boxed.rs"              ] pub mod boxed;
#[path = "traits/_traits.rs"            ] pub mod traits; #[doc(hidden)] pub use traits::*;
#[path = "util/_util.rs"                ] mod util;

pub mod error;
mod layout;                 pub use layout::*;
