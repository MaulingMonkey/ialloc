use crate::{thin};
use crate::{AllocNN};



unsafe impl<A: thin::FreeNullable> thin::Free for A {
    unsafe fn free(&self, ptr: AllocNN) { unsafe { thin::FreeNullable::free(self, ptr.as_ptr()) } }
}

unsafe impl<A: thin::SizeOf> thin::SizeOfDebug for A {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> { unsafe { thin::SizeOf::size_of(self, ptr) } }
}
