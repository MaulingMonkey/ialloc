use crate::*;
use crate::meta::*;
use super::ffi;

use core::alloc::Layout;
use core::ptr::NonNull;



/// [`::operator new[](size_t, align_val_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteArrayAligned;



// meta::*

impl Meta for NewDeleteArrayAligned {
    type Error                  = ();

    /// | Platform          | Value     |
    /// | ------------------| ----------|
    /// | OS X 64-bit       | 2 GiB (macOS 11.7.6 20G1231 [seems to](https://github.com/MaulingMonkey/ialloc/actions/runs/4999128292/jobs/8955213565) return only 2 GiB alignment when 4+ GiB is requested)
    /// | Linux 64-bit      | [`Alignment::MAX`] (2<sup>63</sup> B)
    /// | Windows 64-bit    | [`Alignment::MAX`] (2<sup>63</sup> B)
    /// | \* 32-bit         | [`Alignment::MAX`] (2 GiB)
    const MAX_ALIGN : Alignment = if cfg!(target_os = "macos") { ALIGN_MIN_2_GiB_MAX } else { Alignment::MAX };

    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = false;            // platform behavior too inconsistent
}

// SAFETY: ✔️ global state only
unsafe impl Stateless for NewDeleteArrayAligned {}



// fat::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`fat::test::alignment`]
/// | `size`        | ✔️ Validated via [`fat::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`NewDeleteArrayAligned`] is `'static` - allocations by [`::operator new[](size_t, align_val_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) live until [`::operator delete[](void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)ed.
/// | `compatible`  | ✔️ [`NewDeleteArrayAligned`] uses exclusively intercompatible arrayed, explicit-alignment operators from the same stdlib.
/// | `compatible`  | ✔️ [`::operator delete[](void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is compatible with [`::operator new[](size_t, align_val_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) (C++17)
/// | `exclusive`   | ✔️ [`::operator new[](size_t, align_val_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) allocations are exclusive/unique
/// | `exceptions`  | ✔️ [`::operator new[](size_t, align_val_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) is `noexcept` and will return null on error (C++17)
/// | `threads`     | ✔️ these operators postdate `std::thread` (C++11) and should be thread safe on platforms supporting threads.
/// | `zeroed`      | ✔️ Validated via [`fat::test::zeroed_alloc`]
///
// SAFETY: per above
unsafe impl fat::Alloc for NewDeleteArrayAligned {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ OS X can underalign if we don't perform this explicit check (see `MAX_ALIGN`'s notes).
        if Self::MAX_ALIGN != Alignment::MAX && layout.align() > Self::MAX_ALIGN.as_usize() { return Err(()) }
        // SAFETY: per trait safety above
        NonNull::new(unsafe { ffi::operator_new_array_align_nothrow(layout.size(), layout.align()) }.cast()).ok_or(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`NewDeleteArrayAligned`] uses exclusively intercompatible arrayed, explicit-alignment operators from the same stdlib.
/// | `compatible`  | ✔️ [`::operator delete[](void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is compatible with [`::operator new[](size_t, align_val_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) (C++17)
/// | `exceptions`  | ✔️ [`::operator delete[](void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is `noexcept` and returns no errors (C++17)
/// | `threads`     | ✔️ these operators postdate `std::thread` (C++11) and should be thread safe on platforms supporting threads.
///
// SAFETY: per above
unsafe impl fat::Free for NewDeleteArrayAligned {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        // SAFETY: ✔️ `ptr` belongs to `self` per [`fat::Free::free`]'s documented safety preconditions
        unsafe { ffi::operator_delete_array_align(ptr.as_ptr().cast(), layout.align()) };
    }
}

// SAFETY: ✔️ default Realloc impl is soundly implemented in terms of Alloc+Free
unsafe impl fat::Realloc for NewDeleteArrayAligned {}



#[test] fn fat_alignment()          { fat::test::alignment(NewDeleteArrayAligned) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(NewDeleteArrayAligned) }
#[test] fn fat_uninit()             { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(NewDeleteArrayAligned) } } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(NewDeleteArrayAligned) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_conservative(NewDeleteArrayAligned) }
