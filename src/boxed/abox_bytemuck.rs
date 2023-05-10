#![cfg(feature = "bytemuck")]

use crate::boxed::ABox;
use crate::error::ExcessiveSliceRequestedError;
use crate::util;
use crate::zsty::*;

use bytemuck::*;

use core::alloc::Layout;



impl<T: Zeroable, A: Alloc + Free> ABox<T, A> {
    // Sized, Alloc

    /// Allocate a new box initialized to `0` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, _>::try_new_bytemuck_zeroed_in(Malloc).unwrap();
    /// let a = ABox::<(),  _>::try_new_bytemuck_zeroed_in(Malloc).unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - Null can't allocate anything
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::<u32, _>::try_new_bytemuck_zeroed_in(Null).unwrap_err();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - requires too much alignment for Malloc
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, _>::try_new_bytemuck_zeroed_in(Malloc).unwrap();
    /// ```
    #[track_caller] pub fn try_new_bytemuck_zeroed_in(allocator: A) -> Result<Self, A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        let layout = Layout::new::<T>();
        let data = allocator.alloc_zeroed(layout)?.cast();
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }

    /// Allocate a new box of `len` values initialized to `0` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when excessively large allocations are requested
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, _>::try_new_bytemuck_zeroed_slice_in(0, Malloc).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  _>::try_new_bytemuck_zeroed_slice_in(0, Malloc).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, _>::try_new_bytemuck_zeroed_slice_in(32, Malloc).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::try_new_bytemuck_zeroed_slice_in(32, Malloc).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::try_new_bytemuck_zeroed_slice_in(usize::MAX, Malloc).unwrap();
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```
    /// // will return Err(...) - too much memory requested
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let err = ABox::<u32, _>::try_new_bytemuck_zeroed_slice_in(usize::MAX, Malloc).err().unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let err = ABox::<u32, _>::try_new_bytemuck_zeroed_slice_in(usize::MAX/8+1, Malloc).err().unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// // won't compile - requires too much alignment for Malloc
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, _>::try_new_bytemuck_zeroed_slice_in(1, Malloc).unwrap();
    /// ```
    #[track_caller] pub fn try_new_bytemuck_zeroed_slice_in(len: usize, allocator: A) -> Result<ABox<[T], A>, A::Error> where ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        let layout = Layout::array::<T>(len).map_err(|_| ExcessiveSliceRequestedError{ requested: len }.into())?;
        let data = util::nn::slice_from_raw_parts(allocator.alloc_zeroed(layout)?.cast(), len);
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }

    /// Allocate a new box initialized to `0` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, _>::new_bytemuck_zeroed_in(Malloc);
    /// let a = ABox::<(),  _>::new_bytemuck_zeroed_in(Malloc);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - Null can't allocate anything
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, _>::new_bytemuck_zeroed_in(Null);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - requires too much alignment for Malloc
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, _>::new_bytemuck_zeroed_in(Malloc);
    /// ```
    #[cfg(feature = "panicy-memory")] #[track_caller] pub fn new_bytemuck_zeroed_in(allocator: A) -> Self {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        Self::try_new_bytemuck_zeroed_in(allocator).expect("unable to allocate")
    }

    /// Allocate a new box of `len` values initialized to `0` using `allocator`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when excessively large allocations are requested
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, _>::new_bytemuck_zeroed_slice_in(0, Malloc);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  _>::new_bytemuck_zeroed_slice_in(0, Malloc);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, _>::new_bytemuck_zeroed_slice_in(32, Malloc);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::new_bytemuck_zeroed_slice_in(32, Malloc);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  _>::new_bytemuck_zeroed_slice_in(usize::MAX, Malloc);
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - too much memory requested
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, _>::new_bytemuck_zeroed_slice_in(usize::MAX, Malloc);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, _>::new_bytemuck_zeroed_slice_in(usize::MAX/8+1, Malloc);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// // won't compile - requires too much alignment for Malloc
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, _>::new_bytemuck_zeroed_slice_in(1, Malloc);
    /// ```
    #[cfg(feature = "panicy-memory")] #[track_caller] pub fn new_bytemuck_zeroed_slice_in(len: usize, allocator: A) -> ABox<[T], A> where ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        Self::try_new_bytemuck_zeroed_slice_in(len, allocator).expect("unable to allocate")
    }
}

impl<T: Zeroable, A: Alloc + Free + Default> ABox<T, A> {
    // Sized, Alloc, Default

