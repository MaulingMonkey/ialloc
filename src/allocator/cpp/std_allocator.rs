use crate::*;
use super::ffi;

use core::ffi::c_char;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// Implemented only for T = [`c_char`] <br>
/// [`std::allocator<_>::allocate`](https://en.cppreference.com/w/cpp/memory/allocator/allocate) <br>
/// [`std::allocator<_>::deallocate`](https://en.cppreference.com/w/cpp/memory/allocator/deallocate)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct StdAllocator<T>(PhantomData<fn(usize) -> T>);

impl<T> StdAllocator<T> {
    /// Create a new `std::allocator<T>` wrapper
    pub const fn new() -> Self { Self(PhantomData) }
}

unsafe impl thin::Alloc for StdAllocator<c_char> {
    const MAX_ALIGN : Alignment = ALIGN_1; // XXX: I'm not sure if std::allocator<char>::allocate can/will over-align...? To be investigated!
    // MSVC at least provides NewDelete alignment by default (8/16), but can that be reduced through end user specialization or operator overloads?

    type Error = ();

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::std_allocator_char_allocate(size.get()) }.cast()).ok_or(())
    }
}

unsafe impl nzst::Free for StdAllocator<c_char> {
    unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) {
        unsafe { ffi::std_allocator_char_deallocate(ptr.as_ptr().cast(), layout.size().get()) }
    }
}

unsafe impl nzst::Realloc for StdAllocator<c_char> {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, StdAllocator, c_char};

    impls! {
        unsafe impl ialloc::nzst::Alloc     for StdAllocator<c_char> => ialloc::thin::Alloc;

        unsafe impl ialloc::zsty::Alloc     for StdAllocator<c_char> => ialloc::nzst::Alloc;
        unsafe impl ialloc::zsty::Realloc   for StdAllocator<c_char> => ialloc::nzst::Realloc;
        unsafe impl ialloc::zsty::Free      for StdAllocator<c_char> => ialloc::nzst::Free;
    }
}
