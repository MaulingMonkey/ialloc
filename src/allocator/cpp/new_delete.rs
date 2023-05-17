use crate::*;
use super::ffi;

use core::ptr::NonNull;



/// [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete(void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDelete;

impl meta::Meta for NewDelete {
    type Error = ();

    const MAX_ALIGN : Alignment = if !cfg!(target_env = "msvc") {
        Alignment::of::<f64>() // conservative
        // __STDCPP_DEFAULT_NEW_ALIGNMENT__; // 8/16 - C++17
    } else if core::mem::size_of::<usize>() >= 8 {
        ALIGN_16
    } else {
        ALIGN_8
    };

    const MAX_SIZE : usize = usize::MAX; // *slightly* less in practice
    const ZST_SUPPORTED : bool = false; // works on both MSVC and Linux/Clang, no idea how standard this is however
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Alloc for NewDelete {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsound C++ stdlibs are #[test]ed for at the end of this file.
        NonNull::new(unsafe { ffi::operator_new_nothrow(size) }.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for NewDelete {
    unsafe fn free_nullable(&self, ptr: *mut core::mem::MaybeUninit<u8>) {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ `ptr` is either `nullptr` (safe), or belongs to `self` per thin::Free::free_nullable's documented safety preconditions - and thus was allocated with `::operator new(size, nothrow)`.
        unsafe { ffi::operator_delete(ptr.cast()) };
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl fat::Realloc for NewDelete {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, NewDelete};

    impls! {
        unsafe impl ialloc::fat::Alloc      for NewDelete => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Free       for NewDelete => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(NewDelete) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(NewDelete) }
#[test] fn thin_nullable()          { thin::test::nullable(NewDelete) }
#[test] fn thin_uninit()            { if !cfg!(target_os = "linux") { unsafe { thin::test::uninit_alloc_unsound(NewDelete) } } } // `::operator new` returns zeroed memory on some platforms
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(NewDelete) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_conservative(NewDelete) }

#[test] fn fat_alignment()          { fat::test::alignment(NewDelete) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(NewDelete) }
#[test] fn fat_uninit()             { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(NewDelete) } } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(NewDelete) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_conservative(NewDelete) }
