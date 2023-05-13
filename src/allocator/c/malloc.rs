use crate::*;

use libc::*;

use core::mem::MaybeUninit;
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
/// | [`thin::SizeOfDebug::size_of_debug`]      | `None`                | [`_msize`]    |
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Malloc;

impl meta::Meta for Malloc {
    type Error = ();

    /// | Platform          | Value     |
    /// | ------------------| ----------|
    /// | Windows 32-bit    | [`8` according to Microsoft](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/malloc#return-value)
    /// | Windows 64-bit    | [`16` according to Microsoft](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/malloc#return-value)
    /// | C11               | <code>[_Alignof](https://en.cppreference.com/w/c/language/_Alignof)\([max_align_t](https://en.cppreference.com/w/c/types/max_align_t)\)</code>
    /// | C89               | <code>[_Alignof](https://en.cppreference.com/w/c/language/_Alignof)\(double\)</code>? ("... suitable for any object type with [fundamental alignment](https://en.cppreference.com/w/c/language/object#Alignment)")
    ///
    /// Many systems allow the developer to customize their implementation of `malloc`.
    /// Such custom implementations *could* provide less alignment than those described above.
    /// I consider such a thing to be a bug and undefined behavior *by the customizer*, likely to break a lot more than the code relying on this `MAX_ALIGN`.
    const MAX_ALIGN : Alignment = if cfg!(target_env = "msvc") {
        if core::mem::size_of::<usize>() >= 8 { ALIGN_16 } else { ALIGN_8 }
    } else {
        #[cfg(any(target_env = "msvc", not(c11), not(feature = "libc")))] #[allow(non_camel_case_types)] type max_align_t = f64;
        Alignment::of::<max_align_t>()
    };

    const MAX_SIZE : usize = usize::MAX; // *slightly* less in practice
    const ZST_SUPPORTED : bool = false; // platform behavior too inconsistent
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for Malloc {
    #[track_caller] fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsound C stdlibs are #[test]ed for at the end of this file.
        let alloc = unsafe { malloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[track_caller] fn alloc_zeroed(&self, size: usize) -> Result<NonNull<u8>, Self::Error> {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsound C stdlibs are #[test]ed for at the end of this file.
        // SAFETY: ✔️ `calloc` zeros memory
        let alloc = unsafe { calloc(1, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for Malloc {
    #[track_caller] unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ `ptr` is either `nullptr` (safe), or belongs to `self` per thin::Free::free_nullable's documented safety preconditions - and thus was allocated with one of `malloc`, `calloc`, `realloc, or `_recalloc` - all of which should be safe to `free`.
        unsafe { free(ptr.cast()) }
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Realloc for Malloc {
    const CAN_REALLOC_ZEROED : bool = cfg!(target_env = "msvc");

    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ this "should" be safe for all `new_size`.  Unsound C stdlibs are #[test]ed for at the end of this file.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with one of `malloc`, `calloc`, `realloc, or `_recalloc` - all of which should be safe to `realloc`.
        let alloc = unsafe { realloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[track_caller] unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        #[cfg(target_env = "msvc")] {
            // https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/recalloc
            extern "C" { fn _recalloc(memblock: *mut c_void, num: size_t, size: size_t) -> *mut c_void; }
            // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
            // SAFETY: ✔️ this "should" be safe for all `new_size`.  Unsound C stdlibs are #[test]ed for at the end of this file.
            // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with one of `malloc`, `calloc`, `realloc, or `_recalloc` - all of which should be safe to `_recalloc`.
            // SAFETY: ✔️ `_recalloc` zeros memory
            let alloc = unsafe { _recalloc(ptr.as_ptr().cast(), 1, new_size) };
            NonNull::new(alloc.cast()).ok_or(())
        }
        #[cfg(not(target_env = "msvc"))] {
            let _ = (ptr, new_size);
            Err(())
        }
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::SizeOfDebug for Malloc {
    unsafe fn size_of_debug(&self, _ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> {
        #[cfg(target_env = "msvc")] {
            // https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/msize
            extern "C" { fn _msize(memblock: *mut c_void) -> size_t; }
            // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
            // SAFETY: ✔️ `ptr` belongs to `self` per thin::SizeOfDebug's documented safety preconditions, and thus was allocated with one of `malloc`, `calloc`, `realloc, or `_recalloc` - all of which should be safe to `_msize`.
            let size = unsafe { _msize(_ptr.as_ptr().cast()) };
            if size == !0 { return None } // error - but only if `_ptr` was null (impossible)?
            if size != 0 { return Some(size) }
        }

        None
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, Malloc};

    impls! {
        unsafe impl ialloc::fat::Alloc      for Malloc => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for Malloc => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for Malloc => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()             { thin::test::alignment(Malloc) }
#[test] fn thin_edge_case_sizes()       { thin::test::edge_case_sizes(Malloc) }
#[test] fn thin_nullable()              { thin::test::nullable(Malloc) }
#[test] fn thin_uninit()                { if !cfg!(target_os = "linux") { unsafe { thin::test::uninit_alloc_unsound(Malloc) } } } // malloc returns zeroed memory on some platforms
#[test] fn thin_zeroed()                { thin::test::zeroed_alloc(Malloc) }
#[test] fn thin_zst_support()           { thin::test::zst_supported_conservative(Malloc) }
#[test] fn thin_zst_support_dangle()    { thin::test::zst_supported_conservative(crate::allocator::adapt::DangleZst(Malloc)) }
#[test] fn thin_zst_support_alloc()     { thin::test::zst_supported_conservative(crate::allocator::adapt::AllocZst(Malloc)) }
