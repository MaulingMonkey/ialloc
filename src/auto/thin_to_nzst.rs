use crate::{thin, nzst};
use crate::{AllocNN, LayoutNZ};



unsafe impl<A: thin::Free> nzst::Free for A {
    unsafe fn free(&self, ptr: AllocNN, _layout: LayoutNZ) { unsafe { thin::Free::free(self, ptr) } }
}
