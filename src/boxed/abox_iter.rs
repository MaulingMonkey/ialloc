use crate::boxed::ABox;
use crate::fat::{Free, Realloc};
use crate::meta::ZstSupported;
use crate::vec::AVec;

use core::iter::FusedIterator;



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

#[cfg(global_oom_handling)]
impl<T, A: Realloc + Default + ZstSupported> FromIterator<T> for ABox<[T], A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        AVec::<T, A>::from_iter(iter).into_boxed_slice()
    }
}

// TODO: FromIterator<ABox<str, A>> for String
