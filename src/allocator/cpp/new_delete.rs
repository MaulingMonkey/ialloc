use crate::*;
use crate::meta::*;
use super::ffi;

use core::ptr::NonNull;



/// [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) <br>
/// [`::operator delete(void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct NewDelete;



// meta::*

impl Meta for NewDelete {
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

// SAFETY: ✔️ global state only
unsafe impl Stateless for NewDelete {}



// thin::*

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `align`       | ✔️ Validated via [`thin::test::alignment`]
/// | `size`        | ✔️ Validated via [`thin::test::edge_case_sizes`]
/// | `pin`         | ✔️ [`NewDelete`] is `'static` - allocations by [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) live until [`::operator delete(void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete)ed.
/// | `compatible`  | ✔️ [`NewDelete`] uses exclusively intercompatible un-arrayed, implicit-alignment operators from the same stdlib.
/// | `compatible`  | ✔️ [`::operator delete(void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is compatible with [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) (C++03 § 18.4.1.1 ¶ 12)
/// | `exclusive`   | ✔️ [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) allocations are exclusive/unique
/// | `exceptions`  | ✔️ [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) is `throw()` and will return null on error (C++03 § 18.4.1.1 ¶ 5, or `noexcept` in C++11+)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
/// | `zeroed`      | ✔️ Validated via [`thin::test::zeroed_alloc`]
///
// SAFETY: per above
unsafe impl thin::Alloc for NewDelete {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: per above
        NonNull::new(unsafe { ffi::operator_new_nothrow(size) }.cast()).ok_or(())
    }
}

/// | Safety Item   | Description   |
/// | --------------| --------------|
/// | `compatible`  | ✔️ [`NewDelete`] uses exclusively intercompatible un-arrayed, implicit-alignment operators from the same stdlib.
/// | `compatible`  | ✔️ [`::operator delete(void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is compatible with [`::operator new(size_t, nothrow_t)`](https://en.cppreference.com/w/cpp/memory/new/operator_new) (C++03 § 18.4.1.1 ¶ 12)
/// | `exceptions`  | ✔️ [`::operator delete(void*)`](https://en.cppreference.com/w/cpp/memory/new/operator_delete) is `throw()` and returns no errors (C++03 § 18.4.1.1 ¶ 10) / `noexcept` (C++11+)
/// | `threads`     | ⚠️ thread-unsafe stdlibs existed once upon a time.  I consider linking them in a multithreaded program defacto undefined behavior beyond the scope of this to guard against.
///
// SAFETY: per above
unsafe impl thin::Free for NewDelete {
    unsafe fn free_nullable(&self, ptr: *mut core::mem::MaybeUninit<u8>) {
        // SAFETY: ✔️ `ptr` can be nullptr (C++03 § 18.4.1.1 ¶ 13, validated via [`thin::test::nullable`])
        // SAFETY: ✔️ `ptr` otherwise belongs to `self` per [`thin::Free::free_nullable`]'s documented safety preconditions
        unsafe { ffi::operator_delete(ptr.cast()) };
    }
}



// fat::*

// SAFETY: ✔️ default Realloc impl is soundly implemented in terms of Alloc+Free
unsafe impl fat::Realloc for NewDelete {}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, NewDelete};

    impls! {
        unsafe impl ialloc::fat::Alloc      for NewDelete => ialloc::thin::Alloc;
        unsafe impl ialloc::fat::Free       for NewDelete => ialloc::thin::Free;
    }
}



#[cfg(test)] pub(crate) const OPERATOR_NEW_ZERO_INITS : bool = cfg!(any(
    target_os = "linux",    // from the start of `ialloc` on CI and WSL
    target_os = "macos",    // github's `macos-11` runners didn't zero init, but `macos-14`(? via `macos-latest`) does.
));

#[test] fn thin_alignment()         { thin::test::alignment(NewDelete) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(NewDelete) }
#[test] fn thin_nullable()          { thin::test::nullable(NewDelete) }
#[test] fn thin_uninit()            { if !OPERATOR_NEW_ZERO_INITS { unsafe { thin::test::uninit_alloc_unsound(NewDelete) } } }
//test] fn thin_uninit_realloc()    { thin::test::uninit_realloc(NewDelete) } // does not implement thin::Realloc
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(NewDelete) }
//test] fn thin_zeroed_realloc()    { thin::test::zeroed_realloc(NewDelete) } // does not implement thin::Realloc
#[test] fn thin_zst_support()       { thin::test::zst_supported_conservative(NewDelete) }

#[test] fn fat_alignment()          { fat::test::alignment(NewDelete) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(NewDelete) }
#[test] fn fat_uninit()             { if !OPERATOR_NEW_ZERO_INITS { unsafe { fat::test::uninit_alloc_unsound(NewDelete) } } }
#[test] fn fat_uninit_realloc()     { fat::test::uninit_realloc(NewDelete) }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(NewDelete) }
#[test] fn fat_zeroed_realloc()     { fat::test::zeroed_realloc(NewDelete) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_conservative(NewDelete) }
