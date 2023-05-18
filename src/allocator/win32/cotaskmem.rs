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
/// Uses the default COM / "OLE task memory" allocator provided by [`CoGetMalloc`], which in turn simply uses [`Heap*`](super::Heap) functions under the hood.
/// Consider using [`Heap`] directly instead, unless you're specifically doing COM / have documentation mandating a specific (de)allocator for interop purpouses.
///
/// ## References
/// *   [Memory Allocation in COM](https://learn.microsoft.com/en-us/windows/win32/learnwin32/memory-allocation-in-com) (learn.microsoft.com)
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct CoTaskMem;

impl meta::Meta for CoTaskMem {
    type Error                  = ();
    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = true;
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`CoTaskMem`] is `'static` - allocations by [`CoTaskMemAlloc`] live until [`CoTaskMemRealloc`]ed or [`CoTaskMemFree`]d
/// | `compatible`  | ✔️ [`CoTaskMem`] uses exclusively intercompatible fns
/// | `exclusive`   | ✔️ Allocations by [`CoTaskMemAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`CoTaskMemAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`CoTaskMemAlloc`] uses a no-init `gCMalloc::Alloc` → [`HeapAlloc`] with `dwFlags=0` (e.g. not using [`HEAP_NO_SERIALIZE`]) under the hood
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], trivial default impl
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for CoTaskMem {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: per above
        let alloc = unsafe { CoTaskMemAlloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing CoTaskMemAlloc
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`CoTaskMem`] is `'static` - reallocations by [`CoTaskMemRealloc`] live until [`CoTaskMemRealloc`]ed again or [`CoTaskMemFree`]d
/// | `compatible`  | ✔️ [`CoTaskMem`] uses exclusively intercompatible fns
/// | `exclusive`   | ✔️ Allocations by [`CoTaskMemRealloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`CoTaskMemRealloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`CoTaskMemRealloc`] uses a no-init `gCMalloc::Realloc` → [`HeapRealloc`] with `dwFlags=0` (e.g. not using [`HEAP_NO_SERIALIZE`]) under the hood
/// | `zeroed`      | ✔️ Trivial [`Err`] / not supported
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Realloc for CoTaskMem {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `ptr` belongs to `self` per [`thin::Realloc::realloc_uninit`]'s documented safety preconditions, and thus was allocated with CoTaskMem{Alloc,Realloc}
        let alloc = unsafe { CoTaskMemRealloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: usize) -> Result<AllocNN, Self::Error> {
        Err(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`CoTaskMem`] uses exclusively intercompatible fns
/// | `exceptions`  | ✔️ [`CoTaskMemFree`] is "infalliable".  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`CoTaskMemFree`] uses a no-init `gCMalloc::Free` → [`HeapFree`] with `dwFlags=0` (e.g. not using [`HEAP_NO_SERIALIZE`]) under the hood
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Free for CoTaskMem {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions - and thus was allocated with `CoTaskMem{Alloc,Realloc}`
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
//#[test] fn thin_size()              { ...no CoTaskMemSizeOf... }
#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(CoTaskMem) } }
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(CoTaskMem) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(CoTaskMem) }

#[test] fn fat_alignment()          { fat::test::alignment(CoTaskMem) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(CoTaskMem) }
#[test] fn fat_uninit()             { unsafe { fat::test::uninit_alloc_unsound(CoTaskMem) } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(CoTaskMem) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(CoTaskMem) }
