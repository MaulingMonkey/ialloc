use crate::fat::*;
use crate::vec::AVec;

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};



impl<T: Eq,     A: Free> Eq     for AVec<T, A> {}
impl<T: Ord,    A: Free> Ord    for AVec<T, A> { fn cmp(&self, other: &Self) -> Ordering { <[T]>::cmp(self, other) } }
impl<T: Hash,   A: Free> Hash   for AVec<T, A> { fn hash<H: Hasher>(&self, state: &mut H) { <[T]>::hash::<H>(self, state) } }

#[allow(clippy::partialeq_ne_impl)] // unnecessary but why not
impl<T: PartialEq, A: Free> PartialEq  for AVec<T, A> {
    fn eq(&self, other: &Self) -> bool { <[T]>::eq(self, other.as_slice()) }
    fn ne(&self, other: &Self) -> bool { <[T]>::ne(self, other.as_slice()) }
}

impl<T: PartialOrd, A: Free> PartialOrd for AVec<T, A> {
    fn partial_cmp  (&self, other: &Self) -> Option<Ordering>   { <[T]>::partial_cmp   (self, other) }
    fn ge           (&self, other: &Self) -> bool               { <[T]>::ge            (self, other) }
    fn gt           (&self, other: &Self) -> bool               { <[T]>::gt            (self, other) }
    fn le           (&self, other: &Self) -> bool               { <[T]>::le            (self, other) }
    fn lt           (&self, other: &Self) -> bool               { <[T]>::lt            (self, other) }
}

// TODO:
//  • [ ] PartialEq spam
//  • [ ] PartialOrd spam
