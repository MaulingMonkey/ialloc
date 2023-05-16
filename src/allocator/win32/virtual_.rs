use crate::*;
use super::Error;

use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{PAGE_READWRITE, MEM_COMMIT, MEM_RELEASE};

use core::ptr::{null_mut, NonNull};



/// [`VirtualAlloc`] / [`VirtualFree`]
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[VirtualAlloc](nullptr, size, [MEM_COMMIT], [PAGE_READWRITE])</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[VirtualAlloc](nullptr, size, [MEM_COMMIT], [PAGE_READWRITE])</code>
/// | [`thin::Free::free`]                      | <code>[VirtualFree](ptr, 0, [MEM_RELEASE])</code>
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct VirtualCommit;

impl meta::Meta for VirtualCommit {
    type Error = Error;

    /// ### References
    /// *   [What are the page sizes used by Windows on various processors?](https://devblogs.microsoft.com/oldnewthing/20210510-00/?p=105200) (The Old New Thing)
    const MIN_ALIGN : Alignment = ALIGN_4_KiB;
    const MAX_ALIGN : Alignment = ALIGN_4_KiB;
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = false;
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for VirtualCommit {
    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Error> {
        // SAFETY: ✔️ `lpAddress` is optional and may be null - we have no preference about allocation location
        // SAFETY: ✔️ `size` is in bytes and unbounded - will round up to the next page boundary.  `0` will fail, as #[test]ed.
        // SAFETY: ✔️ `MEM_COMMIT` is addressable.  We'd need `MEM_RESERVE` too if `lpAddress` weren't null.
        // SAFETY: ✔️ `PAGE_READWRITE` is the typical W^X-safe allocation access mode
        // SAFETY: ✔️ returned allocations, if successful, will have at least page alignment (4 KiB), although even larger is common (64 KiB on my machine)
        NonNull::new(unsafe { VirtualAlloc(null_mut(), size, MEM_COMMIT, PAGE_READWRITE) }.cast()).ok_or_else(Error::get_last)
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for VirtualCommit {
    unsafe fn free(&self, ptr: AllocNN) {
        // SAFETY: ✔️ `ptr` belongs to `self` and is thus `MEM_COMMIT` returned directly from `VirtualAlloc`
        // SAFETY: ✔️ `size=0` is required for `MEM_RELEASE` and frees the entire `VirtualAlloc`ed region
        let success = unsafe { VirtualFree(ptr.as_ptr().cast(), 0, MEM_RELEASE) };
        let result = if success == 0 { Err(Error::get_last()) } else { Ok(()) };
        result.expect("VirtualFree failed");
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl fat::Realloc for VirtualCommit {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, VirtualCommit};

    impls! {
        unsafe impl ialloc::fat::Alloc      for VirtualCommit => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Free       for VirtualCommit => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(VirtualCommit) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(VirtualCommit) }
#[test] fn thin_nullable()          { thin::test::nullable(VirtualCommit) }
//#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(VirtualCommit) } } // VirtualCommit is always zeroed
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(VirtualCommit) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(VirtualCommit) }
