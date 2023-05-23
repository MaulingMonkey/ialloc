use crate::*;
use crate::meta::*;

use libc::*;

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// [`malloc`](https://en.cppreference.com/w/c/memory/malloc) / [`realloc`] / [`free`] / ...
///
/// | Rust                                      | C                     | MSVC<br>Only  |
/// | ------------------------------------------| ----------------------| --------------|
/// | [`thin::Alloc::alloc_uninit`]             | [`malloc`](https://en.cppreference.com/w/c/memory/malloc) |               |
/// | [`thin::Alloc::alloc_zeroed`]             | [`calloc`]            |               |
/// | [`thin::Realloc::realloc_uninit`]         | [`realloc`]           |               |
/// | [`thin::Realloc::realloc_zeroed`]         | ❌ N/A               | [`_recalloc`] |
/// | [`thin::Free::free`]                      | [`free`]              |               |
/// | [`thin::SizeOfDebug::size_of_debug`]      | `None`                | [`_msize`]    |
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Malloc;



// meta::*

impl Meta for Malloc {
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

    /// "If the size of the space requested is zero, the behavior is implementation defined: either a null pointer is returned, or the behavior is as if the size were some nonzero value, except that the returned pointer shall not be used to access an object."
    /// C89 § 7.20.3 ¶ 1
    ///
    /// Null pointers will be translated into an [`Err`], so this is at least defined behavior, but consider using an [`adapt`](crate::allocator::adapt) allocator for ZST support.
    ///
    /// | Platform          | Behavior  |
    /// | ------------------| ----------|
    /// | Linux             | Allocate
    /// | OS X              | ???
    /// | Windows           | Allocate
    const ZST_SUPPORTED : bool = false;
}

// SAFETY: ✔️ global state only
unsafe impl DefaultCompatible for Malloc {}



