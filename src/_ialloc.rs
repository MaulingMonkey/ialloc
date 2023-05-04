#![doc = include_str!("../Readme.md")]
#![doc = include_str!("../doc/features.md")]
#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(any(feature = "alloc", doc, test))] extern crate alloc;

type AllocNN    = core::ptr::NonNull<core::mem::MaybeUninit<u8>>;
type AllocNN0   = core::ptr::NonNull<u8>;


#[doc(hidden)] pub mod _macros;

#[path = "auto/_auto.rs"]   mod auto;

/// Allocator implementations
pub mod allocator {
    #[path = "alloc/_alloc.rs"  ] pub mod alloc;
    #[path = "c/_c.rs"          ] pub mod c;
    #[path = "win32/_win32.rs"  ] pub mod win32;
    #[path = "msvc/_msvc.rs"    ] pub mod msvc;
}

mod align;                  pub use align::*;
mod layout;                 pub use layout::*;
pub mod nzst;
pub mod thin;
pub mod zsty;
