//! Macro implementation details.  These are supposed to be `#[doc(hidden)]` from view and not used directly.

pub mod prelude {
    pub use crate as ialloc;
    pub use core;

    pub use crate::{Alignment, LayoutNZ, nzst, thin, zsty};
    pub use core::alloc::{Layout, *}; // AllocError (unstable)
    pub use core::convert::TryFrom;
    pub use core::mem::MaybeUninit;
    pub use core::num::NonZeroUsize;
    pub use core::option::{Option::{self, Some, None}};
    pub use core::ptr::{NonNull, null_mut, slice_from_raw_parts_mut};
    pub use core::primitive::{u8, usize};
    pub use core::result::{Result::{self, Ok, Err}};
}

/// Implement [`ialloc`](crate) (and/or [`core`]) traits in terms of other traits
#[macro_export] macro_rules! impls {
    () => {};

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::GlobalAlloc for $ty:ty => $(::)? ialloc::zsty::Realloc; $($tt:tt)* ) => {

        const _ : () = {
            use $crate::_impls::prelude::*;
            use zsty::*;

            unsafe impl $(<$($gdef)*>)? core::alloc::GlobalAlloc for $ty {
                #[track_caller] unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                    Alloc::alloc_uninit(self, layout).map_or(null_mut(), |p| p.as_ptr().cast())
                }

                #[track_caller] unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
                    Alloc::alloc_zeroed(self, layout).map_or(null_mut(), |p| p.as_ptr().cast())
                }

                #[track_caller] unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                    NonNull::new(ptr).map(|ptr| unsafe { Free::free(self, ptr.cast(), layout) });
                }

                #[track_caller] unsafe fn realloc(&self, ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
                    let Some(ptr) = NonNull::new(ptr) else { return null_mut() };
                    let Ok(new_layout) = Layout::from_size_align(new_size, old_layout.align()) else { return null_mut() };
                    unsafe { Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_or(null_mut(), |p| p.as_ptr().cast())
                }
            }
        };

        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::Allocator(unstable $(1.50$(.0)?)?) for $ty:ty => $(::)? ialloc::zsty::Realloc; $($tt:tt)* ) => {

        const _ : () = {
            use $crate::_impls::prelude::*;
            use zsty::*;

            unsafe impl $(<$($gdef)*>)? core::alloc::Allocator for $ty {
                #[track_caller] fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
                    let alloc = Alloc::alloc_uninit(self, layout).map_err(|_| AllocError)?;
                    NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), layout.size())).ok_or(AllocError)
                }

                #[track_caller] fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
                    let alloc = Alloc::alloc_zeroed(self, layout).map_err(|_| AllocError)?;
                    NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), layout.size())).ok_or(AllocError)
                }

                #[track_caller] unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
                    unsafe { Free::free(self, ptr.cast(), layout) }
                }

                #[track_caller] unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
                    let alloc = unsafe { Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
                    NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), new_layout.size())).ok_or(AllocError)
                }

                #[track_caller] unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
                    let alloc = unsafe { Realloc::realloc_zeroed(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
                    NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), new_layout.size())).ok_or(AllocError)
                }

                #[track_caller] unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
                    let alloc = unsafe { Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
                    NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), new_layout.size())).ok_or(AllocError)
                }
            }
        };

        $crate::impls!($($tt)*);
    };



    // ... => core::ops::Deref

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::Alloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? thin::Alloc for $ty {
                type Error = <<$ty as core::ops::Deref>::Target as thin::Alloc>::Error;
                #[inline(always)] #[track_caller] fn alloc_uninit(&self, size: NonZeroUsize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { thin::Alloc::alloc_uninit(&**self, size) }
                #[inline(always)] #[track_caller] fn alloc_zeroed(&self, size: NonZeroUsize) -> Result<NonNull<            u8 >, Self::Error> { thin::Alloc::alloc_zeroed(&**self, size) }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::Free for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? thin::Free for $ty {
                #[inline(always)] #[track_caller] unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>) { unsafe { thin::Free::free(&**self, ptr) } }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::Realloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? thin::Realloc for $ty {
                const CAN_REALLOC_ZEROED : bool = <$ty as thin::Realloc>::CAN_REALLOC_ZEROED;
                #[inline(always)] #[track_caller] unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: NonZeroUsize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { thin::Realloc::realloc_uninit(&**self, ptr, new_size) } }
                #[inline(always)] #[track_caller] unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, new_size: NonZeroUsize) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { thin::Realloc::realloc_zeroed(&**self, ptr, new_size) } }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::SizeOf for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? thin::SizeOf for $ty {
                #[inline(always)] #[track_caller] unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> { unsafe { thin::SizeOf::size_of(&**self, ptr) } }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::SizeOfDebug for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? thin::SizeOfDebug for $ty {
                #[inline(always)] #[track_caller] unsafe fn size_of(&self, ptr: NonNull<MaybeUninit<u8>>) -> Option<usize> { unsafe { thin::SizeOfDebug::size_of(&**self, ptr) } }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::nzst::Alloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? nzst::Alloc for $ty {
                type Error = <<$ty as core::ops::Deref>::Target as nzst::Alloc>::Error;
                #[inline(always)] #[track_caller] fn alloc_uninit(&self, layout: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { nzst::Alloc::alloc_uninit(&**self, layout) }
                #[inline(always)] #[track_caller] fn alloc_zeroed(&self, layout: LayoutNZ) -> Result<NonNull<            u8 >, Self::Error> { nzst::Alloc::alloc_zeroed(&**self, layout) }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::nzst::Free for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? nzst::Free for $ty {
                #[inline(always)] #[track_caller] unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, layout: LayoutNZ) { unsafe { nzst::Free::free(&**self, ptr, layout) } }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::nzst::Realloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? nzst::Realloc for $ty {
                #[inline(always)] #[track_caller]  unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { nzst::Realloc::realloc_uninit(&**self, ptr, old_layout, new_layout) } }
                #[inline(always)] #[track_caller]  unsafe fn realloc_zeroed(&self, ptr: NonNull<MaybeUninit<u8>>, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> { unsafe { nzst::Realloc::realloc_zeroed(&**self, ptr, old_layout, new_layout) } }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::GlobalAlloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? core::alloc::GlobalAlloc for $ty {
                #[inline(always)] #[track_caller]  unsafe fn alloc         (&self, layout: Layout) -> *mut u8 { unsafe { core::alloc::GlobalAlloc::alloc(&**self, layout) } }
                #[inline(always)] #[track_caller]  unsafe fn alloc_zeroed  (&self, layout: Layout) -> *mut u8 { unsafe { core::alloc::GlobalAlloc::alloc_zeroed(&**self, layout) } }
                #[inline(always)] #[track_caller]  unsafe fn dealloc       (&self, ptr: *mut u8, layout: Layout) { unsafe { core::alloc::GlobalAlloc::dealloc(&**self, ptr, layout) } }
                #[inline(always)] #[track_caller]  unsafe fn realloc       (&self, ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 { unsafe { core::alloc::GlobalAlloc::realloc(&**self, ptr, old_layout, new_size) } }
            }
        };
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::Allocator(unstable $(1.50$(.0)?)?) for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        const _ : () = {
            use $crate::_impls::prelude::*;
            unsafe impl $(<$($gdef)*>)? core::alloc::Allocator for $ty {
                #[inline(always)] #[track_caller]  fn allocate             (&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> { core::alloc::Allocator::allocate(&**self, layout) }
                #[inline(always)] #[track_caller]  fn allocate_zeroed      (&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> { core::alloc::Allocator::allocate_zeroed(&**self, layout) }
                #[inline(always)] #[track_caller]  unsafe fn deallocate    (&self, ptr: NonNull<u8>, layout: Layout) { unsafe { core::alloc::Allocator::deallocate(&**self, ptr, layout) } }
                #[inline(always)] #[track_caller]  unsafe fn grow          (&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> { unsafe { core::alloc::Allocator::grow(&**self, ptr, old_layout, new_layout) } }
                #[inline(always)] #[track_caller]  unsafe fn grow_zeroed   (&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> { unsafe { core::alloc::Allocator::grow_zeroed(&**self, ptr, old_layout, new_layout) } }
                #[inline(always)] #[track_caller]  unsafe fn shrink        (&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> { unsafe { core::alloc::Allocator::shrink(&**self, ptr, old_layout, new_layout) } }
            }
        };
        $crate::impls!($($tt)*);
    };
}
