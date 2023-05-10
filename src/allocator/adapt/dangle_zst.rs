use crate::*;
use crate::util::nn::dangling;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// If the underlying allocator doesn't support ZSTs, add support by returning dangling pointers for ZSTs.<br>
/// This is efficient, but awkward for C/C++ interop, where the underlying allocator likely chokes on dangling pointers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct DangleZst<A>(pub A);

impl<A> core::ops::Deref for DangleZst<A> { fn deref(&self) -> &Self::Target { &self.0 } type Target = A; }

impl<A: meta::Meta> meta::Meta for DangleZst<A> {
    type Error                  = A::Error;
    const MAX_ALIGN : Alignment = A::MAX_ALIGN;
    const MAX_SIZE  : usize     = A::MAX_SIZE;
    const ZST_SUPPORTED : bool  = true;
}

unsafe impl<A: fat::Alloc> fat::Alloc for DangleZst<A> {
    fn alloc_uninit(&self, layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if !A::ZST_SUPPORTED && layout.size() == 0 { return Ok(dangling(layout)) }
        self.0.alloc_uninit(layout)
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        if !A::ZST_SUPPORTED && layout.size() == 0 { return Ok(dangling(layout)) }
        self.0.alloc_zeroed(layout)
    }
}

unsafe impl<A: fat::Free> fat::Free for DangleZst<A> {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, layout: Layout) {
        if !A::ZST_SUPPORTED && layout.size() == 0 { debug_assert_eq!(ptr, dangling(layout)); return }
        unsafe { self.0.free(ptr, layout) }
    }
}

unsafe impl<A: fat::Realloc> fat::Realloc for DangleZst<A> {
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if A::ZST_SUPPORTED || (old_layout.size() > 0 && new_layout.size() > 0) {
            unsafe { self.0.realloc_uninit(ptr, old_layout, new_layout) }
        } else {
            let alloc = self.alloc_uninit(new_layout)?;
            let n = old_layout.size().min(new_layout.size());
            {
                let src : &    [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts    (ptr  .as_ptr(), n) };
                let dst : &mut [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), n) };
                dst.copy_from_slice(src);
            }
            unsafe { self.free(ptr, old_layout) };
            Ok(alloc)
        }
    }

    unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if A::ZST_SUPPORTED || (old_layout.size() > 0 && new_layout.size() > 0) {
            unsafe { self.0.realloc_zeroed(ptr, old_layout, new_layout) }
        } else {
            let alloc = self.alloc_zeroed(new_layout)?.cast();
            let n = old_layout.size().min(new_layout.size());
            {
                let src : &    [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts    (ptr  .as_ptr(), n) };
                let dst : &mut [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), n) };
                dst.copy_from_slice(src);
            }
            unsafe { self.free(ptr, old_layout) };
            Ok(alloc)
        }
    }
}

#[no_implicit_prelude] mod cleanroom {
    #[allow(unused_imports)] use super::{impls, fat, DangleZst};

    impls! {
        unsafe impl[A: ::core::alloc::GlobalAlloc   ] core::alloc::GlobalAlloc  for DangleZst<A> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl[A: fat::Realloc                ] core::alloc::Allocator(unstable 1.50) for DangleZst<A> => ialloc::fat::Realloc;
    }
}
