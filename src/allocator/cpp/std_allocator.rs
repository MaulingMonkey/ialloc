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

unsafe impl thin::Alloc for StdAllocator<c_char> {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::std_allocator_char_allocate(size) }.cast()).ok_or(())
    }
}

unsafe impl zsty::Free for StdAllocator<c_char> {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        unsafe { ffi::std_allocator_char_deallocate(ptr.as_ptr().cast(), layout.size()) }
    }
}

unsafe impl zsty::Realloc for StdAllocator<c_char> {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, StdAllocator, c_char};

    impls! {
        unsafe impl ialloc::zsty::Alloc     for StdAllocator<c_char> => ialloc::thin::Alloc;
    }
}



// inconsistent behavior across platforms
// also no thin::Free support
//#[test] fn thin_zst_support() { assert!(thin::zst_supported_accurate(StdAllocator::<c_char>::new())) }
