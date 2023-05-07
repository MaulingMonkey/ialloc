use crate::*;

use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::winbase::{LocalAlloc, LocalReAlloc, LocalFree, LocalSize};
use winapi::um::minwinbase::LMEM_ZEROINIT;

use core::mem::MaybeUninit;
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

    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing

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
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        assert!(unsafe { LocalFree(ptr.cast()) }.is_null());
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
        unsafe impl ialloc::nzst::Alloc     for Local => ialloc::thin::Alloc;
        unsafe impl ialloc::nzst::Realloc   for Local => ialloc::thin::Realloc;
        unsafe impl ialloc::nzst::Free      for Local => ialloc::thin::Free;
    }
}



#[test] fn test_nullable() {
    use crate::thin::Free;
    unsafe { Local.free_nullable(core::ptr::null_mut()) }
}

#[test] fn test_align() {
    use crate::thin::*;
    for size in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        let size = NonZeroUsize::new(size).unwrap();
        std::dbg!(size);
        let mut addr_bits = 0;
        for _ in 0 .. 1000 {
            let alloc = Local.alloc_uninit(size).unwrap();
            addr_bits |= alloc.as_ptr() as usize;
            unsafe { Local.free(alloc) };
        }
        let align = 1 << addr_bits.trailing_zeros(); // usually 16, occasionally 32+
        assert!(align >= Local::MIN_ALIGN.as_usize());
        assert!(align >= Local::MAX_ALIGN.as_usize());
    }
}
