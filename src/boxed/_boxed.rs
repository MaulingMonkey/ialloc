//! [`alloc::boxed::Box`] alternatives

mod abox;                   pub use abox::*;
mod abox_alloc;
mod abox_bytemuck;
mod abox_clone;
mod abox_cmp;
mod abox_convert_infallible_;
mod abox_convert_infallible_alloc;
mod abox_convert_infallible_std;
mod abox_default;
mod abox_fmt;
mod abox_io;
mod abox_iter;
mod abox_realloc;
mod abox_traits;
