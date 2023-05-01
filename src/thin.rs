//! C/C++y allocator traits operating on thin pointers, implying alignment, etc.
//!
//! C and C++ allocators often merely accept a pointer for dealloc/realloc/size queries.
//! This module provides traits for such functionality.

use crate::*;

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
#[cfg(doc)] use core::ptr::NonNull;



/// <code>[alloc_size](Self::alloc_size)(ptr: [NonNull]<[MaybeUninit]<[u8]>>) -> [Result]<[usize]></code>
///
/// ### Safety
/// It wouldn't be entirely unreasonable for an implementor to implement realloc in terms of this trait.
/// Such an implementor would generally rely on the `ptr[..a.alloc_size(ptr)]` being valid memory when `ptr` is a valid allocation owned by `a`.
/// By implementing this trait, you pinky promise that such a size is valid.
pub unsafe trait AllocSize {
    type Error : core::fmt::Debug;

    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn alloc_size(&self, ptr: AllocNN) -> Result<usize, Self::Error>;
}

// TODO: SafeAllocSize - like alloc_size, but a safe fn?



const ALIGN_USIZE : Alignment = Alignment::of::<usize>();

/// Allocation functions with alignment (up to <code>[Alloc]::[MAX_ALIGN](Self::MAX_ALIGN)</code>) implied by size:
/// <code>
/// fn [alloc_uninit](Self::alloc_uninit)(size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;
/// fn [alloc_zeroed](Self::alloc_zeroed)(size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;
/// </code><br>
///
/// ## Alignment Guarantees
///
/// | Requested Size                                                                        | Guaranteed Alignment (if successful)  |
/// | --------------------------------------------------------------------------------------| --------------------------------------|
/// | 0                                                                                     | Doesn't compile thanks to [`NonZeroUsize`]
/// | <code>1 .. [Alloc]::[MIN_ALIGN](Self::MIN_ALIGN)</code>                               | <code>[Alloc]::[MIN_ALIGN](Self::MIN_ALIGN)</code>
/// | <code>[Alloc]::[MIN_ALIGN](Self::MIN_ALIGN) .. [MAX_ALIGN](Self::MAX_ALIGN)</code>    | The largest power of two that fits within `size`
/// | <code>[Alloc]::[MAX_ALIGN](Self::MAX_ALIGN) ..</code>                                 | <code>[Alloc]::[MAX_ALIGN](Self::MAX_ALIGN)</code>
pub unsafe trait Alloc {
    type Error;

    /// The minimum alignment guaranteed by this allocator.
    ///
    /// This defaults to a conservative `1`.  Real allocators often provide more, but you don't particularly *need* more.
    const MIN_ALIGN : Alignment = ALIGN_1;

    /// The maximum alignment guaranteed by this allocator.
    ///
    /// This defaults to a conservative <code>[Alignment]::of::&lt;[usize]&gt;()</code>.<br>
    /// Real allocators often guarantee <code>[Alignment]::of::&lt;libc::max_align_t&gt;()</code>.<br>
    /// This can be even larger than <code>[Alignment]::of::&lt;u128&gt;()</code> on something as common as AMD64!<br>
    ///
    /// Common values on AMD64:
    ///
    /// | Type                  | Align         | Size      | Notes    |
    /// | ----------------------| --------------| ----------| ---------|
    /// | [`()`](unit)          | 1             | **0**     | align &gt; size
    /// | [`u8`]                | 1             | 1         |
    /// | [`u16`]               | 2             | 2         |
    /// | [`u32`]               | 4             | 4         |
    /// | [`u64`]               | 8             | 8         |
    /// | [`usize`]             | 8             | 8         |
    /// | [`u128`]              | **8 or 16**   | 16        | align &lt; size, sometimes
    /// | `max_align_t`         | 16            | **32**    | align &lt; size
    /// | `[max_align_t; 0]`    | 16            | 0         | align &gt; size
    const MAX_ALIGN : Alignment = ALIGN_USIZE;

    /// Allocate at least `size` bytes of uninitialized memory.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[dealloc](Free::dealloc)</code>
    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error>;

    /// Allocate at least `size` bytes of zeroed memory.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[dealloc](Free::dealloc)</code>
    fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<AllocNN0, Self::Error> {
        let alloc = self.alloc_uninit(size)?;
        unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), size.get()) }.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}



/// <code>[dealloc](Self::dealloc)(ptr: [NonNull]<[MaybeUninit]<[u8]>>)</code>
pub trait Free {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after dealloc
    unsafe fn dealloc(&self, ptr: AllocNN);
}

impl<A: thin::Free> nzst::Free for A {
    unsafe fn dealloc(&self, ptr: AllocNN, _layout: LayoutNZ) { unsafe { thin::Free::dealloc(self, ptr) } }
}



/// <code>[dealloc](Self::dealloc)(ptr: *const <[MaybeUninit]<[u8]>>)</code>
pub trait FreeNullable {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` may be null, in which case this is a noop.
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after dealloc
    unsafe fn dealloc(&self, ptr: *const MaybeUninit<u8>);
}

impl<A: FreeNullable> thin::Free for A {
    unsafe fn dealloc(&self, ptr: AllocNN) { unsafe { FreeNullable::dealloc(self, ptr.as_ptr()) } }
}
