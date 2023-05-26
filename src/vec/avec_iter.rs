use crate::fat::*;
use crate::meta::*;
use crate::vec::AVec;

use core::alloc::Layout;
use core::iter::FusedIterator;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



#[cfg(global_oom_handling)] impl<T, A: Realloc + Default + ZstSupported> FromIterator<T> for AVec<T, A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v = Self::new();
        v.extend(iter);
        v
    }
}

impl<T, A: Free> IntoIterator for AVec<T, A> {
    type Item = T;
    type IntoIter = IntoIter<T, A>;
    fn into_iter(self) -> Self::IntoIter {
        let (data, len, cap, alloc) = self.into_raw_parts_with_allocator();
        let i = 0;
        IntoIter { data, i, len, cap, alloc }
    }
}

/// [`AVec`] converted into an iterator (e.g. the result of <code>avec.[into_iter](AVec::into_iter)\(\)</code>)
pub struct IntoIter<T, A: Free> {
    data:   NonNull<T>,
    i:      usize,
    len:    usize,
    cap:    usize,
    alloc:  A,
}

impl<T, A: Free> Drop for IntoIter<T, A> {
    fn drop(&mut self) {
        let to_drop = unsafe { core::ptr::slice_from_raw_parts_mut(self.data.as_ptr().add(self.i), self.len - self.i) };
        unsafe { core::ptr::drop_in_place(to_drop) };
        unsafe { self.alloc.free(self.data.cast(), Layout::array::<MaybeUninit<T>>(self.cap).unwrap()) };
    }
}

impl<T, A: Free> Iterator for IntoIter<T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.len { return None }
        let item = unsafe { core::ptr::read(self.data.as_ptr().add(self.i)) };
        self.i += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len - self.i;
        (n, Some(n))
    }
}

impl<T, A: Free> DoubleEndedIterator for IntoIter<T, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.i >= self.len { return None }
        self.len -= 1;
        Some(unsafe { core::ptr::read(self.data.as_ptr().add(self.len)) })
    }
}

impl<T, A: Free> ExactSizeIterator for IntoIter<T, A> {}
impl<T, A: Free> FusedIterator for IntoIter<T, A> {}

// UNNECESSARY:
//  • Iterator/ExactSizeIterator/... - see slice instead
