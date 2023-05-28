#![allow(dead_code)] // some used for test code

use core::marker::PhantomData;
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



#[cfg(any(feature = "std", test))] std::thread_local! { static TESTER_COUNTS: [core::cell::Cell<usize>; 256] = [(); 256].map(|_| core::cell::Cell::new(0)); }

#[cfg(any(feature = "std", test))] #[derive(Debug)] pub struct Tester {
    data: u8,
    _phantom: PhantomData<*const ()>,
}

#[cfg(any(feature = "std", test))] impl Tester {
    pub fn new(data: u8) -> Self { TESTER_COUNTS.with(|tc| tc[data as usize].set(tc[data as usize].get() + 1)); Self { data, _phantom: PhantomData } }
    pub fn get(&self) -> u8 { self.data }
    pub fn counts() -> [usize; 256] { TESTER_COUNTS.with(|tc| tc.clone().map(|c| c.get())) }
}

#[cfg(any(feature = "std", test))] impl core::ops::Deref for Tester {
    type Target = u8;
    fn deref(&self) -> &u8 { &self.data }
}

#[cfg(any(feature = "std", test))] impl Drop for Tester {
    fn drop(&mut self) {
        let data = self.data as usize;
        TESTER_COUNTS.with(|tc| tc[data].set(tc[data].get().checked_sub(1).expect("count went negative, a util::drop::Tester was presumably dropped multiple times")))
    }
}

#[cfg(any(feature = "std", test))] impl Clone for Tester {
    fn clone(&self) -> Self { Self::new(self.data) }
}

#[cfg(any(feature = "std", test))] impl Default for Tester {
    fn default() -> Self { Self::new(0) }
}
