use crate::*;
use crate::meta::Meta;
use super::ffi;

use core::alloc::Layout;
use core::ptr::NonNull;



/// [`::operator new(size_t, align_val_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete(void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteAligned;

impl Meta for NewDeleteAligned {
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

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Alloc for NewDeleteAligned {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ OS X can underalign if we don't perform this explicit check.
        if Self::MAX_ALIGN != Alignment::MAX && layout.align() > Self::MAX_ALIGN.as_usize() { return Err(()) }

        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ this "should" allocate correctly for all `size`.  #[test]ed for via fat::test::edge_case_sizes at the end of this file.
        // SAFETY: ✔️ this "should" align correctly for all `align <= MAX_ALIGN`.  #[test]ed for via fat::test::alignment at the end of this file.
        // SAFETY: ✔️ should not throw
        NonNull::new(unsafe { ffi::operator_new_align_nothrow(layout.size(), layout.align()) }.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Free for NewDeleteAligned {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ `ptr` belongs to `self` per thin::Free::free's documented safety preconditions - and thus was allocated with `::operator new(size_t, align_val_t, nothrow_t)`
        unsafe { ffi::operator_delete_align(ptr.as_ptr().cast(), layout.align()) };
    }
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Realloc for NewDeleteAligned {}



#[test] fn fat_alignment()          { fat::test::alignment(NewDeleteAligned) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(NewDeleteAligned) }
#[test] fn fat_uninit()             { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(NewDeleteAligned) } } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(NewDeleteAligned) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_conservative(NewDeleteAligned) }
