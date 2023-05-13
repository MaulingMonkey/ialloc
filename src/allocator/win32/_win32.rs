#![cfg(all(target_os = "windows", feature = "win32"))]
//! [`CoTaskMem`], [`CryptMem`], [`Global`], [`Heap`], [`ProcessHeap`], [`Local`]
//!
//! | Allocator                     | [`thin::Alloc`]       | [`thin::Realloc`]     | [`thin::Free`]    | [`thin::SizeOf`]      |
//! | ------------------------------| ----------------------| ----------------------| ------------------| ----------------------|
//! | [`CoTaskMem`]†                | [`CoTaskMemAlloc`]    | [`CoTaskMemRealloc`]  | [`CoTaskMemFree`] | ❌                    |
//! | [`CryptMem`]†                 | [`CryptMemAlloc`]     | [`CryptMemRealloc`]   | [`CryptMemFree`]  | ❌                    |
//! | [`Global`]†                   | [`GlobalAlloc`]       | [`GlobalReAlloc`]     | [`GlobalFree`]    | [`GlobalSize`]        |
//! | <code>[Heap]\(HANDLE\)</code> | [`HeapAlloc`]         | [`HeapReAlloc`]       | [`HeapFree`]      | [`HeapSize`]          |
//! | [`HeapNoSerialize`]           | [`HeapAlloc`]         | [`HeapReAlloc`]       | [`HeapFree`]      | [`HeapSize`]          |
//! | [`ProcessHeap`]               | [`HeapAlloc`]         | [`HeapReAlloc`]       | [`HeapFree`]      | [`HeapSize`]          |
//! | [`Local`]†                    | [`LocalAlloc`]        | [`LocalReAlloc`]      | [`LocalFree`]     | [`LocalSize`]         |
//! |
//! | (TODO)                        |
//! | `IMalloc`†                    | [`IMalloc::Alloc`]    | [`IMalloc::Realloc`]  | [`IMalloc::Free`] | [`IMalloc::GetSize`]  |
//! |
//! | (TODO)                        | [`fat::Alloc`]        | [`fat::Realloc`]      | [`fat::Free`]     | [`thin::SizeOf`]      |
//! | `Virtual(Commit?)`            | [`VirtualAlloc`]      | ❌                    | [`VirtualFree`]   | ❌                    |
//!
//! ## † Legacy Allocator Notes
//!
//! Many of these allocators are, these days, simply wrappers around [`Heap`] allocations - possibly with extra overhead.
//! I would generally recommend using [`Heap`] directly instead of these allocators, unless you have explicit reason to do otherwise, such as:
//! *   Microsoft documentation dictating a specific allocator to use when freeing memory.
//! *   Interoperability with third party code using a specific allocator you can't change.
//!
//! Microsoft makes similar recommendations:
//!
//! > The global and local functions are supported for porting from 16-bit code, or for maintaining source code compatibility with 16-bit Windows.
//! > Starting with 32-bit Windows, the global and local functions are implemented as wrapper functions that call the corresponding [heap functions] using a handle to the process's default heap.
//! > Therefore, the global and local functions have greater overhead than other memory management functions.
//! >
//! > The [heap functions] provide more features and control than the global and local functions.
//! > New applications should use the heap functions unless documentation specifically states that a global or local function should be used.
//! > For example, some Windows functions allocate memory that must be freed with [`LocalFree`], and the global functions are still used with Dynamic Data Exchange (DDE), the clipboard functions, and OLE data objects.
//! > For a complete list of global and local functions, see the table in [Memory Management Functions](https://learn.microsoft.com/en-us/windows/win32/memory/memory-management-functions).
//! >
//! > <https://learn.microsoft.com/en-us/windows/win32/memory/global-and-local-functions>
//!
//! As currently tested on my machine (this is of course subject to change!) - these win32 allocators are, at the time of writing this (May 12th, 2023), implemented in terms of:
//!
//! | Allocator             | Eventually implemented in terms of    |
//! | ----------------------| --------------------------------------|
//! | [`CoTaskMem`]         | `IMalloc` → [`Heap`]
//! | [`CryptMem`]          | [`Local`] w/ [`LMEM_ZEROINIT`] → [`Heap`] w/ [`HEAP_ZERO_MEMORY`]
//! | [`Global`]            | [`Heap`]
//! | [`Local`]             | [`Heap`]
//! | `IMalloc`             | [`Heap`]
//!
//! [heap functions]:   https://learn.microsoft.com/en-us/windows/win32/memory/heap-functions
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
