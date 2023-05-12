use crate::*;

use winapi::um::heapapi::{HeapAlloc, HeapReAlloc, HeapFree, HeapSize, GetProcessHeap};
use winapi::um::winnt::{HANDLE, HEAP_ZERO_MEMORY};

use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// [`HeapAlloc`] / [`HeapReAlloc`] / [`HeapFree`] / [`HeapSize`]
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[HeapAlloc](heap, 0, size)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[HeapAlloc](heap, HEAP_ZERO_MEMORY, size)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[HeapReAlloc](heap, 0, ptr, size)</code>
/// | [`thin::Realloc::realloc_zeroed`]         | <code>[HeapReAlloc](heap, HEAP_ZERO_MEMORY, ptr, size)</code>
/// | [`thin::Free::free`]                      | <code>[HeapFree]\(heap, 0, ptr\)</code>
/// | [`thin::SizeOf::size_of`]                 | <code>[HeapSize]\(heap, 0, ptr\)</code>
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct HeapRef<'a> {
    handle:     HANDLE,
    _phantom:   PhantomData<&'a ()>,
}

// SAFETY: ✔️ All construction paths forbid `HEAP_NO_SERIALIZE`, so simultanious access of the `HANDLE` should be safe.
unsafe impl Sync for HeapRef<'_> {}

// SAFETY: ✔️ The ref doesn't own the `HANDLE`, giving new threads should be safe given that this is already `Sync`.
unsafe impl Send for HeapRef<'_> {}

impl<'a> HeapRef<'a> {
    /// Wrap a [`HeapAlloc`]-compatible `HANDLE`.
    ///
    /// ### Safety
    /// *   `handle` must be a valid [`HeapAlloc`]-compatible `HANDLE`.
    /// *   `handle` must be a growable heap
    /// *   `handle` must only be accessed in a serialized fashion (e.g. not creating using, nor never used with, `HEAP_NO_SERIALIZE`)
    /// *   `handle` must remain valid for the lifetime of `'a'`.
    ///
    #[doc = include_str!("_refs.md")]
    pub unsafe fn new(handle: HANDLE) -> Self { Self { handle, _phantom: PhantomData } }
}

impl HeapRef<'static> {
    /// <code>[GetProcessHeap]\(\)</code>
    #[doc = include_str!("_refs.md")]
    pub fn process() -> Self {
        // SAFETY: ⚠️ I assert that undefined behavior must've already happened if things have gone so catastrophically wrong as for this to fail.
        let heap = unsafe { GetProcessHeap() };

        // SAFETY: I assert that undefined behavior must've already happened if things have gone so catastrophically wrong for any of the following assumptions to not be true:
        // SAFETY: ✔️ `GetProcessHeap()` is a valid [`HeapAlloc`]-compatible
        // SAFETY: ✔️ `GetProcessHeap()` is a growable heap
        // SAFETY: ⚠️ `GetProcessHeap()` is only accessed without `HEAP_NO_SERIALIZE`, or by code that is already undefined behavior as third party injected DLLs presumably use said heap from their own threads.
        // SAFETY: ⚠️ `GetProcessHeap()` is valid for the lifetime of the process / `'static`, as any code closing it presumably invokes undefined behavior by third party injected DLLs.
        unsafe { Self::new(heap) }
    }
}