/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`fat::test::alignment`] ("The pointer returned if the allocation succeeds is suitably aligned so that it may be assigned to a pointer to any type of object and then used to access such an object or an array of such objects in the space allocated (until the space is explicitly deallocated)." C89 § 7.20.3 ¶ 1)
/// | `size`        | ✔️ Validated via [`fat::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`Malloc`] is `'static` - allocations by [`malloc`](https://en.cppreference.com/w/c/memory/malloc) / [`calloc`] live until [`free`]d ("The lifetime of an allocated object extends from the allocation until the deallocation." C89 § 7.20.3 ¶ 1)
/// | `compatible`  | ✔️ [`Malloc`] uses exclusively intercompatible [`malloc`](https://en.cppreference.com/w/c/memory/malloc) / [`realloc`] / [`calloc`] / [`free`] / [`_recalloc`]
/// | `exclusive`   | ✔️ Allocations by [`malloc`](https://en.cppreference.com/w/c/memory/malloc) / [`calloc`] are exclusive/unique ("Each such allocation shall yield a pointer to an object disjoint from any other object." C89 § 7.20.3 ¶ 1)
/// | `exceptions`  | ✔️ [`malloc`](https://en.cppreference.com/w/c/memory/malloc) / [`calloc`] throw no exceptions (it's C) and return null on errors (C89 § 7.20.3.1 ¶ 3, C89 § 7.20.3.3 ¶ 3)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], correct use of [`calloc`] ("The space is initialized to all bits zero" C89 § 7.20.3.1 ¶ 2)
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for Malloc {
    #[track_caller] fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsound C stdlibs are #[test]ed for at the end of this file.
        let alloc = unsafe { malloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[track_caller] fn alloc_zeroed(&self, size: usize) -> Result<NonNull<u8>, Self::Error> {
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsound C stdlibs are #[test]ed for at the end of this file.
        // SAFETY: ✔️ `calloc` zeros memory
        let alloc = unsafe { calloc(1, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`free`] is compatible with [`malloc`](https://en.cppreference.com/w/c/memory/malloc) / [`realloc`] / [`calloc`] (C89 § 7.20.3.2 ¶ 2) and with [`_recalloc`]
/// | `exceptions`  | ✔️ [`free`] throws no exceptions (it's C) and returns no errors (C89 § 7.20.3.2 ¶ 3)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Free for Malloc {
    #[track_caller] unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (C89 § 7.20.3.2 ¶ 2, validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions
        unsafe { free(ptr.cast()) }
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`Malloc`] is `'static` - reallocations by [`realloc`] / [`_recalloc`] live until [`realloc`] / [`_recalloc`]ed again or [`free`]d
/// | `compatible`  | ✔️ [`Malloc`] uses exclusively intercompatible [`malloc`](https://en.cppreference.com/w/c/memory/malloc) / [`realloc`] / [`calloc`] / [`free`] / [`_recalloc`]
/// | `exclusive`   | ✔️ Reallocations by [`realloc`] / [`_recalloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`realloc`] / [`_recalloc`] throw no exceptions (it's C) and return `nullptr` on error (C89 § 7.20.3.4 ¶ 4)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
/// | `zeroed`      | ⚠️ untested, but we use [`_recalloc`] or default impl appropriately...
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Realloc for Malloc {
    const CAN_REALLOC_ZEROED : bool = cfg!(target_env = "msvc");

    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with one of `malloc`, `calloc`, `realloc, or `_recalloc` - all of which should be safe to `realloc`.
        let alloc = unsafe { realloc(ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    #[track_caller] unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        #[cfg(target_env = "msvc")] {
            // https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/recalloc
            extern "C" { fn _recalloc(memblock: *mut c_void, num: size_t, size: size_t) -> *mut c_void; }
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

/// | Item          | Description   |
/// | --------------| --------------|
/// | `size`        | ✔️ validated via [`thin::test::size_over_alloc`]
/// | `compatible`  | ✔️ [`_msize`] is explicitly documented to be compatible with [`malloc`](https://en.cppreference.com/w/c/memory/malloc), [`realloc`], [`calloc`] - and is implicitly compatible with [`_recalloc`]
/// | `exceptions`  | ✔️ [`_msize`] returns `-1` / sets `errno` on error instead of throwing exceptions
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::SizeOfDebug for Malloc {
    unsafe fn size_of_debug(&self, _ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> {
        #[cfg(not(target_env = "msvc"))] { None }
        #[cfg(    target_env = "msvc" )] {
            // https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/msize
            extern "C" { fn _msize(memblock: *mut c_void) -> size_t; }
            // SAFETY: ✔️ `ptr` belongs to `self` per thin::SizeOfDebug's documented safety preconditions, and thus was allocated with one of `malloc`, `calloc`, `realloc, or `_recalloc` - all of which should be safe to `_msize`.
            let size = unsafe { _msize(_ptr.as_ptr().cast()) };
            if size == !0 { return None } // error - but only if `_ptr` was null (impossible)?
            Some(size)
        }
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
#[test] fn thin_size()                  { thin::test::size_over_alloc(Malloc) }
#[test] fn thin_uninit()                { if !cfg!(target_os = "linux") { unsafe { thin::test::uninit_alloc_unsound(Malloc) } } } // malloc returns zeroed memory on some platforms
#[test] fn thin_zeroed()                { thin::test::zeroed_alloc(Malloc) }
#[test] fn thin_zst_support()           { thin::test::zst_supported_conservative(Malloc) }
#[test] fn thin_zst_support_dangle()    { thin::test::zst_supported_conservative(crate::allocator::adapt::DangleZst(Malloc)) }
#[test] fn thin_zst_support_alloc()     { thin::test::zst_supported_conservative(crate::allocator::adapt::AllocZst(Malloc)) }

#[test] fn fat_alignment()              { fat::test::alignment(Malloc) }
#[test] fn fat_edge_case_sizes()        { fat::test::edge_case_sizes(Malloc) }
#[test] fn fat_uninit()                 { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(Malloc) } } } // malloc returns zeroed memory on some platforms
#[test] fn fat_zeroed()                 { fat::test::zeroed_alloc(Malloc) }
#[test] fn fat_zst_support()            { fat::test::zst_supported_conservative(Malloc) }
#[test] fn fat_zst_support_dangle()     { fat::test::zst_supported_conservative(crate::allocator::adapt::DangleZst(Malloc)) }
#[test] fn fat_zst_support_alloc()      { fat::test::zst_supported_conservative(crate::allocator::adapt::AllocZst(Malloc)) }
