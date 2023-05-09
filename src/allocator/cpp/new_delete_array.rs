use crate::*;
use super::ffi;

use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteArray;

impl meta::Meta for NewDeleteArray {
    type Error = ();

    const MAX_ALIGN : Alignment = if !cfg!(target_env = "msvc") {
        Alignment::of::<f64>() // conservative
        // __STDCPP_DEFAULT_NEW_ALIGNMENT__; // 8/16 - C++17
    } else if core::mem::size_of::<usize>() >= 8 {
        ALIGN_16
    } else {
        ALIGN_8
    };

    const MAX_SIZE : usize = usize::MAX; // *slightly* less in practice
    const ZST_SUPPORTED : bool = false; // platform behavior too inconsistent
}

unsafe impl thin::Alloc for NewDeleteArray {
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

        unsafe impl ialloc::zsty::Alloc     for NewDeleteArray => ialloc::nzst::Alloc;
        unsafe impl ialloc::zsty::Realloc   for NewDeleteArray => ialloc::nzst::Realloc;
        unsafe impl ialloc::zsty::Free      for NewDeleteArray => ialloc::nzst::Free;
    }
}
