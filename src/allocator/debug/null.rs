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

unsafe impl thin::Alloc for Null {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> { Err(()) }
}

unsafe impl thin::Free for Null {
    unsafe fn free(&self, ptr: AllocNN) { panic!("bug: Null allocator can't allocate anything, ergo freeing anything it supposedly allocated is a serious bug") }
}

unsafe impl thin::Realloc for Null {
    const CAN_REALLOC_ZEROED : bool = true;
    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { Err(()) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { Err(()) }
}

unsafe impl thin::SizeOf for Null {}
unsafe impl thin::SizeOfDebug for Null {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> { panic!("bug: Null allocator can't allocate anything, ergo querying the size of anything it supposedly allocated is a serious bug") }
}



// fat::*

unsafe impl fat::Alloc for Null {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> { Err(()) }
}

unsafe impl fat::Free for Null {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) { panic!("bug: Null allocator can't allocate anything, ergo freeing anything it supposedly allocated is a serious bug") }
}

unsafe impl fat::Realloc for Null {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
}



// XXX: Null doesn't allocate *anything*, but is meant to be "compatible" with everything
//#[test] fn thin_zst_support() { assert!(thin::zst_supported_accurate(Null)) }
