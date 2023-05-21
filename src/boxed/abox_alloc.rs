use crate::boxed::ABox;
use crate::error::ExcessiveSliceRequestedError;
use crate::fat::*;
use crate::meta::*;
use crate::util;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::mem::{align_of, size_of};



impl<T, A: Alloc + Free> ABox<T, A> {
    // Sized, Alloc

    /// If you hit this assertion, it's unlikely that `A` can ever successfully allocate an instance of `T` except by happenstance and accident.
    /// Unless you've written some obscenely generic code that intentionally handles containers that might never be able to allocate, this is likely a bug.
    pub(super) const ASSERT_A_CAN_ALLOC_T : () = {
        assert!(align_of::<T>() <= A::MAX_ALIGN.as_usize(), "Alignment::of::<T>() > A::MAX_ALIGN - the allocator cannot allocate memory sufficiently aligned for instances of T on it's own");
        assert!(size_of::<T>() > 0 || A::ZST_SUPPORTED, "T is a ZST but A does not support allocating ZSTs - consider wrapping the allocator in an `adapt` allocator such as `AllocZst` or `DangleZst`");
    };

    /// If you hit this assertion, `A` cannot generically allocate a slice possibly 0 instances of `T`.
    /// Unless you've written some obscenely generic code that intentionally handles containers that might never be able to allocate, this is likely a bug.
    pub(super) const ASSERT_A_CAN_ALLOC_T_SLICE : () = {
        assert!(align_of::<T>() <= A::MAX_ALIGN.as_usize(), "Alignment::of::<T>() > A::MAX_ALIGN - the allocator cannot allocate memory sufficiently aligned for instances of T on it's own");
        assert!(A::ZST_SUPPORTED, "[T] could be a ZST but A does not support allocating ZSTs - consider wrapping the allocator in an `adapt` allocator such as `AllocZst` or `DangleZst`");
    };

    /// Allocate a new box initialized to `value` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::try_new_in(42_u32, Malloc).unwrap();
    /// let a = ABox::try_new_in((), DangleZst(Malloc)).unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::try_new_in(42_u32, Null).unwrap_err();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::try_new_in((), Malloc).unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::try_new_in(Page([0u8; 4096]), Malloc).unwrap();
    /// ```
    pub fn try_new_in(value: T, allocator: A) -> Result<Self, A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Ok(ABox::write(Self::try_new_uninit_in(allocator)?, value))
    }

    /// Allocate a new uninitialized box using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, _>::try_new_uninit_in(Malloc).unwrap();
    /// let a = ABox::<(), _>::try_new_uninit_in(DangleZst(Malloc)).unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::<u32, _>::try_new_uninit_in(Null).unwrap_err();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<(), _>::try_new_uninit_in(Malloc).unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, _>::try_new_uninit_in(Malloc).unwrap();
    /// ```
    pub fn try_new_uninit_in(allocator: A) -> Result<ABox<MaybeUninit<T>, A>, A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        let layout = Layout::new::<T>();
        let data = allocator.alloc_uninit(layout)?.cast();
        // SAFETY: ✔️ we just allocated `data` with `allocator`
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }

    /// Allocate a new uninitialized box of `len` values using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile if the allocator doesn't support ZSTs
    /// *   Returns <code>[Err]\(...\)</code> when excessively large allocations are requested
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let alloc = DangleZst(Malloc);
    /// let a = ABox::<u32, _>::try_new_uninit_slice_in(0, alloc).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  _>::try_new_uninit_slice_in(0, alloc).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, _>::try_new_uninit_slice_in(32, alloc).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::try_new_uninit_slice_in(32, alloc).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::try_new_uninit_slice_in(usize::MAX, alloc).unwrap();
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```
    /// // will return Err(...) - too much memory requested
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # let alloc = DangleZst(Malloc);
    /// let err = ABox::<u32, _>::try_new_uninit_slice_in(usize::MAX, alloc).err().unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # let alloc = DangleZst(Malloc);
    /// let err = ABox::<u32, _>::try_new_uninit_slice_in(usize::MAX/8+1, alloc).err().unwrap();
    /// ```
    ///
    /// ```compile_fail,E0277
    /// // won't compile - Malloc doesn't support ZSTs like empty slices
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, _>::try_new_uninit_slice_in(0, Malloc).unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # let alloc = DangleZst(Malloc);
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, _>::try_new_uninit_slice_in(1, alloc).unwrap();
    /// ```
    pub fn try_new_uninit_slice_in(len: usize, allocator: A) -> Result<ABox<[MaybeUninit<T>], A>, A::Error> where A : ZstSupported, ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T_SLICE;
        let layout = Layout::array::<T>(len).map_err(|_| ExcessiveSliceRequestedError{ requested: len }.into())?;
        let data = util::nn::slice_from_raw_parts(allocator.alloc_uninit(layout)?.cast(), len);
        // SAFETY: ✔️ we just allocated `data` with `allocator`
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }
}

