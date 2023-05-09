use crate::*;

use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::winbase::{GlobalAlloc, GlobalReAlloc, GlobalFree, GlobalSize, GMEM_ZEROINIT};

use core::mem::MaybeUninit;
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

impl meta::Meta for Global {
    type Error                  = ();
    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = false;
}

unsafe impl thin::Alloc for Global {
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
        unsafe impl ialloc::nzst::Alloc     for Global => ialloc::thin::Alloc;
        unsafe impl ialloc::nzst::Realloc   for Global => ialloc::thin::Realloc;
        unsafe impl ialloc::nzst::Free      for Global => ialloc::thin::Free;

        unsafe impl ialloc::zsty::Alloc     for Global => ialloc::nzst::Alloc;
        unsafe impl ialloc::zsty::Realloc   for Global => ialloc::nzst::Realloc;
        unsafe impl ialloc::zsty::Free      for Global => ialloc::nzst::Free;
    }
}



#[test] fn test_nullable() {
    use crate::thin::Free;
    unsafe { Global.free_nullable(core::ptr::null_mut()) }
}

#[test] fn test_align() {
    use crate::{meta::*, thin::*};
    for size in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        let size = NonZeroUsize::new(size).unwrap();
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
