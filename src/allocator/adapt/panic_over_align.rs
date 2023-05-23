use crate::{*, Alignment};
use crate::meta::*;

use core::alloc::Layout;



/// Adapt a [`thin`] allocator to a wider interface, [`panic!`]ing if more than [`Meta::MAX_ALIGN`] is requested.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct PanicOverAlign<A>(pub A);

#[inline(never)] #[track_caller] fn invalid_requested_alignment(requested: usize, supported: usize) -> ! {
    panic!("requested alignment {requested:?} > supported {supported:?}")
}

#[inline(always)] #[track_caller] fn assert_valid_alignment(requested: impl Into<usize>, supported: impl Into<usize>) {
    let requested = requested.into();
    let supported = supported.into();
    if requested > supported { invalid_requested_alignment(requested, supported) }
}

#[inline(always)] #[track_caller] fn freed_old_alignment(freed: impl Into<usize>, supported: impl Into<usize>) {
    let freed = freed.into();
    let supported = supported.into();
    if freed > supported { bug::ub::invalid_free_align_for_allocator(freed) }
}

impl<A> core::ops::Deref for PanicOverAlign<A> {
    type Target = A;
    #[inline(always)] fn deref(&self) -> &Self::Target { &self.0 }
}



// meta::*

impl<A: Meta> Meta for PanicOverAlign<A> {
    type Error                  = A::Error;
    const MAX_ALIGN : Alignment = A::MAX_ALIGN;
    const MAX_SIZE  : usize     = A::MAX_SIZE;
    const ZST_SUPPORTED : bool  = A::ZST_SUPPORTED;
}

impl<A: ZstSupported> ZstSupported for PanicOverAlign<A> {}

// SAFETY: ✔️ per underlying allocator
unsafe impl<A: ZstInfalliable> ZstInfalliable for PanicOverAlign<A> {}

// SAFETY: ✔️ per underlying allocator
unsafe impl<A: Stateless> Stateless for PanicOverAlign<A> {}



// fat::*

unsafe impl<A: fat::Alloc> fat::Alloc for PanicOverAlign<A> {
    #[track_caller] fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        assert_valid_alignment(layout.align(), A::MAX_ALIGN);
        A::alloc_uninit(self, layout)
    }
    #[track_caller] fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        assert_valid_alignment(layout.align(), A::MAX_ALIGN);
        A::alloc_zeroed(self, layout)
    }
}

unsafe impl<A: fat::Free> fat::Free for PanicOverAlign<A> {
    #[inline(always)] #[track_caller] unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        freed_old_alignment(layout.align(), A::MAX_ALIGN);
        unsafe { A::free(self, ptr, layout) }
    }
}

unsafe impl<A: fat::Realloc> fat::Realloc for PanicOverAlign<A> {
    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        freed_old_alignment(old_layout.align(), A::MAX_ALIGN);
        assert_valid_alignment(new_layout.align(), A::MAX_ALIGN);
        unsafe { A::realloc_uninit(self, ptr, old_layout, new_layout) }
    }
    #[track_caller] unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        freed_old_alignment(old_layout.align(), A::MAX_ALIGN);
        assert_valid_alignment(new_layout.align(), A::MAX_ALIGN);
        unsafe { A::realloc_zeroed(self, ptr, old_layout, new_layout) }
    }
}



#[no_implicit_prelude] mod cleanroom {
    use super::{impls, thin, fat, PanicOverAlign};

    impls! {
        unsafe impl[A: fat::Realloc         ] core::alloc::GlobalAlloc  for PanicOverAlign<A> => ialloc::fat::Realloc;

        unsafe impl[A: thin::Alloc          ] ialloc::thin::Alloc       for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::Free           ] ialloc::thin::Free        for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::Realloc        ] ialloc::thin::Realloc     for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::SizeOf         ] ialloc::thin::SizeOf      for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::SizeOfDebug    ] ialloc::thin::SizeOfDebug for PanicOverAlign<A> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl[A: fat::Realloc         ] core::alloc::Allocator(unstable 1.50) for PanicOverAlign<A> => ialloc::fat::Realloc;
    }
}



#[cfg(all(c89, allocator_api = "*"))] #[test] fn allocator_api() {
    use crate::allocator::{adapt::PanicOverAlign, c::Malloc};
    use alloc::vec::Vec;

    let mut v = Vec::new_in(PanicOverAlign(Malloc));
    v.push(1);
    v.push(2);
    v.push(3);
    let v2 = v.clone();
    assert_eq!(3, v.len());
    assert_eq!(3, v2.len());
}

#[cfg(all(c89, allocator_api = "*"))] #[should_panic] #[test] fn allocator_api_overalign() {
    use crate::allocator::{adapt::PanicOverAlign, c::Malloc};
    use alloc::vec::Vec;

    #[derive(Clone, Copy)] #[repr(C, align(4096))] struct Page([u8; 4096]);
    impl Page { pub fn new() -> Self { Self([0u8; 4096]) } }

    let mut v = Vec::new_in(PanicOverAlign(Malloc));
    v.push(Page::new());
    v.push(Page::new());
    v.push(Page::new());
    let v2 = v.clone();
    assert_eq!(3, v.len());
    assert_eq!(3, v2.len());
}