impl<T, A: Alloc + Free + Default> ABox<T, A> {
    // Sized, Alloc, Default

    /// Allocate a new box initialized to `value`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<_, Malloc>::try_new(42_u32).unwrap();
    /// let a = ABox::<_, DangleZst<Malloc>>::try_new(()).unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::<u32, Null>::try_new(42_u32).unwrap_err();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<_, Malloc>::try_new(()).unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a : ABox<Page, Malloc> = ABox::try_new(Page([0u8; 4096])).unwrap();
    /// ```
    pub fn try_new(value: T) -> Result<Self, A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Self::try_new_in(value, A::default())
    }

    /// Allocate a new uninitialized box.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::try_new_uninit().unwrap();
    /// let a = ABox::<(), DangleZst<Malloc>>::try_new_uninit().unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::<u32, Null>::try_new_uninit().unwrap_err();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<(), Malloc>::try_new_uninit().unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, Malloc>::try_new_uninit().unwrap();
    /// ```
    pub fn try_new_uninit() -> Result<ABox<MaybeUninit<T>, A>, A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Self::try_new_uninit_in(A::default())
    }

    /// Allocate a new uninitialized box of `len` values.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile if the allocator doesn't support ZSTs
    /// *   Returns <code>[Err]\(...\)</code> when excessively large allocations are requested
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// type A = DangleZst<Malloc>;
    /// let a = ABox::<u32, A>::try_new_uninit_slice(0).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  A>::try_new_uninit_slice(0).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, A>::try_new_uninit_slice(32).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  A>::try_new_uninit_slice(32).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  A>::try_new_uninit_slice(usize::MAX).unwrap();
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```
    /// // will return Err(...) - too much memory requested
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # type A = DangleZst<Malloc>;
    /// let err = ABox::<u32, A>::try_new_uninit_slice(usize::MAX).err().unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # type A = DangleZst<Malloc>;
    /// let err = ABox::<u32, A>::try_new_uninit_slice(usize::MAX/8+1).err().unwrap();
    /// ```
    ///
    /// ```compile_fail,E0277
    /// // won't compile - Malloc doesn't support ZSTs like empty slices
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::try_new_uninit_slice(0).unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # type A = DangleZst<Malloc>;
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, A>::try_new_uninit_slice(1).unwrap();
    /// ```
    pub fn try_new_uninit_slice(len: usize) -> Result<ABox<[MaybeUninit<T>], A>, A::Error> where A : ZstSupported, ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T_SLICE;
        Self::try_new_uninit_slice_in(len, A::default())
    }
}

