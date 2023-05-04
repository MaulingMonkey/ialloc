use crate::*;

use core::mem::MaybeUninit;
use core::ptr::{NonNull, null_mut};

/// > This function sets `errno` to `ENOMEM` if the memory allocation failed or if the requested size was greater than `_HEAP_MAXREQ`.
/// >
/// > <https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-malloc>
///
/// ```cpp
/// // Maximum heap request the heap manager will attempt
/// #ifdef _WIN64
///     #define _HEAP_MAXREQ 0xFFFFFFFFFFFFFFE0
/// #else
///     #define _HEAP_MAXREQ 0xFFFFFFE0
/// #endif
/// ```
///
/// `C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\ucrt\malloc.h`
const _HEAP_MAXREQ : usize = usize::MAX & !0x1F;



/// [`_aligned_malloc`] / [`_aligned_realloc`] / [`_aligned_free`] / ...
///
/// | Rust                              | C (Release CRT)       | ~~Debug CRT~~ (N/A)       |
/// | ----------------------------------| ----------------------| --------------------------|
/// | [`nzst::Alloc::alloc_uninit`]     | [`_aligned_malloc`]   | [`_aligned_malloc_dbg`]   |
/// | [`nzst::Alloc::alloc_zeroed`]     | [`_aligned_recalloc`] | [`_aligned_recalloc_dbg`] |
/// | [`nzst::Realloc::realloc_uninit`] | [`_aligned_realloc`]  | [`_aligned_realloc_dbg`]  |
/// | [`nzst::Realloc::realloc_zeroed`] | [`_aligned_recalloc`] | [`_aligned_recalloc_dbg`] |
/// | [`nzst::Free::free`]              | [`_aligned_free`]     | [`_aligned_free_dbg`]     |
/// | [`thin::Free::free`]              | [`_aligned_free`]     | [`_aligned_free_dbg`]     |
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct AlignedMalloc;

unsafe impl nzst::Alloc for AlignedMalloc {
    type Error = ();

    #[track_caller] fn alloc_uninit(&self, layout: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let alloc = unsafe { _aligned_malloc(layout.size().get(), layout.align().as_usize()) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[track_caller] fn alloc_zeroed(&self, layout: LayoutNZ) -> Result<NonNull<u8>, Self::Error> {
        let alloc = unsafe { _aligned_recalloc(null_mut(), 1, layout.size().get(), layout.align().as_usize()) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl nzst::Free for AlignedMalloc {
    #[track_caller] unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, _layout: LayoutNZ) {
        unsafe { _aligned_free(ptr.as_ptr().cast()) }
    }
}

unsafe impl nzst::Realloc for AlignedMalloc {
    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, _old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { _aligned_realloc(ptr.as_ptr().cast(), new_layout.size().get(), new_layout.align().as_usize()) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, _old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { _aligned_recalloc(ptr.as_ptr().cast(), 1, new_layout.size().get(), new_layout.align().as_usize()) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}



// thin::{Alloc, Realloc, ReallocZeroed} could infer an alignment, but that seems like a mild possible footgun

unsafe impl thin::FreeNullable for AlignedMalloc {
    #[track_caller] unsafe fn free(&self, ptr: *mut MaybeUninit<u8>) {
        unsafe { _aligned_free(ptr.cast()) }
    }
}

// thin::SizeOf is not applicable: _aligned_msize requires alignment/offset, which isn't available for thin::SizeOf::size_of



use ffi::*;
mod ffi {
    use libc::*;
    use core::ptr::NonNull;

    extern "cdecl" {
        // C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\ucrt\corecrt_malloc.h
        pub fn _aligned_free(block: *mut c_void);
        pub fn _aligned_malloc(size: size_t, alignment: size_t) -> *mut c_void;
        pub fn _aligned_offset_malloc(size: size_t, alignment: size_t, offset: size_t) -> *mut c_void;
        pub fn _aligned_msize(block: NonNull<c_void>, alignment: size_t, offset: size_t) -> size_t;
        pub fn _aligned_offset_realloc(block: *mut c_void, size: size_t, alignment: size_t, offset: size_t) -> *mut c_void;
        pub fn _aligned_offset_recalloc(block: *mut c_void, count: size_t, size: size_t, alignment: size_t, offset: size_t) -> *mut c_void;
        pub fn _aligned_realloc(block: *mut c_void, size: size_t, alignment: size_t) -> *mut c_void;
        pub fn _aligned_recalloc(block: *mut c_void, count: size_t, size: size_t, alignment: size_t) -> *mut c_void;
    }

    #[cfg(never)] // XXX: rustc always links against non-debug Windows runtime: https://github.com/rust-lang/rust/issues/39016
    extern "cdecl" {
        // C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\ucrt\crtdbg.h
        pub fn _aligned_free_dbg(block: *mut c_void);
        pub fn _aligned_malloc_dbg(size: size_t, alignment: size_t, file_name: abistr::CStrPtr<u8>, line_number: c_int) -> *mut c_void;
        pub fn _aligned_msize_dbg(block: NonNull<c_void>, alignment: size_t, offset: size_t) -> size_t;
        pub fn _aligned_offset_malloc_dbg(size: size_t, alignment: size_t, offset: size_t, file_name: abistr::CStrPtr<u8>, line_number: c_int) -> *mut c_void;
        pub fn _aligned_offset_realloc_dbg(block: *mut c_void, size: size_t, alignment: size_t, offset: size_t, file_name: abistr::CStrPtr<u8>, line_number: c_int) -> *mut c_void;
        pub fn _aligned_offset_recalloc_dbg(block: *mut c_void, count: size_t, size: size_t, alignment: size_t, offset: size_t, file_name: abistr::CStrPtr<u8>, line_number: c_int) -> *mut c_void;
        pub fn _aligned_realloc_dbg(block: *mut c_void, size: size_t, alignment: size_t, file_name: abistr::CStrPtr<u8>, line_number: c_int) -> *mut c_void;
        pub fn _aligned_recalloc_dbg(block: *mut c_void, count: size_t, size: size_t, alignment: size_t, file_name: abistr::CStrPtr<u8>, line_number: c_int) -> *mut c_void;
    }
}
