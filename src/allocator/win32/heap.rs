use crate::*;
use super::Error;

use winapi::um::heapapi::{HeapAlloc, HeapReAlloc, HeapFree, HeapSize, GetProcessHeap, HeapDestroy, HeapCreate};
use winapi::um::winnt::{HANDLE, HEAP_ZERO_MEMORY, HEAP_NO_SERIALIZE, HEAP_GENERATE_EXCEPTIONS};

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// [`HeapAlloc`] / [`HeapReAlloc`] / [`HeapFree`] / [`HeapSize`]
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[HeapAlloc](heap, 0, size)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[HeapAlloc](heap, [HEAP_ZERO_MEMORY], size)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[HeapReAlloc](heap, 0, ptr, size)</code>
/// | [`thin::Realloc::realloc_zeroed`]         | <code>[HeapReAlloc](heap, [HEAP_ZERO_MEMORY], ptr, size)</code>
/// | [`thin::Free::free`]                      | <code>[HeapFree]\(heap, 0, ptr\)</code>
/// | [`thin::SizeOf::size_of`]                 | <code>[HeapSize]\(heap, 0, ptr\)</code>
///
/// ## Recommended Reading
/// *   [Heap Functions](https://learn.microsoft.com/en-us/windows/win32/memory/heap-functions)
/// *   [Low-fragmentation Heap](https://learn.microsoft.com/en-us/windows/win32/memory/low-fragmentation-heap)
///
#[doc = include_str!("_refs.md")]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] // SAFETY: this cannot be Clone or Copy as this owns the `HANDLE`
#[repr(transparent)] // SAFETY: Heap::borrow makes use of this
pub struct Heap(HANDLE); // SAFETY: This should always be a valid `Heap*` compatible handle

impl Drop for Heap {
    fn drop(&mut self) {
        // SAFETY: ✔️ We supposedly exclusively own `self.0`
        let succeeds = unsafe { HeapDestroy(self.0) };
        if succeeds == 0 {
            let err = super::get_last_error();
            panic!("HeapDestroy({:?}) failed with GetLastError() == 0x{:08x}", self.0, err);
        }
    }
}

// SAFETY: ✔️ All construction paths forbid `HEAP_NO_SERIALIZE`, so simultanious access of the `HANDLE` should be safe.
unsafe impl Sync for Heap {}

// SAFETY: ✔️ All construction paths forbid `HEAP_NO_SERIALIZE`, this seems like it *should* be safe...
unsafe impl Send for Heap {}

#[test] fn cross_thread_destroy_fuzz_test() {
    let threads = (0..50).map(|_| {
        let heap = create_test_heap(None, None);
        thin::test::alignment(&heap); // put the heap to some use
        std::thread::spawn(move || std::mem::drop(heap)) // Drop on another thread for testing purpouses
    }).collect::<std::vec::Vec<_>>();
    for thread in threads { thread.join().unwrap() }
}

impl Heap {
    /// Borrow a [`HeapAlloc`]-compatible `HANDLE`.
    ///
    /// ### Safety
    /// *   `*handle` must be a valid [`HeapAlloc`]-compatible `HANDLE`.
    /// *   `*handle` must be a growable heap
    /// *   `*handle` must only be accessed in a serialized fashion (e.g. not creating using, nor ever used with, [`HEAP_NO_SERIALIZE`])
    /// *   `*handle` must remain valid while borrowed
    ///
    #[doc = include_str!("_refs.md")]
    pub unsafe fn borrow(handle: &HANDLE) -> &Self {
        // SAFETY: ✔️ `Self` is a `#[repr(transparent)]` wrapper around `HANDLE`, so this should be safe
        unsafe { core::mem::transmute(handle) }
    }

