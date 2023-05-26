use crate::boxed::ABox;
use crate::meta::*;
use crate::fat::*;
use crate::vec::AVec;

use core::mem::MaybeUninit;
use core::ptr::NonNull;



impl<T, A: Free + ZstSupported> From<ABox<[T], A>> for AVec<T, A> {
    fn from(value: ABox<[T], A>) -> Self {
        let (data, allocator) = ABox::into_raw_with_allocator(value);
        let len  : usize = data.len();
        let data : *mut [T] = data.as_ptr();
        let data : *mut MaybeUninit<T> = data as _;
        let data = unsafe { NonNull::new_unchecked(data) };
        unsafe { Self::from_raw_parts_in(data.cast(), len, len, allocator) }
    }
}

#[cfg(nope)] // TODO: impl Unpin, pin, pin_in, into_pin, ...
impl<T: ?Sized, A: Free + 'static> From<ABox<T, A>> for Pin<ABox<T, A>> {
    fn from(value: ABox<T, A>) -> Self {
        todo!()
    }
}

impl<A: Free> From<ABox<str, A>> for ABox<[u8], A> {
    fn from(value: ABox<str, A>) -> Self {
        let (data, allocator) = ABox::into_raw_with_allocator(value);
        unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut [u8]), allocator) }
    }
}
