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
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct CoTaskMem;

impl meta::Meta for CoTaskMem {
    type Error                  = ();
    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

unsafe impl thin::Alloc for CoTaskMem {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { CoTaskMemAlloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing CoTaskMemAlloc
}

unsafe impl thin::Realloc for CoTaskMem {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let new_size = super::check_size(new_size)?;
        let alloc = unsafe { CoTaskMemRealloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: usize) -> Result<AllocNN, Self::Error> {
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
        unsafe impl ialloc::fat::Alloc      for CoTaskMem => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for CoTaskMem => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for CoTaskMem => ialloc::thin::Free;
    }
}



#[test] fn test_nullable() {
    use crate::thin::Free;
    unsafe { CoTaskMem.free_nullable(core::ptr::null_mut()) }
}

#[test] fn test_align() {
    use crate::{meta::*, thin::*};
    for size in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        std::dbg!(size);
        let mut addr_bits = 0;
        for _ in 0 .. 1000 {
            let alloc = CoTaskMem.alloc_uninit(size).unwrap();
            addr_bits |= alloc.as_ptr() as usize;
            unsafe { CoTaskMem.free(alloc) };
        }
        let align = 1 << addr_bits.trailing_zeros(); // usually 16, occasionally 32+
        assert!(align >= CoTaskMem::MIN_ALIGN.as_usize());
        assert!(align >= CoTaskMem::MAX_ALIGN.as_usize());
    }
}



#[test] fn thin_zst_support() { thin::test::zst_supported_accurate(CoTaskMem) }
