use crate::*;
use super::ffi;

use core::ptr::NonNull;



/// [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDeleteArray;

impl meta::Meta for NewDeleteArray {
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
unsafe impl thin::Alloc for NewDeleteArray {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ this "should" be safe for all `size`.  Unsound C++ stdlibs are #[test]ed for at the end of this file.
        NonNull::new(unsafe { ffi::operator_new_array_nothrow(size) }.cast()).ok_or(())
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl thin::Free for NewDeleteArray {
    unsafe fn free_nullable(&self, ptr: *mut core::mem::MaybeUninit<u8>) {
        // SAFETY: ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
        // SAFETY: ✔️ `ptr` is either `nullptr` (safe), or belongs to `self` per thin::Free::free_nullable's documented safety preconditions - and thus was allocated with `::operator new(size)`.
        unsafe { ffi::operator_delete_array(ptr.cast()) };
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
unsafe impl fat::Realloc for NewDeleteArray {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, NewDeleteArray};

    impls! {
        unsafe impl ialloc::fat::Alloc      for NewDeleteArray => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Free       for NewDeleteArray => ialloc::thin::Free;
    }
}



#[test] fn thin_alignment()         { thin::test::alignment(NewDeleteArray) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(NewDeleteArray) }
#[test] fn thin_nullable()          { thin::test::nullable(NewDeleteArray) }
#[test] fn thin_uninit()            { if !cfg!(target_os = "linux") { unsafe { thin::test::uninit_alloc_unsound(NewDeleteArray) } } } // `::operator new[]` returns zeroed memory on some platforms
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(NewDeleteArray) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_conservative(NewDeleteArray) }
