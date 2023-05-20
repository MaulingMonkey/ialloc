#[cfg(doc)] use crate::fat;
use crate::boxed::ABox;
use crate::fat::*;

use core::mem::MaybeUninit;



/// [`fat::Alloc`]-friendly [`alloc::vec::Vec`] alternative
pub struct AVec<T, A: Free> {
    data:   ABox<[MaybeUninit<T>], A>,
    len:    usize,
}

impl<T, A: Free> Drop for AVec<T, A> { fn drop(&mut self) { self.clear() } }

impl<T, A: Free> AVec<T, A> {
    #[inline(always)] pub fn allocator(&self) -> &A { ABox::allocator(&self.data) }
    #[inline(always)] pub fn as_ptr(&self) -> *const T { self.data.as_ptr().cast() }
    #[inline(always)] pub fn as_mut_ptr(&mut self) -> *mut T { self.data.as_mut_ptr().cast() }
    #[inline(always)] pub fn as_slice(&self) -> &[T] { unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len) } }
    #[inline(always)] pub fn as_slice_mut(&mut self) -> &mut [T] { unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } }
    #[inline(always)] pub fn capacity(&self) -> usize { self.data.len() }
    #[inline(always)] pub fn is_empty(&self) -> bool { self.len() == 0 }
    #[inline(always)] pub fn len(&self) -> usize { self.len }
    #[inline(always)] pub unsafe fn set_len(&mut self, new_len: usize) { self.len = new_len; }
    #[inline(always)] pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] { self.data.get_mut(self.len..).unwrap_or(&mut []) }

    // TODO: append

    pub fn clear(&mut self) {
        let to_drop = core::ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len);
        self.len = 0;
        unsafe { to_drop.drop_in_place() };
    }

    // TODO: dedup, dedup_by, dedup_by_key
    // TODO: drain, drain_filter
    // TODO: extend_from_slice, extend_from_within
    // TODO: from_raw_parts, from_raw_parts_in
    // TODO: insert
    // TODO: into_boxed_slice, into_flattened
    // TODO: into_raw_parts, into_raw_parts_with_allocator
    // TODO: leak
    // TODO: new, new_in

    pub fn pop(&mut self) -> Option<T> {
        let idx_to_pop = self.len.checked_sub(1)?;
        self.len = idx_to_pop;
        unsafe { Some(self.as_mut_ptr().add(idx_to_pop).read()) }
    }

    // TODO: push

    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        if self.len < self.capacity() {
            unsafe { self.as_mut_ptr().add(self.len).write(value) };
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }

    // TODO: pub?
    fn try_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            let count = self.len - index;
            let src = unsafe { self.as_mut_ptr().add(index+1) };
            let dst = unsafe { self.as_mut_ptr().add(index+0) };
            let value : T = unsafe { dst.read() };
            self.len -= 1;
            unsafe { core::ptr::copy(src, dst, count) };
            Some(value)
        } else {
            None
        }
    }

    #[cfg(feature = "panicy-bounds")] pub fn remove(&mut self, index: usize) -> T { self.try_remove(index).expect("index out of bounds") }

    // TODO: reserve, reserve_exact
    // TODO: resize, resize_with
    // TODO: retain, retain_mut
    // TODO: shrink_to, shrink_to_fit
    // TODO: splice
    // TODO: split_at_sparse_mut
    // TODO: split_off

    // TODO: pub?
    fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            self.data.swap(index, self.len-1);
            self.pop()
        } else {
            None
        }
    }

    #[cfg(feature = "panicy-bounds")] pub fn swap_remove(&mut self, index: usize) -> T { self.try_swap_remove(index).expect("index out of bounds") }

    pub fn truncate(&mut self, len: usize) {
        if len > self.len { return }
        let to_drop = core::ptr::slice_from_raw_parts_mut(unsafe { self.as_mut_ptr().add(len) }, self.len - len);
        self.len = len;
        unsafe { to_drop.drop_in_place() };
    }

    // TODO: try_reserve, try_reserve_exact
    // TODO: with_capacity, with_capacity_in
}
