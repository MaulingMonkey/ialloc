use crate::*;

use winapi::um::winbase::{GlobalAlloc, GlobalReAlloc, GlobalFree, GlobalSize, GMEM_ZEROINIT};

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// [`GlobalAlloc`] / [`GlobalReAlloc`] / [`GlobalFree`] / [`GlobalSize`]
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[`GlobalAlloc`](0, size)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[`GlobalAlloc`](GMEM_ZEROINIT, size)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[`GlobalReAlloc`](ptr, size, 0)</code>
/// | [`thin::Realloc::realloc_zeroed`]         | <code>[`GlobalReAlloc`](ptr, size, GMEM_ZEROINIT)</code>
/// | [`thin::Free::free`]                      | [`GlobalFree`]
/// | [`thin::SizeOf::size_of`]                 | [`GlobalSize`]
///
/// ## Legacy Notes
///
/// "The global and local functions are supported for porting from 16-bit code, or for maintaining source code compatibility with 16-bit Windows.
/// Starting with 32-bit Windows, the global and local functions are implemented as wrapper functions that call the corresponding [heap functions] using a handle to the process's default heap.
/// Therefore, the global and local functions have greater overhead than other memory management functions."
///
/// "The [heap functions] provide more features and control than the global and local functions.
/// New applications should use the heap functions unless documentation specifically states that a global or local function should be used.
/// For example, some Windows functions allocate memory that must be freed with [`LocalFree`], and the global functions are still used with Dynamic Data Exchange (DDE), the clipboard functions, and OLE data objects.
/// For a complete list of global and local functions, see the table in [Memory Management Functions](https://learn.microsoft.com/en-us/windows/win32/memory/memory-management-functions)."
///
/// <https://learn.microsoft.com/en-us/windows/win32/memory/global-and-local-functions>
///
/// [heap functions]:   https://learn.microsoft.com/en-us/windows/win32/memory/heap-functions
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Global;

impl meta::Meta for Global {
    type Error                  = ();
    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = true;
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for Global {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ this "should" be thread safe according to random SO threads, and the underlying Heap* allocs are, but it'd be worth #[test]ing.
        // SAFETY: ✔️ this "should" be safe for all `size`.  This is #[test]ed for at the end of this file.
        // SAFETY: ✔️ no oddball flags like `GMEM_MOVEABLE` that would ruin dereferencability of the returned `alloc`
        let alloc = unsafe { GlobalAlloc(0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        // SAFETY: ⚠️ this "should" be thread safe according to random SO threads, and the underlying Heap* allocs are, but it'd be worth #[test]ing.
        // SAFETY: ✔️ this "should" be safe for all `size`.  This is #[test]ed for at the end of this file.
        // SAFETY: ✔️ no oddball flags like `GMEM_MOVEABLE` that would ruin dereferencability of the returned `alloc`
        // SAFETY: ✔️ `GMEM_ZEROINIT` should ensure the newly allocated memory is zeroed.
        let alloc = unsafe { GlobalAlloc(GMEM_ZEROINIT, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Realloc for Global {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ this "should" be thread safe according to random SO threads, and the underlying Heap* allocs are, but it'd be worth #[test]ing.
        // SAFETY: ⚠️ this "should" be safe for all `size`.  This is not yet #[test]ed.
        // SAFETY: ✔️ no oddball flags like `GMEM_MOVEABLE` that would ruin dereferencability of the returned `alloc`
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions - and thus was allocated with `Global{,Re}Alloc` - which should be safe to `GlobalReAlloc`.
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), new_size, 0) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ this "should" be thread safe according to random SO threads, and the underlying Heap* allocs are, but it'd be worth #[test]ing.
        // SAFETY: ⚠️ this "should" be safe for all `size`.  This is not yet #[test]ed.
        // SAFETY: ✔️ no oddball flags like `GMEM_MOVEABLE` that would ruin dereferencability of the returned `alloc`
        // SAFETY: ✔️ `GMEM_ZEROINIT` should ensure the newly allocated memory is zeroed.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions - and thus was allocated with `Global{,Re}Alloc` - which should be safe to `GlobalReAlloc`.
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), new_size, GMEM_ZEROINIT) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for Global {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ⚠️ this "should" be thread safe according to random SO threads, and the underlying Heap* allocs are, but it'd be worth #[test]ing.
        // SAFETY: ✔️ `ptr` is either `nullptr` (safe), or belongs to `self` per thin::Free::free_nullable's documented safety preconditions - and thus was allocated with `Global{,Re}Alloc` - which should be safe to `GlobalFree`.
        if !unsafe { GlobalFree(ptr.cast()) }.is_null() && cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::SizeOf for Global {}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::SizeOfDebug for Global {
    unsafe fn size_of_debug(&self, ptr: AllocNN) -> Option<usize> {
        super::clear_last_error();
        // SAFETY: ⚠️ this "should" be thread safe according to random SO threads, and the underlying Heap* allocs are, but it'd be worth #[test]ing.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::SizeOfDebug's documented safety preconditions - and thus was allocated with `Global{,Re}Alloc` - which should be safe to `GlobalSize`.
        let size = unsafe { GlobalSize(ptr.as_ptr().cast()) };
        if size == 0 {
            let err = super::get_last_error();
            if err != 0 { return None }
        }
        Some(size)
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, Global};

    impls! {
        unsafe impl ialloc::fat::Alloc      for Global => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for Global => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for Global => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(Global) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(Global) }
#[test] fn thin_nullable()          { thin::test::nullable(Global) }
#[test] fn thin_size()              { thin::test::size_over_alloc(Global) }
#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(Global) } }
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(Global) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(Global) }

#[test] fn fat_alignment()          { fat::test::alignment(Global) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(Global) }
#[test] fn fat_uninit()             { unsafe { fat::test::uninit_alloc_unsound(Global) } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(Global) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(Global) }
