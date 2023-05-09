use crate::*;
use crate::util::nn::dangling;



/// Adapt a [`nzst`] allocator to [`zsty`], returning dangling pointers for ZSTs.<br>
/// This is efficient, but awkward for C/C++ interop, where the underlying allocator likely chokes on dangling pointers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct DangleZst<A>(pub A);

impl<A> core::ops::Deref for DangleZst<A> { fn deref(&self) -> &Self::Target { &self.0 } type Target = A; }

unsafe impl<A: nzst::Alloc> zsty::Alloc for DangleZst<A> {
    const MAX_ALIGN : Alignment = <A as nzst::Alloc>::MAX_ALIGN;
    type Error = <A as nzst::Alloc>::Error;

    fn alloc_uninit(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
        if let Ok(layout) = LayoutNZ::try_from(layout) {
            nzst::Alloc::alloc_uninit(&self.0, layout)
        } else { // Zero sized alloc
            Ok(dangling(layout))
        }
    }

    fn alloc_zeroed(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::primitive::u8>, Self::Error> {
        if let Ok(layout) = LayoutNZ::from_layout(layout) {
            nzst::Alloc::alloc_zeroed(&self.0, layout)
        } else { // Zero sized alloc
            Ok(dangling(layout))
        }
    }
}

unsafe impl<A: nzst::Free> zsty::Free for DangleZst<A> {
    unsafe fn free(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, layout: ::core::alloc::Layout) {
        match LayoutNZ::from_layout(layout) {
            Ok(layout) => unsafe { nzst::Free::free(&self.0, ptr, layout) },
            Err(_zsty) =>        { debug_assert_eq!(ptr, dangling(layout)) },
        }
    }
}

unsafe impl<A: nzst::Realloc> zsty::Realloc for DangleZst<A> {
    unsafe fn realloc_uninit(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
        match (LayoutNZ::from_layout(old_layout), LayoutNZ::from_layout(new_layout)) {
            (Err(_old_zsty), Ok(new_layout)) =>        { debug_assert_eq!(ptr, dangling(old_layout       )); nzst::Alloc::alloc_uninit(&self.0, new_layout) },
            (Ok(old_layout), Ok(new_layout)) => unsafe { debug_assert_ne!(ptr, dangling(old_layout.into())); nzst::Realloc::realloc_uninit(&self.0, ptr, old_layout, new_layout) },
            (Ok(old_layout), Err(_new_zsty)) => unsafe { debug_assert_ne!(ptr, dangling(old_layout.into())); nzst::Free::free(&self.0, ptr, old_layout); Ok(dangling(new_layout)) },
            (Err(_old_zsty), Err(_new_zsty)) =>        { debug_assert_eq!(ptr, dangling(old_layout       )); Ok(dangling(new_layout)) },
        }
    }

    unsafe fn realloc_zeroed(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
        match (LayoutNZ::from_layout(old_layout), LayoutNZ::from_layout(new_layout)) {
            (Err(_old_zsty), Ok(new_layout)) =>        { debug_assert_eq!(ptr, dangling(old_layout       )); Ok(nzst::Alloc::alloc_zeroed(&self.0, new_layout)?.cast()) },
            (Ok(old_layout), Ok(new_layout)) => unsafe { debug_assert_ne!(ptr, dangling(old_layout.into())); nzst::Realloc::realloc_zeroed(&self.0, ptr, old_layout, new_layout) },
            (Ok(old_layout), Err(_new_zsty)) => unsafe { debug_assert_ne!(ptr, dangling(old_layout.into())); nzst::Free::free(&self.0, ptr, old_layout); Ok(dangling(new_layout)) },
            (Err(_old_zsty), Err(_new_zsty)) =>        { debug_assert_eq!(ptr, dangling(old_layout       )); Ok(dangling(new_layout)) },
        }
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, nzst, DangleZst};

    impls! {
        unsafe impl[A: ::core::alloc::GlobalAlloc   ] core::alloc::GlobalAlloc  for DangleZst<A> => core::ops::Deref;

        unsafe impl[A: nzst::Alloc                  ] ialloc::nzst::Alloc       for DangleZst<A> => core::ops::Deref;
        unsafe impl[A: nzst::Free                   ] ialloc::nzst::Free        for DangleZst<A> => core::ops::Deref;
        unsafe impl[A: nzst::Realloc                ] ialloc::nzst::Realloc     for DangleZst<A> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl[A: nzst::Realloc                ] core::alloc::Allocator(unstable 1.50) for DangleZst<A> => ialloc::zsty::Realloc;
    }
}
