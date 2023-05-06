use crate::boxed::ABox;
use crate::zsty::*;



// Allocating traits, falliable counterparts

#[cfg(feature = "panicy-memory")] impl<T: Clone, A: Alloc + Free + Clone> Clone for ABox<T, A> {
    fn clone(&self) -> Self                 { Self::new_in(T::clone(self), self.allocator().clone()) }
    fn clone_from(&mut self, source: &Self) { T::clone_from(self, source) }
}

#[cfg(feature = "panicy-memory")] impl<T: Default, A: Alloc + Free + Default> Default for ABox<T, A> {
    fn default() -> Self { Self::new(T::default()) }
}

/// Non-panicing alternatives to [`Clone`] / support for alternative allocators.
impl<T: Clone, A: Free> ABox<T, A> {
    pub fn clone_from(&mut self, source: &ABox<T, impl Free>) { T::clone_from(self, source) }
    pub fn try_clone       (&self               ) -> Result<ABox<T, A >, A ::Error> where A  : Alloc + Clone { ABox::try_new_in(T::clone(self), self.allocator().clone()) }
    pub fn try_clone_in<A2>(&self, allocator: A2) -> Result<ABox<T, A2>, A2::Error> where A2 : Alloc + Free  { ABox::try_new_in(T::clone(self), allocator) }

    #[cfg(feature = "panicy-memory")] pub fn clone_in<A2>(&self, allocator: A2) -> ABox<T, A2> where A2 : Alloc + Free { ABox::new_in(T::clone(self), allocator) }
}

/// Non-panicing alternatives to [`Default`] / support for alternative allocators.
impl<T: Default, A: Free> ABox<T, A> {
    pub fn try_default       ()              -> Result<ABox<T, A >, A ::Error> where T : Default, A : Alloc + Default { ABox::try_new_in(T::default(), A::default()) }
    pub fn try_default_in<A2>(allocator: A2) -> Result<ABox<T, A2>, A2::Error> where T : Default, A2: Alloc + Free    { ABox::try_new_in(T::default(), allocator) }

    #[cfg(feature = "panicy-memory")] pub fn default_in<A2>(allocator: A2) -> ABox<T, A2> where T : Default, A2 : Alloc + Free { ABox::new_in(T::default(), allocator) }
}



// TODO:
//  • [ ] impl Clone   for       slice types
//  • [ ] impl Default for empty slice types