    /// Allocate a new box initialized to `0`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::try_new_bytemuck_zeroed().unwrap();
    /// let a = ABox::<(),  Malloc>::try_new_bytemuck_zeroed().unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - Null can't allocate anything
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let err = ABox::<u32, Null>::try_new_bytemuck_zeroed().unwrap_err();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - requires too much alignment for Malloc
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, Malloc>::try_new_bytemuck_zeroed().unwrap();
    /// ```
    #[track_caller] #[inline(always)] pub fn try_new_bytemuck_zeroed() -> Result<Self, A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        Self::try_new_bytemuck_zeroed_in(A::default())
    }

    /// Allocate a new box of `len` values initialized to `0`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   Returns <code>[Err]\(...\)</code> when excessively large allocations are requested
    /// *   Returns <code>[Err]\(...\)</code> when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::try_new_bytemuck_zeroed_slice(0).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  Malloc>::try_new_bytemuck_zeroed_slice(0).unwrap();
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, Malloc>::try_new_bytemuck_zeroed_slice(32).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  Malloc>::try_new_bytemuck_zeroed_slice(32).unwrap();
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  Malloc>::try_new_bytemuck_zeroed_slice(usize::MAX).unwrap();
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```
    /// // will return Err(...) - too much memory requested
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let err = ABox::<u32, Malloc>::try_new_bytemuck_zeroed_slice(usize::MAX).err().unwrap();
    /// ```
    ///
    /// ```
    /// // will return Err(...) - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let err = ABox::<u32, Malloc>::try_new_bytemuck_zeroed_slice(usize::MAX/8+1).err().unwrap();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// // won't compile - requires too much alignment for Malloc
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, Malloc>::try_new_bytemuck_zeroed_slice(1).unwrap();
    /// ```
    #[track_caller] #[inline(always)] pub fn try_new_bytemuck_zeroed_slice(len: usize) -> Result<ABox<[T], A>, A::Error> where ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        Self::try_new_bytemuck_zeroed_slice_in(len, A::default())
    }

    /// Allocate a new box initialized to `0`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::new_bytemuck_zeroed();
    /// let a = ABox::<(),  Malloc>::new_bytemuck_zeroed();
    /// ```
    ///
    /// ```should_panic
    /// // will panic - Null can't allocate anything
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// let a = ABox::<u32, Null>::new_bytemuck_zeroed();
    /// ```
    ///
    /// ```compile_fail,E0080
    /// // won't compile - requires too much alignment for Malloc
    /// # use ialloc::{allocator::{c::Malloc, debug::Null}, boxed::ABox};
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, Malloc>::new_bytemuck_zeroed();
    /// ```
    #[cfg(feature = "panicy-memory")] #[track_caller] #[inline(always)] pub fn new_bytemuck_zeroed() -> Self {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        Self::new_bytemuck_zeroed_in(A::default())
    }

    /// Allocate a new box of `len` values initialized to `0`.
    ///
    /// ## Failure Modes
    /// *   Fails to compile on impossible alignments (e.g. attempting to allocate 4 KiB alignment pages via 8/16 byte aligned malloc)
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when excessively large allocations are requested
    /// *   [`panic!`]s or [`handle_alloc_error`](alloc::alloc::handle_alloc_error)s when out of memory
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::new_bytemuck_zeroed_slice(0);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<(),  Malloc>::new_bytemuck_zeroed_slice(0);
    /// # assert_eq!(a.len(), 0);
    /// let a = ABox::<u32, Malloc>::new_bytemuck_zeroed_slice(32);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  Malloc>::new_bytemuck_zeroed_slice(32);
    /// # assert_eq!(a.len(), 32);
    /// let a = ABox::<(),  Malloc>::new_bytemuck_zeroed_slice(usize::MAX);
    /// # assert_eq!(a.len(), usize::MAX);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - too much memory requested
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::new_bytemuck_zeroed_slice(usize::MAX);
    /// ```
    ///
    /// ```should_panic
    /// // will panic - still too much memory (half the address space → `Layout` overflows)
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// let a = ABox::<u32, Malloc>::new_bytemuck_zeroed_slice(usize::MAX/8+1);
    /// ```
    ///
    /// ```compile_fail,E0080
    /// # use ialloc::{allocator::alloc::Global, boxed::ABox};
    /// // won't compile - requires too much alignment for Malloc
    /// #[repr(C, align(4096))] pub struct Page([u8; 4096]);
    /// unsafe impl bytemuck::Zeroable for Page {}
    /// let a = ABox::<Page, Malloc>::new_bytemuck_zeroed_slice(1);
    /// ```
    #[cfg(feature = "panicy-memory")] #[track_caller] #[inline(always)] pub fn new_bytemuck_zeroed_slice(len: usize) -> ABox<[T], A> where ExcessiveSliceRequestedError : Into<A::Error> {
        let _ = Self::ASSERT_A_CAN_ALLOC_ALIGNED_T;
        Self::new_bytemuck_zeroed_slice_in(len, A::default())
    }
}
