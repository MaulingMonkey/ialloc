#![allow(unused_variables)]

use crate::*;
use core::alloc::Layout;



/// Never allocates anything, not even ZSTs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct Null;

impl meta::Meta for Null {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::MAX;
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = true;
}



// thin::*

// SAFETY: ✔️ always failing to allocate is a trivally safe implementation of this trait
unsafe impl thin::Alloc for Null {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> { Err(()) }
}

// SAFETY: ✔️ this trait cannot be safely called, and simply panicing in response is a reasonable response to the caller's UB
unsafe impl thin::Free for Null {
    #[track_caller] #[inline(never)] unsafe fn free(&self, ptr: AllocNN) {
        // SAFETY: ✔️ violation of thin::Free::free's documented safety precondition that `ptr` belong to `self`
        unsafe { ub!("bug: undefined behavior: {ptr:?} does not belong to `self` as the Null allocator can't allocate anything to free in the first place") }
    }
}

// SAFETY: ✔️ always failing to (re)allocate is a trivally safe implementation of this trait
unsafe impl thin::Realloc for Null {
    const CAN_REALLOC_ZEROED : bool = true;
    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { Err(()) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { Err(()) }
}

// SAFETY: ✔️ always failing is a trivally safe implementation of this trait
unsafe impl thin::SizeOf for Null {}

// SAFETY: ✔️ always failing is a trivally safe implementation of this trait
unsafe impl thin::SizeOfDebug for Null {
    #[track_caller] #[inline(never)] unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> {
        // SAFETY: ✔️ violation of thin::SizeOfDebug::size_of's documented safety precondition that `ptr` belong to `self`
        unsafe { ub!("bug: undefined behavior: {ptr:?} does not belong to `self` as the Null allocator can't allocate anything to query in the first place") }
    }
}



// fat::*

// SAFETY: ✔️ always failing to allocate is a trivally safe implementation of this trait
unsafe impl fat::Alloc for Null {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> { Err(()) }
}

// SAFETY: ✔️ always failing is a trivally safe implementation of this trait
unsafe impl fat::Free for Null {
    #[track_caller] #[inline(never)] unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        // SAFETY: ✔️ violation of fat::Free::free's documented safety precondition that `ptr` belong to `self`
        unsafe { ub!("bug: undefined behavior: {ptr:?} does not belong to `self` as the Null allocator can't allocate anything to free in the first place") }
    }
}

// SAFETY: ✔️ always failing to (re)allocate is a trivally safe implementation of this trait
unsafe impl fat::Realloc for Null {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
}



//#[test] fn thin_zst_support() { thin::test::zst_supported_conservative(Null) } // XXX: Null intentionally claims support for anything but won't ever allocate
