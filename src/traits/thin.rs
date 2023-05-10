//! C/C++y allocator traits operating on thin pointers, implying alignment, etc.
//!
//! C and C++ allocators often merely accept a pointer for free/realloc/size queries.
//! This module provides traits for such functionality.

use crate::*;
use crate::meta::Meta;

use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// Allocation functions with alignment (up to <code>[Meta]::[MAX_ALIGN](Meta::MAX_ALIGN)</code>) implied by size:
/// <code>
/// fn [alloc_uninit](Self::alloc_uninit)(size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;
/// fn [alloc_zeroed](Self::alloc_zeroed)(size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;
/// </code><br>
///
/// ## Alignment Guarantees
///
/// | Requested Size                                                                        | Guaranteed Alignment (if successful)  |
/// | --------------------------------------------------------------------------------------| --------------------------------------|
/// | <code>0 .. [Meta]::[MIN_ALIGN](Meta::MIN_ALIGN)</code>                                | <code>[Meta]::[MIN_ALIGN](Meta::MIN_ALIGN)</code>
/// | <code>[Meta]::[MIN_ALIGN](Meta::MIN_ALIGN) .. [MAX_ALIGN](Meta::MAX_ALIGN)</code>     | The largest power of two that fits within `size`
/// | <code>[Meta]::[MAX_ALIGN](Meta::MAX_ALIGN) ..</code>                                  | <code>[Meta]::[MAX_ALIGN](Meta::MAX_ALIGN)</code>
pub unsafe trait Alloc : Meta {
    /// Allocate at least `size` bytes of uninitialized memory.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error>;

    /// Allocate at least `size` bytes of zeroed memory.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        let alloc = self.alloc_uninit(size)?;
        unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), size) }.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}



/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]<[MaybeUninit]<[u8]>>)</code><br>
/// <br>
pub unsafe trait Free : meta::Meta {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>) { unsafe { self.free_nullable(ptr.as_ptr()) } }

    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` may be null, in which case this is a noop
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) { if let Some(ptr) = NonNull::new(ptr) { unsafe { self.free(ptr) } } }
}



/// Reallocation function:<br>
/// <code>[realloc_uninit](Self::realloc_uninit)(ptr: [NonNull]<[MaybeUninit]<[u8]>>, new_size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[realloc_zeroed](Self::realloc_zeroed)(ptr: [NonNull]<[MaybeUninit]<[u8]>>, new_size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
pub unsafe trait Realloc : Alloc + Free {
    /// Indicate if [`realloc_zeroed`](Self::realloc_zeroed) is supported / likely to work.
    const CAN_REALLOC_ZEROED : bool;

    /// Reallocate an existing allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after a succesful realloc (`realloc_uninit` returns <code>[Ok]\(...\)</code>)
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error>;

    /// Reallocate an existing allocation, `ptr`, belonging to `self`.
    ///
    /// Any newly allocated memory will be zeroed.
    /// This might be *no* new memory, even if you pass a larger `new_size` than you did previously - the implementation could've rounded both the old and new sizes up to the same value.
    /// As such, it's almost certainly a bug to use this to (re)allocate memory that came from <code>{,[re](Self::realloc_uninit)}[alloc_uninit](Alloc::alloc_uninit)</code>.
    /// Reallocate memory that came from <code>{,[re](Self::realloc_zeroed)}[alloc_zeroed](Alloc::alloc_zeroed)</code> instead.
    ///
    /// Additionally, not all implementations of [`thin::Realloc`] actually support this function.
    /// Check [`Self::CAN_REALLOC_ZEROED`].
    /// While it should be safe to call this function even if [`CAN_REALLOC_ZEROED`](Self::CAN_REALLOC_ZEROED) is false, [`realloc_zeroed`](Self::realloc_zeroed) will likely return <code>[Err]\(...\)</code>.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after a succesful realloc (`realloc_uninit` returns <code>[Ok]\(...\)</code>)
    unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error>;
}



/// Allocation size query (reliable for `self`-owned allocations):<br>
/// <code>[size_of](Self::size_of)(ptr: [NonNull]<[MaybeUninit]<[u8]>>) -> [Option]<[usize]></code><br>
/// <br>
///
/// ### Safety
/// It wouldn't be entirely unreasonable for an implementor to implement realloc in terms of this trait.
/// Such an implementor would generally rely on the `ptr[..a.size_of(ptr)]` being valid memory when `ptr` is a valid allocation owned by `a`.
/// By implementing this trait, you pinky promise that such a size is valid.
pub unsafe trait SizeOf : SizeOfDebug {
    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> { unsafe { SizeOfDebug::size_of(self, ptr) } }
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
pub unsafe trait SizeOfDebug : meta::Meta {
    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize>;
}



unsafe impl<'a, A: Alloc> Alloc for &'a A {
    fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { A::alloc_uninit(self, size) }
    fn alloc_zeroed(&self, size: usize) -> Result<NonNull<            u8 >, Self::Error> { A::alloc_zeroed(self, size) }
}

unsafe impl<'a, A: Free> Free for &'a A {
    unsafe fn free(         &self, ptr: NonNull<MaybeUninit<u8>> ) { unsafe { A::free(         self, ptr) } }
    unsafe fn free_nullable(&self, ptr: *mut    MaybeUninit<u8>  ) { unsafe { A::free_nullable(self, ptr) } }
}

unsafe impl<'a, A: Realloc> Realloc for &'a A {
    const CAN_REALLOC_ZEROED : bool = A::CAN_REALLOC_ZEROED;
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { A::realloc_uninit(self, ptr, new_size) } }
    unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { A::realloc_zeroed(self, ptr, new_size) } }
}

unsafe impl<'a, A: SizeOf> SizeOf for &'a A {
    unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> { unsafe { <A as SizeOf>::size_of(self, ptr) } }
}

unsafe impl<'a, A: SizeOfDebug> SizeOfDebug for &'a A {
    unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> { unsafe { A::size_of(self, ptr) } }
}



/// Testing functions to verify implementations of [`thin`] traits.
pub mod test {
    use super::*;

    /// Assert that [`Meta::ZST_SUPPORTED`] accurately reports if `A` supports ZSTs
    #[track_caller] pub fn zst_supported_accurate<A: Alloc + Free + Meta>(allocator: A) {
        let alloc = allocator.alloc_uninit(0);
        assert_eq!(alloc.is_ok(), A::ZST_SUPPORTED, "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED);
        alloc.ok().map(|alloc| unsafe { allocator.free(alloc) });
    }

    /// Assert that `A` supports ZSTs if [`Meta::ZST_SUPPORTED`] is set.
    #[track_caller] pub fn zst_supported_conservative<A: Alloc + Free + Meta>(allocator: A) {
        let alloc = allocator.alloc_uninit(0);
        if A::ZST_SUPPORTED { assert!(alloc.is_ok(), "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED) }
        alloc.ok().map(|alloc| unsafe { allocator.free(alloc) });
    }

    /// Assert that `A` supports ZSTs if [`Meta::ZST_SUPPORTED`] is set.  Also don't try to [`Free`] the memory involved.
    #[track_caller] pub fn zst_supported_conservative_leak<A: Alloc + Meta>(allocator: A) {
        let alloc = allocator.alloc_uninit(0);
        if A::ZST_SUPPORTED { assert!(alloc.is_ok(), "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED) }
    }
}
