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
//! | (TODO)                        | [`nzst::Alloc`]       | [`nzst::Realloc`]     | [`nzst::Free`]    | [`thin::SizeOf`]      |
//! | `Virtual(Commit?)`            | [`VirtualAlloc`]      | ❌                    | [`VirtualFree`]   | ❌                    |
//!
#![doc = include_str!("_refs.md")]

#[cfg(doc)] use crate::*;

mod cotaskmem;          pub use cotaskmem::*;
mod cryptmem;           pub use cryptmem::*;
mod global;             pub use global::*;
mod heap;               pub use heap::*;
mod local;              pub use local::*;



#[inline(always)] fn check_size<N: TryFrom<usize>>(size: core::num::NonZeroUsize) -> Result<N, ()> {
    let size = size.get();

    // XXX: not entirely sure if this is excessive or not.
    // Will *Alloc(2.5 GiB) reasonably succeed?
    // My understanding is that >isize::MAX allocs/pointer ranges/spatial provenances are super cursed by LLVM / the compiler
    // https://doc.rust-lang.org/core/alloc/struct.Layout.html#method.from_size_align
    // https://doc.rust-lang.org/core/primitive.pointer.html#method.add
    if size > usize::MAX/2 { return Err(()) }

    N::try_from(size).map_err(|_| {})
}