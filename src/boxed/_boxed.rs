//! [`alloc::boxed::Box`] alternatives

mod abox;                   pub use abox::*;
mod abox_alloc;
mod abox_bytemuck;
mod abox_traits_panicy;
mod abox_traits;