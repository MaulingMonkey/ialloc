//! Rusty [ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts)-friendly allocator traits operating on [`Layout`]s

use crate::*;
use crate::boxed::ABox;
use crate::meta::Meta;

use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// Allocation functions:<br>
/// <code>[alloc_uninit](Self::alloc_uninit)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[alloc_zeroed](Self::alloc_zeroed)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
///
/// ## Safety
/// Invariants that [`Alloc`] must uphold for the soundness of various `unsafe` code include:
///
/// | Item          | Description   |
/// | --------------| --------------|
/// | `align`       | Returned allocations must have at least [`Layout::align`] alignment.
/// | `size`        | Returned allocations must be valid to read and/or write for at least [`Layout::size`] bytes.
/// | `pin`         | Returned allocations must remain valid at their initial address for the lifetime of `Self` (typically `'static`!), or until freed by [`Free`] or successful [`Realloc`] - whichever comes first.  In some cases (e.g. structs allocating from arrays on `self` without heap indirection), this might mean that the traits can only be implemented on shared references, rather than on the structs themselves.
/// | `compatible`  | Returned allocations must be compatible with all other [`fat`] traits implemented on the same allocator (at minimum in the sense that they must not cause undefined behavior when passed pointers to said allocations)
/// | `exclusive`   | Returned allocations belong exclusively to the caller.  Double allocating the same pool slot without a free would be undefined behavior.
/// | `exceptions`  | System allocators typically use `extern "C"` FFI, which is *not* safe to unwind exceptions through.  Ensure you catch any expected exceptions such as C++'s [`std::bad_alloc`] and SEH exceptions like `STATUS_HEAP_CORRUPTION` (e.g. don't use `HEAP_GENERATE_EXCEPTIONS`) on the C/C++ side before returning into Rust code.
/// | `threads`     | Allocators are typically implicitly [`Send`]+[`Sync`], which means the underlying FFI calls must be thread safe too.
/// | `zeroed`      | Allocations returned by [`Alloc::alloc_zeroed`] must be zeroed for their entire size.  This might be for more than `size` if [`thin::SizeOfDebug`] is implemented.
///
/// [`std::bad_alloc`]: https://en.cppreference.com/w/cpp/memory/new/bad_alloc
///
pub unsafe trait Alloc : Meta {
    /// Allocate at least `layout.size()` bytes of uninitialized memory aligned to `layout.align()`.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error>;

    /// Allocate at least `layout.size()` bytes of zeroed memory aligned to `layout.align()`.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        let alloc = self.alloc_uninit(layout)?;
        // SAFETY: ✔️ `alloc` was just allocated using `layout`
        let all = unsafe { util::slice::from_raw_bytes_layout_mut(alloc, layout) };
        all.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}

/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]&lt;\_&gt;, layout: [Layout])</code><br>
/// <br>
///
/// ## Safety
/// Invariants that [`Free`] must uphold for the soundness of various `unsafe` code include:
///
/// | Item          | Description   |
/// | --------------| --------------|
/// | `compatible`  | It must be safe to pass the result of any [`fat::Alloc`] or [`fat::Realloc`] function, implemented on the same type, to [`fat::Free`] function (once - with matching `layout` - as twice would be an unsound double free, and a layout mismatch is also unsound.)
/// | `exceptions`  | System allocators typically use `extern "C"` FFI, which is *not* safe to unwind exceptions through.  Avoid C++ or SEH exceptions in favor of error codes or fatal exceptions.
/// | `threads`     | Allocators are typically implicitly [`Send`]+[`Sync`], which means the underlying FFI calls must be thread safe too.
///
pub unsafe trait Free : Meta {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    /// *   `layout` must exactly match the [`Layout`] last used to successfully (re)allocate `ptr`
    unsafe fn free(&self, ptr: AllocNN, layout: Layout);
}

