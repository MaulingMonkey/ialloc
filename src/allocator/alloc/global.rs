use crate::*;

#[cfg(allocator_api = "1.50")] use core::alloc::AllocError;
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
    const ZST_SUPPORTED : bool  = false;
}

unsafe impl fat::Alloc for Global {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        if layout.size() == 0 { return Err(()) }
        let alloc = unsafe { alloc::alloc::alloc(layout) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        if layout.size() == 0 { return Err(()) }
        let alloc = unsafe { alloc::alloc::alloc_zeroed(layout) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl fat::Free for Global {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        if cfg!(debug_assertions) && layout.size() == 0 { bug::ub::invalid_zst_for_allocator(ptr) }
        unsafe { alloc::alloc::dealloc(ptr.as_ptr().cast(), layout) }
    }
}

unsafe impl fat::Realloc for Global {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        if old_layout == new_layout {
            Ok(ptr)
        } else if old_layout.align() == new_layout.align() {
            if new_layout.size() == 0 { return Err(()); }
            let alloc = unsafe { alloc::alloc::realloc(ptr.as_ptr().cast(), old_layout, new_layout.size()) };
            NonNull::new(alloc.cast()).ok_or(())
        } else { // alignment change
            if new_layout.size() == 0 { return Err(()); }
            let alloc = unsafe { alloc::alloc::alloc(new_layout) };
            let alloc : AllocNN = NonNull::new(alloc.cast()).ok_or(())?;
            {
                let old : &    [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts    (ptr.as_ptr().cast(), old_layout.size()) };
                let new : &mut [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(),      new_layout.size()) };
                let n = old.len().min(new.len());
                new[..n].copy_from_slice(&old[..n]);
            }
            unsafe { alloc::alloc::dealloc(ptr.as_ptr().cast(), old_layout) };
            Ok(alloc)
        }
    }

    // realloc_uninit "could" be specialized to use alloc_zeroed on alloc realignment, but it's unclear if that'd be a perf gain (free zeroed memory pages) or perf loss (double zeroing)
}

unsafe impl core::alloc::GlobalAlloc for Global {
    unsafe fn alloc(&self, layout: Layout)                                  -> *mut u8  { unsafe { alloc::alloc::alloc(layout) } }
    unsafe fn alloc_zeroed(&self, layout: Layout)                           -> *mut u8  { unsafe { alloc::alloc::alloc_zeroed(layout) } }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)                              { unsafe { alloc::alloc::dealloc(ptr, layout) } }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8  { unsafe { alloc::alloc::realloc(ptr, layout, new_size) } }
}

#[cfg(allocator_api = "1.50")] unsafe impl core::alloc::Allocator for Global {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = fat::Alloc::alloc_uninit(self, layout).map_err(|_| AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), layout.size()))
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = fat::Alloc::alloc_zeroed(self, layout).map_err(|_| AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { fat::Free::free(self, ptr.cast(), layout) }
    }

    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = unsafe { fat::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), new_layout.size()))
    }

    unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = unsafe { fat::Realloc::realloc_zeroed(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), new_layout.size()))
    }

    unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = unsafe { fat::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
        Ok(util::nn::slice_from_raw_parts(data.cast(), new_layout.size()))
    }
}
