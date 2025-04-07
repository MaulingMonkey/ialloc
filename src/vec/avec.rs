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
    pub(super) data:   ABox<[MaybeUninit<T>], A>,
    pub(super) len:    usize,
}

impl<T, A: Free> Drop for AVec<T, A> { fn drop(&mut self) { self.clear() } }

impl<T, A: Free> AVec<T, A> {
    /// Retrieve the [`fat::Free`] (+ [`fat::Alloc`] + [`fat::Realloc`] + ...) associated with this [`AVec`].
    #[inline(always)] pub fn allocator(&self) -> &A { ABox::allocator(&self.data) }

    /// Get a pointer to the underlying buffer of `T`s without going through a reference to `T` or `[T]` (which could narrow provenance.)
    ///
    /// Unlike <code>[avec](Self).[as_slice](Self::as_slice)\(\).[as_ptr](slice::as_ptr)\(\)</code>, the spatial provenance of this pointer extends into the (likely) uninitialized
    /// data between <code>[avec](Self).[len](Self::len)\(\) .. [avec](Self).[capacity](Self::capacity)\(\)</code>, which can be used to initialize elements in-place before calling <code>[avec](Self).[set_len](Self::set_len)\(\)</code>.
    #[inline(always)] pub fn as_ptr(&self) -> *const T { ABox::as_ptr(&self.data).cast() }

    /// Get a pointer to the underlying buffer of `T`s without going through a reference to `T` or `[T]` (which could narrow provenance.)
    ///
    /// Unlike <code>[avec](Self).[as_slice_mut](Self::as_slice_mut)\(\).[as_mut_ptr](slice::as_mut_ptr)\(\)</code>, the spatial provenance of this pointer extends into the (likely) uninitialized
    /// data between <code>[avec](Self).[len](Self::len)\(\) .. [avec](Self).[capacity](Self::capacity)\(\)</code>, which can be used to initialize elements in-place before calling <code>[avec](Self).[set_len](Self::set_len)\(\)</code>.
    #[inline(always)] pub fn as_mut_ptr(&mut self) -> *mut T { ABox::as_mut_ptr(&mut self.data).cast() }

    /// Return a slice containing the entire vector.  Equivalent to `&avec[..]`.
    #[inline(always)] pub fn as_slice(&self) -> &[T] { unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len) } }

    /// Return a slice containing the entire vector.  Equivalent to `&mut avec[..]`.
    #[inline(always)] pub fn as_slice_mut(&mut self) -> &mut [T] { unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } }

    /// Return the maximum number of elements this vector can store before reallocating.
    #[inline(always)] pub fn capacity(&self) -> usize { self.data.len() }

    /// Return `true` if the vector contains no elements.  Equivalent to <code>avec.[len](Self::len)() == 0</code>.
    #[inline(always)] pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Return the number of elements in the vector.  Valid indicies into said vector are `0 .. avec.len()`.
    #[inline(always)] pub fn len(&self) -> usize { self.len }

    /// Change the length of the vector without:
    /// *   initializing new elements if grown (UB hazard!)
    /// *   reallocating if the length specified is larger than capacity (UB hazard!)
    /// *   [`Drop`]ing old elements if shrunk (leak hazard!)
    ///
    /// ### Safety
    /// *   `new_len` must be less than or equal to <code>[capacity](Self::capacity)()</code>.
    /// *   If <code>new_len &gt; [avec](Self).[len](Self::len)()</code>, the elements between <code>[avec](Self).[len](Self::len)() .. new_len</code> must have been initialized.
    #[inline(always)] pub unsafe fn set_len(&mut self, new_len: usize) { debug_assert!(new_len <= self.capacity(), "undefined behavior: `new_len` exceeds `capacity()`"); self.len = new_len; }

    /// Return a slice to the uninitialized elements between <code>[avec](Self).[len](Self::len)() .. [avec](Self).[capacity](Self::capacity)()</code>.
    #[inline(always)] pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] { self.data.get_mut(self.len..).unwrap_or(&mut []) }

    /// Move all elements from `other` to `self` without [`Clone`]ing.  Results in `other` being emptied.
    ///
    /// Returns an <code>[Err]\(...\)</code> without moving anything if (re)allocating the vector fails.
    fn try_append(&mut self, other: &mut AVec<T, impl Free>) -> Result<(), A::Error> where A : Realloc {
        self.try_reserve(other.len())?;
        debug_assert!(self.len() + other.len() <= self.capacity());
        unsafe { core::ptr::copy_nonoverlapping(other.as_mut_ptr(), self.as_mut_ptr().add(self.len()), other.len()) };
        other.len = 0;
        Ok(())
    }

