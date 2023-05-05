#![cfg_attr(allocator_api = "unstable", feature(allocator_api))]
#![doc = include_str!("../Readme.md")]
#![doc = include_str!("../doc/features.md")]
#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(any(feature = "alloc", doc, test))] extern crate alloc;

type AllocNN    = core::ptr::NonNull<core::mem::MaybeUninit<u8>>;
type AllocNN0   = core::ptr::NonNull<u8>;


#[doc(hidden)] pub mod _impls;

#[path = "auto/_auto.rs"]   mod auto;

/// Allocator implementations
pub mod allocator {
    #[path = "adapt/_adapt.rs"  ] pub mod adapt;
    #[path = "alloc/_alloc.rs"  ] pub mod alloc;
    #[path = "c/_c.rs"          ] pub mod c;
    #[path = "cpp/_cpp.rs"      ] pub mod cpp;
    #[path = "win32/_win32.rs"  ] pub mod win32;
    #[path = "msvc/_msvc.rs"    ] pub mod msvc;
}

/// Allocator traits
pub mod traits {
    pub mod nzst;
    pub mod thin;
    pub mod zsty;
}
#[doc(hidden)] pub use traits::*;

mod align;                  pub use align::*;
mod layout;                 pub use layout::*;
