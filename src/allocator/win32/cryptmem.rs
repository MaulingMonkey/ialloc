use crate::*;

use winapi::um::wincrypt::{CryptMemAlloc, CryptMemRealloc, CryptMemFree};

use core::num::NonZeroUsize;
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

unsafe impl thin::Alloc for CryptMem {
    type Error = ();

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { CryptMemAlloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing CryptMemAlloc
}

unsafe impl thin::Realloc for CryptMem {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let new_size = super::check_size(new_size)?;
        let alloc = unsafe { CryptMemRealloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        Err(())
    }
}

// no zeroing CryptMemRealloc

unsafe impl thin::Free for CryptMem {
    unsafe fn free(&self, ptr: AllocNN) {
        unsafe { CryptMemFree(ptr.as_ptr().cast()) }
    }
}



// TODO: test if CryptMemFree is nullable safe
// TODO: test/improve alignment?
