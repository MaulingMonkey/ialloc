//! Macro implementation details.  These are supposed to be `#[doc(hidden)]` from view and not used directly.

pub mod prelude {
    pub use crate::{self as ialloc, Alignment, LayoutNZ, meta::Meta as _, meta, thin, fat};

    pub use core::prelude::rust_2021::*;
    pub use core::{assert, assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne};
    pub use core::primitive::*;

    pub use core::alloc::{Layout, *}; // AllocError (unstable)
    pub use core::mem::MaybeUninit;
    pub use core::num::NonZeroUsize;
    pub use core::ptr::{NonNull, null_mut, slice_from_raw_parts_mut};

    pub fn dangling<T>(layout: Layout) -> NonNull<T> { crate::util::nn::dangling(layout) }
}

/// Implement [`ialloc`](crate) (and/or [`core`]) traits in terms of other traits
#[macro_export] macro_rules! impls {
    () => {};



    // unsafe impl core::alloc::{...} for {...} => ialloc::fat::{...};

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::GlobalAlloc for $ty:ty => $(::)? ialloc::fat::Realloc; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? ::core::alloc::GlobalAlloc for $ty {
            #[track_caller] unsafe fn alloc(&self, layout: ::core::alloc::Layout) -> *mut ::core::primitive::u8 {
                use $crate::_impls::prelude::*;
                fat::Alloc::alloc_uninit(self, layout).map_or(null_mut(), |p| p.as_ptr().cast())
            }

            #[track_caller] unsafe fn alloc_zeroed(&self, layout: ::core::alloc::Layout) -> *mut ::core::primitive::u8 {
                use $crate::_impls::prelude::*;
                fat::Alloc::alloc_zeroed(self, layout).map_or(null_mut(), |p| p.as_ptr().cast())
            }

            #[track_caller] unsafe fn dealloc(&self, ptr: *mut u8, layout: ::core::alloc::Layout) {
                use $crate::_impls::prelude::*;
                NonNull::new(ptr).map(|ptr| unsafe { fat::Free::free(self, ptr.cast(), layout) });
            }

            #[track_caller] unsafe fn realloc(&self, ptr: *mut u8, old_layout: ::core::alloc::Layout, new_size: ::core::primitive::usize) -> *mut ::core::primitive::u8 {
                use $crate::_impls::prelude::*;
                let Some(ptr) = NonNull::new(ptr) else { return null_mut() };
                let Ok(new_layout) = Layout::from_size_align(new_size, old_layout.align()) else { return null_mut() };
                unsafe { fat::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_or(null_mut(), |p| p.as_ptr().cast())
            }
        }

        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::Allocator(unstable $(1.50$(.0)?)?) for $ty:ty => $(::)? ialloc::fat::Realloc; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? ::core::alloc::Allocator for $ty {
            #[track_caller] fn allocate(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> {
                use $crate::_impls::prelude::*;
                let alloc = fat::Alloc::alloc_uninit(self, layout).map_err(|_| AllocError)?;
                NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), layout.size())).ok_or(AllocError)
            }

            #[track_caller] fn allocate_zeroed(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> {
                use $crate::_impls::prelude::*;
                let alloc = fat::Alloc::alloc_zeroed(self, layout).map_err(|_| AllocError)?;
                NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), layout.size())).ok_or(AllocError)
            }

            #[track_caller] unsafe fn deallocate(&self, ptr: ::core::ptr::NonNull<::core::primitive::u8>, layout: ::core::alloc::Layout) {
                use $crate::_impls::prelude::*;
                unsafe { fat::Free::free(self, ptr.cast(), layout) }
            }

            #[track_caller] unsafe fn grow(&self, ptr: ::core::ptr::NonNull<::core::primitive::u8>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> {
                use $crate::_impls::prelude::*;
                let alloc = unsafe { fat::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
                NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), new_layout.size())).ok_or(AllocError)
            }

            #[track_caller] unsafe fn grow_zeroed(&self, ptr: ::core::ptr::NonNull<::core::primitive::u8>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> {
                use $crate::_impls::prelude::*;
                let alloc = unsafe { fat::Realloc::realloc_zeroed(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
                NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), new_layout.size())).ok_or(AllocError)
            }

            #[track_caller] unsafe fn shrink(&self, ptr: ::core::ptr::NonNull<::core::primitive::u8>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> {
                use $crate::_impls::prelude::*;
                let alloc = unsafe { fat::Realloc::realloc_uninit(self, ptr.cast(), old_layout, new_layout) }.map_err(|_| AllocError)?;
                NonNull::new(slice_from_raw_parts_mut(alloc.as_ptr().cast(), new_layout.size())).ok_or(AllocError)
            }
        }

        $crate::impls!($($tt)*);
    };



    // unsafe impl ialloc::fat::{...} for {...} => ialloc::thin::{...};

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::fat::Alloc for $ty:ty $(where [$($where:tt)*])? => $(::)? ialloc::thin::Alloc; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::fat::Alloc for $ty $(where $($where)*)? {
            fn alloc_uninit(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
                use $crate::_impls::prelude::*;
                if layout.align() > Self::MAX_ALIGN.as_usize() { Err($crate::error::ExcessiveAlignmentRequestedError { requested: Alignment::new(layout.align()).unwrap_or(Alignment::MAX), supported: Self::MAX_ALIGN })? }
                $crate::thin::Alloc::alloc_uninit(self, layout.size())
            }
            fn alloc_zeroed(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::primitive::u8>, Self::Error> {
                use $crate::_impls::prelude::*;
                if layout.align() > Self::MAX_ALIGN.as_usize() { Err($crate::error::ExcessiveAlignmentRequestedError { requested: Alignment::new(layout.align()).unwrap_or(Alignment::MAX), supported: Self::MAX_ALIGN })? }
                $crate::thin::Alloc::alloc_zeroed(self, layout.size())
            }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::fat::Free for $ty:ty $(where [$($where:tt)*])? => $(::)? ialloc::thin::Free; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::fat::Free for $ty $(where $($where)*)? {
            unsafe fn free(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, _layout: ::core::alloc::Layout) {
                use $crate::_impls::prelude::*;
                debug_assert!(_layout.align() <= Self::MAX_ALIGN.as_usize(), "allocation couldn't belong to this allocator: impossible alignment");
                unsafe { $crate::thin::Free::free(self, ptr) }
            }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::fat::Realloc for $ty:ty $(where [$($where:tt)*])? => $(::)? ialloc::thin::Realloc; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::fat::Realloc for $ty $(where $($where)*)? {
            unsafe fn realloc_uninit(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, _old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
                use $crate::_impls::prelude::*;
                debug_assert!(_old_layout.align() <= Self::MAX_ALIGN.as_usize(), "allocation couldn't belong to this allocator: impossible alignment");
                if new_layout.align() > Self::MAX_ALIGN.as_usize() { Err($crate::error::ExcessiveAlignmentRequestedError { requested: Alignment::new(new_layout.align()).unwrap_or(Alignment::MAX), supported: Self::MAX_ALIGN })? }
                unsafe { $crate::thin::Realloc::realloc_uninit(self, ptr, new_layout.size()) }
            }
            unsafe fn realloc_zeroed(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> {
                use $crate::_impls::prelude::*;
                debug_assert!(old_layout.align() <= Self::MAX_ALIGN.as_usize(), "allocation couldn't belong to this allocator: impossible alignment");
                if new_layout.align() > Self::MAX_ALIGN.as_usize() { Err($crate::error::ExcessiveAlignmentRequestedError { requested: Alignment::new(new_layout.align()).unwrap_or(Alignment::MAX), supported: Self::MAX_ALIGN })? }
                if <$ty as $crate::thin::Realloc>::CAN_REALLOC_ZEROED {
                    unsafe { $crate::thin::Realloc::realloc_zeroed(self, ptr, new_layout.size()) }
                } else {
                    let alloc = unsafe { $crate::thin::Realloc::realloc_uninit(self, ptr, new_layout.size())? };
                    if old_layout.size() < new_layout.size() {
                        let all             = unsafe { ::core::slice::from_raw_parts_mut(alloc.as_ptr(), new_layout.size()) };
                        let (_copied, new)  = all.split_at_mut(old_layout.size());
                        new.fill(::core::mem::MaybeUninit::new(0u8));
                    }
                    Ok(alloc.cast())
                }
            }
        }
        $crate::impls!($($tt)*);
    };



    // unsafe impl {...} for {...} => core::ops::Deref;

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::Alloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::thin::Alloc for $ty {
            #[inline(always)] #[track_caller] fn alloc_uninit(&self, size: ::core::primitive::usize) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> { $crate::thin::Alloc::alloc_uninit(&**self, size) }
            #[inline(always)] #[track_caller] fn alloc_zeroed(&self, size: ::core::primitive::usize) -> ::core::result::Result<::core::ptr::NonNull<                         ::core::primitive::u8 >, Self::Error> { $crate::thin::Alloc::alloc_zeroed(&**self, size) }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::Free for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::thin::Free for $ty {
            #[inline(always)] #[track_caller] unsafe fn free(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>) { unsafe { $crate::thin::Free::free(&**self, ptr) } }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::Realloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::thin::Realloc for $ty {
            const CAN_REALLOC_ZEROED : ::core::primitive::bool = <<$ty as ::core::ops::Deref>::Target as $crate::thin::Realloc>::CAN_REALLOC_ZEROED;
            #[inline(always)] #[track_caller] unsafe fn realloc_uninit(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, new_size: ::core::primitive::usize) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> { unsafe { $crate::thin::Realloc::realloc_uninit(&**self, ptr, new_size) } }
            #[inline(always)] #[track_caller] unsafe fn realloc_zeroed(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, new_size: ::core::primitive::usize) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> { unsafe { $crate::thin::Realloc::realloc_zeroed(&**self, ptr, new_size) } }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::SizeOf for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::thin::SizeOf for $ty {
            #[inline(always)] #[track_caller] unsafe fn size_of(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>) -> ::core::option::Option<::core::primitive::usize> { unsafe { $crate::thin::SizeOf::size_of(&**self, ptr) } }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::thin::SizeOfDebug for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::thin::SizeOfDebug for $ty {
            #[inline(always)] #[track_caller] unsafe fn size_of(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>) -> ::core::option::Option<::core::primitive::usize> { unsafe { $crate::thin::SizeOfDebug::size_of(&**self, ptr) } }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::fat::Alloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::fat::Alloc for $ty {
            #[inline(always)] #[track_caller] fn alloc_uninit(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> { $crate::fat::Alloc::alloc_uninit(&**self, layout) }
            #[inline(always)] #[track_caller] fn alloc_zeroed(&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<                         ::core::primitive::u8 >, Self::Error> { $crate::fat::Alloc::alloc_zeroed(&**self, layout) }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::fat::Free for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::fat::Free for $ty {
            #[inline(always)] #[track_caller] unsafe fn free(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, layout: ::core::alloc::Layout) { unsafe { $crate::fat::Free::free(&**self, ptr, layout) } }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? ialloc::fat::Realloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? $crate::fat::Realloc for $ty {
            #[inline(always)] #[track_caller] unsafe fn realloc_uninit(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> { unsafe { $crate::fat::Realloc::realloc_uninit(&**self, ptr, old_layout, new_layout) } }
            #[inline(always)] #[track_caller] unsafe fn realloc_zeroed(&self, ptr: ::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<::core::mem::MaybeUninit<::core::primitive::u8>>, Self::Error> { unsafe { $crate::fat::Realloc::realloc_zeroed(&**self, ptr, old_layout, new_layout) } }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::GlobalAlloc for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? ::core::alloc::GlobalAlloc for $ty {
            #[inline(always)] #[track_caller] unsafe fn alloc           (&self, layout: ::core::alloc::Layout) -> *mut u8 { unsafe { ::core::alloc::GlobalAlloc::alloc(&**self, layout) } }
            #[inline(always)] #[track_caller] unsafe fn alloc_zeroed    (&self, layout: ::core::alloc::Layout) -> *mut u8 { unsafe { ::core::alloc::GlobalAlloc::alloc_zeroed(&**self, layout) } }
            #[inline(always)] #[track_caller] unsafe fn dealloc         (&self, ptr: *mut ::core::primitive::u8, layout: ::core::alloc::Layout) { unsafe { ::core::alloc::GlobalAlloc::dealloc(&**self, ptr, layout) } }
            #[inline(always)] #[track_caller] unsafe fn realloc         (&self, ptr: *mut ::core::primitive::u8, old_layout: ::core::alloc::Layout, new_size: ::core::primitive::usize) -> *mut u8 { unsafe { ::core::alloc::GlobalAlloc::realloc(&**self, ptr, old_layout, new_size) } }
        }
        $crate::impls!($($tt)*);
    };

    ( unsafe impl $([$($gdef:tt)*])? $(::)? core::alloc::Allocator(unstable $(1.50$(.0)?)?) for $ty:ty => $(::)? core::ops::Deref; $($tt:tt)* ) => {
        unsafe impl $(<$($gdef)*>)? ::core::alloc::Allocator for $ty {
            #[inline(always)] #[track_caller] fn allocate           (&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> { ::core::alloc::Allocator::allocate(&**self, layout) }
            #[inline(always)] #[track_caller] fn allocate_zeroed    (&self, layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> { ::core::alloc::Allocator::allocate_zeroed(&**self, layout) }
            #[inline(always)] #[track_caller] unsafe fn deallocate  (&self, ptr: ::core::ptr::NonNull<u8>, layout: ::core::alloc::Layout) { unsafe { ::core::alloc::Allocator::deallocate(&**self, ptr, layout) } }
            #[inline(always)] #[track_caller] unsafe fn grow        (&self, ptr: ::core::ptr::NonNull<u8>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> { unsafe { ::core::alloc::Allocator::grow(&**self, ptr, old_layout, new_layout) } }
            #[inline(always)] #[track_caller] unsafe fn grow_zeroed (&self, ptr: ::core::ptr::NonNull<u8>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> { unsafe { ::core::alloc::Allocator::grow_zeroed(&**self, ptr, old_layout, new_layout) } }
            #[inline(always)] #[track_caller] unsafe fn shrink      (&self, ptr: ::core::ptr::NonNull<u8>, old_layout: ::core::alloc::Layout, new_layout: ::core::alloc::Layout) -> ::core::result::Result<::core::ptr::NonNull<[::core::primitive::u8]>, ::core::alloc::AllocError> { unsafe { ::core::alloc::Allocator::shrink(&**self, ptr, old_layout, new_layout) } }
        }
        $crate::impls!($($tt)*);
    };
}
