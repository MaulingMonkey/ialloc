//! [`PanicOverAlign`]

use crate::{*, Alignment};

#[cfg(allocator_api = "1.50")] use core::alloc::AllocError;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::*;



/// Adapt a [`thin`] allocator to a wider interface, `panic!`ing if more than [`thin::Alloc::MAX_ALIGN`] is requested.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct PanicOverAlign<A>(pub A);

impl<A: thin::Alloc> PanicOverAlign<A> {
    #[track_caller] fn layout_to_size(layout: LayoutNZ) -> NonZeroUsize {
        let align = layout.align();
        if align > A::MAX_ALIGN { Self::invalid_alignment(layout.align()) }
        layout.size().max(align.as_nonzero())
    }

    #[inline(never)] #[track_caller] fn invalid_alignment(align: Alignment) -> ! {
        panic!("alignment {align:?} > Self::MAX_ALIGN ({:?})", A::MAX_ALIGN)
    }
}



// thin::*

unsafe impl<A: thin::Alloc> thin::Alloc for PanicOverAlign<A> {
    type Error = A::Error;
    #[inline(always)] #[track_caller] fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN,  Self::Error> { self.0.alloc_uninit(size) }
    #[inline(always)] #[track_caller] fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<AllocNN0, Self::Error> { self.0.alloc_zeroed(size) }
}

unsafe impl<A: thin::Free> thin::Free for PanicOverAlign<A> {
    #[inline(always)] #[track_caller] unsafe fn free(&self, ptr: AllocNN) { unsafe { self.0.free(ptr) } }
    #[inline(always)] #[track_caller] unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) { unsafe { self.0.free_nullable(ptr) } }
}

unsafe impl<A: thin::Realloc> thin::Realloc for PanicOverAlign<A> {
    #[inline(always)] #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> { unsafe { self.0.realloc_uninit(ptr, new_size) } }
}

unsafe impl<A: thin::ReallocZeroed> thin::ReallocZeroed for PanicOverAlign<A> {
    #[inline(always)] #[track_caller] unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> { unsafe { self.0.realloc_zeroed(ptr, new_size) } }
}

unsafe impl<A: thin::SizeOf> thin::SizeOf for PanicOverAlign<A> {}
unsafe impl<A: thin::SizeOfDebug> thin::SizeOfDebug for PanicOverAlign<A> {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> { unsafe { self.0.size_of(ptr) } }
}



// nzst::*

unsafe impl<A: thin::Alloc> nzst::Alloc for PanicOverAlign<A> {
    type Error = A::Error;
    #[track_caller] fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN,  Self::Error> { self.0.alloc_uninit(Self::layout_to_size(layout)) }
    #[track_caller] fn alloc_zeroed(&self, layout: LayoutNZ) -> Result<AllocNN0, Self::Error> { self.0.alloc_zeroed(Self::layout_to_size(layout)) }
}

unsafe impl<A: thin::Alloc + thin::Free> nzst::Free for PanicOverAlign<A> {
    #[track_caller] unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) {
        let _ = Self::layout_to_size(layout); // if this fails, we never could've allocated this allocation
        unsafe { self.0.free(ptr) }
    }
}

unsafe impl<A: thin::ReallocZeroed> nzst::Realloc for PanicOverAlign<A> {
    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        let _ = Self::layout_to_size(old_layout);
        let new_size = Self::layout_to_size(new_layout);
        unsafe { self.0.realloc_uninit(ptr, new_size) }
    }

    #[track_caller] unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        // XXX: should thin::Realloc have a try_realloc_zeroed fn ? a CAN_REALLOC_ZERO bool? I think so...
        let _ = Self::layout_to_size(old_layout);
        let new_size = Self::layout_to_size(new_layout);
        unsafe { self.0.realloc_zeroed(ptr, new_size) }
    }
}



// core::*

unsafe impl<A: thin::ReallocZeroed> core::alloc::GlobalAlloc for PanicOverAlign<A> {
    #[track_caller] unsafe fn alloc(&self, layout: Layout) -> *mut u8 { zsty::Alloc::alloc_uninit(self, layout).map_or(null_mut(), |p| p.as_ptr().cast()) }
    #[track_caller] unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { zsty::Alloc::alloc_zeroed(self, layout).map_or(null_mut(), |p| p.as_ptr().cast()) }
    #[track_caller] unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { if let Some(ptr) = NonNull::new(ptr) { unsafe { zsty::Free::free(self, ptr.cast(), layout) } } }
    #[track_caller] unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else { return null_mut() };
        if let Some(ptr) = NonNull::new(ptr) {
            unsafe { zsty::Realloc::realloc_uninit(self, ptr.cast(), layout, new_layout) }
        } else {
            zsty::Alloc::alloc_uninit(self, new_layout)
        }.map_or(null_mut(), |p| p.as_ptr().cast())
    }
}

#[cfg(allocator_api = "1.50")] unsafe impl<A: thin::ReallocZeroed> core::alloc::Allocator for PanicOverAlign<A> {
    #[track_caller] fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = zsty::Alloc::alloc_uninit(self, layout).map_err(|_| AllocError)?.as_ptr().cast();
        Ok(unsafe { NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data, layout.size())) })
    }

    #[track_caller] fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = zsty::Alloc::alloc_zeroed(self, layout).map_err(|_| AllocError)?.as_ptr().cast();
        Ok(unsafe { NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data, layout.size())) })
    }

    #[track_caller] unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { zsty::Free::free(self, ptr.cast(), layout) }
    }

    #[track_caller] unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = unsafe { zsty::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?.as_ptr().cast();
        Ok(unsafe { NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data, new_layout.size())) })
    }

    #[track_caller] unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = unsafe { zsty::Realloc::realloc_zeroed(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?.as_ptr().cast();
        Ok(unsafe { NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data, new_layout.size())) })
    }

    #[track_caller] unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = unsafe { zsty::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?.as_ptr().cast();
        Ok(unsafe { NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data, new_layout.size())) })
    }
}
