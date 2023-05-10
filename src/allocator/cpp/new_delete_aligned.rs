use crate::*;
use super::ffi;

use core::alloc::Layout;
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

unsafe impl fat::Alloc for NewDeleteAligned {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        NonNull::new(unsafe { ffi::operator_new_align_nothrow(layout.size(), layout.align()) }.cast()).ok_or(())
    }
}

unsafe impl fat::Free for NewDeleteAligned {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        unsafe { ffi::operator_delete_align(ptr.as_ptr().cast(), layout.align()) };
    }
}

unsafe impl fat::Realloc for NewDeleteAligned {}
