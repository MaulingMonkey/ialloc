use crate::{thin};
use crate::{AllocNN};



unsafe impl<A: thin::FreeNullable> thin::Free for A {
    unsafe fn free(&self, ptr: AllocNN) { unsafe { thin::FreeNullable::free(self, ptr.as_ptr()) } }
}
