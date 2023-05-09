//! [`Null`]

#![allow(unused_variables)]

use crate::*;
use core::alloc::Layout;



/// Never allocates anything, not even ZSTs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct Null;



// thin::*

unsafe impl thin::Alloc for Null {
    const MAX_ALIGN : Alignment = ALIGN_MAX;
    type Error = ();
    fn alloc_uninit(&self, size: core::num::NonZeroUsize) -> Result<AllocNN, Self::Error> { Err(()) }
}

unsafe impl thin::Free for Null {
    unsafe fn free(&self, ptr: AllocNN) { panic!("bug: Null allocator can't allocate anything, ergo freeing anything it supposedly allocated is a serious bug") }
}

unsafe impl thin::Realloc for Null {
    const CAN_REALLOC_ZEROED : bool = true;
    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: core::num::NonZeroUsize) -> Result<AllocNN, Self::Error> { Err(()) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: core::num::NonZeroUsize) -> Result<AllocNN, Self::Error> { Err(()) }
}

unsafe impl thin::SizeOf for Null {}
unsafe impl thin::SizeOfDebug for Null {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> { panic!("bug: Null allocator can't allocate anything, ergo querying the size of anything it supposedly allocated is a serious bug") }
}



// nzst::*

unsafe impl nzst::Alloc for Null {
    const MAX_ALIGN : Alignment = ALIGN_MAX;
    type Error = ();
    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN, Self::Error> { Err(()) }
    fn alloc_zeroed(&self, layout: LayoutNZ) -> Result<AllocNN0, Self::Error> { Err(()) }
}

unsafe impl nzst::Free for Null {
    unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) { panic!("bug: Null allocator can't allocate anything, ergo freeing anything it supposedly allocated is a serious bug") }
}

unsafe impl nzst::Realloc for Null {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> { Err(()) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> { Err(()) }
}




// zsty::*

unsafe impl zsty::Alloc for Null {
    const MAX_ALIGN : Alignment = ALIGN_MAX;
    type Error = ();
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> { Err(()) }
}

unsafe impl zsty::Free for Null {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) { panic!("bug: Null allocator can't allocate anything, ergo freeing anything it supposedly allocated is a serious bug") }
}

unsafe impl zsty::Realloc for Null {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { Err(()) }
}
