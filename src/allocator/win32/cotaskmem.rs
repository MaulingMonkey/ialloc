use crate::*;

use winapi::um::combaseapi::{CoTaskMemAlloc, CoTaskMemRealloc, CoTaskMemFree};

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// [`CoTaskMemAlloc`] / [`CoTaskMemRealloc`] / [`CoTaskMemFree`]
///
/// | Rust                              | C                     |
/// | ----------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]     | [`CoTaskMemAlloc`]
/// | [`thin::Realloc::realloc_uninit`] | [`CoTaskMemRealloc`]
/// | [`thin::Free::free`]              | [`CoTaskMemFree`]
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct CoTaskMem;

unsafe impl thin::Alloc for CoTaskMem {
    type Error = ();

    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { CoTaskMemAlloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing CoTaskMemAlloc
}

unsafe impl thin::Realloc for CoTaskMem {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let new_size = super::check_size(new_size)?;
        let alloc = unsafe { CoTaskMemRealloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        Err(())
    }
}

unsafe impl thin::Free for CoTaskMem {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        unsafe { CoTaskMemFree(ptr.cast()) }
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, CoTaskMem};

    impls! {
        unsafe impl ialloc::nzst::Alloc     for CoTaskMem => ialloc::thin::Alloc;
        unsafe impl ialloc::nzst::Realloc   for CoTaskMem => ialloc::thin::Realloc;
        unsafe impl ialloc::nzst::Free      for CoTaskMem => ialloc::thin::Free;
    }
}



// TODO: test/improve alignment?
//
// Docs boldly claim:
// > The storage space pointed to by the return value is guaranteed to be suitably aligned for storage of any type of object.
// > <https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cotaskmemrealloc>
// But I'm skeptical that means `alignof(max_align_t)` instead of "our x86 instructions won't fail"
