//! C/C++y allocator traits operating on thin pointers, implying alignment, etc.
//!
//! C and C++ allocators often merely accept a pointer for free/realloc/size queries.
//! This module provides traits for such functionality.

use crate::*;
use crate::error::ExcessiveSliceRequestedError;
use crate::meta::Meta;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// Allocation functions with alignment (up to <code>[Meta]::[MAX_ALIGN](Meta::MAX_ALIGN)</code>) implied by size:
/// <code>
/// fn [alloc_uninit](Self::alloc_uninit)(size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;
/// fn [alloc_zeroed](Self::alloc_zeroed)(size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;
/// </code><br>
///
/// ## Safety
/// *   Allocations created by this trait must be compatible with any other [`thin`] traits implemented on this allocator type.
/// *   Returned allocations must be valid for at least `size` bytes.
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
    fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        self.alloc_zeroed(size).map(|p| p.cast())
    }

    /// Allocate at least `size` bytes of zeroed memory.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        let layout = Layout::from_size_align(size, 1).map_err(|_| ExcessiveSliceRequestedError { requested: size })?;
        let alloc = self.alloc_uninit(size)?;
        // SAFETY: ✔️ `alloc` was just allocated using `layout`
        unsafe { util::slice::from_raw_bytes_layout_mut(alloc, layout) }.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}



