#![doc = include_str!("../Readme.md")]
#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(test)] extern crate alloc;

type AllocNN    = core::ptr::NonNull<core::mem::MaybeUninit<u8>>;
type AllocNN0   = core::ptr::NonNull<u8>;


#[doc(hidden)] pub mod _macros;

#[path = "auto/_auto.rs"]   mod auto;

mod align;                  pub use align::*;
mod layout;                 pub use layout::*;
pub mod nzst;
pub mod thin;
pub mod zsty;
