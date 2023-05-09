use crate::*;

use winapi::um::wincrypt::{CryptMemAlloc, CryptMemRealloc, CryptMemFree};

use core::mem::MaybeUninit;
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

impl meta::Meta for CryptMem {
    type Error                  = ();
    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = false;
}

unsafe impl thin::Alloc for CryptMem {
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

unsafe impl thin::Free for CryptMem {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        unsafe { CryptMemFree(ptr.cast()) }
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, CryptMem};

    impls! {
        unsafe impl ialloc::nzst::Alloc     for CryptMem => ialloc::thin::Alloc;
        unsafe impl ialloc::nzst::Realloc   for CryptMem => ialloc::thin::Realloc;
        unsafe impl ialloc::nzst::Free      for CryptMem => ialloc::thin::Free;

        unsafe impl ialloc::zsty::Alloc     for CryptMem => ialloc::nzst::Alloc;
        unsafe impl ialloc::zsty::Realloc   for CryptMem => ialloc::nzst::Realloc;
        unsafe impl ialloc::zsty::Free      for CryptMem => ialloc::nzst::Free;
    }
}



#[test] fn test_nullable() {
    use crate::thin::Free;
    unsafe { CryptMem.free_nullable(core::ptr::null_mut()) }
}

#[test] fn test_align() {
    use crate::{meta::*, thin::*};
    for size in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        let size = NonZeroUsize::new(size).unwrap();
        std::dbg!(size);
        let mut addr_bits = 0;
        for _ in 0 .. 1000 {
            let alloc = CryptMem.alloc_uninit(size).unwrap();
            addr_bits |= alloc.as_ptr() as usize;
            unsafe { CryptMem.free(alloc) };
        }
        let align = 1 << addr_bits.trailing_zeros(); // usually 16, occasionally 32+
        assert!(align >= CryptMem::MIN_ALIGN.as_usize());
        assert!(align >= CryptMem::MAX_ALIGN.as_usize());
    }
}
