use crate::*;
use crate::meta::*;
use super::*;

use winapi::shared::winerror::SUCCEEDED;
use winapi::um::combaseapi::{CoGetMalloc};

use core::mem::MaybeUninit;
use core::ptr::{NonNull, null_mut};



/// [`IMalloc::Alloc`] / [`IMalloc::Realloc`] / [`IMalloc::Free`] / [`IMalloc::GetSize`]
///
/// | Rust                              | C++                   |
/// | ----------------------------------| ----------------------|
/// | [`thin::Alloc::alloc_uninit`]     | [`IMalloc::Alloc`]
/// | [`thin::Realloc::realloc_uninit`] | [`IMalloc::Realloc`]
/// | [`thin::Free::free`]              | [`IMalloc::Free`]
/// | [`thin::SizeOf::size_of`]         | [`IMalloc::GetSize`]
///
/// Uses the [`IMalloc`](https://learn.microsoft.com/en-us/windows/win32/api/objidl/nn-objidl-imalloc) interface as used for COM (de)allocations.
/// Consider using [`Heap`](super::Heap) directly instead, unless you're specifically doing COM / have documentation mandating a specific (de)allocator for interop purpouses.
///
/// ## References
/// *   [`CoTaskMem`](super::CoTaskMem) (stateless equivalent)
/// *   [Memory Allocation in COM](https://learn.microsoft.com/en-us/windows/win32/learnwin32/memory-allocation-in-com) (learn.microsoft.com)
///
#[doc = include_str!("_refs.md")]
#[derive(Clone)] #[repr(transparent)] pub struct IMalloc(mcom::Rc<winapi::um::objidlbase::IMalloc>);

impl From<mcom::Rc<winapi::um::objidlbase::IMalloc>> for IMalloc { fn from(value: mcom::Rc<winapi::um::objidlbase::IMalloc>) -> Self { Self(value) } }
impl From<IMalloc> for mcom::Rc<winapi::um::objidlbase::IMalloc> { fn from(value: IMalloc) -> Self { value.0 } }
impl TryFrom<CoTaskMem> for IMalloc { fn try_from(_: CoTaskMem) -> Result<Self, Self::Error> { Self::co_get_malloc_1() } type Error = Error; }
// TODO: test interop between CoTaskMem ←→ IMalloc

impl IMalloc {
    /// <code>[CoGetMalloc]\(1, &amp;mut result)</code>
    ///
    #[doc = include_str!("_refs.md")]
    pub fn co_get_malloc_1() -> Result<Self, Error> {
        let mut result = null_mut();
        let hr = unsafe { CoGetMalloc(1, &mut result) };
        if !SUCCEEDED(hr) { return Err(Error(winresult::ErrorHResultOrCode::from(hr))) }
        Ok(Self(unsafe { mcom::Rc::from_raw(result) }))
    }

    /// [`IMalloc::DidAlloc`](https://learn.microsoft.com/en-us/windows/win32/api/objidl/nf-objidl-imalloc-didalloc)
    pub fn did_alloc(&self, ptr: *mut MaybeUninit<u8>) -> Option<bool> {
        match unsafe { self.0.DidAlloc(ptr.cast()) } {
            1  => Some(true),
            0  => Some(false),
            -1 => None,
            _  => None, // unexpected
        }
    }

    /// [`IMalloc::HeapMinimize`](https://learn.microsoft.com/en-us/windows/win32/api/objidl/nf-objidl-imalloc-heapminimize)
    pub fn heap_minimize(&self) {
        unsafe { self.0.HeapMinimize() }
    }
}



// meta::*

impl Meta for IMalloc {
    type Error                  = ();
    const MIN_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_ALIGN : Alignment = super::MEMORY_ALLOCATION_ALIGNMENT; // Verified through testing
    const MAX_SIZE  : usize     = usize::MAX;

    /// -   `IMalloc::Alloc(0)` allocates successfully.
    /// -   `IMalloc::Realloc(ptr, 0)` **frees**.
    ///     Note that [`thin::Realloc`] and [`fat::Realloc`] resolve this by always requesting at least 1 byte.
    /// -   `IMalloc::GetSize(ptr)` will return inconsistent results for ZSTs as a result.
    ///
    const ZST_SUPPORTED : bool  = true;
}

impl ZstSupported for IMalloc {}