    /// [`HeapCreate`]
    ///
    /// ### Safety
    /// *   <code>options &amp; [HEAP_NO_SERIALIZE]</code> is forbidden: [`Heap`] is [`Send`]+[`Sync`] so that would be undefined behavior.
    /// *   <code>options &amp; [HEAP_GENERATE_EXCEPTIONS]</code> is forbidden: Rust assumes C-ABI / no SEH exceptions
    /// *   New `options` may be added which are similarly undefined behavior.
    /// *   The function may make a best-effort attempt to [`panic!`] instead of invoking UB.
    /// *   No idea what happens if `initial_size` > `maximum_size`.
    ///
    #[doc = include_str!("_refs.md")]
    pub unsafe fn try_create(options: u32, initial_size: Option<NonZeroUsize>, maximum_size: Option<NonZeroUsize>) -> Result<Self, Error> {
        assert!(options & HEAP_NO_SERIALIZE == 0, "bug: undefined behavior: Heap::try_create cannot be used with HEAP_NO_SERIALIZE");
        assert!(options & HEAP_GENERATE_EXCEPTIONS == 0, "bug: undefined behavior: Heap::try_create cannot be used with HEAP_GENERATE_EXCEPTIONS");
        let initial_size = initial_size.map_or(0, |nz| nz.get());
        let maximum_size = maximum_size.map_or(0, |nz| nz.get());

        // SAFETY: ✔️ preconditions documented in Safety docs
        let handle = unsafe { HeapCreate(options, initial_size, maximum_size) };
        if handle.is_null() { return Err(Error::get_last()) }
        Ok(Self(handle))
    }

    /// [`HeapCreate`]
    ///
    /// ### Safety
    /// *   <code>options &amp; [HEAP_NO_SERIALIZE]</code> is forbidden: [`Heap`] is [`Send`]+[`Sync`] so that would be undefined behavior.
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

    /// <code>[GetProcessHeap]\(\)</code>
    #[doc = include_str!("_refs.md")]
    pub fn with_process<R>(f: impl FnOnce(&Self) -> R) -> R {
        // SAFETY: ⚠️ I assert that undefined behavior must've already happened if things have gone so catastrophically wrong as for this to fail.
        let heap = unsafe { GetProcessHeap() };

        // SAFETY: I assert that undefined behavior must've already happened if things have gone so catastrophically wrong for any of the following assumptions to not be true:
        // SAFETY: ✔️ `GetProcessHeap()` is a valid [`HeapAlloc`]-compatible `HANDLE`
        // SAFETY: ✔️ `GetProcessHeap()` is a growable heap
        // SAFETY: ⚠️ `GetProcessHeap()` is never used with [`HEAP_NO_SERIALIZE`] ("This value should not be specified when accessing the process's default heap.")
        // SAFETY: ⚠️ `GetProcessHeap()` is valid for the lifetime of the process / `'static`, as any code closing it presumably invokes undefined behavior by third party injected DLLs.
        f(unsafe { Self::borrow(&heap) })
    }
}

