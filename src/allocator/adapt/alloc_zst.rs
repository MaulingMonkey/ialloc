use crate::*;
use crate::error::ExcessiveAlignmentRequestedError;
use crate::meta::*;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// If the underlying allocator doesn't support ZSTs, add support by increasing sizes to at least 1 byte.<br>
/// This potentially wastes a little memory and performance - but allows for C/C++ interop with fewer edge cases.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct AllocZst<A>(pub A);

impl<A> core::ops::Deref for AllocZst<A> { fn deref(&self) -> &Self::Target { &self.0 } type Target = A; }

impl<A: Meta> AllocZst<A> {
    fn fix_thin_size(size: usize) -> usize {
        if A::ZST_SUPPORTED { size }
        else { size.max(1) }
    }

    fn fix_layout(layout: Layout) -> Result<Layout, A::Error> {
        if A::ZST_SUPPORTED || layout.size() != 0 {
            Ok(layout)
        } else {
            const ALIGN_HALF_MAX : Alignment = Alignment::constant(Alignment::MAX.as_usize()/2);
            Layout::from_size_align(1, layout.align()).map_err(|_| ExcessiveAlignmentRequestedError {
                requested: Alignment::from(layout),
                supported: ALIGN_HALF_MAX, // only alignment that would cause 1 to OOB is ALIGN_MAX
            }.into())
        }
    }
}



// meta::*

impl<A: Meta> Meta for AllocZst<A> {
    type Error                  = A::Error;
    const MAX_ALIGN : Alignment = A::MAX_ALIGN;
    const MAX_SIZE  : usize     = A::MAX_SIZE;
    const ZST_SUPPORTED : bool  = true;
}

impl<A: Meta> ZstSupported for AllocZst<A> {}

unsafe impl<A: DefaultCompatible> DefaultCompatible for AllocZst<A> {}



// thin::*

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl<A: thin::Alloc> thin::Alloc for AllocZst<A> {
    fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let size = Self::fix_thin_size(size);
        self.0.alloc_uninit(size)
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        let size = Self::fix_thin_size(size);
        self.0.alloc_zeroed(size)
    }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl<A: thin::Free> thin::Free for AllocZst<A> {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>) { unsafe { self.0.free(ptr) } }
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) { unsafe { self.0.free_nullable(ptr) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl<A: thin::Realloc> thin::Realloc for AllocZst<A> {
    const CAN_REALLOC_ZEROED : bool = A::CAN_REALLOC_ZEROED;

    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let new_size = Self::fix_thin_size(new_size);
        unsafe { self.0.realloc_uninit(ptr, new_size) }
    }

    unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let new_size = Self::fix_thin_size(new_size);
        unsafe { self.0.realloc_zeroed(ptr, new_size) }
    }
}



// fat::*

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl<A: fat::Alloc> fat::Alloc for AllocZst<A> {
    fn alloc_uninit(&self, layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let layout = Self::fix_layout(layout)?;
        self.0.alloc_uninit(layout)
    }

    fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        let layout = Self::fix_layout(layout)?;
        self.0.alloc_zeroed(layout)
    }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl<A: fat::Free> fat::Free for AllocZst<A> {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, layout: Layout) {
        let layout = Self::fix_layout(layout).expect("bug: undefined behavior: invalid old_layout");
        unsafe { self.0.free(ptr, layout) }
    }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl<A: fat::Realloc> fat::Realloc for AllocZst<A> {
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
    #[allow(unused_imports)] use super::{impls, fat, AllocZst};

    impls! {
        unsafe impl[A: ::core::alloc::GlobalAlloc   ] core::alloc::GlobalAlloc  for AllocZst<A> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl[A: fat::Realloc                ] core::alloc::Allocator(unstable 1.50) for AllocZst<A> => ialloc::fat::Realloc;
    }
}
