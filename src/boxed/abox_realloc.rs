use crate::boxed::ABox;
use crate::error::ExcessiveSliceRequestedError;
use crate::fat::*;
use crate::util;

use core::alloc::Layout;
use core::mem::MaybeUninit;



impl<T, A: Realloc> ABox<[MaybeUninit<T>], A> {
    pub fn try_realloc_uninit_slice(this: &mut Self, new_len: usize) -> Result<(), A::Error> {
        let new_layout  = Layout::array::<MaybeUninit<T>>(new_len).map_err(|_| ExcessiveSliceRequestedError { requested: new_len })?;
        let old_layout  = Self::layout(this);
        let allocator   = Self::allocator(this);
        let data        = Self::data(this).cast::<MaybeUninit<u8>>();
        let data        = unsafe { allocator.realloc_uninit(data, old_layout, new_layout)? };
        unsafe { Self::set_data(this, util::nn::slice_from_raw_parts(data.cast(), new_len)) };
        Ok(())
    }
}

#[cfg(global_oom_handling)] impl<T, A: Realloc> ABox<[MaybeUninit<T>], A> {
    pub fn realloc_uninit_slice(this: &mut Self, new_len: usize) {
        Self::try_realloc_uninit_slice(this, new_len).expect("unable to reallocate")
    }
}
