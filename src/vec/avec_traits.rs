use crate::fat::*;
use crate::meta::*;
use crate::vec::AVec;

use core::borrow::{Borrow, BorrowMut};
use core::fmt::{self, Debug, Formatter};
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice::SliceIndex;



// (Auto)Derefs

impl<T, A: Free> Deref          for AVec<T, A> { fn deref(&self)            -> &[T]         { self.as_slice()     } type Target = [T]; }
impl<T, A: Free> DerefMut       for AVec<T, A> { fn deref_mut(&mut self)    -> &mut [T]     { self.as_slice_mut() } }
impl<T, A: Free> AsMut<[T]>     for AVec<T, A> { fn as_mut(&mut self)       -> &mut [T]     { self } }
impl<T, A: Free> AsMut<Self>    for AVec<T, A> { fn as_mut(&mut self)       -> &mut Self    { self } }
impl<T, A: Free> AsRef<[T]>     for AVec<T, A> { fn as_ref(&self)           -> &[T]         { self } }
impl<T, A: Free> AsRef<Self>    for AVec<T, A> { fn as_ref(&self)           -> &Self        { self } }
impl<T, A: Free> Borrow<[T]>    for AVec<T, A> { fn borrow(&self)           -> &[T]         { self } }
impl<T, A: Free> BorrowMut<[T]> for AVec<T, A> { fn borrow_mut(&mut self)   -> &mut [T]     { self } }



// core::fmt::*

impl<T: Debug, A: Free + Debug> Debug for AVec<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { f.debug_struct("AVec").field("allocator", self.allocator()).field("capacity", &self.capacity()).field("data", &self.as_slice()).finish() } }



#[cfg(global_oom_handling)] impl<T, A: Realloc + ZstSupported> Extend<T> for AVec<T, A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for item in iter { self.push(item) }
    }
}

#[cfg(global_oom_handling)] impl<'a, T: Copy + 'a, A: Realloc + ZstSupported> Extend<&'a T> for AVec<T, A> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for item in iter { self.push(*item) }
    }
    // unstable:
    // fn extend_one(&mut self, item: &'a T) { self.push(*item) }
    // fn extend_reserve(&mut self, additional: usize) { self.reserve(additional) }
}

// TODO:
//  • [ ] From
//  • [ ] TryFrom

impl<T, A: Free, I: SliceIndex<[T]>> Index<I> for AVec<T, A> {
    type Output = I::Output;
    fn index(&self, index: I) -> &I::Output { self.as_slice().index(index) }
}

impl<T, A: Free, I: SliceIndex<[T]>> IndexMut<I> for AVec<T, A> {
    fn index_mut(&mut self, index: I) -> &mut I::Output { self.as_slice_mut().index_mut(index) }
}

#[cfg(feature = "std")]
impl<A: Realloc> std::io::Write for AVec<u8, A> {
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.try_extend_from_slice(buf) {
            Ok(()) => Ok(buf.len()),
            Err(_err) => Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory)),
        }
    }
}
