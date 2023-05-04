use crate::*;
use super::ffi;

use core::ffi::c_char;
use core::marker::PhantomData;
use core::ptr::NonNull;



/// Implemented only for T = [`c_char`] <br>
/// [`std::allocator<_>::allocate`](https://en.cppreference.com/w/cpp/memory/allocator/allocate) <br>
/// [`std::allocator<_>::deallocate`](https://en.cppreference.com/w/cpp/memory/allocator/deallocate)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct StdAllocator<T>(PhantomData<fn(usize) -> T>);

unsafe impl nzst::Alloc for StdAllocator<c_char> {
    type Error = ();

    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::std_allocator_char_allocate(layout.size().get()) }.cast()).ok_or(())
    }
}

unsafe impl nzst::Free for StdAllocator<c_char> {
    unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) {
        unsafe { ffi::std_allocator_char_deallocate(ptr.as_ptr().cast(), layout.size().get()) }
    }
}