// thin::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`IMalloc`] is `'static` - allocations by [`IMalloc::Alloc`] live until [`IMalloc::Realloc`]ed, [`IMalloc::Free`]d, or theoretically with some impls, perhaps if the last reference to the underlying `IMalloc` is released (not merely moved.)
/// | `compatible`  | ✔️ [`IMalloc`] uses exclusively intercompatible fns
/// | `exclusive`   | ✔️ Allocations by [`IMalloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`IMalloc::Alloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`IMalloc`] is <code>\![Send] + \![Sync]</code>
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`], trivial default impl
///
#[doc = include_str!("_refs.md")]
// SAFETY: per above
unsafe impl thin::Alloc for IMalloc {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { self.0.Alloc(size) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    // no zeroing IMalloc::Alloc
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `size`        | ⚠️ untested, but *should* be safe if [`thin::Alloc`] was
/// | `pin`         | ✔️ [`IMalloc`] is `'static` - reallocations by [`IMalloc::Realloc`] live until [`IMalloc::Realloc`]ed, [`IMalloc::Free`]d, or theoretically with some impls, perhaps if the last reference to the underlying `IMalloc` is released (not merely moved.)
/// | `compatible`  | ✔️ [`IMalloc`] uses exclusively intercompatible fns
/// | `exclusive`   | ✔️ Reallocations by [`IMalloc`] are exclusive/unique
/// | `exceptions`  | ✔️ [`IMalloc::Realloc`] returns null on error per docs / lack of [`HEAP_GENERATE_EXCEPTIONS`].  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`IMalloc`] is <code>\![Send] + \![Sync]</code>
/// | `zeroed`      | ✔️ Trivial [`Err`] / not supported
/// | `preserved`   | ⚠️ untested, but *should* be the case...
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Realloc for IMalloc {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { self.0.Realloc(ptr.as_ptr().cast(), new_size.max(1)) };
        NonNull::new(alloc.cast()).ok_or(())
    }

    unsafe fn realloc_zeroed(&self, _ptr: AllocNN, _new_size: usize) -> Result<AllocNN, Self::Error> {
        Err(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`IMalloc`] uses exclusively intercompatible fns
/// | `exceptions`  | ✔️ [`IMalloc::Free`] returns no errors per docs.  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`IMalloc`] is <code>\![Send] + \![Sync]</code>
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::Free for IMalloc {
    unsafe fn free_nullable(&self, ptr: *mut MaybeUninit<u8>) {
        unsafe { self.0.Free(ptr.cast()) }
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `size`        | ✔️ Verified by [`thin::test::size_exact_alloc`]
/// | `compatible`  | ✔️ [`IMalloc`] uses exclusively intercompatible fns
/// | `exceptions`  | ✔️ [`IMalloc::GetSize`] returns `-1` on error per docs.  Non-unwinding fatalish heap corruption exceptions will only occur after previous undefined behavior.
/// | `threads`     | ✔️ [`IMalloc`] is <code>\![Send] + \![Sync]</code>
///
#[doc = include_str!("_refs.md")]
#[allow(clippy::missing_safety_doc)]
// SAFETY: per above
unsafe impl thin::SizeOfDebug for IMalloc {
    unsafe fn size_of_debug(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> {
        match unsafe { self.0.GetSize(ptr.as_ptr().cast()) } {
            usize::MAX  => None,
            size        => Some(size),
        }
    }
}

// SAFETY: ✔️ same preconditions as thin::SizeOfDebug
unsafe impl thin::SizeOf for IMalloc {}



// fat::*

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, IMalloc};

    impls! {
        unsafe impl ialloc::fat::Alloc      for IMalloc => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Realloc    for IMalloc => ialloc::thin::Realloc;
        unsafe impl ialloc::fat::Free       for IMalloc => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn thin_nullable()          { thin::test::nullable(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn thin_size()              { thin::test::size_exact_alloc(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(IMalloc::co_get_malloc_1().unwrap()) } }
#[test] fn thin_uninit_realloc()    { thin::test::uninit_realloc(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn thin_zeroed_realloc()    { thin::test::zeroed_realloc(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(IMalloc::co_get_malloc_1().unwrap()) }

#[test] fn fat_alignment()          { fat::test::alignment(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn fat_uninit()             { unsafe { fat::test::uninit_alloc_unsound(IMalloc::co_get_malloc_1().unwrap()) } }
#[test] fn fat_uninit_realloc()     { fat::test::uninit_realloc(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn fat_zeroed_realloc()     { fat::test::zeroed_realloc(IMalloc::co_get_malloc_1().unwrap()) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(IMalloc::co_get_malloc_1().unwrap()) }
