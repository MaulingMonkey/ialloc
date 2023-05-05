#![cfg(c89)]
//! [`Malloc`] (C89), [`AlignedMalloc`] (MSVC, C11, or C++17)
//!
//! | [`Malloc`] <br> Rust Traits               | C                     | MSVC<br>Only  |
//! | ------------------------------------------| ----------------------| --------------|
//! | [`thin::Alloc::alloc_uninit`]             | [`malloc`]            |               |
//! | [`thin::Alloc::alloc_zeroed`]             | [`calloc`]            |               |
//! | [`thin::Realloc::realloc_uninit`]         | [`realloc`]           |               |
//! | [`thin::Realloc::realloc_zeroed`]         | ❌ N/A               | [`_recalloc`] |
//! | [`thin::Free::free`]                      | [`free`]              |               |
//! | [`thin::SizeOfDebug::size_of`]            | `None`                | [`_msize`]    |
//!
//! | [`AlignedMalloc`] <br> Rust Traits    | MSVC Release CRT <br> ~~MSVC Debug CRT~~                                                                                              | !MSVC<br>C11 or C++17     |
//! | --------------------------------------| --------------------------------------------------------------------------------------------------------------------------------------| --------------------------|
//! | [`nzst::Alloc::alloc_uninit`]         | <code>[_aligned_malloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-malloc-dbg)}</code>     | [`aligned_alloc`]
//! | [`nzst::Alloc::alloc_zeroed`]         | <code>[_aligned_recalloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-recalloc-dbg)}</code> | &emsp;&emsp;+ [`memset`]
//! | [`nzst::Realloc::realloc_uninit`]     | <code>[_aligned_realloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-realloc-dbg)}</code>   | [`realloc`] or [`aligned_alloc`] + [`memcpy`]
//! | [`nzst::Realloc::realloc_zeroed`]     | <code>[_aligned_recalloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-recalloc-dbg)}</code> | &emsp;&emsp;+ [`memset`]
//! | [`nzst::Free::free`]                  | <code>[_aligned_free]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-free-dbg)}</code>         | [`free`] or [`free_aligned_sized`] (C23)
//! | [`thin::Free::free`]                  | <code>[_aligned_free]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-free-dbg)}</code>         | [`free`]
//!
#![doc = include_str!("_refs.md")]

#[cfg(doc)] use crate::*;

#[cfg(c89)] mod malloc;
#[cfg(c89)] pub use malloc::Malloc;

#[cfg(any(msvc, c11, cpp17))]   mod aligned_malloc;
#[cfg(any(msvc, c11, cpp17))]   pub use aligned_malloc::*;
