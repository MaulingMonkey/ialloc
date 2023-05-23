use crate::*;
use crate::meta::*;
use super::ffi;

use core::alloc::Layout;
use core::ffi::c_char;
use core::marker::PhantomData;
use core::ptr::NonNull;



/// Implemented only for T = [`c_char`] <br>
/// [`std::allocator<T>::allocate`] <br>
/// [`std::allocator<T>::deallocate`]
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct StdAllocator<T>(PhantomData<fn(usize) -> T>);

impl<T> StdAllocator<T> {
    /// Create a new [`std::allocator<T>`] wrapper
    ///
    #[doc = include_str!("_refs.md")]
    pub const fn new() -> Self { Self(PhantomData) }
}



// meta::*

impl<T> Meta for StdAllocator<T> {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::of::<T>(); // more in practice, but this is what I'll rely on
    const MAX_SIZE  : usize     = usize::MAX;           // less in practice
    const ZST_SUPPORTED : bool  = false;                // supported on some linux, unsupported on windows
}

/// SAFETY: ✔️ <code>[std::allocator]&lt;char&gt;</code> is stateless (see `is_always_equal` checks in `ffi.cpp`)
///
#[doc = include_str!("_refs.md")]
unsafe impl Stateless for StdAllocator<c_char> {} // likely applicable to most std::allocator<T> where T is a builtin



// thin::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`fat::test::alignment`]
/// | `size`        | ✔️ Validated via [`fat::test::edge_case_sizes`]
/// | `pin`         | ✔️ <code>[StdAllocator]&lt;c_char&gt;</code> is `'static` - allocations by [`std::allocator<T>::allocate`] live until [`std::allocator<T>::deallocate`]d.
/// | `compatible`  | ✔️ <code>[StdAllocator]&lt;c_char&gt;</code> uses exclusively intercompatible <code>[std::allocator]&lt;char&gt;</code> functions
/// | `compatible`  | ✔️ <code>[std::allocator]&lt;char&gt;</code> is stateless (see `is_always_equal` checks in `ffi.cpp`)
/// | `exclusive`   | ✔️ [`std::allocator<T>::allocate`] allocations are exclusive/unique
/// | `exceptions`  | ✔️ [`std::allocator<T>::allocate`] can throw [`std::bad_alloc`] - the FFI wrapper around it catches [`std::bad_alloc`] and returns `nullptr` instead.
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
/// | `zeroed`      | ✔️ Validated via [`fat::test::zeroed_alloc`]
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for StdAllocator<c_char> {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: per above
        NonNull::new(unsafe { ffi::std_allocator_char_allocate(size) }.cast()).ok_or(())
    }
}

// SAFETY: ⚠️ DO NOT IMPLEMENT: thin::Free
// [`std::allocator<T>::deallocate`] requires a size, and thus cannot implement this interface without an adapter allocator



// fat::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ <code>[StdAllocator]&lt;c_char&gt;</code> uses exclusively intercompatible <code>[std::allocator]&lt;char&gt;</code> functions
/// | `compatible`  | ✔️ <code>[std::allocator]&lt;char&gt;</code> is stateless (see `is_always_equal` checks in `ffi.cpp`)
/// | `exceptions`  | ⚠️ [`std::allocator<T>::deallocate`] "Does not throw exceptions" (C++03 § 20.1.5 ¶ 2 Table 32), although it's neither `throw()` nor `noexcept`.
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl fat::Free for StdAllocator<c_char> {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        // SAFETY: ✔️ `ptr` belongs to `self` per [`fat::Free::free`]'s documented safety preconditions - and thus was allocated with `std::allocator<char>{}.allocate(layout.size())`
        unsafe { ffi::std_allocator_char_deallocate(ptr.as_ptr().cast(), layout.size()) }
    }
}

// SAFETY: ✔️ default Realloc impl is soundly implemented in terms of Alloc+Free
unsafe impl fat::Realloc for StdAllocator<c_char> {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, StdAllocator, c_char};

    impls! {
        // SAFETY: ✔️ all {thin, fat}::* impls intercompatible with each other where implemented
        unsafe impl ialloc::fat::Alloc for StdAllocator<c_char> => ialloc::thin::Alloc;
    }
}



#[test] fn thin_zst_support()       { thin::test::zst_supported_conservative_leak(StdAllocator::<c_char>::new()) }

#[test] fn fat_alignment()          { fat::test::alignment(StdAllocator::<c_char>::new()) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(StdAllocator::<c_char>::new()) }
#[test] fn fat_uninit()             { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(StdAllocator::<c_char>::new()) } } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(StdAllocator::<c_char>::new()) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_conservative(StdAllocator::<c_char>::new()) }
