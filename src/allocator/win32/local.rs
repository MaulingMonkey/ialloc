use crate::*;
use crate::meta::*;
use super::Error;

use winapi::um::winbase::{LocalAlloc, LocalReAlloc, LocalFree, LocalSize};
use winapi::um::minwinbase::LMEM_ZEROINIT;

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// [`LocalAlloc`] / [`LocalReAlloc`] / [`LocalFree`] / [`LocalSize`]
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[`LocalAlloc`](0, size)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[`LocalAlloc`](LMEM_ZEROINIT, size)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[`LocalReAlloc`](ptr, size, 0)</code>
/// | [`thin::Realloc::realloc_zeroed`]         | <code>[`LocalReAlloc`](ptr, size, LMEM_ZEROINIT)</code>
/// | [`thin::Free::free`]                      | [`LocalFree`]
/// | [`thin::SizeOf::size_of`]                 | [`LocalSize`]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Local;



// meta::*

impl Meta for Local {
    type Error                  = Error;
    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = true;
}

impl ZstSupported for Local {}

// SAFETY: ✔️ global state only
unsafe impl Stateless for Local {}



// thin::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`], no oddball flags like [`LMEM_MOVEABLE`] that would ruin dereferencability of the returned allocation
/// | `pin`         | ✔️ [`Local`] is `'static` - allocations by [`LocalAlloc`] live until [`LocalReAlloc`]ed or [`LocalFree`]d (as we don't use [`LMEM_MOVEABLE`])
/// | `compatible`  | ✔️ [`Local`] uses exclusively intercompatible `Local*` fns
/// | `exclusive`   | ✔️ Allocations by [`LocalAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`LocalAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`LocalAlloc`] *eventually* calls [`HeapAlloc`], without [`HEAP_NO_SERIALIZE`], which *should* be thread safe - as claimed by random Stack Overflow threads.
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], [`LMEM_ZEROINIT`] used appropriately
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for Local {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: per above
        let alloc = unsafe { LocalAlloc(0, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        // SAFETY: ✔️ `LMEM_ZEROINIT` should ensure the newly allocated memory is zeroed.
        let alloc = unsafe { LocalAlloc(LMEM_ZEROINIT, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`Local`] is `'static` - reallocations by [`LocalReAlloc`] live until [`LocalReAlloc`]ed again or [`LocalFree`]d
/// | `compatible`  | ✔️ [`Local`] uses exclusively intercompatible `Local*` fns
/// | `exclusive`   | ✔️ Allocations by [`LocalReAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`LocalReAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`LocalReAlloc`] *eventually* calls [`HeapReAlloc`] without [`HEAP_NO_SERIALIZE`], which *should* be thread safe...
/// | `zeroed`      | ⚠️ untested, but we use [`LMEM_ZEROINIT`] appropriately...
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Realloc for Local {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions - and thus was allocated with `Local{,Re}Alloc` - which should be safe to `LocalReAlloc`.
        let alloc = unsafe { LocalReAlloc(ptr.as_ptr().cast(), new_size, 0) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `LMEM_ZEROINIT` should ensure the newly reallocated memory is zeroed.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions - and thus was allocated with `Local{,Re}Alloc` - which should be safe to `LocalReAlloc`.
        let alloc = unsafe { LocalReAlloc(ptr.as_ptr().cast(), new_size, LMEM_ZEROINIT) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`Local`] uses exclusively intercompatible `Local*` fns
/// | `exceptions`  | ✔️ [`LocalFree`] is "infalliable".  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`LocalFree`] *eventually* calls [`HeapFree`] without [`HEAP_NO_SERIALIZE`], which *should* be thread safe...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Free for Local {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions - and thus was allocated with `Local{,Re}Alloc`
        if unsafe { LocalFree(ptr.cast()) }.is_null() { return }
        if cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}

// SAFETY: ✔️ same preconditions as thin::SizeOfDebug
unsafe impl thin::SizeOf for Local {}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `size`        | ✔️ Validated via [`thin::test::size_exact_alloc`]
/// | `compatible`  | ✔️ [`Local`] uses exclusively intercompatible `Local*` fns
/// | `exceptions`  | ✔️ [`LocalSize`] returns `0` for errors.  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ⚠️ [`LocalSize`] *eventually* calls [`HeapSize`] without [`HEAP_NO_SERIALIZE`], which *should* be thread safe...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::SizeOfDebug for Local {
    unsafe fn size_of_debug(&self, ptr: AllocNN) -> Option<usize> {
        super::clear_last_error();
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::SizeOfDebug's documented safety preconditions - and thus was allocated with `Local{,Re}Alloc` - which should be safe to `LocalSize`.
        let size = unsafe { LocalSize(ptr.as_ptr().cast()) };
        if size == 0 {
            let err = super::get_last_error();
            if err != 0 { return None }
        }
        Some(size)
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, Local};

    impls! {
        unsafe impl ialloc::fat::Alloc      for Local => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for Local => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for Local => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(Local) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(Local) }
#[test] fn thin_nullable()          { thin::test::nullable(Local) }
#[test] fn thin_size()              { thin::test::size_exact_alloc(Local) }
#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(Local) } }
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(Local) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(Local) }

#[test] fn fat_alignment()          { fat::test::alignment(Local) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(Local) }
#[test] fn fat_uninit()             { unsafe { fat::test::uninit_alloc_unsound(Local) } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(Local) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(Local) }
