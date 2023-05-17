use crate::*;
use super::ffi;

use core::alloc::Layout;
use core::ffi::c_char;
use core::marker::PhantomData;
use core::ptr::NonNull;



/// Implemented only for T = [`c_char`] <br>
/// [`std::allocator<_>::allocate`](https://en.cppreference.com/w/cpp/memory/allocator/allocate) <br>
/// [`std::allocator<_>::deallocate`](https://en.cppreference.com/w/cpp/memory/allocator/deallocate)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct StdAllocator<T>(PhantomData<fn(usize) -> T>);

impl<T> StdAllocator<T> {
    /// Create a new `std::allocator<T>` wrapper
    pub const fn new() -> Self { Self(PhantomData) }
}

impl<T> meta::Meta for StdAllocator<T> {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::of::<T>(); // more in practice, but this is what I'll rely on
    const MAX_SIZE  : usize     = usize::MAX;           // less in practice
    const ZST_SUPPORTED : bool  = false;                // supported on some linux, unsupported on windows
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for StdAllocator<c_char> {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ this "should" allocate correctly for all `size`.  #[test]ed for indirectly through fat::test::edge_case_sizes at the end of this file.
        // SAFETY: ✔️ this "should" align correctly for all `size`.     #[test]ed for indirectly through fat::test::alignment at the end of this file.
        // SAFETY: ✔️ FFI wrapper catches std::bad_alloc.
        NonNull::new(unsafe { ffi::std_allocator_char_allocate(size) }.cast()).ok_or(())
    }
}

// DO NOT IMPLEMENT: thin::Free
// std::allocator requires a size, and thus cannot implement this interface without an adapter allocator

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Free for StdAllocator<c_char> {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Free::free's documented safety preconditions - and thus was allocated with `std::allocator<char>{}.allocate(layout.size())`
        unsafe { ffi::std_allocator_char_deallocate(ptr.as_ptr().cast(), layout.size()) }
    }
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
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
