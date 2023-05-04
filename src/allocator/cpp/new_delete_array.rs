use crate::*;
use super::ffi;

use core::ptr::NonNull;



/// [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteArray;

unsafe impl nzst::Alloc for NewDeleteArray {
    type Error = ();

    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::operator_new_array_nothrow(layout.size().get()) }.cast()).ok_or(())
    }
}

unsafe impl nzst::Free for NewDeleteArray {
    unsafe fn free(&self, ptr: AllocNN, _layout: LayoutNZ) {
        unsafe { ffi::operator_delete_array(ptr.as_ptr().cast()) };
    }
}

unsafe impl thin::Free for NewDeleteArray {
    unsafe fn free(&self, ptr: AllocNN) {
        unsafe { ffi::operator_delete_array(ptr.as_ptr().cast()) };
    }
}
