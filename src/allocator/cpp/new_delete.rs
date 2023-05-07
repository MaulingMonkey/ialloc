use crate::*;
use super::ffi;

use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete(void*, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDelete;

unsafe impl thin::Alloc for NewDelete {
    type Error = ();

    //const MAX_ALIGN : Alignment = __STDCPP_DEFAULT_NEW_ALIGNMENT__; // 8/16 - C++17

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::operator_new_nothrow(size.get()) }.cast()).ok_or(())
    }
}

unsafe impl thin::Free for NewDelete {
    unsafe fn free(&self, ptr: AllocNN) {
        unsafe { ffi::operator_delete(ptr.as_ptr().cast()) };
    }
}

unsafe impl nzst::Realloc for NewDelete {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, NewDelete};

    impls! {
        unsafe impl ialloc::nzst::Alloc     for NewDelete => ialloc::thin::Alloc;
        unsafe impl ialloc::nzst::Free      for NewDelete => ialloc::thin::Free;
    }
}
