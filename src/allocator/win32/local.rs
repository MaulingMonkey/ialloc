use crate::*;

use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::winbase::{LocalAlloc, LocalReAlloc, LocalFree, LocalSize};
use winapi::um::minwinbase::LMEM_ZEROINIT;

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// [`LocalAlloc`] / [`LocalReAlloc`] / [`LocalFree`] / [`LocalSize`]
/// (prefer [`ProcessHeap`](crate::allocator::win32::ProcessHeap)† unless required by doc)
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
/// ## † Legacy Notes
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

impl meta::Meta for Local {
    type Error = ();

    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

unsafe impl thin::Alloc for Local {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { LocalAlloc(0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { LocalAlloc(LMEM_ZEROINIT, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Realloc for Local {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { LocalReAlloc(ptr.as_ptr().cast(), size, 0) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { LocalReAlloc(ptr.as_ptr().cast(), size, LMEM_ZEROINIT) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Free for Local {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        if !unsafe { LocalFree(ptr.cast()) }.is_null() && cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}

unsafe impl thin::SizeOf for Local {}
unsafe impl thin::SizeOfDebug for Local {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> {
        unsafe { SetLastError(0) };
        let size = unsafe { LocalSize(ptr.as_ptr().cast()) };
        if size == 0 {
            let err = unsafe { GetLastError() };
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



#[test] fn thin_alignment()     { thin::test::alignment(Local) }
#[test] fn thin_nullable()      { thin::test::nullable(Local) }
#[test] fn thin_zst_support()   { thin::test::zst_supported_accurate(Local) }
