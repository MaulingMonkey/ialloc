use crate::*;

use winapi::um::errhandlingapi::{GetLastError, SetLastError};
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
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Global;

impl meta::Meta for Global {
    type Error                  = ();
    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

unsafe impl thin::Alloc for Global {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { GlobalAlloc(0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { GlobalAlloc(GMEM_ZEROINIT, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Realloc for Global {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), size, 0) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(new_size)?;
        let alloc = unsafe { GlobalReAlloc(ptr.as_ptr().cast(), size, GMEM_ZEROINIT) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Free for Global {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        assert!(unsafe { GlobalFree(ptr.cast()) }.is_null());
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
        unsafe impl ialloc::fat::Alloc      for Global => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for Global => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for Global => ialloc::thin::Free;
    }
}



#[test] fn test_nullable() {
    use crate::thin::Free;
    unsafe { Global.free_nullable(core::ptr::null_mut()) }
}

#[test] fn test_align() {
    use crate::{meta::*, thin::*};
    for size in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        std::dbg!(size);
        let mut addr_bits = 0;
        for _ in 0 .. 1000 {
            let alloc = Global.alloc_uninit(size).unwrap();
            addr_bits |= alloc.as_ptr() as usize;
            unsafe { Global.free(alloc) };
        }
        let align = 1 << addr_bits.trailing_zeros(); // usually 16, occasionally 32+
        assert!(align >= Global::MIN_ALIGN.as_usize());
        assert!(align >= Global::MAX_ALIGN.as_usize());
    }
}



#[test] fn thin_zst_support() { thin::test::zst_supported_accurate(Global) }
