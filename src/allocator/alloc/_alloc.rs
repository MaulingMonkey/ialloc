//! [`Global`]
#![cfg(feature = "alloc")]

use crate::*;

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// Use <code>[alloc::alloc]::{[alloc](alloc::alloc::alloc), [alloc_zeroed](alloc::alloc::alloc_zeroed), [realloc](alloc::alloc::realloc), [dealloc](alloc::alloc::realloc)}</code>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Global;

unsafe impl nzst::Alloc for Global {
    type Error = ();

    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { alloc::alloc::alloc(*layout) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, layout: LayoutNZ) -> Result<AllocNN0, Self::Error> {
        let alloc = unsafe { alloc::alloc::alloc_zeroed(*layout) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl nzst::Free for Global {
    unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) {
        unsafe { alloc::alloc::dealloc(ptr.as_ptr().cast(), *layout) }
    }
}

unsafe impl nzst::Realloc for Global {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        if old_layout == new_layout {
            Ok(ptr)
        } else if old_layout.align() == new_layout.align() {
            let alloc = unsafe { alloc::alloc::realloc(ptr.as_ptr().cast(), *old_layout, new_layout.size().get()) };
            NonNull::new(alloc.cast()).ok_or(())
        } else { // alignment change
            let alloc = unsafe { alloc::alloc::alloc(*new_layout) };
            let alloc : AllocNN = NonNull::new(alloc.cast()).ok_or(())?;
            {
                let old : &    [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts    (ptr.as_ptr().cast(), old_layout.size().get()) };
                let new : &mut [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(),      new_layout.size().get()) };
                let n = old.len().min(new.len());
                new[..n].copy_from_slice(&old[..n]);
            }
            unsafe { alloc::alloc::dealloc(ptr.as_ptr().cast(), *old_layout) };
            Ok(alloc)
        }
    }

    // realloc_uninit "could" be specialized to use alloc_zeroed on alloc realignment, but it's unclear if that'd be a perf gain (free zeroed memory pages) or perf loss (double zeroing)
}

unsafe impl core::alloc::GlobalAlloc for Global {
    unsafe fn alloc(&self, layout: core::alloc::Layout)                                     -> *mut u8  { unsafe { alloc::alloc::alloc(layout) } }
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout)                              -> *mut u8  { unsafe { alloc::alloc::alloc_zeroed(layout) } }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout)                                 { unsafe { alloc::alloc::dealloc(ptr, layout) } }
    unsafe fn realloc(&self, ptr: *mut u8, layout: core::alloc::Layout, new_size: usize)    -> *mut u8  { unsafe { alloc::alloc::realloc(ptr, layout, new_size) } }
}

// unsafe impl core::alloc::Allocator for Global {
//    ...
// }
