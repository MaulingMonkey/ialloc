#![cfg(all(target_env = "msvc", feature = "msvc"))]
//! [`AlignedMalloc`]
//!
//! | Rust                              | C (Release CRT)       | ~~Debug CRT~~ (N/A)       |
//! | ----------------------------------| ----------------------| --------------------------|
//! | **[`AlignedMalloc`]**             |                       |                           |
//! | [`nzst::Alloc::alloc_uninit`]     | [`_aligned_malloc`]   | [`_aligned_malloc_dbg`]   |
//! | [`nzst::Alloc::alloc_zeroed`]     | [`_aligned_recalloc`] | [`_aligned_recalloc_dbg`] |
//! | [`nzst::Realloc::realloc_uninit`] | [`_aligned_realloc`]  | [`_aligned_realloc_dbg`]  |
//! | [`nzst::Realloc::realloc_zeroed`] | [`_aligned_recalloc`] | [`_aligned_recalloc_dbg`] |
//! | [`nzst::Free::free`]              | [`_aligned_free`]     | [`_aligned_free_dbg`]     |
//! | [`thin::Free::free`]              | [`_aligned_free`]     | [`_aligned_free_dbg`]     |
//!
#![doc = include_str!("_refs.md")]

#[cfg(doc)] use crate::*;

mod aligned_malloc;     pub use aligned_malloc::*;