/// Reallocation function:<br>
/// <code>[realloc_uninit](Self::realloc_uninit)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[realloc_zeroed](Self::realloc_zeroed)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
///
/// ## Safety
/// Invariants that [`Realloc`] must uphold for the soundness of various `unsafe` code include:
///
/// | Item          | Description   |
/// | --------------| --------------|
/// | `align`       | Returned allocations must have at least [`Layout::align`] alignment.
/// | `size`        | Returned allocations must be valid to read and/or write for at least [`Layout::size`] bytes.
/// | `pin`         | Returned allocations must remain valid at their initial address for the lifetime of `Self` (typically `'static`!), or until freed by [`Free`] or successful [`Realloc`] - whichever comes first.  In some cases (e.g. structs allocating from arrays on `self` without heap indirection), this might mean that the traits can only be implemented on shared references, rather than on the structs themselves.
/// | `compatible`  | Returned allocations must be compatible with all other [`thin`] traits implemented on the same allocator (at minimum in the sense that they must not cause undefined behavior when passed pointers to said allocations)
/// | `exclusive`   | Returned allocations belong exclusively to the caller.  Double allocating the same pool slot without a free would be undefined behavior.
/// | `exceptions`  | System allocators typically use `extern "C"` FFI, which is *not* safe to unwind exceptions through.  Ensure you catch any expected exceptions such as C++'s [`std::bad_alloc`] and SEH exceptions like `STATUS_HEAP_CORRUPTION` (e.g. don't use `HEAP_GENERATE_EXCEPTIONS`) on the C/C++ side before returning into Rust code.
/// | `threads`     | Allocators are typically implicitly [`Send`]+[`Sync`], which means the underlying FFI calls must be thread safe too.
/// | `zeroed`      | Allocations returned by [`Realloc::realloc_zeroed`] must be zeroed between their old and new actualized size - which might be for more than `new_size` (especially if [`thin::SizeOfDebug`] is implemented.)
/// | `preserved`   | Allocations returned by [`Realloc`] must preserve their previous contents (although truncation if `new_size` is smaller than the old size is OK)
///
/// [`std::bad_alloc`]: https://en.cppreference.com/w/cpp/memory/new/bad_alloc
///
pub unsafe trait Realloc : Alloc + Free {
    /// Reallocate an existing allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after a succesful realloc (`realloc_uninit` returns <code>[Ok]\(...\)</code>)
    /// *   `old_layout` must exactly match the [`Layout`] last used to successfully (re)allocate `ptr`
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        if old_layout == new_layout { return Ok(ptr) }
        let alloc = self.alloc_uninit(new_layout)?;
        {
            // SAFETY: ✔️ `ptr` is valid for `old_layout` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
            // SAFETY: ✔️ `alloc` was just allocated using `new_layout`
            #![allow(clippy::undocumented_unsafe_blocks)]

            let old : &    [MaybeUninit<u8>] = unsafe { util::slice::from_raw_bytes_layout    (ptr,   old_layout) };
            let new : &mut [MaybeUninit<u8>] = unsafe { util::slice::from_raw_bytes_layout_mut(alloc, new_layout) };
            let n = old.len().min(new.len());
            new[..n].copy_from_slice(&old[..n]);
        }
        // SAFETY: ✔️ `ptr` is valid for `old_layout` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
        // SAFETY: ✔️ `ptr` belongs to `self` by `fat::Realloc::realloc_uninit`'s documented safety preconditions
        unsafe { self.free(ptr, old_layout) };
        Ok(alloc)
    }

    /// Reallocate an existing allocation, `ptr`, belonging to `self`.
    ///
    /// Any newly allocated memory will be zeroed.
    /// Any memory beyond `old_layout.size()` **may or may not** be zeroed.
    /// This might be *no* new memory, even if `new_layout.size()` > `old_layout.size()` - the implementation could've rounded both the old and new sizes up to the same value.
    /// As such, it's almost certainly a bug to use this to (re)allocate memory that came from <code>{,[re](Self::realloc_uninit)}[alloc_uninit](Alloc::alloc_uninit)</code>.
    /// Reallocate memory that came from <code>{,[re](Self::realloc_zeroed)}[alloc_zeroed](Alloc::alloc_zeroed)</code> instead.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after a succesful realloc (`realloc_uninit` returns <code>[Ok]\(...\)</code>)
    /// *   `old_layout` must exactly match the [`Layout`] last used to successfully (re)allocate `ptr`
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ realloc_uninit has same prereqs as realloc_zeroed
        let alloc = unsafe { self.realloc_uninit(ptr, old_layout, new_layout) }?;
        if old_layout.size() < new_layout.size() {
            // SAFETY: ✔️ `alloc` was just (re)allocated using `new_layout`
            let all = unsafe { util::slice::from_raw_bytes_layout_mut(alloc, new_layout) };
            let (_copied, new)  = all.split_at_mut(old_layout.size());
            new.fill(MaybeUninit::new(0u8));
        }
        Ok(alloc.cast())
    }
}



#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Alloc> Alloc for &'a A {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN,  Self::Error> { A::alloc_uninit(self, layout) }
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> { A::alloc_zeroed(self, layout) }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Free> Free for &'a A {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) { unsafe { A::free(self, ptr, layout) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Realloc> Realloc for &'a A {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { unsafe { A::realloc_uninit(self, ptr, old_layout, new_layout) } }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { unsafe { A::realloc_zeroed(self, ptr, old_layout, new_layout) } }
}



/// Testing functions to verify implementations of [`fat`] traits.
pub mod test {
    use super::*;

