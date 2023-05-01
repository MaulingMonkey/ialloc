//! C/C++y allocator traits operating on thin pointers
//!
//! C and C++ allocators often merely accept a pointer for dealloc/realloc/size queries.
//! This module provides traits for such functionality.

use crate::*;

#[cfg(doc)] use core::mem::MaybeUninit;
#[cfg(doc)] use core::ptr::NonNull;



/// <code>[alloc_size](Self::alloc_size)(ptr: [NonNull]<[MaybeUninit]<[u8]>>) -> [Result]<[usize]></code>
///
/// ### Safety
/// It wouldn't be entirely unreasonable for an implementor to implement realloc in terms of this trait.
/// Such an implementor would generally rely on the `ptr[..a.alloc_size(ptr)]` being valid memory when `ptr` is a valid allocation owned by `a`.
/// By implementing this trait, you pinky promise that such a size is valid.
pub unsafe trait AllocSize {
    type Error : core::fmt::Debug;

    /// Attempt to retrieve the size of the allocation `ptr`, owned by `self`.
    ///
    /// ### Safety
    /// *   May exhibit UB if `ptr` is not an allocation belonging to `self`.
    /// *   Returns the allocation size, but some or all of the data in said allocation might be uninitialized.
    unsafe fn alloc_size(&self, ptr: AllocNN) -> Result<usize, Self::Error>;
}

// TODO: SafeAllocSize - like alloc_size, but a safe fn?



/// <code>[dealloc](Self::dealloc)(ptr: [NonNull]<[MaybeUninit]<[u8]>>)</code>
pub trait Free {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after dealloc
    unsafe fn dealloc(&self, ptr: AllocNN);
}

impl<A: thin::Free> nzst::Free for A {
    unsafe fn dealloc(&self, ptr: AllocNN, _layout: LayoutNZ) { unsafe { thin::Free::dealloc(self, ptr) } }
}
