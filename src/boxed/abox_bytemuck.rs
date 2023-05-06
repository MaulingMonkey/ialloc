#![cfg(feature = "bytemuck")]

use crate::boxed::ABox;
use crate::util;
use crate::zsty::*;

use bytemuck::*;

use core::alloc::{Layout, LayoutError};



impl<T: Zeroable, A: Alloc + Free> ABox<T, A> {
    // Sized, Alloc

    #[track_caller] pub fn try_new_bytemuck_zeroed_in(allocator: A) -> Result<Self, A::Error> {
        let layout = Layout::new::<T>();
        let data = allocator.alloc_zeroed(layout)?.cast();
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }

    #[track_caller] pub fn try_new_bytemuck_zeroed_slice_in(len: usize, allocator: A) -> Result<ABox<[T], A>, A::Error> where LayoutError : Into<A::Error> {
        let layout = Layout::array::<T>(len).map_err(|e| e.into())?;
        let data = util::nn::slice_from_raw_parts(allocator.alloc_zeroed(layout)?.cast(), len);
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }

    #[cfg(feature = "panicy-memory")] #[track_caller] pub fn new_bytemuck_zeroed_in(allocator: A) -> Self {
        Self::try_new_bytemuck_zeroed_in(allocator).expect("unable to allocate")
    }

    #[cfg(feature = "panicy-memory")] #[track_caller] pub fn new_bytemuck_zeroed_slice_in(len: usize, allocator: A) -> ABox<[T], A> where LayoutError : Into<A::Error> {
        Self::try_new_bytemuck_zeroed_slice_in(len, allocator).expect("unable to allocate")
    }
}

impl<T: Zeroable, A: Alloc + Free + Default> ABox<T, A> {
    // Sized, Alloc, Default
    #[track_caller] #[inline(always)] pub fn try_new_bytemuck_zeroed() -> Result<Self, A::Error> { Self::try_new_bytemuck_zeroed_in(A::default()) }
    #[track_caller] #[inline(always)] pub fn try_new_bytemuck_zeroed_slice(len: usize) -> Result<ABox<[T], A>, A::Error> where LayoutError : Into<A::Error> { Self::try_new_bytemuck_zeroed_slice_in(len, A::default()) }
    #[cfg(feature = "panicy-memory")] #[track_caller] #[inline(always)] pub fn new_bytemuck_zeroed() -> Self { Self::new_bytemuck_zeroed_in(A::default()) }
    #[cfg(feature = "panicy-memory")] #[track_caller] #[inline(always)] pub fn new_bytemuck_zeroed_slice(len: usize) -> ABox<[T], A> where LayoutError : Into<A::Error> { Self::new_bytemuck_zeroed_slice_in(len, A::default()) }
}
