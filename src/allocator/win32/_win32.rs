#![cfg(all(target_os = "windows", feature = "win32"))]
//! [`CoTaskMem`], [`CryptMem`], [`Global`], [`Heap`], [`ProcessHeap`], [`Local`]
//!
//! | Allocator                     | [`thin::Alloc`]       | [`thin::Realloc`]     | [`thin::Free`]    | [`thin::SizeOf`]      |
//! | ------------------------------| ----------------------| ----------------------| ------------------| ----------------------|
//! | [`CoTaskMem`]                 | [`CoTaskMemAlloc`]    | [`CoTaskMemRealloc`]  | [`CoTaskMemFree`] | ❌                    |
//! | [`CryptMem`]                  | [`CryptMemAlloc`]     | [`CryptMemRealloc`]   | [`CryptMemFree`]  | ❌                    |
//! | [`Global`]                    | [`GlobalAlloc`]       | [`GlobalReAlloc`]     | [`GlobalFree`]    | [`GlobalSize`]        |
//! | <code>[Heap]\(HANDLE\)</code> | [`HeapAlloc`]         | [`HeapReAlloc`]       | [`HeapFree`]      | [`HeapSize`]          |
//! | [`ProcessHeap`]               | [`HeapAlloc`]         | [`HeapReAlloc`]       | [`HeapFree`]      | [`HeapSize`]          |
//! | [`Local`]                     | [`LocalAlloc`]        | [`LocalReAlloc`]      | [`LocalFree`]     | [`LocalSize`]         |
//! |
//! | (TODO)                        |
//! | `IMalloc`                     | [`IMalloc::Alloc`]    | [`IMalloc::Realloc`]  | [`IMalloc::Free`] | [`IMalloc::GetSize`]  |
//! |
//! | (TODO)                        | [`fat::Alloc`]        | [`fat::Realloc`]      | [`fat::Free`]     | [`thin::SizeOf`]      |
//! | `Virtual(Commit?)`            | [`VirtualAlloc`]      | ❌                    | [`VirtualFree`]   | ❌                    |
//!
#![doc = include_str!("_refs.md")]

#[cfg(doc)] use crate::*;

mod cotaskmem;          pub use cotaskmem::*;
mod cryptmem;           pub use cryptmem::*;
mod global;             pub use global::*;
mod heap;               pub use heap::*;
mod local;              pub use local::*;

/// | Arch      | Value |
/// | ----------| -----:|
/// | i686      |  8    |
/// | x86_64    | 16    |
const MEMORY_ALLOCATION_ALIGNMENT : crate::Alignment = crate::Alignment::constant(winapi::um::winnt::MEMORY_ALLOCATION_ALIGNMENT);
