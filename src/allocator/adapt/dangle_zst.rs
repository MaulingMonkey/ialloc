use crate::*;
use crate::error::ExcessiveAlignmentRequestedError;
use crate::meta::*;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// If the underlying allocator doesn't support ZSTs, add support by returning dangling pointers for ZSTs.<br>
/// This is efficient, but awkward for C/C++ interop, where the underlying allocator likely chokes on dangling pointers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct DangleZst<A>(pub A);

impl<A: Meta> DangleZst<A> {
    // TODO: replace with NonNull::new when that stabilizes as a const fn
    const DANGLE : NonNull<MaybeUninit<u8>> = unsafe { NonNull::new_unchecked(A::MAX_ALIGN.as_usize() as _) };
}

impl<A> core::ops::Deref for DangleZst<A> { fn deref(&self) -> &Self::Target { &self.0 } type Target = A; }



// meta::*

impl<A: Meta> Meta for DangleZst<A> {
    type Error                  = A::Error;
    const MAX_ALIGN : Alignment = A::MAX_ALIGN;
    const MAX_SIZE  : usize     = A::MAX_SIZE;
    const ZST_SUPPORTED : bool  = true;
}

impl<A: Meta> ZstSupported for DangleZst<A> {}

unsafe impl<A: Meta> ZstInfalliable for DangleZst<A> {}

unsafe impl<A: Stateless> Stateless for DangleZst<A> {}



// thin::*

unsafe impl<A: thin::Alloc> thin::Alloc for DangleZst<A> {
    fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if size == 0 { return Ok(Self::DANGLE) }
        self.0.alloc_uninit(size)
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        if size == 0 { return Ok(Self::DANGLE.cast()) }
        self.0.alloc_zeroed(size)
    }
}

unsafe impl<A: thin::Free> thin::Free for DangleZst<A> {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>) {
        if ptr == Self::DANGLE { return }
        unsafe { self.0.free(ptr) }
    }
}

unsafe impl<A: thin::Realloc> thin::Realloc for DangleZst<A> {
    const CAN_REALLOC_ZEROED : bool = A::CAN_REALLOC_ZEROED;

    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if ptr == Self::DANGLE  { return self.0.alloc_uninit(new_size) }
        if new_size == 0        { unsafe { self.0.free(ptr) }; return Ok(Self::DANGLE) }
        unsafe { self.0.realloc_uninit(ptr, new_size) }
    }

    unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if ptr == Self::DANGLE  { return self.0.alloc_zeroed(new_size).map(|a| a.cast()) }
        if new_size == 0        { unsafe { self.0.free(ptr) }; return Ok(Self::DANGLE) }
        unsafe { self.0.realloc_uninit(ptr, new_size) }
    }
}



// fat::*

unsafe impl<A: fat::Alloc> fat::Alloc for DangleZst<A> {
    fn alloc_uninit(&self, layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if layout.size() > 0 {
            self.0.alloc_uninit(layout)
        } else if layout.align() <= A::MAX_ALIGN.as_usize() {
            Ok(Self::DANGLE)
        } else {
            Err(ExcessiveAlignmentRequestedError{ requested: layout.into(), supported: A::MAX_ALIGN }.into())
        }
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        if layout.size() > 0 {
            self.0.alloc_zeroed(layout)
        } else if layout.align() <= A::MAX_ALIGN.as_usize() {
            Ok(Self::DANGLE.cast())
        } else {
            Err(ExcessiveAlignmentRequestedError{ requested: layout.into(), supported: A::MAX_ALIGN }.into())
        }
    }
}

unsafe impl<A: fat::Free> fat::Free for DangleZst<A> {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, layout: Layout) {
        if layout.size() > 0 {
            unsafe { self.0.free(ptr, layout) };
        } else if cfg!(debug_assertions) {
            if ptr != Self::DANGLE                      { bug::ub::invalid_ptr_for_allocator(ptr) }
            if layout.align() > A::MAX_ALIGN.as_usize() { bug::ub::invalid_free_align_for_allocator(layout.align()) }
        }
    }
}

unsafe impl<A: fat::Realloc> fat::Realloc for DangleZst<A> {
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        if old_layout.size() > 0 && new_layout.size() > 0 {
            unsafe { self.0.realloc_uninit(ptr, old_layout, new_layout) }
        } else {
            if cfg!(debug_assertions) && old_layout.size() == 0 {
                if ptr != Self::DANGLE                          { bug::ub::invalid_ptr_for_allocator(ptr) }
                if old_layout.align() > A::MAX_ALIGN.as_usize() { bug::ub::invalid_free_align_for_allocator(old_layout.align()) }
            }
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
        if old_layout.size() > 0 && new_layout.size() > 0 {
            unsafe { self.0.realloc_zeroed(ptr, old_layout, new_layout) }
        } else {
            if cfg!(debug_assertions) && old_layout.size() == 0 {
                if ptr != Self::DANGLE                          { bug::ub::invalid_ptr_for_allocator(ptr) }
                if old_layout.align() > A::MAX_ALIGN.as_usize() { bug::ub::invalid_free_align_for_allocator(old_layout.align()) }
            }
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
        unsafe impl[A: fat::Realloc                 ] core::alloc::Allocator(unstable 1.50) for DangleZst<A> => ialloc::fat::Realloc;
    }
}