    /// Move all elements from `other` to `self` without [`Clone`]ing.  Results in `other` being emptied.
    ///
    /// Panics without moving anything if (re)allocating the vector fails.
    #[cfg(global_oom_handling)] pub fn append(&mut self, other: &mut AVec<T, impl Free>) where A : Realloc { self.try_append(other).expect("out of memory") }

    /// Remove all elements from `self` by [`Drop`]ing them.
    pub fn clear(&mut self) { self.truncate(0) }

    // TODO: dedup, dedup_by, dedup_by_key
    // TODO: drain, drain_filter

    /// Append all elements from `slice` to `self` by [`Clone`]ing.
    ///
    /// Returns an <code>[Err]\(...\)</code> without [`Clone`]ing anything if (re)allocating the vector fails.
    pub(crate) fn try_extend_from_slice(&mut self, slice: &[T]) -> Result<(), A::Error> where T : Clone, A : Realloc {
        self.try_reserve(slice.len())?;
        for value in slice.iter().cloned() { unsafe { self.push_within_capacity_unchecked(value) } }
        Ok(())
    }

    /// Append all elements from `slice` to `self` by [`Clone`]ing.
    ///
    /// Panics without [`Clone`]ing anything if (re)allocating the vector fails.
    #[cfg(global_oom_handling)] pub fn extend_from_slice(&mut self, slice: &[T]) where T : Clone, A : Realloc { self.try_extend_from_slice(slice).expect("out of memory") }

    /// [`Clone`] elements from within `self` and append them to `self`.
    ///
    /// Panics without [`Clone`]ing anything if (re)allocating the vector fails.
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

    /// Construct an [`AVec`] from it's raw pointer, length, capacity, and default allocator.
    ///
    /// You *generally* only want to do this when you previously broke down a vector of the same type with <code>[into_raw_parts](Self::into_raw_parts)</code>.
    ///
    /// ### Safety
    /// *   `(data, capacity, align_of::<T>())` should exactly describe an allocation belonging to allocator `A`.
    /// *   `0 .. length` should be initialized elements of type `T`.
    /// *   [`AVec`] takes exclusive ownership of said allocation.
    pub unsafe fn from_raw_parts(data: NonNull<T>, length: usize, capacity: usize) -> Self where A : Stateless {
        unsafe { Self::from_raw_parts_in(data, length, capacity, A::default()) }
    }

    /// Construct an [`AVec`] from it's raw pointer, length, capacity, and allocator.
    ///
    /// You *generally* only want to do this when you previously broke down a vector of the same type with <code>[into_raw_parts_with_allocator](Self::into_raw_parts_with_allocator)</code>.
    ///
    /// ### Safety
    /// *   `(data, capacity, align_of::<T>())` should exactly describe an allocation belonging to `allocator`.
    /// *   `0 .. length` should be initialized elements of type `T`.
    /// *   [`AVec`] takes exclusive ownership of said allocation.
    pub unsafe fn from_raw_parts_in(data: NonNull<T>, length: usize, capacity: usize, allocator: A) -> Self {
        let data = crate::util::nn::slice_from_raw_parts(data.cast(), capacity);
        let data = unsafe { ABox::from_raw_in(data, allocator) };
        Self { data, len: length }
    }

    // TODO: insert

    /// Convert an <code>[AVec]&lt;T, A&gt;</code> into an <code>[ABox]&lt;\[T\], A&gt;</code>.
    ///
    /// Returns an <code>[Err]\(...\)</code> if the allocation could not be shrunk to match it's length exactly.
    fn try_into_boxed_slice(self) -> Result<ABox<[T], A>, (Self, A::Error)> where A : Realloc {
        let mut v = self;
        if let Err(err) = v.try_shrink_to_fit() { return Err((v, err)) }

        // decompose without Drop
        let v = ManuallyDrop::new(v);
        let data = unsafe { core::ptr::read(&v.data) };
        core::mem::forget(v);

        //let (raw, allocator) = data.into_raw_with_allocator();
        //Ok(ABox::from_raw_in(raw, allocator))
        Ok(unsafe { data.assume_init() })
    }

