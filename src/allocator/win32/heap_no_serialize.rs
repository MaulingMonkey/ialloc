use crate::*;
use super::Error;

use winapi::um::heapapi::{HeapAlloc, HeapReAlloc, HeapFree, HeapSize, HeapDestroy, HeapCreate};
use winapi::um::winnt::{HANDLE, HEAP_ZERO_MEMORY, HEAP_NO_SERIALIZE, HEAP_GENERATE_EXCEPTIONS};

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// [`HeapAlloc`] / [`HeapReAlloc`] / [`HeapFree`] / [`HeapSize`] w/ [`HEAP_NO_SERIALIZE`]
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[HeapAlloc](heap, [HEAP_NO_SERIALIZE], size)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[HeapAlloc](heap, [HEAP_NO_SERIALIZE]\|[HEAP_ZERO_MEMORY], size)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[HeapReAlloc](heap, [HEAP_NO_SERIALIZE], ptr, size)</code>
/// | [`thin::Realloc::realloc_zeroed`]         | <code>[HeapReAlloc](heap, [HEAP_NO_SERIALIZE]\|[HEAP_ZERO_MEMORY], ptr, size)</code>
/// | [`thin::Free::free`]                      | <code>[HeapFree]\(heap, [HEAP_NO_SERIALIZE], ptr\)</code>
/// | [`thin::SizeOf::size_of`]                 | <code>[HeapSize]\(heap, [HEAP_NO_SERIALIZE], ptr\)</code>
///
/// ## Recommended Reading
/// *   [Heap Functions](https://learn.microsoft.com/en-us/windows/win32/memory/heap-functions)
/// *   [Low-fragmentation Heap](https://learn.microsoft.com/en-us/windows/win32/memory/low-fragmentation-heap)
///
#[doc = include_str!("_refs.md")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] // SAFETY: this cannot be Clone or Copy as this owns the `HANDLE`
#[repr(transparent)] // SAFETY: HeapNoSerialize::borrow makes use of this
pub struct HeapNoSerialize(HANDLE); // SAFETY: This should always be a valid `Heap*` compatible handle, to a heap used exclusively by this thread

impl Drop for HeapNoSerialize {
    fn drop(&mut self) {
        // SAFETY: ✔️ We supposedly exclusively own `self.0`
        let succeeds = unsafe { HeapDestroy(self.0) };
        if succeeds == 0 {
            let err = super::get_last_error();
            panic!("HeapDestroy({:?}) failed with GetLastError() == 0x{:08x}", self.0, err);
        }
    }
}

/// ```no_compile,E0277
/// # use ialloc::allocator::win32::HeapNoSerialize;
/// let heap = HeapNoSerialize::create(0, None, None);
/// assert_send(heap);
/// fn assert_send(_: impl Send) {}
/// ```
///
/// ```no_compile,E0277
/// # use ialloc::allocator::win32::HeapNoSerialize;
/// let heap = HeapNoSerialize::create(0, None, None);
/// assert_sync(heap);
/// fn assert_sync(_: impl Sync) {}
/// ```
#[allow(dead_code)] struct AssertNotSendSync;

impl HeapNoSerialize {
    /// Borrow a [`HeapAlloc`]-compatible `HANDLE`.
    ///
    /// ### Safety
    /// *   `*handle` must be a valid [`HeapAlloc`]-compatible `HANDLE`.
    /// *   `*handle` must be a growable heap
    /// *   `*handle` must only be accessed by the current thread
    /// *   `*handle` must remain valid while borrowed
    ///
    #[doc = include_str!("_refs.md")]
    pub unsafe fn borrow(handle: &HANDLE) -> &Self {
        // SAFETY: ✔️ `Self` is a `#[repr(transparent)]` wrapper around `HANDLE`, so this should be safe
        unsafe { core::mem::transmute(handle) }
    }

