use crate::*;

use winapi::um::heapapi::{HeapAlloc, HeapReAlloc, HeapFree, HeapSize, GetProcessHeap};
use winapi::um::winnt::{HANDLE, HEAP_ZERO_MEMORY};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Heap(HANDLE);

unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}

impl Heap {
    /// Wrap a [`HeapAlloc`]-compatible `HANDLE`.
    ///
    /// ### Safety
    /// *   `handle` must be a valid [`HeapAlloc`]-compatible `HANDLE`.
    /// *   `handle` must be a growable heap
    /// *   `handle` must only be accessed in a serialized fashion (e.g. never used with `HEAP_NO_SERIALIZE`)
    /// *   `handle` must remain valid for the lifetime of `Self`.
    ///
    #[doc = include_str!("_refs.md")]
    pub unsafe fn new(handle: HANDLE) -> Self { Self(handle) }

    /// <code>[GetProcessHeap]\(\)</code>
    ///
    #[doc = include_str!("_refs.md")]
    pub fn process() -> Self { unsafe { Self::new(GetProcessHeap()) } }
}

impl meta::Meta for Heap {
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

unsafe impl thin::Alloc for Heap {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { HeapAlloc(self.0, 0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { HeapAlloc(self.0, HEAP_ZERO_MEMORY, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Realloc for Heap {
    const CAN_REALLOC_ZEROED : bool = true;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let new_size = super::check_size(new_size)?;
        let alloc = unsafe { HeapReAlloc(self.0, 0, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let new_size = super::check_size(new_size)?;
        let alloc = unsafe { HeapReAlloc(self.0, HEAP_ZERO_MEMORY, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Free for Heap {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // "This pointer can be NULL."
        // https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapfree#parameters
        if unsafe { HeapFree(self.0, 0, ptr.cast()) } == 0 && cfg!(debug_assertions) { bug::ub::free_failed(ptr) }
    }
}

unsafe impl thin::SizeOf for Heap {}
unsafe impl thin::SizeOfDebug for Heap {
    unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> {
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

unsafe impl thin::Alloc for ProcessHeap {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error>  { Heap::process().alloc_uninit(size) }
    fn alloc_zeroed(&self, size: usize) -> Result<AllocNN0, Self::Error> { Heap::process().alloc_zeroed(size) }
}

unsafe impl thin::Realloc for ProcessHeap {
    const CAN_REALLOC_ZEROED : bool = Heap::CAN_REALLOC_ZEROED;
    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { unsafe { Heap::process().realloc_uninit(ptr, new_size) } }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> { unsafe { Heap::process().realloc_zeroed(ptr, new_size) } }
}
unsafe impl thin::Free          for ProcessHeap {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>) { unsafe { Heap::process().free(ptr) } }
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) { unsafe { Heap::process().free_nullable(ptr) } }
}
unsafe impl thin::SizeOf        for ProcessHeap {}
unsafe impl thin::SizeOfDebug   for ProcessHeap { unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> { unsafe { Heap::process().size_of(ptr) } } }



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



#[test] fn thin_alignment() {
    thin::test::alignment(ProcessHeap);
    thin::test::alignment(Heap::process());
}

#[test] fn thin_nullable() {
    thin::test::nullable(ProcessHeap);
    thin::test::nullable(Heap::process());
}

#[test] fn thin_zst_support() {
    thin::test::zst_supported_accurate(ProcessHeap);
    thin::test::zst_supported_accurate(Heap::process());
}
