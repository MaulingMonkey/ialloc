use crate::fat::Free;
use crate::vec::AVec;

use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;



impl<T, A: Free, I: SliceIndex<[T]>> Index<I> for AVec<T, A> {
    type Output = I::Output;
    fn index(&self, index: I) -> &I::Output { self.as_slice().index(index) }
}

impl<T, A: Free, I: SliceIndex<[T]>> IndexMut<I> for AVec<T, A> {
    fn index_mut(&mut self, index: I) -> &mut I::Output { self.as_slice_mut().index_mut(index) }
}