/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]<[MaybeUninit]<[u8]>>)</code><br>
/// <br>
///
/// ## Safety
/// *   This trait must be able to free allocations made by any other [`thin`] traits implemented on this allocator type.
pub unsafe trait Free : meta::Meta {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>) {
        // SAFETY: ✔️ free_nullable has ≈identical prereqs
        unsafe { self.free_nullable(ptr.as_ptr()) }
    }

    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` may be null, in which case this is a noop
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ free has ≈identical prereqs
        if let Some(ptr) = NonNull::new(ptr) { unsafe { self.free(ptr) } }
    }
}



/// Reallocation function:<br>
/// <code>[realloc_uninit](Self::realloc_uninit)(ptr: [NonNull]<[MaybeUninit]<[u8]>>, new_size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[realloc_zeroed](Self::realloc_zeroed)(ptr: [NonNull]<[MaybeUninit]<[u8]>>, new_size: [usize]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
///
/// ## Safety
/// *   This trait must be able to reallocate allocations made by any other [`thin`] traits implemented on this allocator type.
/// *   Returned allocations must be valid for at least `new_size` bytes.
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
/// <code>[size_of](Self::size_of)(ptr: [NonNull]<[MaybeUninit]<[u8]>>) -> [usize]</code><br>
/// <br>
///
/// ## Safety
/// *   This trait must be able to safely query the size of allocations made by any [`thin`] traits implemented on this allocator type.
/// *   If given a valid `ptr`, you promise that `ptr[..a.size_of(ptr)]` is dereferenceable.
pub unsafe trait SizeOf : SizeOfDebug {
    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> usize {
        // SAFETY: ✔️ SizeOfDebug::size_of_debug has identical prereqs
        unsafe { SizeOfDebug::size_of_debug(self, ptr) }.unwrap()
    }
}



/// Allocation size query (unreliable / for debug purpouses only):<br>
/// <code>[size_of_debug](Self::size_of_debug)(ptr: [NonNull]<[MaybeUninit]<[u8]>>) -> [Option]<[usize]></code><br>
/// <br>
///
/// ## Safety
/// *   This trait must be able to safely query the size of allocations made by any [`thin`] traits implemented on this allocator type.
/// *   If given a valid `ptr`, by returning <code>[Some]\(...\)</code>, you promise that `ptr[..a.size_of_debug(ptr)]` is dereferenceable.
///
/// ## Remarks
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
/// let Some(size) = (unsafe{ alloc.size_of_debug(ptr) }) else { return };
/// let slice : &[MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), size) };
/// # }
/// ```
pub unsafe trait SizeOfDebug : meta::Meta {
    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn size_of_debug(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize>;
}



#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Alloc> Alloc for &'a A {
    fn alloc_uninit(&self, size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { A::alloc_uninit(self, size) }
    fn alloc_zeroed(&self, size: usize) -> Result<NonNull<            u8 >, Self::Error> { A::alloc_zeroed(self, size) }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Free> Free for &'a A {
    unsafe fn free(         &self, ptr: NonNull<MaybeUninit<u8>> ) { unsafe { A::free(         self, ptr) } }
    unsafe fn free_nullable(&self, ptr: *mut    MaybeUninit<u8>  ) { unsafe { A::free_nullable(self, ptr) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Realloc> Realloc for &'a A {
    const CAN_REALLOC_ZEROED : bool = A::CAN_REALLOC_ZEROED;
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { A::realloc_uninit(self, ptr, new_size) } }
    unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: usize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { A::realloc_zeroed(self, ptr, new_size) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: SizeOf> SizeOf for &'a A {
    unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> usize { unsafe { A::size_of(self, ptr) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: SizeOfDebug> SizeOfDebug for &'a A {
    unsafe fn size_of_debug(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> { unsafe { A::size_of_debug(self, ptr) } }
}



/// Testing functions to verify implementations of [`thin`] traits.
pub mod test {
    use super::*;

    /// "Thin Test Box"
    #[allow(clippy::upper_case_acronyms)]
    struct TTB<A: Free>(A, NonNull<MaybeUninit<u8>>);
    impl<A: Free> Drop for TTB<A> {
        fn drop(&mut self) {
            // SAFETY: ✔️ we exclusively own the alloc `self.1`
            unsafe { self.0.free(self.1) };
        }
    }
    impl<A: Free> TTB<A> {
        pub fn try_new_uninit(allocator: A, size: usize) -> Result<Self, A::Error> where A : Alloc { let alloc = allocator.alloc_uninit(size)?; Ok(Self(allocator, alloc)) }
        pub fn try_new_zeroed(allocator: A, size: usize) -> Result<Self, A::Error> where A : Alloc { let alloc = allocator.alloc_zeroed(size)?; Ok(Self(allocator, alloc.cast())) }
        fn as_ptr(&self) -> *mut MaybeUninit<u8> { self.1.as_ptr() }
        fn as_nonnull(&self) -> NonNull<MaybeUninit<u8>> { self.1 }
    }

    /// Assert that `allocator` meets all it's alignment requirements
    pub fn alignment<A: Alloc + Free>(allocator: A) {
        // First, a quick test
        let mut align = A::MAX_ALIGN;
        loop {
            let unaligned_mask = align.as_usize() - 1;
            if let Ok(alloc) = TTB::try_new_uninit(&allocator, align.as_usize()) {
                let alloc = alloc.as_ptr();
                let addr = alloc as usize;
                assert_eq!(0, addr & unaligned_mask, "allocation for size {align:?} @ {alloc:?} had less than expected alignment ({align:?} <= MAX_ALIGN)");
            }
            let Some(next) = Alignment::new(align.as_usize() >> 1) else { break };
            align = next;
        }

        // Something a little more stress testy
        for size in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
            std::dbg!(size);
            let mut addr_bits = 0;
            for _ in 0 .. 100 {
                if let Ok(alloc) = TTB::try_new_uninit(&allocator, size) {
                    addr_bits |= alloc.as_ptr() as usize;
                }
            }
            if addr_bits == 0 { continue }
            let align = 1 << addr_bits.trailing_zeros(); // usually 16, occasionally 32+
            let expected_align = A::MAX_ALIGN.as_usize().min(size).max(A::MIN_ALIGN.as_usize());
            assert!(align >= expected_align);
        }
    }

    /// Check edge cases near 2 GiB, 4 GiB, usize::MAX/2, and usize::MAX watermarks.
    pub fn edge_case_sizes<A: Alloc + Free>(allocator: A) {
        let boundaries = if cfg!(target_pointer_width = "64") {
            &[0, (u32::MAX/2) as usize, (u32::MAX  ) as usize, usize::MAX/2, usize::MAX][..]
        } else {
            &[0, usize::MAX/2, usize::MAX][..]
        };
        for boundary in boundaries.iter().copied() {
            for offset in -64_isize .. 64_isize {
                let Some(size) = boundary.checked_add_signed(offset) else { continue };
                std::dbg!(size);
                let Ok(alloc) = TTB::try_new_uninit(&allocator, size) else { continue };
                if let Some(last_byte_index) = size.checked_sub(1) {
                    let last_byte_index = last_byte_index.min(isize::MAX as usize);
                    // SAFETY: ✔️ in bounds of allocated object
                    // SAFETY: ✔️ cannot overflow an isize (capped immediately above)
                    // SAFETY: ✔️ does not wrap around address space
                    let last_byte = unsafe { alloc.as_ptr().add(last_byte_index) };
                    // SAFETY: ✔️ in bounds of allocated object
                    unsafe { last_byte.write_volatile(MaybeUninit::new(42u8)) };
                }
            }
        }
    }

    /// Assert that `allocator` meets all it's nullable allocation requirements
    pub fn nullable<A: Free>(allocator: A) {
        // SAFETY: ✔️ freeing null should always be safe
        unsafe { allocator.free_nullable(core::ptr::null_mut()) }
    }

    /// Assert that `allocator` always reports an exact allocation size
    pub fn size_exact_alloc<A: Alloc + Free + SizeOfDebug>(allocator: A) {
        for size in [0, 1, 3, 7, 15, 31, 63, 127] {
            let Ok(alloc) = TTB::try_new_uninit(&allocator, size) else { continue };
            // SAFETY: ✔️ `alloc` belongs to `allocator`
            let query_size = unsafe { allocator.size_of_debug(alloc.as_nonnull()) }.unwrap_or(size);
            assert_eq!(size, query_size, "allocator returns oversized allocs, use thin::test::size_over_alloc instead");
        }
    }

    /// Assert that `allocator` sometimes reports a larger-than-requested allocation size
    pub fn size_over_alloc<A: Alloc + Free + SizeOfDebug>(allocator: A) {
        let mut any_sized = false;
        let mut over = false;
        for size in [0, 1, 3, 7, 15, 31, 63, 127] {
            let Ok(alloc) = TTB::try_new_uninit(&allocator, size) else { continue };
            // SAFETY: ✔️ `alloc` belongs to `allocator`
            let Some(query_size) = (unsafe { allocator.size_of_debug(alloc.as_nonnull()) }) else { continue };
            any_sized = true;
            over |= size < query_size;
            assert!(size <= query_size, "allocator returns undersized allocs");
        }
        assert!(over || !any_sized, "no allocations were oversized");
    }

    /// **UNSOUND:** Verify `A` allocates uninitialized memory by reading `MaybeUninit<u8>`s.
    ///
    /// This is technically completely unnecessary - but educational for verifying assumptions.  Use this only in non-production unit tests.
    #[allow(clippy::missing_safety_doc)] // It's in the first line instead
    pub unsafe fn uninit_alloc_unsound<A: Alloc + Free>(allocator: A) {
        let mut any = false;
        for _ in 0 .. 1000 {
            if let Ok(alloc) = TTB::try_new_uninit(&allocator, 1) {
                any = true;

                // SAFETY: ✔️ should be safe to access the first byte of a 1 byte alloc
                let byte = unsafe { &mut *alloc.as_ptr() };

                // SAFETY: ❌ this is unsound per the fn preamble!
                let is_uninit = unsafe { byte.assume_init() } != 0;

                byte.write(0xFF); // ensure we'll detect "uninitialized" memory if this alloc is reused

                if is_uninit { return } // success!
            }
        }
        assert!(!any, "A::alloc_uninit appears to allocate zeroed memory");
    }

    /// Assert that `allocator` always provides zeroed memory when requested
    pub fn zeroed_alloc<A: Alloc + Free>(allocator: A) {
        for _ in 0 .. 1000 {
            if let Ok(alloc) = TTB::try_new_zeroed(&allocator, 1) {
                // SAFETY: ✔️ should be safe to access the first byte of a 1 byte alloc - and being zeroed, it should be safe to strip MaybeUninit
                let byte : &mut u8 = unsafe { &mut *alloc.as_ptr().cast::<u8>() };

                let is_zeroed = *byte == 0u8;
                *byte = 0xFF; // ensure we'll detect "unzeroed" memory if this alloc is reused without zeroing

                assert!(is_zeroed, "A::alloc_zeroed returned unzeroed memory!");
            }
        }
    }

    /// Assert that [`Meta::ZST_SUPPORTED`] accurately reports if `A` supports ZSTs
    pub fn zst_supported_accurate<A: Alloc + Free>(allocator: A) {
        let alloc = TTB::try_new_uninit(&allocator, 0);
        let alloc = alloc.as_ref().map(|a| a.as_ptr());
        assert_eq!(alloc.is_ok(), A::ZST_SUPPORTED, "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED);
    }

    /// Assert that `A` supports ZSTs if [`Meta::ZST_SUPPORTED`] is set.
    pub fn zst_supported_conservative<A: Alloc + Free>(allocator: A) {
        let alloc = TTB::try_new_uninit(&allocator, 0);
        let alloc = alloc.as_ref().map(|a| a.as_ptr());
        if A::ZST_SUPPORTED { assert!(alloc.is_ok(), "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED) }
    }

    /// Assert that `A` supports ZSTs if [`Meta::ZST_SUPPORTED`] is set.  Also don't try to [`Free`] the memory involved.
    pub fn zst_supported_conservative_leak<A: Alloc>(allocator: A) {
        let alloc = allocator.alloc_uninit(0);
        if A::ZST_SUPPORTED { assert!(alloc.is_ok(), "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED) }
    }
}
