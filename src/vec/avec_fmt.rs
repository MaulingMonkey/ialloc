use crate::fat::*;
use crate::vec::AVec;

use core::fmt::{self, Debug, Formatter};



impl<T: Debug, A: Free + Debug> Debug for AVec<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AVec")
            .field("allocator", self.allocator())
            .field("capacity", &self.capacity())
            .field("data", &self.as_slice())
        .finish()
    }
}
