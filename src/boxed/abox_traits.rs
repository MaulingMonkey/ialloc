use crate::boxed::ABox;
use crate::fat::*;
use crate::meta::ZstSupported;
use crate::vec::AVec;

use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt::{self, Debug, Display, Formatter, Pointer};
use core::hash::{Hash, Hasher};
use core::iter::FusedIterator;
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



// core::fmt::*

impl<T: Debug,   A: Free + Debug> Debug   for ABox<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { f.debug_struct("ABox").field("data", &**self).field("allocator", Self::allocator(self)).finish() } }
impl<T: Display, A: Free        > Display for ABox<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { T::fmt(self, f) } }
impl<T: ?Sized,  A: Free        > Pointer for ABox<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Pointer::fmt(&self.data(), f) } }



// Misc. Operators

impl<T: ?Sized + Eq,        A: Free> Eq     for ABox<T, A> {}
impl<T: ?Sized + Ord,       A: Free> Ord    for ABox<T, A> { fn cmp(&self, other: &Self) -> Ordering { T::cmp(self, other) } }
impl<T: ?Sized + Hash,      A: Free> Hash   for ABox<T, A> { fn hash<H: Hasher>(&self, state: &mut H) { T::hash::<H>(self, state) } }

#[allow(clippy::partialeq_ne_impl)] // unnecessary but why not
impl<T: ?Sized + PartialEq, A: Free> PartialEq  for ABox<T, A> {
    fn eq(&self, other: &Self) -> bool { T::eq(self, other) }
    fn ne(&self, other: &Self) -> bool { T::ne(self, other) }
}

impl<T: ?Sized + PartialOrd, A: Free> PartialOrd for ABox<T, A> {
    fn partial_cmp  (&self, other: &Self) -> Option<Ordering>   { T::partial_cmp   (self, other) }
    fn ge           (&self, other: &Self) -> bool               { T::ge            (self, other) }
    fn gt           (&self, other: &Self) -> bool               { T::gt            (self, other) }
    fn le           (&self, other: &Self) -> bool               { T::le            (self, other) }
    fn lt           (&self, other: &Self) -> bool               { T::lt            (self, other) }
}

impl<T: ?Sized + Hasher, A: Free> Hasher for ABox<T, A> {
    fn finish       (&self) -> u64              { T::finish(self) }
    fn write        (&mut self, bytes: &[u8])   { T::write(self, bytes) }
    // write_length_prefix: nightly
    // write_str:           nightly

    fn write_u8     (&mut self, i: u8)          { T::write_u8(self, i) }
    fn write_u16    (&mut self, i: u16)         { T::write_u16(self, i) }
    fn write_u32    (&mut self, i: u32)         { T::write_u32(self, i) }
    fn write_u64    (&mut self, i: u64)         { T::write_u64(self, i) }
    fn write_u128   (&mut self, i: u128)        { T::write_u128(self, i) }
    fn write_usize  (&mut self, i: usize)       { T::write_usize(self, i) }

    fn write_i8     (&mut self, i: i8)          { T::write_i8(self, i) }
    fn write_i16    (&mut self, i: i16)         { T::write_i16(self, i) }
    fn write_i32    (&mut self, i: i32)         { T::write_i32(self, i) }
    fn write_i64    (&mut self, i: i64)         { T::write_i64(self, i) }
    fn write_i128   (&mut self, i: i128)        { T::write_i128(self, i) }
    fn write_isize  (&mut self, i: isize)       { T::write_isize(self, i) }
}

#[cfg(feature = "std")]
#[allow(deprecated)]
impl<T: std::error::Error, A: Free> std::error::Error for ABox<T, A> where Self : Debug + Display {
    fn description(&self)   -> &str                                         { (**self).description() }
    fn cause(&self)         -> Option<&dyn std::error::Error>               { (**self).cause() }
    fn source(&self)        -> Option<&(dyn std::error::Error + 'static)>   { (**self).source() }
}

impl<T: ?Sized + Iterator, A: Free> Iterator for ABox<T, A> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> { (**self).next() }
    fn size_hint(&self) -> (usize, Option<usize>) { (**self).size_hint() }
    fn nth(&mut self, n: usize) -> Option<Self::Item> { (**self).nth(n) }
    // XXX: last()
}

impl<T: ?Sized + ExactSizeIterator, A: Free> ExactSizeIterator for ABox<T, A> {
    fn len(&self) -> usize { (**self).len() }
}

impl<T: ?Sized + DoubleEndedIterator, A: Free> DoubleEndedIterator for ABox<T, A> {
    fn next_back(&mut self) -> Option<Self::Item> { (**self).next_back() }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> { (**self).nth_back(n) }
}

impl<T: ?Sized + FusedIterator, A: Free> FusedIterator for ABox<T, A> {}

#[cfg(feature = "alloc")]
#[cfg(global_oom_handling)]
impl<A: Free> Extend<ABox<str, A>> for alloc::string::String {
    fn extend<I: IntoIterator<Item = ABox<str, A>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

#[cfg(global_oom_handling)]
impl<T, A: Realloc + Default + ZstSupported> FromIterator<T> for ABox<[T], A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        AVec::<T, A>::from_iter(iter).into_boxed_slice()
    }
}

// TODO: FromIterator<ABox<str, A>> for String

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
