use crate::*;
use crate::meta::*;

use core::alloc::Layout;
use core::mem::{MaybeUninit, size_of};
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
/// | [`fat::Alloc::alloc_uninit`]      | <code>[_aligned_malloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-malloc-dbg)}</code>†    | [`aligned_alloc`]
/// | [`fat::Alloc::alloc_zeroed`]      | <code>[_aligned_recalloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-recalloc-dbg)}</code>†| &emsp;&emsp;+ [`memset`]
/// | [`fat::Realloc::realloc_uninit`]  | <code>[_aligned_realloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-realloc-dbg)}</code>†  | [`realloc`] or [`aligned_alloc`] + [`memcpy`]
/// | [`fat::Realloc::realloc_zeroed`]  | <code>[_aligned_recalloc]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-recalloc-dbg)}</code>†| &emsp;&emsp;+ [`memset`]
/// | [`fat::Free::free`]               | <code>[_aligned_free]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-free-dbg)}</code>†        | [`free`] or [`free_aligned_sized`]† (C23)
/// | [`thin::Free::free`]              | <code>[_aligned_free]{,[~~_dbg~~](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/aligned-free-dbg)}</code>†        | [`free`]
///
/// ## † ⚠️ FFI Safety Caveats ⚠️
/// *   **OS X:** POSIX may require alignment to be a multiple of `sizeof(void*)`.  This impl rounds [`Layout`] alignments up to that, and size up to alignment, which may make it incompatible with naively passing the same values to [`free_aligned_sized`].
/// *   **Windows:** this uses `_aligned_*` which is *not* compatible with [`free`].
/// *   **Windows:** I reserve the right to call `_aligned_*_dbg` variants in the future if debug CRT support is added.
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct AlignedMalloc;

impl AlignedMalloc {
    /// Not all valid [`Layout`]s are valid on all implementations of this allocator.
    /// Notably, OS X has several (POSIX-sanctioned?) edge cases.
    /// This may increase alignment and/or size.
    fn fix_layout(layout: Layout) -> Result<Layout, ()> {
        if cfg!(target_os = "macos") {                                              // `aligned_alloc` is what I would consider to be a miserable pile of bugs, at least on 64-bit macOS 11.7.6 20G1231.
            if layout.align() > Self::MAX_ALIGN.as_usize() { return Err(()) }       // 1. it will "succeed" when requesting 4 GiB+ alignment - but only provide 2 GiB alignment.  Manually reject these bogus fulfillments of our requests.
            let layout = layout.align_to(size_of::<*const ()>()).map_err(|_| {})?;  // 2. it will fail if the requested alignment is less than 8.  Even for something like size=align=1.  For no good reason whatsoever.  So... increase alignment:
            let layout = layout.pad_to_align();                                     // 3. it might succeed if size=0, align=8, but will fail with size=1, align=8.  I'll interpret this as requiring size to be a multiple of alignment... make it so.
            // some of these spicy preconditions may stem from forwarding blindly to POSIX - see e.g. <https://man7.org/linux/man-pages/man3/posix_memalign.3.html> which requires, among other things, that `alignment` be a multiple of `sizeof(void *)`.
            Ok(layout)
        } else {
            Ok(layout)
        }
    }
}



// meta::*

impl Meta for AlignedMalloc {
    type Error                  = ();

    /// | Platform          | Value     |
    /// | ------------------| ----------|
    /// | OS X 64-bit       | 2 GiB (macOS 11.7.6 20G1231 [seems to](https://github.com/MaulingMonkey/ialloc/actions/runs/4999128292/jobs/8955213565) return only 2 GiB alignment when 4+ GiB is requested)
    /// | Linux 64-bit      | [`Alignment::MAX`] (2<sup>63</sup> B)
    /// | Windows 64-bit    | [`Alignment::MAX`] (2<sup>63</sup> B)
    /// | \* 32-bit         | [`Alignment::MAX`] (2 GiB)
    const MAX_ALIGN : Alignment = if cfg!(target_os = "macos") { ALIGN_MIN_2_GiB_MAX } else { Alignment::MAX };
    // MSVC MIN_ALIGN is 4 ..= 8

    const MAX_SIZE  : usize     = usize::MAX;

    /// | Platform          | Behavior |
    /// | ------------------| ---------|
    /// | Linux             | Succeeds?
    /// | Windows           | Fails?  [`_aligned_malloc`] explicitly documents "If \[...\] `size` is zero, this function invokes the invalid parameter handler, as described in [Parameter validation](https://learn.microsoft.com/en-us/cpp/c-runtime-library/parameter-validation). If execution is allowed to continue, this function returns `NULL` and sets `errno` to `EINVAL`."
    ///
    #[doc = include_str!("_refs.md")]
    const ZST_SUPPORTED : bool  = false;
}

// SAFETY: ✔️ global state only
unsafe impl Stateless for AlignedMalloc {}



