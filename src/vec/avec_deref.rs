use crate::fat::*;
use crate::vec::AVec;

use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};



// (Auto)Derefs

impl<T, A: Free> Deref          for AVec<T, A> { fn deref(&self)            -> &[T]         { self.as_slice()     } type Target = [T]; }
impl<T, A: Free> DerefMut       for AVec<T, A> { fn deref_mut(&mut self)    -> &mut [T]     { self.as_slice_mut() } }
impl<T, A: Free> AsMut<[T]>     for AVec<T, A> { fn as_mut(&mut self)       -> &mut [T]     { self } }
impl<T, A: Free> AsMut<Self>    for AVec<T, A> { fn as_mut(&mut self)       -> &mut Self    { self } }
impl<T, A: Free> AsRef<[T]>     for AVec<T, A> { fn as_ref(&self)           -> &[T]         { self } }
impl<T, A: Free> AsRef<Self>    for AVec<T, A> { fn as_ref(&self)           -> &Self        { self } }
impl<T, A: Free> Borrow<[T]>    for AVec<T, A> { fn borrow(&self)           -> &[T]         { self } }
impl<T, A: Free> BorrowMut<[T]> for AVec<T, A> { fn borrow_mut(&mut self)   -> &mut [T]     { self } }