    /// Convert an <code>[AVec]&lt;T, A&gt;</code> into an <code>[ABox]&lt;\[T\], A&gt;</code>.
    ///
    /// Panics if the allocation could not be shrunk to match it's length exactly.
    #[cfg(global_oom_handling)] pub fn into_boxed_slice(self) -> ABox<[T], A> where A : Realloc { self.try_into_boxed_slice().map_err(|(_, err)| err).expect("unable to shrink alloc") }

    // TODO: into_flattened

    /// Convert an [`AVec`] into it's raw pointer, length, and capacity.
    ///
    /// This will leak memory unless you later free said memory yourself (perhaps by reconstructing an [`AVec`] through [`AVec::from_raw_parts`].)
    pub fn into_raw_parts(self) -> (NonNull<T>, usize, usize) where A : Stateless {
        let (data, len, cap, _) = self.into_raw_parts_with_allocator();
        (data, len, cap)
    }

    /// Convert an [`AVec`] into it's raw pointer, length, capacity, and allocator.
    ///
    /// This will leak memory unless you later free said memory yourself (perhaps by reconstructing an [`AVec`] through [`AVec::from_raw_parts_in`].)
    pub fn into_raw_parts_with_allocator(self) -> (NonNull<T>, usize, usize, A) {
        let this            = ManuallyDrop::new(self);
        let len             = this.len;
        let data            = unsafe { core::ptr::read(&this.data) };
        let _               = this;
        let (data, alloc)   = ABox::into_raw_with_allocator(data);
        let cap             = data.len();
        (data.cast(), len, cap, alloc)
    }

    // TODO: leak

    /// Create an empty [`AVec`] using a default allocator.
    pub fn new() -> Self where A : Alloc + Default + ZstInfalliableOrGlobalOomHandling { Self::try_with_capacity(0).unwrap() }

    /// Create an empty [`AVec`] using `allocator`.
    pub fn new_in(allocator: A) -> Self where A : Alloc + ZstInfalliableOrGlobalOomHandling { Self::try_with_capacity_in(0, allocator).unwrap() }

    /// Remove and return the last (highest index) element from the [`AVec`], if any.
    pub fn pop(&mut self) -> Option<T> {
        let idx_to_pop = self.len.checked_sub(1)?;
        self.len = idx_to_pop;
        unsafe { Some(self.as_mut_ptr().add(idx_to_pop).read()) }
    }

    /// Attempt to push `value` to the end of the [`AVec`].
    ///
    /// Returns <code>[Err]\(\(value, ...\)\)</code> if <code>[len](Self::len)() == [capacity](Self::capacity)()</code> and reallocation fails.
    pub(crate) fn try_push(&mut self, value: T) -> Result<(), (T, A::Error)> where A : Realloc {
        if let Err(e) = self.try_reserve(1) { return Err((value, e)) }
        debug_assert!(self.len < self.capacity());
        Ok(unsafe { self.push_within_capacity_unchecked(value) })
    }

    /// Attempt to push `value` to the end of the [`AVec`].
    ///
    /// Panics if <code>[len](Self::len)() == [capacity](Self::capacity)()</code> and reallocation fails.
    #[cfg(global_oom_handling)] pub fn push(&mut self, value: T) where A : Realloc { self.try_push(value).map_err(|(_, e)| e).expect("out of memory") }

    /// Attempt to push `value` to the end of the [`AVec`].
    ///
    /// ### Safety
    /// *   There must be unused capacity (e.g. it must be the case that <code>[len](Self::len)() &lt; [capacity](Self::capacity)()</code>.)
    unsafe fn push_within_capacity_unchecked(&mut self, value: T) {
        unsafe { self.as_mut_ptr().add(self.len).write(value) };
        self.len += 1;
    }

