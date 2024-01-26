#![cfg_attr(allocator_api = "unstable", feature(allocator_api))]
#![doc = include_str!("../Readme.md")]
#![no_std]

#![forbid(unreachable_patterns)] // often indicates e.g. a typoed "constant" in a match statement
#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(non_snake_case)] // often indicates e.g. a typoed "constant" in a match statement
#![warn(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::let_unit_value)] // very common for const assertions
#![cfg_attr(not(feature = "default"), allow(dead_code, unused_imports))] // suppress noisy "dead code" warnings in non-default build configs

#[cfg(any(feature = "alloc", doc, test))] extern crate alloc;
#[cfg(any(feature = "std",   doc, test))] extern crate std;

type AllocNN    = core::ptr::NonNull<core::mem::MaybeUninit<u8>>;
type AllocNN0   = core::ptr::NonNull<u8>;


#[macro_use] mod _macros;
#[doc(hidden)] pub mod _impls;

pub use align::alignment::*;
pub(crate) use align::alignn::AlignN;
#[doc(hidden)] pub use align::constants::*;
mod align {
    pub mod alignment;
    pub mod alignn;
    pub mod constants;
}

#[cfg(doc)] #[doc = include_str!("../doc/assumptions.md")] pub mod _assumptions {}
#[cfg(doc)] #[doc = include_str!("../doc/features.md")] pub mod _features {}

#[path = "allocator/_allocator.rs"      ] pub mod allocator;
#[path = "boxed/_boxed.rs"              ] pub mod boxed;
#[path = "traits/_traits.rs"            ] pub mod traits; #[doc(hidden)] pub use traits::*;
#[path = "util/_util.rs"                ] mod util;
#[path = "vec/_vec.rs"                  ] pub mod vec;

#[doc(hidden)] pub mod bug;
pub mod error;
