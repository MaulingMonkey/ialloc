use crate::*;



/// Adapt a [`nzst`] allocator to [`zsty`], always allocating at least 1 byte from the underlying allocator, even for ZSTs.<br>
/// This potentially wastes a little memory and performance - but allows for C/C++ interop with fewer edge cases.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct AllocZst<A>(pub A);

impl<A> core::ops::Deref for AllocZst<A> { fn deref(&self) -> &Self::Target { &self.0 } type Target = A; }

impl<A: meta::Meta> meta::Meta for AllocZst<A> {
    type Error                  = A::Error;
    const MAX_ALIGN : Alignment = A::MAX_ALIGN;
    const MAX_SIZE  : usize     = A::MAX_SIZE;
    const ZST_SUPPORTED : bool  = true;
}

unsafe impl<A: nzst::Alloc> zsty::Alloc for AllocZst<A> {
    fn alloc_uninit(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
        let layout = LayoutNZ::from_layout_min_size_1(layout)?;
        nzst::Alloc::alloc_uninit(&self.0, layout)
    }

    fn alloc_zeroed(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::primitive::u8>, Self::Error> {
        let layout = LayoutNZ::from_layout_min_size_1(layout)?;
        nzst::Alloc::alloc_zeroed(&self.0, layout)
    }
}

unsafe impl<A: nzst::Free> zsty::Free for AllocZst<A> {
    unsafe fn free(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, layout: ::core::alloc::Layout) {
        let layout = LayoutNZ::from_layout_min_size_1(layout).expect("bug: undefined behavior: invalid old_layout");
        unsafe { nzst::Free::free(&self.0, ptr, layout) }
    }
}

unsafe impl<A: nzst::Realloc> zsty::Realloc for AllocZst<A> {
    unsafe fn realloc_uninit(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
        let old_layout = LayoutNZ::from_layout_min_size_1(old_layout).expect("bug: undefined behavior: invalid old_layout");
        let new_layout = LayoutNZ::from_layout_min_size_1(new_layout)?;
        unsafe { nzst::Realloc::realloc_uninit(&self.0, ptr, old_layout, new_layout) }
    }

    unsafe fn realloc_zeroed(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
        let old_layout = LayoutNZ::from_layout_min_size_1(old_layout).expect("bug: undefined behavior: invalid old_layout");
        let new_layout = LayoutNZ::from_layout_min_size_1(new_layout)?;
        unsafe { nzst::Realloc::realloc_zeroed(&self.0, ptr, old_layout, new_layout) }
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, nzst, AllocZst};

    impls! {
        unsafe impl[A: ::core::alloc::GlobalAlloc   ] core::alloc::GlobalAlloc  for AllocZst<A> => core::ops::Deref;

        unsafe impl[A: nzst::Alloc                  ] ialloc::nzst::Alloc       for AllocZst<A> => core::ops::Deref;
        unsafe impl[A: nzst::Free                   ] ialloc::nzst::Free        for AllocZst<A> => core::ops::Deref;
        unsafe impl[A: nzst::Realloc                ] ialloc::nzst::Realloc     for AllocZst<A> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl[A: nzst::Realloc                ] core::alloc::Allocator(unstable 1.50) for AllocZst<A> => ialloc::zsty::Realloc;
    }
}
