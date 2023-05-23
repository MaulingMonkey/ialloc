#[cfg(doc)] use crate::fat;
use crate::boxed::ABox;
use crate::error::ExcessiveSliceRequestedError;
use crate::meta::*;
use crate::fat::*;

use core::mem::ManuallyDrop;
use core::mem::MaybeUninit;
use core::ops::{RangeBounds, Bound};
use core::ptr::NonNull;



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

    fn try_append(&mut self, other: &mut AVec<T, impl Free>) -> Result<(), A::Error> where A : Realloc {
        self.try_reserve(other.len())?;
        debug_assert!(self.len() + other.len() <= self.capacity());
        unsafe { core::ptr::copy_nonoverlapping(other.as_mut_ptr(), self.as_mut_ptr().add(self.len()), other.len()) };
        other.len = 0;
        Ok(())
    }

    #[cfg(global_oom_handling)] pub fn append(&mut self, other: &mut AVec<T, impl Free>) where A : Realloc { self.try_append(other).expect("out of memory") }

    pub fn clear(&mut self) { self.truncate(0) }

    // TODO: dedup, dedup_by, dedup_by_key
    // TODO: drain, drain_filter

    pub(crate) fn try_extend_from_slice(&mut self, slice: &[T]) -> Result<(), A::Error> where T : Clone, A : Realloc {
        self.try_reserve(slice.len())?;
        for value in slice.iter().cloned() { unsafe { self.push_within_capacity_unchecked(value) } }
        Ok(())
    }

    #[cfg(global_oom_handling)] pub fn extend_from_slice(&mut self, slice: &[T]) where T : Clone, A : Realloc { self.try_extend_from_slice(slice).expect("out of memory") }

    #[cfg(global_oom_handling)]
    #[cfg(feature = "panicy-bounds")]
    pub fn extend_from_within<R: RangeBounds<usize>>(&mut self, src: R) where T : Clone, A : Realloc {
        let start = match src.start_bound() {
            Bound::Unbounded    => 0,
            Bound::Included(i)  => *i,
            Bound::Excluded(i)  => i.checked_add(1).expect("start out of bounds"),
        };
        let end = match src.end_bound() {
            Bound::Unbounded    => self.len,
            Bound::Included(i)  => i.checked_add(1).expect("end out of bounds"),
            Bound::Excluded(i)  => *i,
        };
        assert!(start <= end);
        assert!(end <= self.len);
        self.reserve(end-start);

        for i in start .. end {
            let value = unsafe { self.get_unchecked(i) }.clone();
            unsafe { self.push_within_capacity_unchecked(value) }
        }
    }

    // TODO: pub
    #[allow(dead_code)]
    pub(crate) unsafe fn from_raw_parts(data: NonNull<MaybeUninit<T>>, length: usize, capacity: usize) -> Self where A : Stateless {
        unsafe { Self::from_raw_parts_in(data, length, capacity, A::default()) }
    }

    // TODO: pub
    pub(crate) unsafe fn from_raw_parts_in(data: NonNull<MaybeUninit<T>>, length: usize, capacity: usize, allocator: A) -> Self {
        let data = crate::util::nn::slice_from_raw_parts(data, capacity);
        let data = unsafe { ABox::from_raw_in(data, allocator) };
        Self { data, len: length }
    }

    // TODO: insert

    fn try_into_boxed_slice(self) -> Result<ABox<[T], A>, (Self, A::Error)> where A : Realloc {
        let mut v = self;
        if let Err(err) = v.try_shrink_to_fit() { return Err((v, err)) }

        // decompose without Drop
        let v = ManuallyDrop::new(v);
        let data = unsafe { std::ptr::read(&v.data) };
        core::mem::forget(v);

        //let (raw, allocator) = data.into_raw_with_allocator();
        //Ok(ABox::from_raw_in(raw, allocator))
        Ok(unsafe { data.assume_init() })
    }

    #[cfg(global_oom_handling)] pub fn into_boxed_slice(self) -> ABox<[T], A> where A : Realloc { self.try_into_boxed_slice().map_err(|(_, err)| err).expect("unable to shrink alloc") }

    // TODO: into_flattened
    // TODO: into_raw_parts, into_raw_parts_with_allocator
    // TODO: leak

    #[cfg(    global_oom_handling )] pub fn new() -> Self where A : Alloc + Default + ZstSupported { Self::with_capacity(0) }
    #[cfg(not(global_oom_handling))] pub fn new() -> Self where A : Alloc + Default + ZstInfalliable { Self::try_with_capacity(0).expect("zero-sized allocation failed despite ZstInfalliable") }
    #[cfg(    global_oom_handling )] pub fn new_in(allocator: A) -> Self where A : Alloc + ZstSupported { Self::with_capacity_in(0, allocator) }
    #[cfg(not(global_oom_handling))] pub fn new_in(allocator: A) -> Self where A : Alloc + ZstInfalliable { Self::try_with_capacity_in(0, allocator).expect("zero-sized allocation failed despite ZstInfalliable") }

    pub fn pop(&mut self) -> Option<T> {
        let idx_to_pop = self.len.checked_sub(1)?;
        self.len = idx_to_pop;
        unsafe { Some(self.as_mut_ptr().add(idx_to_pop).read()) }
    }

    pub(crate) fn try_push(&mut self, value: T) -> Result<(), (T, A::Error)> where A : Realloc {
        if let Err(e) = self.try_reserve(1) { return Err((value, e)) }
        debug_assert!(self.len < self.capacity());
        Ok(unsafe { self.push_within_capacity_unchecked(value) })
    }

    #[cfg(global_oom_handling)] pub fn push(&mut self, value: T) where A : Realloc { self.try_push(value).map_err(|(_, e)| e).expect("out of memory") }

    unsafe fn push_within_capacity_unchecked(&mut self, value: T) {
        unsafe { self.as_mut_ptr().add(self.len).write(value) };
        self.len += 1;
    }

    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        if self.len < self.capacity() {
            Ok(unsafe { self.push_within_capacity_unchecked(value) })
        } else {
            Err(value)
        }
    }

    pub(crate) fn try_remove(&mut self, index: usize) -> Option<T> {
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

    #[cfg(global_oom_handling)] pub fn reserve(&mut self, additional: usize) where A : Realloc { self.try_reserve(additional).expect("unable to reserve more memory") }
    #[cfg(global_oom_handling)] pub fn reserve_exact(&mut self, additional: usize) where A : Realloc { self.try_reserve_exact(additional).expect("unable to reserve more memory") }

    pub(crate) fn try_resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) -> Result<(), A::Error> where A : Realloc {
        if let Some(additional) = new_len.checked_sub(self.len) {
            self.try_reserve(additional)?;
            while self.len() < new_len { unsafe { self.push_within_capacity_unchecked(f()) } }
        } else {
            self.truncate(new_len);
        }
        Ok(())
    }

    fn try_resize(&mut self, new_len: usize, value: T) -> Result<(), A::Error> where T : Clone, A : Realloc {
        self.try_resize_with(new_len, || value.clone())
    }

    #[cfg(global_oom_handling)] pub fn resize(&mut self, new_len: usize, value: T) where T : Clone, A : Realloc { self.try_resize(new_len, value).expect("unable to reserve more memory") }
    #[cfg(global_oom_handling)] pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, f: F) where A : Realloc { self.try_resize_with(new_len, f).expect("unable to reserve more memory") }

    // TODO: retain, retain_mut

    pub(crate) fn try_shrink_to(&mut self, min_capacity: usize) -> Result<(), A::Error> where A : Realloc { let c = min_capacity.max(self.len()); ABox::try_realloc_uninit_slice(&mut self.data, c) }
    pub(crate) fn try_shrink_to_fit(&mut self) -> Result<(), A::Error> where A : Realloc { self.try_shrink_to(self.len()) }
    #[cfg(global_oom_handling)] pub fn shrink_to(&mut self, min_capacity: usize) where A : Realloc { self.try_shrink_to(min_capacity).expect("unable to reallocate") }
    #[cfg(global_oom_handling)] pub fn shrink_to_fit(&mut self) where A : Realloc { self.try_shrink_to_fit().expect("unable to reallocate") }

    // TODO: splice
    // TODO: split_at_sparse_mut
    // TODO: split_off

    pub(crate) fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            self.data.swap(index, self.len-1);
            self.pop()
        } else {
            None
        }
    }

    #[cfg(feature = "panicy-bounds")] pub fn swap_remove(&mut self, index: usize) -> T { self.try_swap_remove(index).expect("index out of bounds") }

    pub fn truncate(&mut self, len: usize) {
        if let Some(to_drop) = self.len.checked_sub(len) {
            let to_drop = core::ptr::slice_from_raw_parts_mut(unsafe { self.as_mut_ptr().add(len) }, to_drop);
            self.len = len;
            unsafe { to_drop.drop_in_place() };
        }
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), A::Error> where A : Realloc {
        let new_capacity = self.len().checked_add(additional).ok_or_else(|| ExcessiveSliceRequestedError { requested: !0 })?;
        if new_capacity <= self.capacity() { return Ok(()) }
        let new_capacity = new_capacity.max(self.capacity().saturating_mul(2));
        ABox::try_realloc_uninit_slice(&mut self.data, new_capacity)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), A::Error> where A : Realloc {
        let new_capacity = self.len().checked_add(additional).ok_or_else(|| ExcessiveSliceRequestedError { requested: !0 })?;
        if new_capacity <= self.capacity() { return Ok(()) }
        ABox::try_realloc_uninit_slice(&mut self.data, new_capacity)
    }

    pub(crate) fn try_with_capacity_in(capacity: usize, allocator: A) -> Result<Self, A::Error> where A : Alloc + ZstSupported { Ok(Self { data: ABox::try_new_uninit_slice_in(capacity, allocator)?, len: 0 }) }
    pub(crate) fn try_with_capacity(   capacity: usize) -> Result<Self, A::Error> where A : Alloc + Default + ZstSupported     { Self::try_with_capacity_in(capacity, A::default()) }
    #[cfg(global_oom_handling)] pub fn with_capacity_in(capacity: usize, allocator: A) -> Self where A : Alloc + ZstSupported  { Self::try_with_capacity_in(capacity, allocator).expect("out of memory") }
    #[cfg(global_oom_handling)] pub fn with_capacity(   capacity: usize) -> Self where A : Alloc + Default + ZstSupported      { Self::try_with_capacity(capacity ).expect("out of memory") }
}
