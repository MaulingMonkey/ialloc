use crate::*;

use winapi::um::heapapi::{HeapAlloc, HeapReAlloc, HeapFree, HeapSize, GetProcessHeap};
use winapi::um::winnt::{HANDLE, HEAP_ZERO_MEMORY};

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

const MEMORY_ALLOCATION_ALIGNMENT : Alignment = Alignment::constant(winapi::um::winnt::MEMORY_ALLOCATION_ALIGNMENT);



/// [`HeapAlloc`] / [`HeapReAlloc`] / [`HeapFree`] / [`HeapSize`]
///
/// | Rust                                      | C                     |
/// | ------------------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]             | <code>[HeapAlloc](heap, 0, size)</code>
/// | [`thin::Alloc::alloc_zeroed`]             | <code>[HeapAlloc](heap, HEAP_ZERO_MEMORY, size)</code>
/// | [`thin::Realloc::realloc_uninit`]         | <code>[HeapReAlloc](heap, 0, ptr, size)</code>
/// | [`thin::ReallocZeroed::realloc_zeroed`]   | <code>[HeapReAlloc](heap, HEAP_ZERO_MEMORY, ptr, size)</code>
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

unsafe impl thin::Alloc for Heap {
    type Error = ();

    /// The alignment of memory returned by HeapAlloc is MEMORY_ALLOCATION_ALIGNMENT in WinNT.h:
    /// ```cpp
    /// #if defined(_WIN64) || defined(_M_ALPHA)
    /// #define MEMORY_ALLOCATION_ALIGNMENT 16
    /// #else
    /// #define MEMORY_ALLOCATION_ALIGNMENT 8
    /// #endif
    /// ```
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapalloc#remarks>
    const MAX_ALIGN : Alignment = MEMORY_ALLOCATION_ALIGNMENT;

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { HeapAlloc(self.0, 0, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<AllocNN0, Self::Error> {
        let size = super::check_size(size)?;
        let alloc = unsafe { HeapAlloc(self.0, HEAP_ZERO_MEMORY, size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Realloc for Heap {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let new_size = super::check_size(new_size)?;
        let alloc = unsafe { HeapReAlloc(self.0, 0, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::ReallocZeroed for Heap {
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let new_size = super::check_size(new_size)?;
        let alloc = unsafe { HeapReAlloc(self.0, HEAP_ZERO_MEMORY, ptr.as_ptr().cast(), new_size) };
        NonNull::new(alloc.cast()).ok_or(())
    }
}

unsafe impl thin::Free for Heap {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        // "This pointer can be NULL."
        // https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapfree#parameters
        assert!(0 != unsafe { HeapFree(self.0, 0, ptr.cast()) });
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
/// | [`thin::ReallocZeroed::realloc_zeroed`]   | <code>[HeapReAlloc]\([GetProcessHeap]\(\), HEAP_ZERO_MEMORY, ptr, size\)</code>
/// | [`thin::Free::free`]                      | <code>[HeapFree]\([GetProcessHeap]\(\), 0, ptr\)</code>
/// | [`thin::SizeOf::size_of`]                 | <code>[HeapSize]\([GetProcessHeap]\(\), 0, ptr\)</code>
///
#[doc = include_str!("_refs.md")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct ProcessHeap;

unsafe impl thin::Alloc for ProcessHeap {
    type Error = ();

    /// The alignment of memory returned by HeapAlloc is MEMORY_ALLOCATION_ALIGNMENT in WinNT.h:
    /// ```cpp
    /// #if defined(_WIN64) || defined(_M_ALPHA)
    /// #define MEMORY_ALLOCATION_ALIGNMENT 16
    /// #else
    /// #define MEMORY_ALLOCATION_ALIGNMENT 8
    /// #endif
    /// ```
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapalloc#remarks>
    const MAX_ALIGN : Alignment = MEMORY_ALLOCATION_ALIGNMENT;

    fn alloc_uninit(&self, size: NonZeroUsize) -> Result<AllocNN, Self::Error>  { Heap::process().alloc_uninit(size) }
    fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<AllocNN0, Self::Error> { Heap::process().alloc_zeroed(size) }
}

unsafe impl thin::Realloc       for ProcessHeap { unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> { unsafe { Heap::process().realloc_uninit(ptr, new_size) } } }
unsafe impl thin::ReallocZeroed for ProcessHeap { unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> { unsafe { Heap::process().realloc_zeroed(ptr, new_size) } } }
unsafe impl thin::Free          for ProcessHeap {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>) { unsafe { Heap::process().free(ptr) } }
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) { unsafe { Heap::process().free_nullable(ptr) } }
}
unsafe impl thin::SizeOf        for ProcessHeap {}
unsafe impl thin::SizeOfDebug   for ProcessHeap { unsafe fn size_of(&self, ptr: AllocNN) -> Option<usize> { unsafe { Heap::process().size_of(ptr) } } }
