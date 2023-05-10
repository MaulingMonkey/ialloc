use crate::*;
use crate::meta::Meta;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// If the underlying allocator doesn't support ZSTs, add support by increasing sizes to at least 1 byte.<br>
/// This potentially wastes a little memory and performance - but allows for C/C++ interop with fewer edge cases.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct AllocZst<A>(pub A);

impl<A> core::ops::Deref for AllocZst<A> { fn deref(&self) -> &Self::Target { &self.0 } type Target = A; }

impl<A: Meta> AllocZst<A> {
    fn fix_layout(layout: Layout) -> Result<Layout, A::Error> {
        if A::ZST_SUPPORTED || layout.size() != 0 { return Ok(layout) }
        LayoutNZ::from_layout_min_size_1(layout).map(|l| l.into()).map_err(|e| e.into())
    }
}

impl<A: Meta> Meta for AllocZst<A> {
    type Error                  = A::Error;
    const MAX_ALIGN : Alignment = A::MAX_ALIGN;
    const MAX_SIZE  : usize     = A::MAX_SIZE;
    const ZST_SUPPORTED : bool  = true;
}

unsafe impl<A: zsty::Alloc> zsty::Alloc for AllocZst<A> {
    fn alloc_uninit(&self, layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let layout = Self::fix_layout(layout)?;
        self.0.alloc_uninit(layout)
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        let layout = Self::fix_layout(layout)?;
        self.0.alloc_zeroed(layout)
    }
}

unsafe impl<A: zsty::Free> zsty::Free for AllocZst<A> {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, layout: Layout) {
        let layout = Self::fix_layout(layout).expect("bug: undefined behavior: invalid old_layout");
        unsafe { self.0.free(ptr, layout) }
    }
}

unsafe impl<A: zsty::Realloc> zsty::Realloc for AllocZst<A> {
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let old_layout = Self::fix_layout(old_layout).expect("bug: undefined behavior: invalid old_layout");
        let new_layout = Self::fix_layout(new_layout)?;
        unsafe { self.0.realloc_uninit(ptr, old_layout, new_layout) }
    }

    unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let old_layout = Self::fix_layout(old_layout).expect("bug: undefined behavior: invalid old_layout");
        let new_layout = Self::fix_layout(new_layout)?;
        unsafe { self.0.realloc_zeroed(ptr, old_layout, new_layout) }
    }
}

#[no_implicit_prelude] mod cleanroom {
    #[allow(unused_imports)] use super::{impls, zsty, AllocZst};

    impls! {
        unsafe impl[A: ::core::alloc::GlobalAlloc   ] core::alloc::GlobalAlloc  for AllocZst<A> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl[A: zsty::Realloc                ] core::alloc::Allocator(unstable 1.50) for AllocZst<A> => ialloc::zsty::Realloc;
    }
}
