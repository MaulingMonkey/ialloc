#![cfg(all(target_os = "windows", feature = "win32"))]
//! [`CoTaskMem`], [`CryptMem`], [`Global`], [`Heap`], [`ProcessHeap`], [`Local`]
//!
//! | Allocator                     | [`thin::Alloc`]       | [`thin::Realloc`]     | [`thin::Free`]    | [`thin::SizeOf`]      |
//! | ------------------------------| ----------------------| ----------------------| ------------------| ----------------------|
//! | [`CoTaskMem`]                 | [`CoTaskMemAlloc`]    | [`CoTaskMemRealloc`]  | [`CoTaskMemFree`] | ❌                    |
//! | [`CryptMem`]                  | [`CryptMemAlloc`]     | [`CryptMemRealloc`]   | [`CryptMemFree`]  | ❌                    |
//! | [`Global`]                    | [`GlobalAlloc`]       | [`GlobalReAlloc`]     | [`GlobalFree`]    | [`GlobalSize`]        |
//! | <code>[Heap]\(HANDLE\)</code> | [`HeapAlloc`]         | [`HeapReAlloc`]       | [`HeapFree`]      | [`HeapSize`]          |
//! | [`HeapNoSerialize`]           | [`HeapAlloc`]         | [`HeapReAlloc`]       | [`HeapFree`]      | [`HeapSize`]          |
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
mod heap_no_serialize;  pub use heap_no_serialize::*;
mod local;              pub use local::*;

/// | Arch      | Value |
/// | ----------| -----:|
/// | i686      |  8    |
/// | x86_64    | 16    |
const MEMORY_ALLOCATION_ALIGNMENT : crate::Alignment = crate::Alignment::constant(winapi::um::winnt::MEMORY_ALLOCATION_ALIGNMENT);

/// <code>[SetLastError](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setlasterror)\(0\)</code>
fn clear_last_error() {
    // SAFETY: ✔️ if writing this TLS var is ever unsafe, something has gone *horrifically* wrong.
    unsafe { winapi::um::errhandlingapi::SetLastError(0) };
}

/// [`GetLastError`](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
fn get_last_error() -> u32 {
    // SAFETY: ✔️ if accessing this TLS var is ever unsafe, something has gone *horrifically* wrong.
    unsafe { winapi::um::errhandlingapi::GetLastError() }
}