impl meta::Meta for HeapRef<'_> {
    type Error = ();

    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing

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
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for HeapRef<'_> {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ thread safe - we don't use HEAP_NO_SERIALIZE, and preclude others from using it in the safety docs for all construction paths of `HeapRef`.
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsoundness is #[test]ed for at the end of this file.
        let alloc = unsafe { HeapAlloc(self.handle, 0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        // SAFETY: ✔️ thread safe - we don't use HEAP_NO_SERIALIZE and preclude using it in all construction paths for `HeapRef`.
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsoundness is #[test]ed for at the end of this file.
        // SAFETY: ✔️ HeapAlloc zeros memory when we use HEAP_ZERO_MEMORY
        let alloc = unsafe { HeapAlloc(self.handle, HEAP_ZERO_MEMORY, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Realloc for HeapRef<'_> {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ thread safe - we don't use HEAP_NO_SERIALIZE and preclude using it in all construction paths for `HeapRef`.
        // SAFETY: ⚠️ this "should" be safe for all `size`.  Unsoundness is not yet #[test]ed for.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with `Heap{,Re}Alloc`
        let alloc = unsafe { HeapReAlloc(self.handle, 0, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ thread safe - we don't use HEAP_NO_SERIALIZE and preclude using it in all construction paths for `HeapRef`.
        // SAFETY: ⚠️ this "should" be safe for all `size`.  Unsoundness is not yet #[test]ed for.
        // SAFETY: ✔️ HeapReAlloc zeros memory when we use HEAP_ZERO_MEMORY
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Realloc's documented safety preconditions, and thus was allocated with `Heap{,Re}Alloc`
        let alloc = unsafe { HeapReAlloc(self.handle, HEAP_ZERO_MEMORY, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for HeapRef<'_> {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // "This pointer can be NULL."
        // https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapfree#parameters
        //
        // SAFETY: ✔️ thread safe - we don't use HEAP_NO_SERIALIZE and preclude using it in all construction paths for `HeapRef`.
        // SAFETY: ✔️ `ptr` is either `nullptr` (safe, tested), or belongs to `self` per thin::Free::free_nullable's documented safety preconditions - and thus was allocated with `Heap{,Re}Alloc`
        if unsafe { HeapFree(self.handle, 0, ptr.cast()) } == 0 && cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}


// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::SizeOf for HeapRef<'_> {}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::SizeOfDebug for HeapRef<'_> {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> {
        // SAFETY: ✔️ thread safe - we don't use HEAP_NO_SERIALIZE and preclude using it in all construction paths for `HeapRef`.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::SizeOfDebug's documented safety preconditions, and thus was allocated with `Heap{,Re}Alloc`
        let size = unsafe { HeapSize(self.handle, 0, ptr.as_ptr().cast()) };
        if size == !0 { return None }
        Some(size)
    }
}



/// [`HeapAlloc`] / [`HeapReAlloc`] / [`HeapFree`] / [`HeapSize`] on <code>[GetProcessHeap]\(\)</code>
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[HeapAlloc]\([GetProcessHeap]\(\), 0, size\)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[HeapAlloc]\([GetProcessHeap]\(\), HEAP_ZERO_MEMORY, size\)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[HeapReAlloc]\([GetProcessHeap]\(\), 0, ptr, size\)</code>
/// | [`thin::Realloc::realloc_zeroed`]         | <code>[HeapReAlloc]\([GetProcessHeap]\(\), HEAP_ZERO_MEMORY, ptr, size\)</code>
/// | [`thin::Free::free`]                      | <code>[HeapFree]\([GetProcessHeap]\(\), 0, ptr\)</code>
/// | [`thin::SizeOf::size_of`]                 | <code>[HeapSize]\([GetProcessHeap]\(\), 0, ptr\)</code>
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct ProcessHeap;

impl meta::Meta for ProcessHeap {
    type Error = ();

    //const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing

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
    const MAX_SIZE  : usize     = usize::MAX/2;
    const ZST_SUPPORTED : bool  = true;
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::Alloc for ProcessHeap {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error>  { HeapRef::process().alloc_uninit(size) }
    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> { HeapRef::process().alloc_zeroed(size) }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::Realloc for ProcessHeap {
    const CAN_REALLOC_ZEROED : bool = HeapRef::CAN_REALLOC_ZEROED;
    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { unsafe { HeapRef::process().realloc_uninit(ptr, new_size) } }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { unsafe { HeapRef::process().realloc_zeroed(ptr, new_size) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::Free          for ProcessHeap {
    unsafe fn free         (&self, ptr: NonNull<MaybeUninit<u8>>) { unsafe { HeapRef::process().free(ptr) } }
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>    ) { unsafe { HeapRef::process().free_nullable(ptr) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::SizeOf        for ProcessHeap {}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ implemented against same traits with same prereqs
unsafe impl thin::SizeOfDebug   for ProcessHeap { unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> { unsafe { HeapRef::process().size_of(ptr) } } }



#[no_implicit_prelude] mod cleanroom {
    use super::{impls, HeapRef, ProcessHeap};

    impls! {
        unsafe impl ialloc::fat::Alloc      for HeapRef<'_> => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for HeapRef<'_> => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for HeapRef<'_> => ialloc::thin::Free;

        unsafe impl ialloc::fat::Alloc      for ProcessHeap => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for ProcessHeap => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for ProcessHeap => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment() {
    thin::test::alignment(ProcessHeap);
    thin::test::alignment(HeapRef::process());
}

#[test] fn thin_edge_case_sizes() {
    thin::test::edge_case_sizes(ProcessHeap);
    thin::test::edge_case_sizes(HeapRef::process());
}

#[test] fn thin_nullable() {
    thin::test::nullable(ProcessHeap);
    thin::test::nullable(HeapRef::process());
}

#[test] fn thin_zst_support() {
    thin::test::zst_supported_accurate(ProcessHeap);
    thin::test::zst_supported_accurate(HeapRef::process());
}
