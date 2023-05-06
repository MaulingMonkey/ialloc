use crate::*;
use zsty::*;

use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit, size_of};
use core::ptr::*;



/// [`zsty::Alloc`]-friendly [`alloc::boxed::Box`] alternative
pub struct ABox<T: ?Sized, A: Free> {
    allocator:  A,
    data:       NonNull<T>,
    _phantom:   PhantomData<T>,
}

unsafe impl<T: ?Sized + Send, A: Free + Send> Send for ABox<T, A> {}
unsafe impl<T: ?Sized + Sync, A: Free + Sync> Sync for ABox<T, A> {} // A: Sync is mainly required to safely clone/default allocator

impl<T: ?Sized, A: Free> Drop for ABox<T, A> {
    fn drop(&mut self) {
        let layout = self.layout();
        unsafe { self.data.as_ptr().drop_in_place() };
        unsafe { self.allocator.free(self.data.cast(), layout) };
    }
}

impl<T: ?Sized, A: Free> ABox<T, A> {
    #[inline(always)] pub fn allocator(&self) -> &A { &self.allocator }
    #[inline(always)] pub(super) fn data(&self) -> NonNull<T> { self.data }
    #[inline(always)] fn layout(&self) -> Layout { Layout::for_value(&**self) }

    pub unsafe fn from_raw_in(data: NonNull<T>, allocator: A) -> Self {
        Self { data, allocator, _phantom: PhantomData }
    }

    const ASSERT_A_IS_ZST_FROM_RAW : () = assert!(size_of::<A>() == 0, "A is not a ZST - it is unlikely that `data` happens to be compatible with `A::default()`.  Prefer `ABox::from_raw_in` to specify an allocator instead.");
    pub unsafe fn from_raw(data: NonNull<T>) -> Self where A : Default {
        let _ = Self::ASSERT_A_IS_ZST_FROM_RAW;
        unsafe { Self::from_raw_in(data, A::default()) }
    }

    pub fn into_raw_with_allocator(this: Self) -> (NonNull<T>, A) {
        let this        = ManuallyDrop::new(this);
        let data        = this.data;
        let allocator   = unsafe { core::ptr::read(&this.allocator) };
        (data, allocator)
    }

    const ASSERT_A_IS_ZST_INTO_RAW : () = assert!(size_of::<A>() == 0, "A is not a ZST - it is unlikely that `data` can be freed with anything but the discarded allocator.  Prefer `ABox::into_raw_with_allocator` to acquire `data`'s allocator as well.");
    pub fn into_raw(this: Self) -> NonNull<T> {
        let _ = Self::ASSERT_A_IS_ZST_INTO_RAW;
        Self::into_raw_with_allocator(this).0
    }

    pub fn leak<'a>(this: Self) -> &'a mut T where A: 'a { unsafe { ABox::into_raw_with_allocator(this).0.as_mut() } }
}

// TODO:
//  • [ ] downcast
//  • [ ] downcast_unchecked
//  • [ ] into_boxed_slice
//  • [ ] into_pin
//  • [ ] pin
//  • [ ] pin_in

impl<T, A: Free> ABox<T, A> {
    // Sized

    pub fn into_inner(self) -> T { self.into_inner_with_allocator().0 }

    pub fn into_inner_with_allocator(self) -> (T, A) {
        let layout = self.layout();
        let (ptr, allocator) = ABox::into_raw_with_allocator(self);
        let data = unsafe { ptr.as_ptr().read() };
        unsafe { allocator.free(ptr.cast(), layout) };
        (data, allocator)
    }
}

impl<T, A: Free> ABox<MaybeUninit<T>, A> {
    // MaybeUninit<T>

    // XXX: make pub?
    pub(super) unsafe fn assume_init(self) -> ABox<T, A> {
        let (data, allocator) = ABox::into_raw_with_allocator(self);
        unsafe { ABox::from_raw_in(data.cast(), allocator) }
    }

    // XXX: make pub?
    pub(super) fn write(boxed: Self, value: T) -> ABox<T, A> {
        unsafe { boxed.data.as_ptr().write(MaybeUninit::new(value)) };
        unsafe { boxed.assume_init() }
    }
}

impl<T, A: Free> ABox<[MaybeUninit<T>], A> {
    // [MaybeUninit<T>]

    // XXX: make pub?
    #[allow(dead_code)] pub(super) unsafe fn assume_init(self) -> ABox<[T], A> {
        let (data, allocator) = ABox::into_raw_with_allocator(self);
        let data = util::nn::slice_from_raw_parts(data.cast(), data.len());
        unsafe { ABox::from_raw_in(data, allocator) }
    }
}
