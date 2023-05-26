use core::ptr::NonNull;



pub struct InPlaceOnDrop<T: ?Sized>(Option<NonNull<T>>);

impl<T: ?Sized> InPlaceOnDrop<T> {
    #[allow(dead_code)] pub unsafe fn new(to_drop_on_drop: *mut T) -> Self { Self(NonNull::new(to_drop_on_drop)) }
    pub fn forget(self) { core::mem::forget(self) }
    #[allow(dead_code)] pub fn drop(self) { core::mem::drop(self) }

    pub unsafe fn set(&mut self, to_drop_on_drop: *mut T) { self.0 = NonNull::new(to_drop_on_drop) }
}

impl<T: ?Sized> core::ops::Drop for InPlaceOnDrop<T> {
    fn drop(&mut self) {
        if let Some(to_drop) = self.0 {
            unsafe { core::ptr::drop_in_place(to_drop.as_ptr()) }
        }
    }
}

impl<T: ?Sized> Default for InPlaceOnDrop<T> {
    fn default() -> Self { Self(None) }
}
