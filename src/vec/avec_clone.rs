use crate::fat::*;
use crate::meta::*;
use crate::vec::AVec;



#[cfg(global_oom_handling)] impl<T: Clone, A: Realloc + Clone + ZstSupported> Clone for AVec<T, A> {
    fn clone(&self) -> Self {
        let mut v = Self::new_in(self.allocator().clone());
        v.extend_from_slice(self);
        v
    }

    // TODO: clone_from
}

// TODO: falliable/placement clone alternatives
