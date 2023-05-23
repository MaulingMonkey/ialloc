use crate::*;
use crate::meta::*;

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



// meta::*

impl Meta for CryptMem {
    type Error                  = ();
    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = u32::MAX as usize;
    const ZST_SUPPORTED : bool  = true;
}

impl ZstSupported for CryptMem {}

// SAFETY: ✔️ global state only
unsafe impl DefaultCompatible for CryptMem {}



// thin::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`CryptMem`] is `'static` - allocations by [`CryptMemAlloc`] live until [`CryptMemRealloc`]ed or [`CryptMemFree`]d
/// | `compatible`  | ✔️ [`CryptMem`] uses exclusively intercompatible fns
/// | `exclusive`   | ✔️ Allocations by [`CryptMemAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`CryptMemAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`CryptMemAlloc`] uses <code>[LocalAlloc]\([LMEM_ZEROINIT], size\)</code>, and [`Local`](super::Local) claims to be thread safe
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], trivial default impl
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for CryptMem {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        let size = size.try_into().map_err(|_| {})?;
        // SAFETY: per above
        let alloc = unsafe { CryptMemAlloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing CryptMemAlloc
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`CryptMem`] is `'static` - reallocations by [`CryptMemRealloc`] live until [`CryptMemRealloc`]ed again or [`CryptMemFree`]d
/// | `compatible`  | ✔️ [`CryptMem`] uses exclusively intercompatible fns
/// | `exclusive`   | ✔️ Allocations by [`CryptMemRealloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`CryptMemRealloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`CryptMemRealloc`] uses <code>[LocalReAlloc]\(ptr, size, [LMEM_ZEROINIT]\)</code>, and [`Local`](super::Local) claims to be thread safe
/// | `zeroed`      | ✔️ Trivial [`Err`] / not supported
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Realloc for CryptMem {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let new_size = new_size.try_into().map_err(|_| {})?;
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with CryptMem{Alloc,Realloc}
        let alloc = unsafe { CryptMemRealloc(ptr.as_ptr().cast(), new_size) };

        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: usize) -> Result<AllocNN, Self::Error> {
        Err(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`CryptMem`] uses exclusively intercompatible fns
/// | `exceptions`  | ✔️ [`CryptMemFree`] is "infalliable".  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`CryptMemFree`] uses <code>[LocalFree]\(...\)</code>, and [`Local`](super::Local) claims to be thread safe
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Free for CryptMem {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions - and thus was allocated with `CryptMem{Alloc,Realloc}`
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
//#[test] fn thin_size()              { ...no CryptMemSizeOf... }
//#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(CryptMem) } } // Interestingly, CryptMem appears to zero it's memory.  This isn't documented, so I choose not to rely on it, but it's interesting...
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(CryptMem) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(CryptMem) }

#[test] fn fat_alignment()          { fat::test::alignment(CryptMem) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(CryptMem) }
//#[test] fn fat_uninit()             { unsafe { fat::test::uninit_alloc_unsound(CryptMem) } } // CryptMem is always zeroed
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(CryptMem) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(CryptMem) }
