//! [`NonNull`]-related utilities

use core::ptr::NonNull;



/// create `NonNull<[T]>` ← `(data: NonNull<T>, len: usize)`
pub /*const*/ fn slice_from_raw_parts<T>(data: NonNull<T>, len: usize) -> NonNull<[T]> {
    let slice = core::ptr::slice_from_raw_parts_mut(data.as_ptr(), len);
    unsafe { NonNull::new_unchecked(slice) }
}
