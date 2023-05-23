use crate::boxed::ABox;
use crate::fat::*;

use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};



// (Auto)Derefs

impl<T: ?Sized, A: Free> Deref for ABox<T, A> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: ✔️ `ABox::data` should always point at a valid `T` that we have exclusive access to
        unsafe { self.data().as_ref() }
    }
}

impl<T: ?Sized, A: Free> DerefMut for ABox<T, A> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: ✔️ `ABox::data` should always point at a valid `T` that we have exclusive access to
        unsafe { self.data().as_mut() }
    }
}

impl<T: ?Sized, A: Free> AsMut<T>       for ABox<T, A> { fn as_mut(&mut self)        -> &mut T   { self } }
impl<T: ?Sized, A: Free> AsRef<T>       for ABox<T, A> { fn as_ref(&self)            -> &T       { self } }
impl<T: ?Sized, A: Free> Borrow<T>      for ABox<T, A> { fn borrow(&self)            -> &T       { self } }
impl<T: ?Sized, A: Free> BorrowMut<T>   for ABox<T, A> { fn borrow_mut(&mut self)    -> &mut T   { self } }



// TODO:
//  • [ ] impl Generator<...>
//
// TODO:
//  • [ ] impl Future
//  • [ ] impl Unpin
//
// TODO† (unstable: requires fn_traits & unboxed_closures https://github.com/rust-lang/rust/issues/29625):
//  • [ ] impl Fn
//  • [ ] impl FnMut
//  • [ ] impl FnOnce
