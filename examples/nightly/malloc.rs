use ialloc::*;

use core::alloc::{self, Layout, LayoutError};
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



fn main() {
    let mut v = Vec::new_in(ToAllocator(Malloc));
    v.push(1);
    v.push(2);
    v.push(3);
    let v2 = v.clone();
    dbg!((v, v2));
}



#[derive(Clone, Copy, Debug)] struct AllocError;
impl From<LayoutError> for AllocError { fn from(_: LayoutError) -> Self { Self } }
impl From<alloc::AllocError> for AllocError { fn from(_: alloc::AllocError) -> Self { Self } }
impl From<AllocError> for alloc::AllocError { fn from(_: AllocError) -> Self { alloc::AllocError } }



// TODO: exile into a crate
/// malloc / realloc / free
#[derive(Default, Clone, Copy)] struct Malloc;
impl Malloc {
    // TODO: more accurately determine malloc's alignment/size guarantees
    pub const MAX_ALIGN : Alignment     = ALIGN_4;
    pub const MAX_SIZE  : NonZeroUsize  = NonZeroUsize::new(usize::MAX/2).unwrap();
}
unsafe impl nzst::Alloc for Malloc {
    type Error = AllocError;
    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        assert!(layout.size()  <= Self::MAX_SIZE,  "requested allocation beyond malloc's capabilities");
        assert!(layout.align() <= Self::MAX_ALIGN, "requested allocation beyond malloc's capabilities");

        let size = layout.size().get().next_multiple_of(layout.align().get());
        let alloc = unsafe { libc::malloc(size) };
        NonNull::new(alloc.cast()).ok_or(AllocError)
    }
}
unsafe impl nzst::Realloc for Malloc {
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, old: LayoutNZ, new: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        assert!(old.size()  <= Self::MAX_SIZE,  "this allocation couldn't have belonged to this allocator, has too much alignment");
        assert!(old.align() <= Self::MAX_ALIGN, "this allocation couldn't have belonged to this allocator, has too much alignment");
        assert!(new.size()  <= Self::MAX_SIZE,  "requested reallocation beyond malloc's capabilities");
        assert!(new.align() <= Self::MAX_ALIGN, "requested reallocation beyond malloc's capabilities");

        let size = new.size().get().next_multiple_of(new.align().get());
        let alloc = unsafe { libc::realloc(ptr.as_ptr().cast(), size) };
        NonNull::new(alloc.cast()).ok_or(AllocError)
    }
}
impl nzst::Free for Malloc {
    unsafe fn dealloc(&self, ptr: NonNull<MaybeUninit<u8>>, _layout: LayoutNZ) {
        assert!(_layout.size()  <= Self::MAX_SIZE,  "this allocation couldn't have belonged to this allocator, has too much alignment");
        assert!(_layout.align() <= Self::MAX_ALIGN, "this allocation couldn't have belonged to this allocator, has too much alignment");

        unsafe { libc::free(ptr.as_ptr().cast()) }
    }
}



// TODO: exile into a crate
/// Adapt a [`ialloc::zsty`]-style allocator to [`alloc::alloc::Allocator`] (nightl)
#[derive(Default, Clone, Copy)] struct ToAllocator<A: zsty::Alloc<Error = AllocError> + zsty::Free>(pub A);
unsafe impl<A: zsty::Alloc<Error = AllocError> + zsty::Free> alloc::Allocator for ToAllocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, alloc::AllocError> {
        let alloc = zsty::Alloc::alloc_uninit(&self.0, layout)?;
        NonNull::new(core::ptr::slice_from_raw_parts_mut(alloc.as_ptr().cast(), layout.size())).ok_or(alloc::AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        zsty::Free::dealloc(&self.0, ptr.cast(), layout)
    }

    // TODO: leverage zst::Realloc ?
}
