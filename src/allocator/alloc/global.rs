use crate::*;
use crate::meta::*;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



// XXX: Can't implement core::alloc::GlobalAlloc on alloc::alloc::Global
// #[cfg(    allocator_api = "1.50" )] pub use alloc::alloc::Global;
// #[cfg(not(allocator_api = "1.50"))]

/// Use <code>[alloc::alloc]::{[alloc](alloc::alloc::alloc), [alloc_zeroed](alloc::alloc::alloc_zeroed), [realloc](alloc::alloc::realloc), [dealloc](alloc::alloc::realloc)}</code>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Global;

#[cfg(feature = "alloc")] #[cfg(allocator_api = "*")] impl From<Global> for alloc::alloc::Global { fn from(_: Global) -> Self { Self } }
#[cfg(feature = "alloc")] #[cfg(allocator_api = "*")] impl From<alloc::alloc::Global> for Global { fn from(_: alloc::alloc::Global) -> Self { Self } }



// meta::*

impl Meta for Global {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::MAX;
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

impl ZstSupported for Global {}

// SAFETY: ✔️ simply returns dangling pointers when size is zero
unsafe impl ZstInfalliable for Global {}

// SAFETY: ✔️ global state only
unsafe impl Stateless for Global {}



// fat::*

// SAFETY: ✔️ all `impl fat::* for Global` are compatible with each other and return allocations compatible with their alignments
unsafe impl fat::Alloc for Global {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        match layout.size() {
            0                       => Ok(util::nn::dangling(layout)),
            n if n > Self::MAX_SIZE => Err(()),
            _ => {
                debug_assert!(layout.pad_to_align().size() <= Self::MAX_SIZE, "bug: undefined behavior: Layout when padded to alignment exceeds isize::MAX, which violates Layout's invariants");
                // SAFETY: ✔️ we just ensured `layout` has a valid (nonzero, <= isize::MAX) size
                let alloc = unsafe { alloc::alloc::alloc(layout) };
                NonNull::new(alloc.cast()).ok_or(())
            }
        }
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        match layout.size() {
            0                       => Ok(util::nn::dangling(layout)),
            n if n > Self::MAX_SIZE => Err(()),
            _ => {
                debug_assert!(layout.pad_to_align().size() <= Self::MAX_SIZE, "bug: undefined behavior: Layout when padded to alignment exceeds isize::MAX, which violates Layout's invariants");
                // SAFETY: ✔️ we just ensured `layout` has a nonzero size
                let alloc = unsafe { alloc::alloc::alloc_zeroed(layout) };
                NonNull::new(alloc.cast()).ok_or(())
            }
        }
    }
}

// SAFETY: ✔️ all `impl fat::* for Global` are compatible with each other and return allocations compatible with their alignments
unsafe impl fat::Free for Global {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        if layout.size() == 0 { return }
        // SAFETY: ✔️ `ptr` belongs to `self` and `layout` describes the allocation per [`fat::Free::free`]'s documented safety preconditions
        unsafe { alloc::alloc::dealloc(ptr.as_ptr().cast(), layout) }
    }
}

// SAFETY: ✔️ all `impl fat::* for Global` are compatible with each other and return allocations compatible with their alignments
unsafe impl fat::Realloc for Global {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        if new_layout.size() > Self::MAX_SIZE {
            Err(())
        } else if old_layout == new_layout {
            Ok(ptr)
        } else if old_layout.align() != new_layout.align() || old_layout.size() == 0 || new_layout.size() == 0 {
            let alloc = fat::Alloc::alloc_uninit(self, new_layout)?;
            {
                // SAFETY: ✔️ `ptr` is valid for `old_layout` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
                // SAFETY: ✔️ `alloc` was just allocated using `new_layout`
                #![allow(clippy::undocumented_unsafe_blocks)]

                let old : &    [MaybeUninit<u8>] = unsafe { util::slice::from_raw_bytes_layout    (ptr,   old_layout) };
                let new : &mut [MaybeUninit<u8>] = unsafe { util::slice::from_raw_bytes_layout_mut(alloc, new_layout) };
                let n = old.len().min(new.len());
                new[..n].copy_from_slice(&old[..n]);
            }
            // SAFETY: ✔️ `ptr` belongs to `self`, and `old_layout` should describe the allocation, by `fat::Realloc::realloc_uninit`'s documented safety preconditions
            unsafe { fat::Free::free(self, ptr, old_layout) };
            Ok(alloc)
        } else {
            // SAFETY: ✔️ layouts have same alignments
            // SAFETY: ✔️ layouts have nonzero sizes
            // SAFETY: ✔️ `ptr` belongs to `self` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
            // SAFETY: ✔️ `ptr` is valid for `old_layout` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
            // SAFETY: ✔️ `new_layout.size()` was bounds checked at start of fn
            let alloc = unsafe { alloc::alloc::realloc(ptr.as_ptr().cast(), old_layout, new_layout.size()) };
            NonNull::new(alloc.cast()).ok_or(())
        }
    }

    // realloc_uninit "could" be specialized to use alloc_zeroed on alloc realignment, but it's unclear if that'd be a perf gain (free zeroed memory pages) or perf loss (double zeroing)
}



// core::alloc::*

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ alloc::alloc::*'s preconditions are documented in terms of GlobalAlloc's equivalents
unsafe impl core::alloc::GlobalAlloc for Global {
    unsafe fn alloc(&self, layout: Layout)                                  -> *mut u8  { unsafe { alloc::alloc::alloc(layout) } }
    unsafe fn alloc_zeroed(&self, layout: Layout)                           -> *mut u8  { unsafe { alloc::alloc::alloc_zeroed(layout) } }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)                              { unsafe { alloc::alloc::dealloc(ptr, layout) } }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8  { unsafe { alloc::alloc::realloc(ptr, layout, new_size) } }
}

//  • "Allocator is designed to be implemented on ZSTs, references, or smart pointers because having an allocator like MyAlloc([u8; N]) cannot be moved, without updating the pointers to the allocated memory."
//    ✔️ Trivial: `Global` is indeed a ZST containing none of the memory intended for allocation.
//
//  • "Unlike GlobalAlloc, zero-sized allocations are allowed in Allocator. If an underlying allocator does not support this (like jemalloc) or return a null pointer (such as libc::malloc), this must be caught by the implementation."
//    ✔️ `ZST_SUPPORTED` is currently `true`, matching behavior of `alloc::alloc::Global`.
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



#[cfg(test)] const GLOBAL_ALLOC_ZERO_INITS : bool = cfg!(any(
    target_os = "linux",    // from the start of `ialloc` on CI and WSL
    target_os = "macos",    // github's `macos-11` runners didn't zero init, but `macos-14`(? via `macos-latest`) does.
));

#[test] fn fat_alignment()          { fat::test::alignment(Global) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(Global) }
#[test] fn fat_uninit()             { if !GLOBAL_ALLOC_ZERO_INITS { unsafe { fat::test::uninit_alloc_unsound(Global) } } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(Global) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(Global) }
