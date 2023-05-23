//! [`NonNull`]-related utilities

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// create `NonNull<[T]>` ← `(data: NonNull<T>, len: usize)`
pub /*const*/ fn slice_from_raw_parts<T>(data: NonNull<T>, len: usize) -> NonNull<[T]> {
    let slice = core::ptr::slice_from_raw_parts_mut(data.as_ptr(), len);
    // SAFETY: ✔️ `data` is NonNull so the derived `slice` should be too
    unsafe { NonNull::new_unchecked(slice) }
}

pub /*const*/ fn slice_assume_init<T>(data: NonNull<[MaybeUninit<T>]>) -> NonNull<[T]> {
    let len = data.len();
    let ptr = data.as_ptr() as *mut MaybeUninit<T>;
    let ptr = unsafe { NonNull::new_unchecked(ptr.cast()) };
    slice_from_raw_parts(ptr, len)
}

pub fn dangling<T>(layout: Layout) -> NonNull<T> {
    NonNull::new(layout.align() as _).unwrap_or(NonNull::dangling())
}
