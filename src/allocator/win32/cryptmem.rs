use crate::*;

use winapi::um::wincrypt::{CryptMemAlloc, CryptMemRealloc, CryptMemFree};

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// [`CryptMemAlloc`] / [`CryptMemRealloc`] / [`CryptMemFree`]
///
/// | Rust                              | C                     |
/// | ----------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]     | [`CryptMemAlloc`]
/// | [`thin::Realloc::realloc_uninit`] | [`CryptMemRealloc`]
/// | [`thin::Free::free`]              | [`CryptMemFree`]
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct CryptMem;

impl meta::Meta for CryptMem {
    type Error                  = ();
    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for CryptMem {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        let size = size.try_into().map_err(|_| {})?;

        // SAFETY: ⚠️ presumably thread safe per LocalAlloc/HeapAlloc
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsoundness is #[test]ed for at the end of this file.
        let alloc = unsafe { CryptMemAlloc(size) };
        // In practice, this just calls `LocalAlloc(LMEM_ZEROINIT, size)`, which in turn invokes `HeapAlloc`

        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing CryptMemAlloc
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Realloc for CryptMem {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let new_size = new_size.try_into().map_err(|_| {})?;

        // SAFETY: ⚠️ presumably thread safe per LocalReAlloc/HeapReAlloc
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with CryptMem{Alloc,Realloc}
        let alloc = unsafe { CryptMemRealloc(ptr.as_ptr().cast(), new_size) };

        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: usize) -> Result<AllocNN, Self::Error> {
        Err(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for CryptMem {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ⚠️ presumably thread safe per LocalFree/HeapFree
        // SAFETY: ✔️ `ptr` is either `nullptr` (safe, tested), or belongs to `self` per thin::Free::free_nullable's documented safety preconditions - and thus was allocated with CryptMem{Alloc,Realloc}
        unsafe { CryptMemFree(ptr.cast()) }
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, CryptMem};

    impls! {
        unsafe impl ialloc::fat::Alloc      for CryptMem => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for CryptMem => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for CryptMem => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(CryptMem) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(CryptMem) }
#[test] fn thin_nullable()          { thin::test::nullable(CryptMem) }
//#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(CryptMem) } } // Interestingly, CryptMem appears to zero it's memory.  This isn't documented, so I choose not to rely on it, but it's interesting...
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(CryptMem) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(CryptMem) }
