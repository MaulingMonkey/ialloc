use crate::*;
use crate::meta::Meta;
use super::ffi;

use core::alloc::Layout;
use core::ptr::NonNull;



/// [`::operator new[](size_t, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*, align_val_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteArrayAligned;

impl Meta for NewDeleteArrayAligned {
    type Error                  = ();

    const MAX_ALIGN : Alignment = if cfg!(target_os = "macos") {
        // macOS 11.7.6 20G1231 seems to provide at most 4 GiB alignment, returning e.g. 0x7fbf00000000 when 8 GiB alignment is requested
        // https://github.com/MaulingMonkey/ialloc/actions/runs/4998062851/jobs/8953095728
        ALIGN_MIN_4_GiB_MAX
    } else {
        Alignment::MAX
    };

    const MAX_SIZE  : usize     = usize::MAX;       // XXX: less in practice
    const ZST_SUPPORTED : bool  = false;            // platform behavior too inconsistent
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Alloc for NewDeleteArrayAligned {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error> {
        if Self::MAX_ALIGN != Alignment::MAX && layout.align() > Self::MAX_ALIGN.as_usize() { return Err(()) }
        NonNull::new(unsafe { ffi::operator_new_array_align_nothrow(layout.size(), layout.align()) }.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Free for NewDeleteArrayAligned {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) {
        unsafe { ffi::operator_delete_array_align(ptr.as_ptr().cast(), layout.align()) };
    }
}

// SAFETY: ✔️ all fat::* impls intercompatible with each other
unsafe impl fat::Realloc for NewDeleteArrayAligned {}



#[test] fn fat_alignment()          { fat::test::alignment(NewDeleteArrayAligned) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(NewDeleteArrayAligned) }
#[test] fn fat_uninit()             { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(NewDeleteArrayAligned) } } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(NewDeleteArrayAligned) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_conservative(NewDeleteArrayAligned) }