    /// [`HeapCreate`]
    ///
    /// [`HEAP_NO_SERIALIZE`] is recommended but not required.
    ///
    /// ### Safety
    /// *   <code>options &amp; [HEAP_GENERATE_EXCEPTIONS]</code> is forbidden: Rust assumes C-ABI / no SEH exceptions
    /// *   New `options` may be added which are similarly undefined behavior.
    /// *   The function may make a best-effort attempt to [`panic!`] instead of invoking UB.
    /// *   No idea what happens if `initial_size` > `maximum_size`.
    ///
    #[doc = include_str!("_refs.md")]
    pub unsafe fn try_create(options: u32, initial_size: Option<NonZeroUsize>, maximum_size: Option<NonZeroUsize>) -> Result<Self, Error> {
        assert!(options & HEAP_GENERATE_EXCEPTIONS == 0, "bug: undefined behavior: HeapNoSerialize::try_create cannot be used with HEAP_GENERATE_EXCEPTIONS");
        let initial_size = initial_size.map_or(0, |nz| nz.get());
        let maximum_size = maximum_size.map_or(0, |nz| nz.get());

        // SAFETY: ✔️ preconditions documented in Safety docs
        let handle = unsafe { HeapCreate(options, initial_size, maximum_size) };
        if handle.is_null() { return Err(Error::get_last()) }
        Ok(Self(handle))
    }

    /// [`HeapCreate`]
    ///
    /// [`HEAP_NO_SERIALIZE`] is recommended but not required.
    ///
    /// ### Safety
    /// *   <code>options &amp; [HEAP_GENERATE_EXCEPTIONS]</code> is forbidden: Rust assumes C-ABI / no SEH exceptions
    /// *   New `options` may be added which are similarly undefined behavior.
    /// *   The function may make a best-effort attempt to [`panic!`] instead of invoking UB.
    /// *   No idea what happens if `initial_size` > `maximum_size`.
    ///
    #[doc = include_str!("_refs.md")]
    pub unsafe fn create(options: u32, initial_size: Option<NonZeroUsize>, maximum_size: Option<NonZeroUsize>) -> Self {
        // SAFETY: ✔️ create and try_create have identical preconditions
        unsafe { Self::try_create(options, initial_size, maximum_size) }.unwrap_or_else(|err| panic!("HeapCreate failed with GetLastError() == {err:?}"))
    }
}

impl meta::Meta for HeapNoSerialize {
    type Error = Error;

    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing

    /// The alignment of memory returned by `HeapAlloc` is `MEMORY_ALLOCATION_ALIGNMENT` in WinNT.h:
    /// ```cpp
    /// #if defined(_WIN64) || defined(_M_ALPHA)
    /// #define MEMORY_ALLOCATION_ALIGNMENT 16
    /// #else
    /// #define MEMORY_ALLOCATION_ALIGNMENT 8
    /// #endif
    /// ```
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapalloc#remarks>
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = true;
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`HeapNoSerialize`] is `'static` - allocations by [`HeapAlloc`] live until [`HeapReAlloc`]ed, [`HeapFree`]d, or the [`HeapNoSerialize`] is [`Drop`]ed outright (not merely moved.)
/// | `compatible`  | ✔️ [`HeapNoSerialize`] uses exclusively intercompatible `Heap*` fns
/// | `exclusive`   | ✔️ Allocations by [`HeapAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`HeapAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is used, making this not thread safe.  However, we enforce `HeapNoSerialize : !Send + !Sync` as validated by `AssertNotSendSync`.
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], correct use of [`HEAP_ZERO_MEMORY`]
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for HeapNoSerialize {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap, accessed exclusively by this thread, per `Self`'s construction
        let alloc = unsafe { HeapAlloc(self.0, HEAP_NO_SERIALIZE, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap, accessed exclusively by this thread, per `Self`'s construction
        let alloc = unsafe { HeapAlloc(self.0, HEAP_NO_SERIALIZE | HEAP_ZERO_MEMORY, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`HeapNoSerialize`] is `'static` - reallocations by [`HeapReAlloc`] live until [`HeapReAlloc`]ed again or [`HeapFree`]d
/// | `compatible`  | ✔️ [`HeapNoSerialize`] uses exclusively intercompatible `Heap*` fns
/// | `exclusive`   | ✔️ Allocations by [`HeapReAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`HeapReAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is used, making this not thread safe.  However, we enforce `HeapNoSerialize : !Send + !Sync` as validated by `AssertNotSendSync`.
/// | `zeroed`      | ⚠️ untested, but we use [`HEAP_ZERO_MEMORY`] appropriately...
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Realloc for HeapNoSerialize {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap, accessed exclusively by this thread, per `Self`'s construction
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        let alloc = unsafe { HeapReAlloc(self.0, HEAP_NO_SERIALIZE, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap, accessed exclusively by this thread, per `Self`'s construction
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        let alloc = unsafe { HeapReAlloc(self.0, HEAP_NO_SERIALIZE | HEAP_ZERO_MEMORY, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`HeapNoSerialize`] uses exclusively intercompatible `Heap*` fns
/// | `exceptions`  | ✔️ [`HeapFree`] returns `FALSE`/`0` on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is used, making this not thread safe.  However, we enforce `HeapNoSerialize : !Send + !Sync` as validated by `AssertNotSendSync`.
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Free for HeapNoSerialize {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (validated via [`thin::test::nullable`] and documented: "[This pointer can be NULL](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapfree#parameters).")
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions - and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        if unsafe { HeapFree(self.0, HEAP_NO_SERIALIZE, ptr.cast()) } == 0 && cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}

// SAFETY: ✔️ SizeOfDebug has same preconditions
unsafe impl thin::SizeOf for HeapNoSerialize {}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `size`        | ✔️ Validated via [`thin::test::size_exact_alloc`]
/// | `compatible`  | ✔️ [`HeapNoSerialize`] uses exclusively intercompatible `Heap*` fns
/// | `exceptions`  | ✔️ [`HeapSize`] returns `-1` on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is used, making this not thread safe.  However, we enforce `HeapNoSerialize : !Send + !Sync` as validated by `AssertNotSendSync`.
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::SizeOfDebug for HeapNoSerialize {
    unsafe fn size_of_debug(&self, ptr: AllocNN) -> Option<usize> {
        // SAFETY: ✔️ `ptr` belongs to `self` per [`thin::SizeOfDebug::size_of_debug`]'s documented safety preconditions - and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        let size = unsafe { HeapSize(self.0, HEAP_NO_SERIALIZE, ptr.as_ptr().cast()) };
        if size == !0 { return None }
        Some(size)
    }
}



#[no_implicit_prelude] mod cleanroom {
    use super::{impls, HeapNoSerialize};

    impls! {
        unsafe impl ialloc::fat::Alloc      for HeapNoSerialize => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for HeapNoSerialize => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for HeapNoSerialize => ialloc::thin::Free;
    }
}



#[cfg(test)] fn create_test_heap(initial_size: Option<NonZeroUsize>, maximum_size: Option<NonZeroUsize>) -> HeapNoSerialize {
    // SAFETY: ✔️ no forbidden flags used
    unsafe { HeapNoSerialize::create(HEAP_NO_SERIALIZE, initial_size, maximum_size) }
}

#[test] fn thin_alignment() {
    thin::test::alignment(create_test_heap(None, None));
    thin::test::alignment(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn thin_edge_case_sizes() {
    thin::test::edge_case_sizes(create_test_heap(None, None));
    thin::test::edge_case_sizes(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn thin_nullable() {
    thin::test::nullable(create_test_heap(None, None));
    thin::test::nullable(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn thin_size() {
    thin::test::size_exact_alloc(create_test_heap(None, None));
    thin::test::size_exact_alloc(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn thin_uninit() {
    unsafe {
        thin::test::uninit_alloc_unsound(create_test_heap(None, None));
        thin::test::uninit_alloc_unsound(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    }
}

#[test] fn thin_zeroed() {
    thin::test::zeroed_alloc(create_test_heap(None, None));
    thin::test::zeroed_alloc(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn thin_zst_support() {
    thin::test::zst_supported_accurate(create_test_heap(None, None));
    thin::test::zst_supported_accurate(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}



#[test] fn fat_alignment() {
    fat::test::alignment(create_test_heap(None, None));
    fat::test::alignment(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn fat_edge_case_sizes() {
    fat::test::edge_case_sizes(create_test_heap(None, None));
    fat::test::edge_case_sizes(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn fat_uninit() {
    unsafe { fat::test::uninit_alloc_unsound(create_test_heap(None, None)) };
    unsafe { fat::test::uninit_alloc_unsound(create_test_heap(None, NonZeroUsize::new(1024 * 1024))) };
}

#[test] fn fat_zeroed() {
    fat::test::zeroed_alloc(create_test_heap(None, None));
    fat::test::zeroed_alloc(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}

#[test] fn fat_zst_support() {
    fat::test::zst_supported_accurate(create_test_heap(None, None));
    fat::test::zst_supported_accurate(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
}