    /// Attempt to push `value` to the end of the [`AVec`].
    ///
    /// Returns <code>[Err]\(value\)</code> without attempting to reallocate if <code>[len](Self::len)() == [capacity](Self::capacity)()</code>.
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        if self.len < self.capacity() {
            Ok(unsafe { self.push_within_capacity_unchecked(value) })
        } else {
            Err(value)
        }
    }

    /// Remove and return the element at `index`, shifting all elements after it down by 1.
    ///
    /// Returns [`None`] if <code>index >= [len](Self::len)()</code>.
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

    /// Remove and return the element at `index`, shifting all elements after it down by 1.
    ///
    /// Panics if <code>index >= [len](Self::len)()</code>.
    #[cfg(feature = "panicy-bounds")] pub fn remove(&mut self, index: usize) -> T { self.try_remove(index).expect("index out of bounds") }

    /// Reserve enough <code>[capacity](Self::capacity)()</code> for *at least* <code>[len](Self::len)() + additional</code> elements.
    /// This may allocate more capacity than requested to encourage amortized constant behavior via exponential growth patterns.
    ///
    /// Noop if <code>[len](Self::len)() + additional <= [capacity](Self::capacity)()</code>.
    ///
    /// Panics if reallocation was necessary but failed.
    #[cfg(global_oom_handling)] pub fn reserve(&mut self, additional: usize) where A : Realloc { self.try_reserve(additional).expect("unable to reserve more memory") }

    /// Reserve enough <code>[capacity](Self::capacity)()</code> for *exactly* <code>[len](Self::len)() + additional</code> elements.
    /// Beware: This will avoid exponential growth, which can easily lead to O(N<sup>2</sup>) behavior!
    ///
    /// Noop if <code>[len](Self::len)() + additional <= [capacity](Self::capacity)()</code>.
    ///
    /// Panics if reallocation was necessary but failed.
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

    /// Resize `self` to be `new_len` elements.  `value` will be repeatedly [`Clone`]ed if <code>new_len &gt; [len](Self::len)()</code>, or ignored otherwise.
    ///
    /// Panics if reallocation was necessary but failed.
    #[cfg(global_oom_handling)] pub fn resize(&mut self, new_len: usize, value: T) where T : Clone, A : Realloc { self.try_resize(new_len, value).expect("unable to reserve more memory") }

    /// Resize `self` to be `new_len` elements.  If <code>new_len &gt; [len](Self::len)()</code>, `f()` will be called to create new elements, otherwise `f` is ignored.
    ///
    /// Panics if reallocation was necessary but failed.
    #[cfg(global_oom_handling)] pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, f: F) where A : Realloc { self.try_resize_with(new_len, f).expect("unable to reserve more memory") }

    /// Remove all elements `e` of `self` where `!f(e)`.
    pub fn retain    <F: FnMut(&    T) -> bool>(&mut self, mut f: F) { self.retain_mut(|v| f(v)) }

    /// Remove all elements `e` of `self` where `!f(e)`.
    pub fn retain_mut<F: FnMut(&mut T) -> bool>(&mut self, mut f: F) { self.retain_imp(|v| f(v)) }

    /// Shrink <code>self.[capacity](Self::capacity)()</code> to <code>min_capacity.max(self.[len](Self::len)())</code>.
    ///
    /// Returns an <code>[Err]\(...\)</code> if reallocation fails.
    pub(crate) fn try_shrink_to(&mut self, min_capacity: usize) -> Result<(), A::Error> where A : Realloc { let c = min_capacity.max(self.len()); ABox::try_realloc_uninit_slice(&mut self.data, c) }

    /// Shrink <code>self.[capacity](Self::capacity)()</code> to <code>self.[len](Self::len)()</code>.
    ///
    /// Returns an <code>[Err]\(...\)</code> if reallocation fails.
    pub(crate) fn try_shrink_to_fit(&mut self) -> Result<(), A::Error> where A : Realloc { self.try_shrink_to(self.len()) }

    /// Shrink <code>self.[capacity](Self::capacity)()</code> to <code>min_capacity.max(self.[len](Self::len)())</code>.
    ///
    /// Panics if reallocation fails.
    #[cfg(global_oom_handling)] pub fn shrink_to(&mut self, min_capacity: usize) where A : Realloc { self.try_shrink_to(min_capacity).expect("unable to reallocate") }

    /// Shrink <code>self.[capacity](Self::capacity)()</code> to <code>self.[len](Self::len)()</code>.
    ///
    /// Panics if reallocation fails.
    #[cfg(global_oom_handling)] pub fn shrink_to_fit(&mut self) where A : Realloc { self.try_shrink_to_fit().expect("unable to reallocate") }

    // TODO: splice
    // TODO: split_at_sparse_mut
    // TODO: split_off

    /// Attempt to remove and return the element at `index` by swapping it with the last element and <code>[pop](Self::pop)()</code>ing it.
    ///
    /// Returns [`None`] if <code>index &gt;= [len](Self::len)()</code>.
    pub(crate) fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len {
            self.data.swap(index, self.len-1);
            self.pop()
        } else {
            None
        }
    }

    /// Remove and return the element at `index` by swapping it with the last element and <code>[pop](Self::pop)()</code>ing it.
    ///
    /// Panics if <code>index &gt;= [len](Self::len)()</code>.
    #[cfg(feature = "panicy-bounds")] pub fn swap_remove(&mut self, index: usize) -> T { self.try_swap_remove(index).expect("index out of bounds") }

    /// [`Drop`] the elements at <code>self\[len..\]</code>.
    ///
    /// Noop if <code>len &gt;= [self](Self).[len](Self::len)()</code>.
    pub fn truncate(&mut self, len: usize) {
        if let Some(to_drop) = self.len.checked_sub(len) {
            let to_drop = core::ptr::slice_from_raw_parts_mut(unsafe { self.as_mut_ptr().add(len) }, to_drop);
            self.len = len;
            unsafe { to_drop.drop_in_place() };
        }
    }

    /// Reserve enough <code>[capacity](Self::capacity)()</code> for *at least* <code>[len](Self::len)() + additional</code> elements.
    /// This may allocate more capacity than requested to encourage amortized constant behavior via exponential growth patterns.
    ///
    /// Noop if <code>[len](Self::len)() + additional <= [capacity](Self::capacity)()</code>.
    ///
    /// Returns an <code>[Err]\(...\)</code> if reallocation was necessary but failed.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), A::Error> where A : Realloc {
        let new_capacity = self.len().checked_add(additional).ok_or_else(|| ExcessiveSliceRequestedError { requested: !0 })?;
        if new_capacity <= self.capacity() { return Ok(()) }
        let new_capacity = new_capacity.max(self.capacity().saturating_mul(2));
        ABox::try_realloc_uninit_slice(&mut self.data, new_capacity)
    }

    /// Reserve enough <code>[capacity](Self::capacity)()</code> for *exactly* <code>[len](Self::len)() + additional</code> elements.
    /// Beware: This will avoid exponential growth, which can easily lead to O(N<sup>2</sup>) behavior!
    ///
    /// Noop if <code>[len](Self::len)() + additional <= [capacity](Self::capacity)()</code>.
    ///
    /// Returns an <code>[Err]\(...\)</code> if reallocation was necessary but failed.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), A::Error> where A : Realloc {
        let new_capacity = self.len().checked_add(additional).ok_or_else(|| ExcessiveSliceRequestedError { requested: !0 })?;
        if new_capacity <= self.capacity() { return Ok(()) }
        ABox::try_realloc_uninit_slice(&mut self.data, new_capacity)
    }

    /// Create an empty [`AVec`] using `allocator`, with a <code>[capacity](Self::capacity)()</code> of at least `capacity`.
    ///
    /// Returns an <code>[Err]\(...\)</code> if allocation fails.
    pub(crate) fn try_with_capacity_in(capacity: usize, allocator: A) -> Result<Self, A::Error> where A : Alloc + ZstSupported { Ok(Self { data: ABox::try_new_uninit_slice_in(capacity, allocator)?, len: 0 }) }

    /// Create an empty [`AVec`] using a default allocator, with a <code>[capacity](Self::capacity)()</code> of at least `capacity`.
    ///
    /// Returns an <code>[Err]\(...\)</code> if allocation fails.
    pub(crate) fn try_with_capacity(   capacity: usize) -> Result<Self, A::Error> where A : Alloc + Default + ZstSupported     { Self::try_with_capacity_in(capacity, A::default()) }

    /// Create an empty [`AVec`] using `allocator`, with a <code>[capacity](Self::capacity)()</code> of at least `capacity`.
    ///
    /// Panics if allocation fails.
    #[cfg(global_oom_handling)] pub fn with_capacity_in(capacity: usize, allocator: A) -> Self where A : Alloc + ZstSupported  { Self::try_with_capacity_in(capacity, allocator).expect("out of memory") }

    /// Create an empty [`AVec`] using a default allocator, with a <code>[capacity](Self::capacity)()</code> of at least `capacity`.
    ///
    /// Panics if allocation fails.
    #[cfg(global_oom_handling)] pub fn with_capacity(   capacity: usize) -> Self where A : Alloc + Default + ZstSupported      { Self::try_with_capacity(capacity ).expect("out of memory") }
}