    /// "Fat Test Box"
    #[allow(clippy::upper_case_acronyms)]
    struct FTB<A: Free> {
        allocator:  A,
        layout:     Layout,
        data:       NonNull<MaybeUninit<u8>>
    }
    impl<A: Free> Drop for FTB<A> {
        fn drop(&mut self) {
            // SAFETY: ✔️ we exclusively own `self.data`
            // SAFETY: ✔️ `self.data` was allocated by `self.allocator` with layout `self.layout`
            unsafe { self.allocator.free(self.data, self.layout) }
        }
    }
    impl<A: Free> FTB<A> {
        pub fn try_new_uninit(allocator: A, layout: Layout) -> Result<Self, A::Error> where A : Alloc { let data = allocator.alloc_uninit(layout)?; Ok(Self{allocator, layout, data }) }
        fn as_ptr(&self) -> *mut MaybeUninit<u8> { self.data.as_ptr() }
    }

    /// Assert that `allocator` meets all it's alignment requirements
    pub fn alignment<A: Alloc + Free>(allocator: A) {
        let mut ok = true;
        for size in [1, 0] {
            let mut align = ALIGN_1;
            loop {
                let unaligned_mask = align.as_usize() - 1;
                let alloc = Layout::from_size_align(size, align.as_usize()).ok().and_then(|layout| FTB::try_new_uninit(&allocator, layout).ok());
                #[cfg(feature = "std")] std::println!("attempted to allocate size={size} align={align:?} ... {}", if alloc.is_some() { "ok" } else { "FAILED" });
                if let Some(alloc) = alloc {
                    let alloc = alloc.as_ptr();
                    let addr = alloc as usize;
                    assert_eq!(0, addr & unaligned_mask, "allocation for size {align:?} @ {alloc:?} had less than expected alignment ({align:?} <= MAX_ALIGN)");
                } else if align <= A::MAX_ALIGN && align <= ALIGN_4_KiB && (A::ZST_SUPPORTED || size > 0) {
                    ok = false;
                }
                let Some(next) = align.as_usize().checked_shl(1) else { break };
                let Some(next) = Alignment::new(next) else { break };
                align = next;
            }
        }
        assert!(ok, "not all expected alignment allocations succeeded");
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
                let Ok(layout) = Layout::from_size_align(size, 1) else { continue };
                #[cfg(feature = "std")] std::dbg!(size);
                let Ok(alloc) = FTB::try_new_uninit(&allocator, layout) else { continue };
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

    // nullable - fat::Free has no nullable free fns

    // size_* - fat::* has no Size traits

    /// **UNSOUND:** Verify `A` allocates uninitialized memory by reading `MaybeUninit<u8>`s.
    ///
    /// This is technically completely unnecessary - but educational for verifying assumptions.  Use this only in non-production unit tests.
    #[allow(clippy::missing_safety_doc)] // It's in the first line instead
    pub unsafe fn uninit_alloc_unsound<A: Alloc + Free>(allocator: A) {
        let mut any = false;
        for _ in 0 .. 1000 {
            if let Ok(mut byte) = ABox::<u8, _>::try_new_uninit_in(&allocator) {
                any = true;

                // SAFETY: ❌ this is unsound per the fn preamble!
                let is_uninit = unsafe { (*byte).assume_init() } != 0;

                (*byte).write(0xFF); // ensure we'll detect "uninitialized" memory if this alloc is reused

                if is_uninit { return } // success!
            }
        }
        assert!(!any, "A::alloc_uninit appears to allocate zeroed memory");
    }

    /// Assert that `allocator` always provides zeroed memory when requested
    pub fn zeroed_alloc<A: Alloc + Free>(allocator: A) {
        for _ in 0 .. 1000 {
            if let Ok(mut byte) = ABox::<u8, _>::try_new_bytemuck_zeroed_in(&allocator) {
                assert!(*byte == 0u8, "A::alloc_zeroed returned unzeroed memory!");
                *byte = 0xFF; // ensure we'll detect "unzeroed" memory if this alloc is reused without zeroing
            }
        }
    }

    /// Assert that [`Meta::ZST_SUPPORTED`] accurately reports if `A` supports ZSTs
    pub fn zst_supported_accurate<A: Alloc + Free>(allocator: A) {
        let alloc = FTB::try_new_uninit(&allocator, Layout::new::<()>());
        let alloc = alloc.as_ref().map(|a| a.as_ptr());
        assert_eq!(alloc.is_ok(), A::ZST_SUPPORTED, "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED);
    }

    /// Assert that `A` supports ZSTs if [`Meta::ZST_SUPPORTED`] is set.
    pub fn zst_supported_conservative<A: Alloc + Free>(allocator: A) {
        let alloc = FTB::try_new_uninit(&allocator, Layout::new::<()>());
        let alloc = alloc.as_ref().map(|a| a.as_ptr());
        if A::ZST_SUPPORTED { assert!(alloc.is_ok(), "alloc = {alloc:?}, ZST_SUPPORTED = {}", A::ZST_SUPPORTED) }
    }
}
