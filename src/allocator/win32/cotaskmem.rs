use crate::*;

use winapi::um::combaseapi::{CoTaskMemAlloc, CoTaskMemRealloc, CoTaskMemFree};

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// [`CoTaskMemAlloc`] / [`CoTaskMemRealloc`] / [`CoTaskMemFree`]
///
/// | Rust                              | C                     |
/// | ----------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]     | [`CoTaskMemAlloc`]
/// | [`thin::Realloc::realloc_uninit`] | [`CoTaskMemRealloc`]
/// | [`thin::Free::free`]              | [`CoTaskMemFree`]
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct CoTaskMem;

impl meta::Meta for CoTaskMem {
    type Error                  = ();
    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for CoTaskMem {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ thread safe - just uses g_CMalloc::Alloc → HeapAlloc with dwFlags=0 (e.g. not using HEAP_NO_SERIALIZE) under the hood
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsoundness is #[test]ed for at the end of this file.
        let alloc = unsafe { CoTaskMemAlloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing CoTaskMemAlloc
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Realloc for CoTaskMem {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ presumably thread safe - just uses g_CMalloc::Realloc → HeapReAlloc with dwFlags=0 (e.g. not using HEAP_NO_SERIALIZE) under the hood?
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with CoTaskMem{Alloc,Realloc}
        let alloc = unsafe { CoTaskMemRealloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: usize) -> Result<AllocNN, Self::Error> {
        Err(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for CoTaskMem {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ⚠️ presumably thread safe - just uses g_CMalloc::Free → HeapFree with dwFlags=0 (e.g. not using HEAP_NO_SERIALIZE) under the hood?
        // SAFETY: ✔️ `ptr` is either `nullptr` (safe, tested), or belongs to `self` per thin::Free::free_nullable's documented safety preconditions - and thus was allocated with CoTaskMem{Alloc,Realloc}
        unsafe { CoTaskMemFree(ptr.cast()) }
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, CoTaskMem};

    impls! {
        unsafe impl ialloc::fat::Alloc      for CoTaskMem => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for CoTaskMem => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for CoTaskMem => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(CoTaskMem) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(CoTaskMem) }
#[test] fn thin_nullable()          { thin::test::nullable(CoTaskMem) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(CoTaskMem) }
