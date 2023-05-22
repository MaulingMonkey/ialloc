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



// Misc. Operators

#[cfg(feature = "alloc")]
#[cfg(global_oom_handling)]
impl<A: Free> Extend<ABox<str, A>> for alloc::string::String {
    fn extend<I: IntoIterator<Item = ABox<str, A>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

// TODO:
//  • [ ] impl Generator<...>
//
// TODO:
//  • [ ] impl Fn
//  • [ ] impl FnMut
//  • [ ] impl FnOnce
//  • [ ] impl Future
//  • [ ] impl Unpin
//
// TODO:
//  • [ ] impl From<...>
//  • [ ] impl From<...>
//  • [ ] impl TryFrom<...>
