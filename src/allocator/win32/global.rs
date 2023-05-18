use crate::*;
use super::Error;

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
    type Error                  = Error;
    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = true;
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`], no oddball flags like [`GMEM_MOVEABLE`] that would ruin dereferencability of the returned allocation
/// | `pin`         | ✔️ [`Global`] is `'static` - allocations by [`GlobalAlloc`] live until [`GlobalReAlloc`]ed or [`GlobalFree`]d (as we don't use [`GMEM_MOVEABLE`])
/// | `compatible`  | ✔️ [`Global`] uses exclusively intercompatible `Global*` fns
/// | `exclusive`   | ✔️ Allocations by [`GlobalAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`GlobalAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`GlobalAlloc`] *eventually* calls [`HeapAlloc`], without [`HEAP_NO_SERIALIZE`], which *should* be thread safe - as claimed by random Stack Overflow threads.
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], [`GMEM_ZEROINIT`] used appropriately
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for Global {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: per above
        let alloc = unsafe { GlobalAlloc(0, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        // SAFETY: ✔️ `GMEM_ZEROINIT` should ensure the newly allocated memory is zeroed.
        let alloc = unsafe { GlobalAlloc(GMEM_ZEROINIT, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`Global`] is `'static` - reallocations by [`GlobalReAlloc`] live until [`GlobalReAlloc`]ed again or [`GlobalFree`]d
/// | `compatible`  | ✔️ [`Global`] uses exclusively intercompatible `Global*` fns
/// | `exclusive`   | ✔️ Allocations by [`GlobalReAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`GlobalReAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`GlobalReAlloc`] *eventually* calls [`HeapReAlloc`] without [`HEAP_NO_SERIALIZE`], which *should* be thread safe...
/// | `zeroed`      | ⚠️ untested, but we use [`GMEM_ZEROINIT`] appropriately...
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Realloc for Global {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions - and thus was allocated with `Global{,Re}Alloc` - which should be safe to `GlobalReAlloc`.
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), new_size, 0) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `GMEM_ZEROINIT` should ensure the newly reallocated memory is zeroed.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions - and thus was allocated with `Global{,Re}Alloc` - which should be safe to `GlobalReAlloc`.
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), new_size, GMEM_ZEROINIT) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`Global`] uses exclusively intercompatible `Global*` fns
/// | `exceptions`  | ✔️ [`GlobalFree`] is "infalliable".  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`GlobalFree`] *eventually* calls [`HeapFree`] without [`HEAP_NO_SERIALIZE`], which *should* be thread safe...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Free for Global {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions - and thus was allocated with `Global{,Re}Alloc`
        if unsafe { GlobalFree(ptr.cast()) }.is_null() { return }
        if cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}

// SAFETY: ✔️ same preconditions as thin::SizeOfDebug
unsafe impl thin::SizeOf for Global {}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `size`        | ✔️ Validated via [`thin::test::size_exact_alloc`]
/// | `compatible`  | ✔️ [`Global`] uses exclusively intercompatible `Global*` fns
/// | `exceptions`  | ✔️ [`GlobalSize`] returns `0` for errors.  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`GlobalSize`] *eventually* calls [`HeapSize`] without [`HEAP_NO_SERIALIZE`], which *should* be thread safe...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::SizeOfDebug for Global {
    unsafe fn size_of_debug(&self, ptr: AllocNN) -> Option<usize> {
        super::clear_last_error();
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
