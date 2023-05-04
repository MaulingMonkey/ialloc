#![cfg(feature = "c")]
//! [`Malloc`] (C89)
//!
//! | Rust                                      | C                     | MSVC<br>Only  |
//! | ------------------------------------------| ----------------------| --------------|
//! | [`thin::Alloc::alloc_uninit`]             | [`malloc`]            |               |
//! | [`thin::Alloc::alloc_zeroed`]             | [`calloc`]            |               |
//! | [`thin::Realloc::realloc_uninit`]         | [`realloc`]           |               |
//! | [`thin::ReallocZeroed::realloc_zeroed`]   | ❌ N/A               | [`_recalloc`] |
//! | [`thin::Free::free`]                      | [`free`]              |               |
//! | [`thin::SizeOfDebug::size_of`]            | `None`                | [`_msize`]    |
//!
#![doc = include_str!("_refs.md")]

#[cfg(doc)] use crate::*;

#[cfg(feature = "c89")] mod malloc;
#[cfg(feature = "c89")] pub use malloc::Malloc;
