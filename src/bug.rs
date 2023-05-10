//! Bug reporting panics

use crate::Alignment;

use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



pub trait AsPtr : Copy                  { fn as_ptr(self) -> *mut c_void; }
impl AsPtr for *mut    MaybeUninit<u8>  { fn as_ptr(self) -> *mut c_void { self.cast() } }
impl AsPtr for *mut                u8   { fn as_ptr(self) -> *mut c_void { self.cast() } }
impl AsPtr for NonNull<MaybeUninit<u8>> { fn as_ptr(self) -> *mut c_void { self.as_ptr().cast() } }
impl AsPtr for NonNull<            u8 > { fn as_ptr(self) -> *mut c_void { self.as_ptr().cast() } }

/// Report bugs that indicate Undefined Behavior
pub mod ub {
    use super::*;

    #[track_caller] #[inline(never)] pub fn invalid_free_align_for_allocator(align: impl Into<usize>) -> ! {
        let align = align.into();
        if let Ok(align) = Alignment::try_from(align) {
            panic!("bug: undefined behavior: tried to free an allocation with alignment {align:?}, but that's not supported by the allocator");
        } else {
            panic!("bug: undefined behavior: tried to free an allocation with alignment {align:?}, but that's not supported by the allocator.  It's also not a power of two, which might hint at a corrupt Layout.");
        }
    }

    #[track_caller] #[inline(never)] pub fn invalid_ptr_for_allocator(ptr: impl AsPtr) -> ! {
        let ptr = ptr.as_ptr();
        panic!("bug: undefined behavior: {ptr:?} doesn't belong to this allocator");
    }

    #[track_caller] #[inline(never)] pub fn freed_ptr_for_allocator(ptr: impl AsPtr) -> ! {
        let ptr = ptr.as_ptr();
        panic!("bug: undefined behavior: {ptr:?} belongs to this allocator, but it was already freed");
    }

    #[track_caller] #[inline(never)] pub fn invalid_zst_for_allocator(ptr: impl AsPtr) -> ! {
        let ptr = ptr.as_ptr();
        panic!("bug: undefined behavior: {ptr:?} doesn't belong to this allocator (ZST, but ZSTs are never allocated by this allocator)");
    }

    #[track_caller] #[inline(never)] pub fn free_failed(ptr: impl AsPtr) -> ! {
        let ptr = ptr.as_ptr();
        panic!("bug: undefined behavior: freeing {ptr:?} failed (typically this means the pointer didn't belong to the allocator, or there was heap corruption)");
    }
}
