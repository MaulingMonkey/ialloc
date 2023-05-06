use crate::boxed::ABox;
use crate::util;
use crate::zsty::*;

use core::alloc::{Layout, LayoutError};
use core::mem::MaybeUninit;
use core::mem::align_of;



impl<T, A: Alloc + Free> ABox<T, A> {
    // Sized, Alloc

    /// If you hit this assertion, it's unlikely that `A` can ever successfully allocate an instance of `T` except by happenstance and accident.
    /// Unless you've written some obscenely generic code that intentionally handles containers that might never be able to allocate, this is likely a bug.
    const ASSERT_A_CAN_ALLOC_ALIGNED_T : () = assert!(align_of::<T>() <= A::MAX_ALIGN.as_usize());

    pub fn try_new_in(x: T, allocator: A) -> Result<Self, A::Error> {
        Ok(ABox::write(Self::try_new_uninit_in(allocator)?, x))
    }

    pub fn try_new_uninit_in(allocator: A) -> Result<ABox<MaybeUninit<T>, A>, A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        let layout = Layout::new::<T>();
        let data = allocator.alloc_uninit(layout)?.cast();
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }

    pub fn try_new_uninit_slice_in(len: usize, allocator: A) -> Result<ABox<[MaybeUninit<T>], A>, A::Error> where LayoutError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        let layout = Layout::array::<T>(len).map_err(|e| e.into())?;
        let data = util::nn::slice_from_raw_parts(allocator.alloc_uninit(layout)?.cast(), len);
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }
}

impl<T, A: Alloc + Free + Default> ABox<T, A> {
    // Sized, Alloc, Default
    pub fn try_new(x: T) -> Result<Self, A::Error> { Self::try_new_in(x, A::default()) }
    pub fn try_new_uninit() -> Result<ABox<MaybeUninit<T>, A>, A::Error> { Self::try_new_uninit_in(A::default()) }
    pub fn try_new_uninit_slice(len: usize) -> Result<ABox<[MaybeUninit<T>], A>, A::Error> where LayoutError : Into<A::Error> { Self::try_new_uninit_slice_in(len, A::default()) }
}

#[cfg(feature = "panicy-memory")] impl<T, A: Alloc + Free> ABox<T, A> {
    // Sized, Alloc
    #[track_caller] #[inline(always)] pub fn new_in(x: T, allocator: A) -> Self { Self::try_new_in(x, allocator).expect("unable to allocate") }
    #[track_caller] #[inline(always)] pub fn new_uninit_in(allocator: A) -> ABox<MaybeUninit<T>, A> { Self::try_new_uninit_in(allocator).expect("unable to allocate") }
    #[track_caller] #[inline(always)] pub fn new_uninit_slice_in(len: usize, allocator: A) -> ABox<[MaybeUninit<T>], A> where LayoutError : Into<A::Error> { Self::try_new_uninit_slice_in(len, allocator).expect("unable to allocate") }
}

#[cfg(feature = "panicy-memory")] impl<T, A: Alloc + Free + Default> ABox<T, A> {
    // Sized, Alloc, Default
    #[track_caller] #[inline(always)] pub fn new(x: T) -> Self { Self::new_in(x, A::default()) }
    #[track_caller] #[inline(always)] pub fn new_uninit() -> ABox<MaybeUninit<T>, A> { Self::new_uninit_in(A::default()) }
    #[track_caller] #[inline(always)] pub fn new_uninit_slice(len: usize) -> ABox<[MaybeUninit<T>], A> where LayoutError : Into<A::Error> { Self::new_uninit_slice_in(len, A::default()) }
}
