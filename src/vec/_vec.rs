//! [`alloc::vec::Vec`] alternatives

mod avec;                   pub use avec::*;
mod avec_clone;
mod avec_cmp;
mod avec_default;
mod avec_deref;
mod avec_extend;
mod avec_fmt;
mod avec_from;
mod avec_index;
mod avec_io;
mod avec_iter;              pub use avec_iter::*;
mod avec_retain;
