use crate::fat::*;
use crate::meta::*;
use crate::vec::AVec;



#[cfg(global_oom_handling)] impl<T: Clone, A: Realloc + Clone + ZstSupported> Clone for AVec<T, A> {
    fn clone(&self) -> Self {
        let mut v = Self::with_capacity_in(self.len(), self.allocator().clone());
        v.extend_from_slice(self);
        v
    }

    fn clone_from(&mut self, source: &Self) {
        self.reserve(source.len().saturating_sub(self.len())); // reserve before clear to preserve existing data on panic
        self.clear();
        self.extend_from_slice(source);
    }
}

// TODO: falliable/placement clone alternatives

/// Non-panicing alternatives to [`Clone`] / support for alternative allocators.
impl<T: Clone, A: Realloc + ZstSupported> AVec<T, A> {
    /// Clone the contents of `source` into `self`, reusing the existing allocation of [`AVec`].
    ///
    // TODO: examples
    pub fn try_clone_from(&mut self, source: &AVec<T, impl Free>) -> Result<(), A::Error> {
        self.try_reserve(source.len().saturating_sub(self.len()))?; // reserve before clear to preserve existing data on failure
        self.clear();
        self.try_extend_from_slice(source)
    }

    /// Allocate a new box that clones the contents of `self` using `A::default()`.
    ///
    /// ## Failure Modes
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    // TODO: examples
    pub fn try_clone(&self) -> Result<AVec<T, A>, A::Error> where A : Clone { self.try_clone_in(self.allocator().clone()) }

    /// Allocate a new box that clones the contents of `self` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    // TODO: examples
    pub fn try_clone_in<A2>(&self, allocator: A2) -> Result<AVec<T, A2>, A2::Error> where A2 : Realloc + ZstSupported {
        let mut v = AVec::try_with_capacity_in(self.len(), allocator)?;
        v.try_extend_from_slice(self)?;
        Ok(v)
    }

    /// Allocate a new box that clones the contents of `self` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    // TODO: examples
    #[cfg(global_oom_handling)] pub fn clone_in<A2>(&self, allocator: A2) -> AVec<T, A2> where A2 : Realloc + ZstSupported {
        let mut v = AVec::with_capacity_in(self.len(), allocator);
        v.extend_from_slice(self);
        v
    }
}
