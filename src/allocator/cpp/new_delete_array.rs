use crate::*;
use super::ffi;

use core::ptr::NonNull;



/// [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete[](void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
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

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`NewDeleteArray`] is `'static` - allocations by [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) live until [`::operator delete[](void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)ed.
/// | `compatible`  | ✔️ [`NewDeleteArray`] uses exclusively intercompatible arrayed, implicit-alignment operators from the same stdlib.
/// | `compatible`  | ✔️ [`::operator delete[](void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is compatible with [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) (C++03 § 18.4.1.2 ¶ 11)
/// | `exclusive`   | ✔️ [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) allocations are exclusive/unique
/// | `exceptions`  | ✔️ [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) is `throw()` and will return null on error (C++03 § 18.4.1.2 ¶ 5, or `noexcept` in C++11+)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`]
///
// SAFETY: per above
unsafe impl thin::Alloc for NewDeleteArray {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: per above
        NonNull::new(unsafe { ffi::operator_new_array_nothrow(size) }.cast()).ok_or(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`NewDeleteArray`] uses exclusively intercompatible arrayed, implicit-alignment operators from the same stdlib.
/// | `compatible`  | ✔️ [`::operator delete[](void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is compatible with [`::operator new[](size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) (C++03 § 18.4.1.2 ¶ 11)
/// | `exceptions`  | ✔️ [`::operator delete[](void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is `throw()` and returns no errors (C++03 § 18.4.1.2 ¶ 9) / `noexcept` (C++11+)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
///
// SAFETY: per above
unsafe impl thin::Free for NewDeleteArray {
    unsafe fn free_nullable(&self, ptr: *mut core::mem::MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (C++03 § 18.4.1.2 ¶ 12, validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions
        unsafe { ffi::operator_delete_array(ptr.cast()) };
    }
}

// SAFETY: ✔️ default Realloc impl is soundly implemented in terms of Alloc+Free
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

#[test] fn fat_alignment()          { fat::test::alignment(NewDeleteArray) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(NewDeleteArray) }
#[test] fn fat_uninit()             { if !cfg!(target_os = "linux") { unsafe { fat::test::uninit_alloc_unsound(NewDeleteArray) } } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(NewDeleteArray) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_conservative(NewDeleteArray) }
