use crate::boxed::ABox;
use crate::fat::{Alloc, Free};



#[cfg(global_oom_handling)] impl<T: Clone, A: Alloc + Free + Clone> Clone for ABox<T, A> {
    /// Allocate a new box that clones the contents of `self` using `self.allocator().clone()`
    ///
    /// ## Failure Modes
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    fn clone(&self) -> Self {
        //let _ = Self::ASSERT_A_CAN_ALLOC_T; // implied by `self`
        Self::new_in(T::clone(self), Self::allocator(self).clone())
    }

    /// Clone the contents of `source` into `self` without reallocating the [`ABox`].
    fn clone_from(&mut self, source: &Self) {
        //let _ = Self::ASSERT_A_CAN_ALLOC_T; // implied by `self`
        T::clone_from(self, source)
    }
}

// TODO:
//  • [ ] impl Clone for slice boxes
//  • [ ] impl Clone for str boxes

/// Non-panicing alternatives to [`Clone`] / support for alternative allocators.
impl<T: Clone, A: Free> ABox<T, A> {
    /// Clone the contents of `source` into `self` without reallocating the [`ABox`].
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let mut a = ABox::new_in('a', Malloc);
    /// let     b = ABox::new_in('b', Malloc);
    /// a.clone_from(&b);
    /// assert_eq!(*a, 'b');
    /// ```
    pub fn clone_from(&mut self, source: &ABox<T, impl Free>) {
        //let _ = Self::ASSERT_A_CAN_ALLOC_T; // implied by `self`
        T::clone_from(self, source)
    }

    /// Allocate a new box that clones the contents of `self` using `A::default()`.
    ///
    /// ## Failure Modes
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let a = ABox::new_in('a', Malloc);
    /// let b = a.try_clone().unwrap();
    /// assert_eq!(*b, 'a');
    /// ```
    // TODO: show failure example via allocator with strict memory limits
    pub fn try_clone(&self) -> Result<ABox<T, A>, A ::Error> where A : Alloc + Clone {
        //let _ = Self::ASSERT_A_CAN_ALLOC_T; // implied by `self`
        ABox::try_new_in(T::clone(self), Self::allocator(self).clone())
    }

    /// Allocate a new box that clones the contents of `self` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{alloc::Global, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::new_in('a', Malloc);
    /// let b = a.try_clone_in(Malloc).unwrap();
    /// assert_eq!(*b, 'a');
    /// ```
    ///
    /// ```
    /// // will return Err(...) - Null can't allocate anything
    /// # use ialloc::{allocator::{alloc::Global, c::Malloc, debug::Null}, boxed::ABox};
    /// # let a = ABox::new_in('a', Malloc);
    /// let err = a.try_clone_in(Null).unwrap_err();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::{alloc::Global, c::Malloc, debug::Null}, boxed::ABox};
    /// #[derive(Clone)] #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::new_in(Page([0u8; 4096]), Global);
    ///
    /// // won't compile - requires too much alignment for Malloc
    /// let b = a.try_clone_in(Malloc);
    /// ```
    pub fn try_clone_in<A2>(&self, allocator: A2) -> Result<ABox<T, A2>, A2::Error> where A2 : Alloc + Free {
        let _ = ABox::<T, A2>::ASSERT_A_CAN_ALLOC_T;
        ABox::try_new_in(T::clone(self), allocator)
    }

    /// Allocate a new box that clones the contents of `self` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{alloc::Global, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::new_in('a', Malloc);
    /// let b = a.clone_in(Malloc);
    /// assert_eq!(*b, 'a');
    /// ```
    ///
    /// ```should_panic
    /// // will panic - Null can't allocate anything
    /// # use ialloc::{allocator::{alloc::Global, c::Malloc, debug::Null}, boxed::ABox};
    /// # let a = ABox::new_in('a', Malloc);
    /// let b = a.clone_in(Null);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::{alloc::Global, c::Malloc, debug::Null}, boxed::ABox};
    /// #[derive(Clone)] #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::new_in(Page([0u8; 4096]), Global);
    ///
    /// // won't compile - requires too much alignment for Malloc
    /// let b = a.clone_in(Malloc);
    /// ```
    #[cfg(global_oom_handling)] pub fn clone_in<A2>(&self, allocator: A2) -> ABox<T, A2> where A2 : Alloc + Free {
        let _ = ABox::<T, A2>::ASSERT_A_CAN_ALLOC_T;
        ABox::new_in(T::clone(self), allocator)
    }
}
