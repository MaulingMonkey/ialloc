use crate::boxed::ABox;
use crate::fat::*;

use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt::{self, Debug, Display, Formatter, Pointer};
use core::hash::{Hash, Hasher};
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
// TODO: various numeric traits as well? Or is that overkill?



// Misc. Operators

impl<T: ?Sized + Eq,        A: Free> Eq     for ABox<T, A> {}
impl<T: ?Sized + Ord,       A: Free> Ord    for ABox<T, A> { fn cmp(&self, other: &Self) -> Ordering { T::cmp(self, other) } } // TODO: clamp, min, max? Nah, awkward to impl, and alloc::alloc::Box doesn't bother either.
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



// TODO:
//  • [ ] impl Error
//
// TODO:
//  • [ ] impl DoubleEndedIterator?
//  • [ ] impl ExactSizeIterator?
//  • [ ] impl FromIterator
//  • [ ] impl FusedIterator
//  • [ ] impl Iterator
//  • [ ] impl Extend
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
//
// TODO:
//  • [ ] impl io::As{Fd, RawFd}
//  • [ ] impl io::{,Buf}Read
//  • [ ] impl io::Seek
//  • [ ] impl io::{,Buf}Write
