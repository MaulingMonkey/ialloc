use crate::boxed::ABox;
use crate::meta::*;
use crate::fat::*;
use crate::util;

use core::ffi::CStr;
use core::ptr::NonNull;



impl<T: Copy, A: Alloc + Free + Default + ZstSupported> ABox<[T], A> {
    pub(crate) fn try_from_slice(value: &[T]) -> Result<Self, A::Error> {
        let len : usize = value.len();
        let mut b = ABox::<T, A>::try_new_uninit_slice(len)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        Ok(unsafe { b.assume_init() })
    }
}

impl<A: Alloc + Free + Default + ZstSupported> ABox<CStr, A> {
    pub(crate) fn try_from_cstr(value: &CStr) -> Result<Self, A::Error> {
        let bytes = value.to_bytes_with_nul();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::try_new_uninit_slice(len)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut CStr), allocator) })
    }
}

impl<A: Alloc + Free + Default + ZstSupported> ABox<str, A> {
    pub(crate) fn try_from_str(value: &str) -> Result<Self, A::Error> {
        let bytes = value.as_bytes();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::try_new_uninit_slice(len)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut str), allocator) })
    }
}


impl<T, A: Alloc + Free + Default + ZstSupported> ABox<[T], A> {
    pub(crate) fn try_from_array<const N : usize>(value: [T; N]) -> Result<Self, A::Error> {
        let mut b = ABox::<T, A>::try_new_uninit_slice(N)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), N) };
        core::mem::forget(value);
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        let data = util::nn::slice_assume_init(data);
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }
}
