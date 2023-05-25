use crate::boxed::ABox;
use crate::meta::*;
use crate::fat::*;
use crate::util;

use core::ffi::CStr;
use core::ptr::NonNull;



impl<T: Copy, A: Alloc + Free + ZstSupported> ABox<[T], A> {
    pub(crate) fn try_from_slice(value: &[T]) -> Result<Self, A::Error> where A : Default { Self::try_from_slice_in(value, A::default()) }
    pub(crate) fn try_from_slice_in(value: &[T], allocator: A) -> Result<Self, A::Error> {
        let len : usize = value.len();
        let mut b = ABox::<T, A>::try_new_uninit_slice_in(len, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        Ok(unsafe { b.assume_init() })
    }
}

// TODO: remove A : ZstSupported bound?  CStr is never a ZST as it always has at least a `\0`
impl<A: Alloc + Free + ZstSupported> ABox<CStr, A> {
    pub(crate) fn try_from_cstr(value: &CStr) -> Result<Self, A::Error> where A : Default { Self::try_from_cstr_in(value, A::default()) }
    pub(crate) fn try_from_cstr_in(value: &CStr, allocator: A) -> Result<Self, A::Error> {
        let bytes = value.to_bytes_with_nul();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::try_new_uninit_slice_in(len, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut CStr), allocator) })
    }
}

impl<A: Alloc + Free + ZstSupported> ABox<str, A> {
    pub(crate) fn try_from_str(value: &str) -> Result<Self, A::Error> where A : Default { Self::try_from_str_in(value, A::default()) }
    pub(crate) fn try_from_str_in(value: &str, allocator: A) -> Result<Self, A::Error> {
        let bytes = value.as_bytes();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::try_new_uninit_slice_in(len, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut str), allocator) })
    }
}


impl<T, A: Alloc + Free + ZstSupported> ABox<[T], A> {
    pub(crate) fn try_from_array<const N : usize>(value: [T; N]) -> Result<Self, A::Error> where A : Default { Self::try_from_array_in(value, A::default()) }
    pub(crate) fn try_from_array_in<const N : usize>(value: [T; N], allocator: A) -> Result<Self, A::Error> {
        let mut b = ABox::<T, A>::try_new_uninit_slice_in(N, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), N) };
        core::mem::forget(value);
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        let data = util::nn::slice_assume_init(data);
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }
}
