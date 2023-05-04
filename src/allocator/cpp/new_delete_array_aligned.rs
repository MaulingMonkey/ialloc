use crate::*;
use super::ffi;

use core::ptr::NonNull;



/// [`::operator new[](size_t, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteArrayAligned;

unsafe impl nzst::Alloc for NewDeleteArrayAligned {
    type Error = ();

    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::operator_new_array_align_nothrow(layout.size().get(), layout.align().as_usize()) }.cast()).ok_or(())
    }
}

unsafe impl nzst::Free for NewDeleteArrayAligned {
    unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) {
        unsafe { ffi::operator_delete_array_align(ptr.as_ptr().cast(), layout.align().as_usize()) };
    }
}
