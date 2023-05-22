use crate::fat::*;
use crate::vec::AVec;

use core::fmt::{self, Debug, Formatter};
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;



// core::fmt::*

impl<T: Debug, A: Free + Debug> Debug for AVec<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { f.debug_struct("AVec").field("allocator", self.allocator()).field("capacity", &self.capacity()).field("data", &self.as_slice()).finish() } }



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
