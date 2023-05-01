#![doc = include_str!("../Readme.md")]
#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

mod align;                  pub use align::*;
mod layout;                 pub use layout::*;
