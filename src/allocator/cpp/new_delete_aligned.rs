use crate::*;
use super::ffi;

use core::ptr::NonNull;



/// [`::operator new(size_t, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete(void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteAligned;

impl meta::Meta for NewDeleteAligned {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::MAX;   // XXX: less in practice
    const MAX_SIZE  : usize     = usize::MAX;       // XXX: less in practice
    const ZST_SUPPORTED : bool  = false;            // platform behavior too inconsistent
}

unsafe impl nzst::Alloc for NewDeleteAligned {
    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::operator_new_align_nothrow(layout.size().get(), layout.align().as_usize()) }.cast()).ok_or(())
    }
}

unsafe impl nzst::Free for NewDeleteAligned {
    unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) {
        unsafe { ffi::operator_delete_align(ptr.as_ptr().cast(), layout.align().as_usize()) };
    }
}

unsafe impl nzst::Realloc for NewDeleteAligned {}