impl meta::Meta for Heap {
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
/// | `pin`         | ✔️ [`Heap`] is `'static` - allocations by [`HeapAlloc`] live until [`HeapReAlloc`]ed, [`HeapFree`]d, or the [`Heap`] is [`Drop`]ed outright (not merely moved.)
/// | `compatible`  | ✔️ [`Heap`] uses exclusively intercompatible `Heap*` fns
/// | `exclusive`   | ✔️ Allocations by [`HeapAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`HeapAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is not used, making this thread safe.
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], correct use of [`HEAP_ZERO_MEMORY`]
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for Heap {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap per `Self`'s construction
        let alloc = unsafe { HeapAlloc(self.0, 0, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap per `Self`'s construction
        let alloc = unsafe { HeapAlloc(self.0, HEAP_ZERO_MEMORY, size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`Heap`] is `'static` - reallocations by [`HeapReAlloc`] live until [`HeapReAlloc`]ed again or [`HeapFree`]d
/// | `compatible`  | ✔️ [`Heap`] uses exclusively intercompatible `Heap*` fns
/// | `exclusive`   | ✔️ Allocations by [`HeapReAlloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`HeapReAlloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is not used, making this thread safe.
/// | `zeroed`      | ⚠️ untested, but we use [`HEAP_ZERO_MEMORY`] appropriately...
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Realloc for Heap {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap, per `Self`'s construction
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        let alloc = unsafe { HeapReAlloc(self.0, 0, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `self.0` is a valid heap, per `Self`'s construction
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        let alloc = unsafe { HeapReAlloc(self.0, HEAP_ZERO_MEMORY, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or_else(Error::get_last)
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`Heap`] uses exclusively intercompatible `Heap*` fns
/// | `exceptions`  | ✔️ [`HeapFree`] returns `FALSE`/`0` on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is not used, making this thread safe.
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Free for Heap {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (validated via [`thin::test::nullable`] and documented: "[This pointer can be NULL](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapfree#parameters).")
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions - and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        if unsafe { HeapFree(self.0, 0, ptr.cast()) } == 0 && cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}

// SAFETY: ✔️ SizeOfDebug has same preconditions
unsafe impl thin::SizeOf for Heap {}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `size`        | ✔️ Validated via [`thin::test::size_exact_alloc`]
/// | `compatible`  | ✔️ [`Heap`] uses exclusively intercompatible `Heap*` fns
/// | `exceptions`  | ✔️ [`HeapSize`] returns `-1` on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`HEAP_NO_SERIALIZE`] is not used, making this thread safe.
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::SizeOfDebug for Heap {
    unsafe fn size_of_debug(&self, ptr: AllocNN) -> Option<usize> {
        // SAFETY: ✔️ `ptr` belongs to `self` per [`thin::SizeOfDebug::size_of_debug`]'s documented safety preconditions - and thus was allocated with `Heap{,Re}Alloc` on `self.0`
        let size = unsafe { HeapSize(self.0, 0, ptr.as_ptr().cast()) };
        if size == !0 { return None }
        Some(size)
    }
}



/// [`HeapAlloc`] / [`HeapReAlloc`] / [`HeapFree`] / [`HeapSize`] on <code>[GetProcessHeap]\(\)</code>
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[HeapAlloc]\([GetProcessHeap]\(\), 0, size\)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[HeapAlloc]\([GetProcessHeap]\(\), [HEAP_ZERO_MEMORY], size\)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[HeapReAlloc]\([GetProcessHeap]\(\), 0, ptr, size\)</code>
/// | [`thin::Realloc::realloc_zeroed`]         | <code>[HeapReAlloc]\([GetProcessHeap]\(\), [HEAP_ZERO_MEMORY], ptr, size\)</code>
/// | [`thin::Free::free`]                      | <code>[HeapFree]\([GetProcessHeap]\(\), 0, ptr\)</code>
/// | [`thin::SizeOf::size_of`]                 | <code>[HeapSize]\([GetProcessHeap]\(\), 0, ptr\)</code>
///
/// ## Recommended Reading
/// *   [Heap Functions](https://learn.microsoft.com/en-us/windows/win32/memory/heap-functions)
/// *   [Low-fragmentation Heap](https://learn.microsoft.com/en-us/windows/win32/memory/low-fragmentation-heap)
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct ProcessHeap;

impl meta::Meta for ProcessHeap {
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

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::Alloc for ProcessHeap {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error>  { Heap::with_process(|heap| heap.alloc_uninit(size)) }
    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> { Heap::with_process(|heap| heap.alloc_zeroed(size)) }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::Realloc for ProcessHeap {
    const CAN_REALLOC_ZEROED : bool = Heap::CAN_REALLOC_ZEROED;
    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { Heap::with_process(|heap| unsafe { heap.realloc_uninit(ptr, new_size) }) }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { Heap::with_process(|heap| unsafe { heap.realloc_zeroed(ptr, new_size) }) }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::Free          for ProcessHeap {
    unsafe fn free         (&self, ptr: NonNull<MaybeUninit<u8>>) { Heap::with_process(|heap| unsafe { heap.free(ptr) }) }
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>    ) { Heap::with_process(|heap| unsafe { heap.free_nullable(ptr) }) }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::SizeOf        for ProcessHeap {}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::SizeOfDebug   for ProcessHeap { unsafe fn size_of_debug(&self, ptr: AllocNN) -> Option<usize> { Heap::with_process(|heap| unsafe { heap.size_of_debug(ptr) }) } }



#[no_implicit_prelude] mod cleanroom {
    use super::{impls, Heap, ProcessHeap};

    impls! {
        unsafe impl ialloc::fat::Alloc      for Heap => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for Heap => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for Heap => ialloc::thin::Free;

        unsafe impl ialloc::fat::Alloc      for ProcessHeap => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for ProcessHeap => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for ProcessHeap => ialloc::thin::Free;
    }
}



#[cfg(test)] fn create_test_heap(initial_size: Option<NonZeroUsize>, maximum_size: Option<NonZeroUsize>) -> Heap {
    // SAFETY: ✔️ no forbidden flags used
    unsafe { Heap::create(0, initial_size, maximum_size) }
}

#[test] fn thin_alignment() {
    thin::test::alignment(ProcessHeap);
    thin::test::alignment(create_test_heap(None, None));
    thin::test::alignment(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| thin::test::alignment(heap));
}

#[test] fn thin_edge_case_sizes() {
    thin::test::edge_case_sizes(ProcessHeap);
    thin::test::edge_case_sizes(create_test_heap(None, None));
    thin::test::edge_case_sizes(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| thin::test::edge_case_sizes(heap));
}

#[test] fn thin_nullable() {
    thin::test::nullable(ProcessHeap);
    thin::test::nullable(create_test_heap(None, None));
    thin::test::nullable(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| thin::test::nullable(heap));
}

#[test] fn thin_size() {
    thin::test::size_exact_alloc(ProcessHeap);
    thin::test::size_exact_alloc(create_test_heap(None, None));
    thin::test::size_exact_alloc(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| thin::test::size_exact_alloc(heap));
}

#[test] fn thin_uninit() {
    unsafe {
        thin::test::uninit_alloc_unsound(ProcessHeap);
        thin::test::uninit_alloc_unsound(create_test_heap(None, None));
        thin::test::uninit_alloc_unsound(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
        Heap::with_process(|heap| thin::test::uninit_alloc_unsound(heap));
    }
}

#[test] fn thin_zeroed() {
    thin::test::zeroed_alloc(ProcessHeap);
    thin::test::zeroed_alloc(create_test_heap(None, None));
    thin::test::zeroed_alloc(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| thin::test::zeroed_alloc(heap));
}

#[test] fn thin_zst_support() {
    thin::test::zst_supported_accurate(ProcessHeap);
    thin::test::zst_supported_accurate(create_test_heap(None, None));
    thin::test::zst_supported_accurate(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| thin::test::zst_supported_accurate(heap));
}



#[test] fn fat_alignment() {
    fat::test::alignment(ProcessHeap);
    fat::test::alignment(create_test_heap(None, None));
    fat::test::alignment(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| fat::test::alignment(heap));
}

#[test] fn fat_edge_case_sizes() {
    fat::test::edge_case_sizes(ProcessHeap);
    fat::test::edge_case_sizes(create_test_heap(None, None));
    fat::test::edge_case_sizes(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| fat::test::edge_case_sizes(heap));
}

#[test] fn fat_uninit() {
    unsafe { fat::test::uninit_alloc_unsound(ProcessHeap) };
    unsafe { fat::test::uninit_alloc_unsound(create_test_heap(None, None)) };
    unsafe { fat::test::uninit_alloc_unsound(create_test_heap(None, NonZeroUsize::new(1024 * 1024))) };
    Heap::with_process(|heap| unsafe { fat::test::uninit_alloc_unsound(heap) });
}

#[test] fn fat_zeroed() {
    fat::test::zeroed_alloc(ProcessHeap);
    fat::test::zeroed_alloc(create_test_heap(None, None));
    fat::test::zeroed_alloc(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| fat::test::zeroed_alloc(heap));
}

#[test] fn fat_zst_support() {
    fat::test::zst_supported_accurate(ProcessHeap);
    fat::test::zst_supported_accurate(create_test_heap(None, None));
    fat::test::zst_supported_accurate(create_test_heap(None, NonZeroUsize::new(1024 * 1024)));
    Heap::with_process(|heap| fat::test::zst_supported_accurate(heap));
}

