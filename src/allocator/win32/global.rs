use crate::*;

use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::winbase::{GlobalAlloc, GlobalReAlloc, GlobalFree, GlobalSize, GMEM_ZEROINIT};

use core::num::NonZeroUsize;
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
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Global;

unsafe impl thin::Alloc for Global {
    type Error = ();

    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { GlobalAlloc(0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<AllocNN0, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { GlobalAlloc(GMEM_ZEROINIT, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Realloc for Global {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), size, 0) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), size, GMEM_ZEROINIT) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Free for Global {
    unsafe fn free(&self, ptr: AllocNN) {
        assert!(unsafe { GlobalFree(ptr.as_ptr().cast()) }.is_null());
    }
}

unsafe impl thin::SizeOf for Global {}
unsafe impl thin::SizeOfDebug for Global {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> {
        unsafe { SetLastError(0) };
        let size = unsafe { GlobalSize(ptr.as_ptr().cast()) };
        if size == 0 {
            let err = unsafe { GetLastError() };
            if err != 0 { return None }
        }
        Some(size)
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, Global};

    impls! {
        unsafe impl ialloc::nzst::Alloc     for Global => ialloc::thin::Alloc;
        unsafe impl ialloc::nzst::Realloc   for Global => ialloc::thin::Realloc;
        unsafe impl ialloc::nzst::Free      for Global => ialloc::thin::Free;
    }
}



// TODO: test/improve alignment?
// TODO: check if nullable friendly?
