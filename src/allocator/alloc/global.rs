use crate::*;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



// XXX: Can't implement core::alloc::GlobalAlloc on alloc::alloc::Global
// #[cfg(    allocator_api = "1.50" )] pub use alloc::alloc::Global;
// #[cfg(not(allocator_api = "1.50"))]

/// Use <code>[alloc::alloc]::{[alloc](alloc::alloc::alloc), [alloc_zeroed](alloc::alloc::alloc_zeroed), [realloc](alloc::alloc::realloc), [dealloc](alloc::alloc::realloc)}</code>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Global;

impl meta::Meta for Global {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::MAX;
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = false; // XXX: Awkward for `core::alloc::Allocator`
}

// SAFETY: ✔️ all `impl fat::* for Global` are compatible with each other and return allocations compatible with their alignments
unsafe impl fat::Alloc for Global {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        if layout.size() == 0 { return Err(()) }
        // SAFETY: ✔️ we just ensured `layout` has a nonzero size
        let alloc = unsafe { alloc::alloc::alloc(layout) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        if layout.size() == 0 { return Err(()) }
        // SAFETY: ✔️ we just ensured `layout` has a nonzero size
        let alloc = unsafe { alloc::alloc::alloc_zeroed(layout) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all `impl fat::* for Global` are compatible with each other and return allocations compatible with their alignments
unsafe impl fat::Free for Global {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        if cfg!(debug_assertions) && layout.size() == 0 { bug::ub::invalid_zst_for_allocator(ptr) }
        // SAFETY: ✔ `ptr` should belong to `self` and `layout` should describe the allocation by documented fat::Free::free safety precondition
        unsafe { alloc::alloc::dealloc(ptr.as_ptr().cast(), layout) }
    }
}

// SAFETY: ✔️ all `impl fat::* for Global` are compatible with each other and return allocations compatible with their alignments
unsafe impl fat::Realloc for Global {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        if old_layout == new_layout {
            Ok(ptr)
        } else if old_layout.align() == new_layout.align() {
            if new_layout.size() == 0 { return Err(()); }
            // SAFETY: ✔️ we just ensured `new_layout` has a nonzero size
            // SAFETY: ✔️ `ptr` belongs to `self` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
            // SAFETY: ✔️ `ptr` is valid for `old_layout` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
            let alloc = unsafe { alloc::alloc::realloc(ptr.as_ptr().cast(), old_layout, new_layout.size()) };
            NonNull::new(alloc.cast()).ok_or(())
        } else { // alignment change
            if new_layout.size() == 0 { return Err(()); }
            // SAFETY: ✔️ we just ensured `new_layout` has a nonzero size
            let alloc = unsafe { alloc::alloc::alloc(new_layout) };
            let alloc : AllocNN = NonNull::new(alloc.cast()).ok_or(())?;
            {
                // SAFETY: ✔️ `ptr` is valid for `old_layout` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
                // SAFETY: ✔️ `alloc` was just allocated using `new_layout`
                #![allow(clippy::undocumented_unsafe_blocks)]

                let old : &    [MaybeUninit<u8>] = unsafe { util::slice::from_raw_bytes_layout    (ptr,   old_layout) };
                let new : &mut [MaybeUninit<u8>] = unsafe { util::slice::from_raw_bytes_layout_mut(alloc, new_layout) };
                let n = old.len().min(new.len());
                new[..n].copy_from_slice(&old[..n]);
            }
            // SAFETY: ✔ `ptr` should belong to `self`, and `old_layout` should describe the allocation, by `fat::Realloc::realloc_uninit`'s documented safety preconditions
            unsafe { alloc::alloc::dealloc(ptr.as_ptr().cast(), old_layout) };
            Ok(alloc)
        }
    }

    // realloc_uninit "could" be specialized to use alloc_zeroed on alloc realignment, but it's unclear if that'd be a perf gain (free zeroed memory pages) or perf loss (double zeroing)
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ alloc::alloc::*'s preconditions are documented in terms of GlobalAlloc's equivalents
unsafe impl core::alloc::GlobalAlloc for Global {
    unsafe fn alloc(&self, layout: Layout)                                  -> *mut u8  { unsafe { alloc::alloc::alloc(layout) } }
    unsafe fn alloc_zeroed(&self, layout: Layout)                           -> *mut u8  { unsafe { alloc::alloc::alloc_zeroed(layout) } }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)                              { unsafe { alloc::alloc::dealloc(ptr, layout) } }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8  { unsafe { alloc::alloc::realloc(ptr, layout, new_size) } }
}

#[cfg(never)] // SAFETY: ❌ NOT AT THIS TIME
//
//  • "Allocator is designed to be implemented on ZSTs, references, or smart pointers because having an allocator like MyAlloc([u8; N]) cannot be moved, without updating the pointers to the allocated memory."
//    ✔️ Trivial: `Global` is indeed a ZST containing none of the memory intended for allocation.
//
//  • "Unlike GlobalAlloc, zero-sized allocations are allowed in Allocator. If an underlying allocator does not support this (like jemalloc) or return a null pointer (such as libc::malloc), this must be caught by the implementation."
//    ❌ `ZST_SUPPORTED` is currently `false` to encourage users to explicitly chose `AllocZst` or `DangleZst`.
//        While `DangleZst` is the correct choice for `core::alloc::Allocator`, I'm unsure if I should do so implicitly.
//
//
// ## Currently allocated memory
//
//  • "Some of the methods require that a memory block be currently allocated via an allocator. This means that [...]"
//    ✔️ Trivial: All impl fns use `fat::*` traits, which impose equivalent allocation validity requirements.
//
//
// ## Memory fitting
//
// ⚠️ `fat::*`'s layout requirements are currently a bit stricter than core::alloc::Allocator 's:
// <https://doc.rust-lang.org/std/alloc/trait.Allocator.html#memory-fitting>
//
// Both interfaces require identical `layout.align()`ments.
// `fat::*` requires identical `layout.size()`s.
// `core::alloc::Allocator` allows a `layout.size()` in the range `min ..= max` where:
//  • `min` is the size of the layout most recently used to allocate the block, and
//  • `max` is the latest actual size returned from `allocate`, `grow`, or `shrink`.
//
// The implementation of this interface unifies requirements by ensuring `min` = `max`.
// That is, all interfaces return slices based *exactly* on `layout.size()`.
//
//
// ## Trait-level Safety Requirements:
//
//  • "Memory blocks returned from an allocator must point to valid memory and retain their validity until the instance and all of its copies and clones are dropped,"
//    ✔️ Trivial: all allocations remain valid until deallocated, even if all `Global`s are dropped.
//
//  • "copying, cloning, or moving the allocator must not invalidate memory blocks returned from this allocator. A copied or cloned allocator must behave like the same allocator, and"
//    ✔️ Trivial: `Global` contains none of the memory nor bookkeeping for memory blocks
//
//  • "any pointer to a memory block which is currently allocated may be passed to any other method of the allocator."
//    ✔️ Trivial: all fns use `fat::*` allocators exclusively, which are intended to be intercompatible.
//
#[cfg(allocator_api = "1.50")] unsafe impl core::alloc::Allocator for Global {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let data = fat::Alloc::alloc_uninit(self, layout).map_err(|_| core::alloc::AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), layout.size()))
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let data = fat::Alloc::alloc_zeroed(self, layout).map_err(|_| core::alloc::AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { fat::Free::free(self, ptr.cast(), layout) }
    }

    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let data = unsafe { fat::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| core::alloc::AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), new_layout.size()))
    }

    unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let data = unsafe { fat::Realloc::realloc_zeroed(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| core::alloc::AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), new_layout.size()))
    }

    unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let data = unsafe { fat::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| core::alloc::AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), new_layout.size()))
    }
}
