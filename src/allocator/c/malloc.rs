use crate::*;

use libc::*;

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// [`malloc`] / [`realloc`] / [`free`] / ...
///
/// | Rust                                      | C                     | MSVC<br>Only  |
/// | ------------------------------------------| ----------------------| --------------|
/// | [`thin::Alloc::alloc_uninit`]             | [`malloc`]            |               |
/// | [`thin::Alloc::alloc_zeroed`]             | [`calloc`]            |               |
/// | [`thin::Realloc::realloc_uninit`]         | [`realloc`]           |               |
/// | [`thin::Realloc::realloc_zeroed`]         | ❌ N/A               | [`_recalloc`] |
/// | [`thin::Free::free`]                      | [`free`]              |               |
/// | [`thin::SizeOfDebug::size_of`]            | `None`                | [`_msize`]    |
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Malloc;

impl Malloc {
    #[inline(always)] fn check_size(size: NonZeroUsize) -> Result<usize, ()> {
        // XXX: not entirely sure if this is excessive or not.
        // Is there any 32-bit platform for which malloc(2.5 GiB) succeeds?
        // My understanding is that >isize::MAX allocs/pointer ranges/spatial provenances are super cursed by LLVM / the compiler
        // https://doc.rust-lang.org/core/alloc/struct.Layout.html#method.from_size_align
        // https://doc.rust-lang.org/core/primitive.pointer.html#method.add
        let size = size.get();
        if size > usize::MAX/2 { return Err(()) }
        Ok(size)
    }
}

unsafe impl thin::Alloc for Malloc {
    type Error = ();

    #[track_caller] fn alloc_uninit(&self, size: NonZeroUsize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let size = Self::check_size(size)?;
        let alloc = unsafe { malloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[track_caller] fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<NonNull<u8>, Self::Error> {
        let size = Self::check_size(size)?;
        let alloc = unsafe { calloc(1, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Free for Malloc {
    #[track_caller] unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        unsafe { free(ptr.cast()) }
    }
}

unsafe impl thin::Realloc for Malloc {
    const CAN_REALLOC_ZEROED : bool = cfg!(target_env = "msvc");

    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let new_size = Self::check_size(new_size)?;
        let alloc = unsafe { realloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[track_caller] unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        #[cfg(target_env = "msvc")] {
            // https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/recalloc
            extern "C" { fn _recalloc(memblock: *mut c_void, num: size_t, size: size_t) -> *mut c_void; }
            let new_size = Self::check_size(new_size)?;
            let alloc = unsafe { _recalloc(ptr.as_ptr().cast(), 1, new_size) };
            NonNull::new(alloc.cast()).ok_or(())
        }
        #[cfg(not(target_env = "msvc"))] {
            let _ = (ptr, new_size);
            Err(())
        }
    }
}

unsafe impl thin::SizeOfDebug for Malloc {
    unsafe fn size_of(&self, _ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> {
        #[cfg(target_env = "msvc")] {
            // https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/msize
            extern "C" { fn _msize(memblock: *mut c_void) -> size_t; }
            let size = unsafe { _msize(_ptr.as_ptr().cast()) };
            if size == !0 { return None } // error - but only if `_ptr` was null (impossible)?
            if size != 0 { return Some(size) }
        }

        None
    }
}



#[cfg(allocator_api = "*")] #[test] fn allocator_api() {
    use crate::allocator::{adapt::PanicOverAlign, c::Malloc};
    use alloc::vec::Vec;

    let mut v = Vec::new_in(PanicOverAlign(Malloc));
    v.push(1);
    v.push(2);
    v.push(3);
    let v2 = v.clone();
    assert_eq!(3, v.len());
    assert_eq!(3, v2.len());
}
