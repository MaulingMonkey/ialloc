use crate::*;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

/// "This function sets `errno` to `ENOMEM` if the memory allocation failed or if the requested size was greater than `_HEAP_MAXREQ`."<br>
/// <https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-malloc>
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
///
/// N.B. this is significantly larger than the isize::MAX supported by much of LLVM / Rust
const _HEAP_MAXREQ : usize = usize::MAX & !0x1F;



/// [`_aligned_malloc`] / [`_aligned_realloc`] / [`_aligned_free`] / ...
///
/// | Rust                              | MSVC Release CRT <br> ~~MSVC Debug CRT~~                                                                                              | !MSVC<br>C11 or C++17     |
/// | ----------------------------------| --------------------------------------------------------------------------------------------------------------------------------------| --------------------------|
/// | [`fat::Alloc::alloc_uninit`]      | <code>[_aligned_malloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-malloc-dbg)}</code>     | [`aligned_alloc`]
/// | [`fat::Alloc::alloc_zeroed`]      | <code>[_aligned_recalloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-recalloc-dbg)}</code> | &emsp;&emsp;+ [`memset`]
/// | [`fat::Realloc::realloc_uninit`]  | <code>[_aligned_realloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-realloc-dbg)}</code>   | [`realloc`] or [`aligned_alloc`] + [`memcpy`]
/// | [`fat::Realloc::realloc_zeroed`]  | <code>[_aligned_recalloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-recalloc-dbg)}</code> | &emsp;&emsp;+ [`memset`]
/// | [`fat::Free::free`]               | <code>[_aligned_free]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-free-dbg)}</code>         | [`free`] or [`free_aligned_sized`] (C23)
/// | [`thin::Free::free`]              | <code>[_aligned_free]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-free-dbg)}</code>         | [`free`]
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct AlignedMalloc;

impl meta::Meta for AlignedMalloc {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::MAX;
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = false;
    // MSVC MIN_ALIGN is 4 ..= 8
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Alloc for AlignedMalloc {
    #[track_caller] fn alloc_uninit(&self, layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        #[cfg(    target_env = "msvc") ] let alloc = unsafe { ffi::_aligned_malloc(layout.size(), layout.align()) };
        #[cfg(not(target_env = "msvc"))] let alloc = unsafe { ffi::aligned_alloc(layout.align(), layout.size()) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[cfg(target_env = "msvc")]
    #[track_caller] fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        let alloc = unsafe { ffi::_aligned_recalloc(core::ptr::null_mut(), 1, layout.size(), layout.align()) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Free for AlignedMalloc {
    #[track_caller] unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, _layout: Layout) {
        #[cfg(target_env = "msvc")] unsafe { ffi::_aligned_free(ptr.as_ptr().cast()) }
        #[cfg(not(target_env = "msvc"))] unsafe {
            #[cfg(c23)] ffi::free_aligned_sized(ptr.as_ptr().cast(), _layout.align(), _layout.size());
            #[allow(dead_code)] ffi::free(ptr.as_ptr().cast());
        }
    }
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Realloc for AlignedMalloc {
    #[cfg(target_env = "msvc")]
    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, _old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { ffi::_aligned_realloc(ptr.as_ptr().cast(), new_layout.size(), new_layout.align()) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[cfg(target_env = "msvc")]
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, _old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { ffi::_aligned_recalloc(ptr.as_ptr().cast(), 1, new_layout.size(), new_layout.align()) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}



// thin::{Alloc, Realloc, ReallocZeroed} could infer an alignment, but that seems like a mild possible footgun

unsafe impl thin::Free for AlignedMalloc {
    #[track_caller] unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        #[cfg(    target_env = "msvc") ] unsafe { ffi::_aligned_free(ptr.cast()) }
        #[cfg(not(target_env = "msvc"))] unsafe { ffi::free(ptr.cast()) }
    }
}

// thin::SizeOf is not applicable: _aligned_msize requires alignment/offset, which isn't available for thin::SizeOf::size_of



mod ffi {
    pub use libc::*;
    #[allow(unused_imports)] use core::ptr::NonNull;

    #[cfg(not(target_env = "msvc"))] extern "C" {
        #[cfg(any(c11, cpp17))] pub fn aligned_alloc(alignment: size_t, size: size_t) -> *mut c_void;
        #[cfg(any(c23       ))] pub fn free_sized(ptr: *mut c_void, size: size_t) -> *mut c_void;
        #[cfg(any(c23       ))] pub fn free_aligned_sized(ptr: *mut c_void, alignment: size_t, size: size_t) -> *mut c_void;
    }

    #[cfg(target_env = "msvc")] extern "cdecl" {
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
    #[cfg(target_env = "msvc")] extern "cdecl" {
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
