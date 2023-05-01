//! C/C++y allocator traits operating on thin pointers, implying alignment, etc.
//!
//! C and C++ allocators often merely accept a pointer for free/realloc/size queries.
//! This module provides traits for such functionality.

use crate::*;

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
#[cfg(doc)] use core::ptr::NonNull;



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
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error>;

    /// Allocate at least `size` bytes of zeroed memory.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<AllocNN0, Self::Error> {
        let alloc = self.alloc_uninit(size)?;
        unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), size.get()) }.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}



/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]<[MaybeUninit]<[u8]>>)</code><br>
/// <br>
pub unsafe trait Free {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    unsafe fn free(&self, ptr: AllocNN);
}



/// Deallocation function (implies [`Free`]):<br>
/// <code>[free](Self::free)(ptr: *mut <[MaybeUninit]<[u8]>>)</code><br>
/// <br>
pub unsafe trait FreeNullable {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` may be null, in which case this is a noop
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    unsafe fn free(&self, ptr: *mut MaybeUninit<u8>);
}



/// Allocation size query (reliable for `self`-owned allocations):<br>
/// <code>[size_of](Self::size_of)(ptr: [NonNull]<[MaybeUninit]<[u8]>>) -> [Option]<[usize]></code><br>
/// <br>
///
/// ### Safety
/// It wouldn't be entirely unreasonable for an implementor to implement realloc in terms of this trait.
/// Such an implementor would generally rely on the `ptr[..a.size_of(ptr)]` being valid memory when `ptr` is a valid allocation owned by `a`.
/// By implementing this trait, you pinky promise that such a size is valid.
pub unsafe trait SizeOf {
    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize>;
}



/// Allocation size query (unreliable / for debug purpouses only):<br>
/// <code>[size_of](Self::size_of)(ptr: [NonNull]<[MaybeUninit]<[u8]>>) -> [Option]<[usize]></code><br>
/// <br>
///
/// This trait may fail (returning [`None`]) even if `ptr` is a thin allocation belonging to `self`.
/// This is intended for size queries where the system may or may not be able to query the underlying system allocator for sizes.
///
/// If all the following hold:
/// * `ptr` is a valid allocation belonging to `self`
/// * <code>[Some]\(size\)</code> was returned
/// * no `&mut T` references alias `ptr[..size]` (hard to verify!)
///
/// It should be valid to construct:
/// ```
/// # use ialloc::thin::*;
/// # use core::mem::*;
/// # use core::ptr::*;
/// # fn wrap(alloc: &impl SizeOfDebug, ptr: NonNull<MaybeUninit<u8>>) {
/// let Some(size) = (unsafe{ alloc.size_of(ptr) }) else { return };
/// let slice : &[MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), size) };
/// # }
/// ```
pub unsafe trait SizeOfDebug {
    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize>;
}
