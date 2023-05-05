use crate::*;

use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::winbase::{LocalAlloc, LocalReAlloc, LocalFree, LocalSize};
use winapi::um::minwinbase::LMEM_ZEROINIT;

use core::num::NonZeroUsize;
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
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Local;

unsafe impl thin::Alloc for Local {
    type Error = ();

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { LocalAlloc(0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<AllocNN0, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { LocalAlloc(LMEM_ZEROINIT, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Realloc for Local {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { LocalReAlloc(ptr.as_ptr().cast(), size, 0) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { LocalReAlloc(ptr.as_ptr().cast(), size, LMEM_ZEROINIT) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Free for Local {
    unsafe fn free(&self, ptr: AllocNN) {
        assert!(unsafe { LocalFree(ptr.as_ptr().cast()) }.is_null());
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



// TODO: test/improve alignment?
// TODO: check if nullable friendly?