#[cfg(global_oom_handling)] impl<T, A: Alloc + Free> ABox<T, A> {
    // Sized, Alloc

    /// Allocate a new box initialized to `value` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::new_in(42_u32, Malloc);
    /// let a = ABox::new_in((), DangleZst(Malloc));
    /// ```
    ///
    /// ```should_panic
    /// // will panic - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::new_in(42_u32, Null);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::new_in((), Malloc);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::new_in(Page([0u8; 4096]), Malloc);
    /// ```
    #[track_caller] #[inline(always)] pub fn new_in(value: T, allocator: A) -> Self {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Self::try_new_in(value, allocator).expect("unable to allocate")
    }

    /// Allocate a new uninitialized box using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, _>::new_uninit_in(Malloc);
    /// let a = ABox::<(), _>::new_uninit_in(DangleZst(Malloc));
    /// ```
    ///
    /// ```should_panic
    /// // will panic - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, _>::new_uninit_in(Null);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<(), _>::new_uninit_in(Malloc);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, _>::new_uninit_in(Malloc);
    /// ```
    #[track_caller] #[inline(always)] pub fn new_uninit_in(allocator: A) -> ABox<MaybeUninit<T>, A> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Self::try_new_uninit_in(allocator).expect("unable to allocate")
    }

    /// Allocate a new uninitialized box of `len` values using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile if the allocator doesn't support ZSTs
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when excessively large allocations are requested
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let alloc = DangleZst(Malloc);
    /// let a = ABox::<u32, _>::new_uninit_slice_in(0, alloc);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  _>::new_uninit_slice_in(0, alloc);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, _>::new_uninit_slice_in(32, alloc);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::new_uninit_slice_in(32, alloc);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::new_uninit_slice_in(usize::MAX, alloc);
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - too much memory requested
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # let alloc = DangleZst(Malloc);
    /// let a = ABox::<u32, _>::new_uninit_slice_in(usize::MAX, alloc);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # let alloc = DangleZst(Malloc);
    /// let a = ABox::<u32, _>::new_uninit_slice_in(usize::MAX/8+1, alloc);
    /// ```
    ///
    /// ```compile_fail,E0277
    /// // won't compile - Malloc doesn't support ZSTs like empty slices
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, _>::new_uninit_slice_in(0, Malloc);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # let alloc = DangleZst(Malloc);
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, _>::new_uninit_slice_in(1, alloc);
    /// ```
    #[track_caller] #[inline(always)] pub fn new_uninit_slice_in(len: usize, allocator: A) -> ABox<[MaybeUninit<T>], A> where A : ZstSupported, ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T_SLICE;
        Self::try_new_uninit_slice_in(len, allocator).expect("unable to allocate")
    }
}

#[cfg(global_oom_handling)] impl<T, A: Alloc + Free + Default> ABox<T, A> {
    // Sized, Alloc, Default

    /// Allocate a new box initialized to `value`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<_, Malloc>::new(42_u32);
    /// let a = ABox::<_, DangleZst<Malloc>>::new(());
    /// ```
    ///
    /// ```should_panic
    /// // will panic - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Null>::new(42_u32);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<_, Malloc>::new(());
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a : ABox<Page, Malloc> = ABox::new(Page([0u8; 4096]));
    /// ```
    #[track_caller] #[inline(always)] pub fn new(value: T) -> Self {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Self::new_in(value, A::default())
    }

    /// Allocate a new uninitialized box.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile for ZSTs if unsupported by the allocator
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::new_uninit();
    /// let a = ABox::<(), DangleZst<Malloc>>::new_uninit();
    /// ```
    ///
    /// ```should_panic
    /// // will panic - Null can't allocate anything
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Null>::new_uninit();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support ZSTs
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<(), Malloc>::new_uninit();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, Malloc>::new_uninit();
    /// ```
    #[track_caller] #[inline(always)] pub fn new_uninit() -> ABox<MaybeUninit<T>, A> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T;
        Self::new_uninit_in(A::default())
    }

    /// Allocate a new uninitialized box of `len` values.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Fails to compile if the allocator doesn't support ZSTs
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when excessively large allocations are requested
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// type A = DangleZst<Malloc>;
    /// let a = ABox::<u32, A>::new_uninit_slice(0);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  A>::new_uninit_slice(0);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, A>::new_uninit_slice(32);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  A>::new_uninit_slice(32);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  A>::new_uninit_slice(usize::MAX);
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - too much memory requested
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # type A = DangleZst<Malloc>;
    /// let a = ABox::<u32, A>::new_uninit_slice(usize::MAX);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # type A = DangleZst<Malloc>;
    /// let a = ABox::<u32, A>::new_uninit_slice(usize::MAX/8+1);
    /// ```
    ///
    /// ```compile_fail,E0277
    /// // won't compile - Malloc doesn't support ZSTs like empty slices
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::new_uninit_slice(0);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - Malloc doesn't support Page alignment
    /// # use ialloc::{allocator::{adapt::DangleZst, c::Malloc, debug::Null}, boxed::ABox};
    /// # type A = DangleZst<Malloc>;
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// let a = ABox::<Page, A>::new_uninit_slice(1);
    /// ```
    #[track_caller] #[inline(always)] pub fn new_uninit_slice(len: usize) -> ABox<[MaybeUninit<T>], A> where A : ZstSupported, ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_T_SLICE;
        Self::new_uninit_slice_in(len, A::default())
    }
}
