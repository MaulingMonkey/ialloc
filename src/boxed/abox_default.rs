use crate::boxed::ABox;
use crate::fat::{Alloc, Free};



// Allocating traits, falliable counterparts

#[cfg(global_oom_handling)] impl<T: Default, A: Alloc + Free + Default> Default for ABox<T, A> {
    /// Allocate a new box containing `T::default()` using `A::default()`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    fn default() -> Self {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Self::new(T::default())
    }
}

// TODO:
//  • [ ] impl Default for slice boxes
//  • [ ] impl Default for str boxes

/// Non-panicing alternatives to [`Default`] / support for alternative allocators.
impl<T: Default, A: Free> ABox<T, A> {
    /// Allocate a new box containing `T::default()` using `A::default()`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::<u32, Null>::try_default().unwrap_err();
    /// let b = ABox::<u32, Malloc>::try_default().unwrap();
    /// assert_eq!(*b, 0);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// // won't compile - requires too much alignment for Malloc
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// impl Default for Page { fn default() -> Self { Self([0u8; 4096]) } }
    /// let b = ABox::<Page, Malloc>::try_default().unwrap();
    /// ```
    pub fn try_default() -> Result<ABox<T, A>, A::Error> where T : Default, A : Alloc + Default {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        ABox::try_new_in(T::default(), A::default())
    }

    /// Allocate a new box containing `T::default()` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::<u32, _>::try_default_in(Null).unwrap_err();
    /// let b = ABox::<u32, _>::try_default_in(Malloc).unwrap();
    /// assert_eq!(*b, 0);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// // won't compile - requires too much alignment for Malloc
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// impl Default for Page { fn default() -> Self { Self([0u8; 4096]) } }
    /// let b = ABox::<Page, _>::try_default_in(Malloc).unwrap();
    /// ```
    pub fn try_default_in(allocator: A) -> Result<ABox<T, A>, A::Error> where T : Default, A: Alloc {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        ABox::try_new_in(T::default(), allocator)
    }

    /// Allocate a new box containing `T::default()` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let b = ABox::<u32, _>::default_in(Malloc);
    /// assert_eq!(*b, 0);
    /// ```
    ///
    /// ```should_panic
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// // will panic - Null can't allocate anything
    /// let b = ABox::<u32, _>::default_in(Null);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// // won't compile - requires too much alignment for Malloc
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// impl Default for Page { fn default() -> Self { Self([0u8; 4096]) } }
    /// let b = ABox::<Page, _>::default_in(Malloc);
    /// ```
    #[cfg(global_oom_handling)] pub fn default_in(allocator: A) -> ABox<T, A> where T : Default, A : Alloc {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        ABox::new_in(T::default(), allocator)
    }
}


