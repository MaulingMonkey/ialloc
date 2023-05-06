use crate::{nzst, zsty};
use crate::{Alignment, AllocNN, AllocNN0, LayoutNZ};

use core::alloc::Layout;
use core::ptr::NonNull;



fn dangling<T>(layout: Layout) -> NonNull<T> { NonNull::new(layout.align() as _).unwrap_or(NonNull::dangling()) }

unsafe impl<A: nzst::Alloc> zsty::Alloc for A {
    const MAX_ALIGN : Alignment = <A as nzst::Alloc>::MAX_ALIGN;

    type Error = A::Error;

    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        if let Ok(layout) = LayoutNZ::from_layout(layout) {
            nzst::Alloc::alloc_uninit(self, layout)
        } else { // Zero sized alloc
            Ok(dangling(layout))
        }
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        if let Ok(layout) = LayoutNZ::from_layout(layout) {
            nzst::Alloc::alloc_zeroed(self, layout)
        } else { // Zero sized alloc
            Ok(dangling(layout))
        }
    }
}

unsafe impl<A: nzst::Free> zsty::Free for A {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        match LayoutNZ::from_layout(layout) {
            Ok(layout) => unsafe { nzst::Free::free(self, ptr, layout) },
            Err(_zsty) =>        { debug_assert_eq!(ptr, dangling(layout)) },
        }
    }
}

unsafe impl<A: nzst::Realloc> zsty::Realloc for A {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        match (LayoutNZ::from_layout(old_layout), LayoutNZ::from_layout(new_layout)) {
            (Err(_old_zsty), Ok(new_layout)) =>        { debug_assert_eq!(ptr, dangling( old_layout)); nzst::Alloc::alloc_uninit(self, new_layout) },
            (Ok(old_layout), Ok(new_layout)) => unsafe { debug_assert_ne!(ptr, dangling(*old_layout)); nzst::Realloc::realloc_uninit(self, ptr, old_layout, new_layout) },
            (Ok(old_layout), Err(_new_zsty)) => unsafe { debug_assert_ne!(ptr, dangling(*old_layout)); nzst::Free::free(self, ptr, old_layout); Ok(dangling(new_layout)) },
            (Err(_old_zsty), Err(_new_zsty)) =>        { debug_assert_eq!(ptr, dangling( old_layout)); Ok(dangling(new_layout)) },
        }
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        match (LayoutNZ::from_layout(old_layout), LayoutNZ::from_layout(new_layout)) {
            (Err(_old_zsty), Ok(new_layout)) =>        { debug_assert_eq!(ptr, dangling( old_layout)); Ok(nzst::Alloc::alloc_zeroed(self, new_layout)?.cast()) },
            (Ok(old_layout), Ok(new_layout)) => unsafe { debug_assert_ne!(ptr, dangling(*old_layout)); nzst::Realloc::realloc_zeroed(self, ptr, old_layout, new_layout) },
            (Ok(old_layout), Err(_new_zsty)) => unsafe { debug_assert_ne!(ptr, dangling(*old_layout)); nzst::Free::free(self, ptr, old_layout); Ok(dangling(new_layout)) },
            (Err(_old_zsty), Err(_new_zsty)) =>        { debug_assert_eq!(ptr, dangling( old_layout)); Ok(dangling(new_layout)) },
        }
    }
}
