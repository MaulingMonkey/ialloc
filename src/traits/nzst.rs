//! Rusty !&zwj;[ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts) allocator traits operating on [`LayoutNZ`]s
//!
//! Zero sized allocations are embraced by Rust yet villified by C and C++.  Exact C++ allocator behavior varies wildly, and can include:
//! *   `alloc(size: 0)` fails to allocate, and...
//!     *   returns nullptr
//!     *   sets an error code
//!     *   throws an exception
//!     *   crashes horribly
//!     *   invokes other horrific undefined behavior
//! *   `alloc(size: 0)` may succeed, and...
//!     *   return a non-unique address to non-dereferencable memory (common in Rust)
//!     *   allocate a unique address to 1+ bytes of memory (not entirely uncommon in C++?)
//!     *   allocate a unique address to non-dereferencable memory (rare but I've heard of it - could involve bitsets...)
//! *   `realloc(ptr: nullptr, ...)` may behave like `alloc(...)`, or may fail even when `alloc` wouldn't.
//! *   `realloc(size: 0? → 0?)` I don't even want to try to imagine!
//!
//! Determining the exact edge cases of every system allocator is difficult.
//! Even worse, the edge cases could change, especially with customizable allocators.
//! This module aims to apply the [Alexander's solution to the Gordian Knot](https://en.wikipedia.org/wiki/Gordian_Knot):
//! Zero sized allocations are simply forbidden by type - all layouts are [`LayoutNZ`]s, all sizes are [`NonZeroUsize`]s.
//!
//! This greatly simplifies the *implementation* of these traits, although direct *consumption* is awkward and discouraged.
//! <code>[zsty]::{[Alloc](zsty::Alloc), [Free](zsty::Free), ...}</code> is auto-implemented in terms of these traits, and provides a zero-alloc friendly interface!

use crate::*;

use core::mem::MaybeUninit;
#[cfg(doc)] use core::num::NonZeroUsize;
#[cfg(doc)] use core::ptr::NonNull;



/// Allocation functions:<br>
/// <code>[alloc_uninit](Self::alloc_uninit)(layout: [LayoutNZ]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[alloc_zeroed](Self::alloc_zeroed)(layout: [LayoutNZ]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
pub unsafe trait Alloc {
    type Error;

    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN, Self::Error>;

    fn alloc_zeroed(&self, layout: LayoutNZ) -> Result<AllocNN0, Self::Error> {
        let alloc = self.alloc_uninit(layout)?;
        unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), layout.size().get()) }.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}

/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]&lt;\_&gt;, layout: [LayoutNZ])</code><br>
/// <br>
pub unsafe trait Free {
    unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ);
}

/// Reallocation function:<br>
/// <code>[realloc_uninit](Self::realloc_uninit)(ptr: [NonNull]&lt;\_&gt;, old: [LayoutNZ], new: [LayoutNZ]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[realloc_zeroed](Self::realloc_zeroed)(ptr: [NonNull]&lt;\_&gt;, old: [LayoutNZ], new: [LayoutNZ]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
pub unsafe trait Realloc : Alloc + Free {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        if old_layout == new_layout { return Ok(ptr) }
        let alloc = self.alloc_uninit(new_layout)?;
        {
            let old : &    [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts    (ptr.as_ptr().cast(), old_layout.size().get()) };
            let new : &mut [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(),      new_layout.size().get()) };
            let n = old.len().min(new.len());
            new[..n].copy_from_slice(&old[..n]);
        }
        unsafe { self.free(ptr, old_layout) };
        Ok(alloc)
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { self.realloc_uninit(ptr, old_layout, new_layout) }?;
        if old_layout.size() < new_layout.size() {
            let all             = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), new_layout.size().get()) };
            let (_copied, new)  = all.split_at_mut(old_layout.size().get());
            new.fill(MaybeUninit::new(0u8));
        }
        Ok(alloc.cast())
    }
}
