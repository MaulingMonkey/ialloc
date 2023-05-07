use crate::*;
use super::ffi;

use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteArray;

unsafe impl thin::Alloc for NewDeleteArray {
    type Error = ();

    //const MAX_ALIGN : Alignment = __STDCPP_DEFAULT_NEW_ALIGNMENT__; // 8/16 - C++17

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::operator_new_array_nothrow(size.get()) }.cast()).ok_or(())
    }
}

unsafe impl thin::Free for NewDeleteArray {
    unsafe fn free(&self, ptr: AllocNN) {
        unsafe { ffi::operator_delete_array(ptr.as_ptr().cast()) };
    }
}

unsafe impl nzst::Realloc for NewDeleteArray {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, NewDeleteArray};

    impls! {
        unsafe impl ialloc::nzst::Alloc     for NewDeleteArray => ialloc::thin::Alloc;
        unsafe impl ialloc::nzst::Free      for NewDeleteArray => ialloc::thin::Free;
    }
}