// fat::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`fat::test::alignment`]
/// | `size`        | ✔️ Validated via [`fat::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`AlignedMalloc`] is `'static` - allocations by [`_aligned_malloc`] (MSVC) / [`aligned_alloc`] (C11) live until [`_aligned_free`] (MSVC) / [`free`]d (C89)
/// | `compatible`  | ⚠️ [`AlignedMalloc`] uses exclusively intercompatible fns - see type-level docs for details, especially "FFI Safety Caveats"
/// | `exclusive`   | ✔️ Allocations by [`_aligned_malloc`] / [`aligned_alloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`_aligned_malloc`] / [`aligned_alloc`] throw no exceptions (C API) and return null on error (possibly setting `errno`)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
/// | `zeroed`      | ✔️ Validated via [`fat::test::zeroed_alloc`]
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl fat::Alloc for AlignedMalloc {
    #[track_caller] fn alloc_uninit(&self, layout: Layout) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        let layout = Self::fix_layout(layout)?;

        // SAFETY: ✔️ `layout` has been checked/fixed for platform validity
        #[cfg(    target_env = "msvc") ] let alloc = unsafe { ffi::_aligned_malloc(layout.size(), layout.align()) };

        // SAFETY: ✔️ `layout` has been checked/fixed for platform validity
        #[cfg(not(target_env = "msvc"))] let alloc = unsafe { ffi::aligned_alloc(layout.align(), layout.size()) };

        NonNull::new(alloc.cast()).ok_or(())
    }

    #[cfg(target_env = "msvc")]
    #[track_caller] fn alloc_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        let layout = Self::fix_layout(layout)?;
        // SAFETY: ✔️ `layout` has been checked/fixed for platform validity
        let alloc = unsafe { ffi::_aligned_recalloc(core::ptr::null_mut(), 1, layout.size(), layout.align()) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ⚠️ [`AlignedMalloc`] uses exclusively intercompatible fns - see type-level docs for details, especially "FFI Safety Caveats"
/// | `exceptions`  | ✔️ [`_aligned_free`] / [`free`] / [`free_aligned_sized`] throw no exceptions (C API) and return no errors.
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl fat::Free for AlignedMalloc {
    #[track_caller] unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, _layout: Layout) {
        // SAFETY: ✔️ `ptr` belongs to `self` per [`fat::Free::free`]'s documented safety preconditions
        // SAFETY: ✔️ `_layout` may have been modified by `Self::fix_layout`, so immediately shadow it to avoid bugs:
        let _layout = Self::fix_layout(_layout);

        #[cfg(all(c23, not(target_env = "msvc")))] if let Ok(_layout) = _layout {
            // SAFETY: per above
            return unsafe { ffi::free_aligned_sized(ptr.as_ptr().cast(), _layout.align(), _layout.size()) };
        }

        // SAFETY: per above
        #[cfg(not(target_env = "msvc"))] unsafe { ffi::free(ptr.as_ptr().cast()) }

        // SAFETY: per above
        #[cfg(target_env = "msvc")] unsafe { ffi::_aligned_free(ptr.as_ptr().cast()) }
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`AlignedMalloc`] is `'static` - reallocations by [`_aligned_realloc`] (MSVC) / [`free`]+[`aligned_alloc`] (C11) live until [`_aligned_free`] (MSVC) / [`free`]d (C89)
/// | `compatible`  | ⚠️ [`AlignedMalloc`] uses exclusively intercompatible fns - see type-level docs for details, especially "FFI Safety Caveats"
/// | `exclusive`   | ✔️ Allocations by [`_aligned_realloc`] / [`free`]+[`aligned_alloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`_aligned_realloc`] / [`free`]+[`aligned_alloc`] throw no exceptions (C API) and return null on error (possibly setting `errno`)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
/// | `zeroed`      | ⚠️
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl fat::Realloc for AlignedMalloc {
    #[cfg(target_env = "msvc")]
    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, _old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        let new_layout = Self::fix_layout(new_layout)?;
        // SAFETY: ✔️ `ptr` belongs to `self` per [`fat::Realloc::realloc_uninit`]'s documented safety preconditions
        // SAFETY: ✔️ `new_layout` has been checked/fixed for platform validity
        let alloc = unsafe { ffi::_aligned_realloc(ptr.as_ptr().cast(), new_layout.size(), new_layout.align()) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[cfg(target_env = "msvc")]
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, _old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        let new_layout = Self::fix_layout(new_layout)?;
        // SAFETY: ✔️ `ptr` belongs to `self` per [`fat::Realloc::realloc_zeroed`]'s documented safety preconditions
        // SAFETY: ✔️ `new_layout` has been checked/fixed for platform validity
        let alloc = unsafe { ffi::_aligned_recalloc(ptr.as_ptr().cast(), 1, new_layout.size(), new_layout.align()) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}



// thin::*

// thin::{Alloc, Realloc, ReallocZeroed} could infer an alignment, but that seems like a mild possible footgun

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ⚠️ [`AlignedMalloc`] uses exclusively intercompatible fns - see type-level docs for details, especially "FFI Safety Caveats"
/// | `exceptions`  | ✔️ [`_aligned_free`] / [`free`] / [`free_aligned_sized`] throw no exceptions (C API) and return no errors.
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Free for AlignedMalloc {
    #[track_caller] unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (C89 § 7.20.3.2 ¶ 2, validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`fat::Realloc::realloc_zeroed`]'s documented safety preconditions
        #[cfg(not(target_env = "msvc"))] unsafe { ffi::free(ptr.cast()) }
        // SAFETY: per above
        #[cfg(    target_env = "msvc") ] unsafe { ffi::_aligned_free(ptr.cast()) }
    }
}

// thin::SizeOf{,Debug} is not applicable: _aligned_msize requires alignment/offset, which isn't available for thin::SizeOf::size_of



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



#[test] fn fat_alignment()              { fat::test::alignment(AlignedMalloc) }
#[test] fn fat_edge_case_sizes()        { fat::test::edge_case_sizes(AlignedMalloc) }
#[test] fn fat_uninit()                 { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(AlignedMalloc) } } } // malloc returns zeroed memory on some platforms
#[test] fn fat_zeroed()                 { fat::test::zeroed_alloc(AlignedMalloc) }
#[test] fn fat_zst_support()            { fat::test::zst_supported_conservative(AlignedMalloc) }
