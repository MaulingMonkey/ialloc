//! Rusty [ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts)-friendly allocator traits operating on [`Layout`]s
//!
//! These traits are meant to generally be auto-implemented in terms of [`nzst`], but ZST/[`Layout`]-friendly for ease of consumption.
//!
//! Mixing [`thin::Free`] and [`zsty`] is likely a bug - the former won't handle the dangling pointers the latter uses for 0-sized allocs.

use crate::*;

use core::alloc::Layout;
use core::mem::MaybeUninit;



/// Allocation functions:<br>
/// <code>[alloc_uninit](Self::alloc_uninit)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[alloc_zeroed](Self::alloc_zeroed)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
pub unsafe trait Alloc {
    type Error;
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error>;
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error>;
}

/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]&lt;\_&gt;, layout: [Layout])</code><br>
/// <br>
pub trait Free {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout);
}

/// Reallocation function:<br>
/// <code>[realloc_uninit](Self::realloc_uninit)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> Result&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[realloc_zeroed](Self::realloc_zeroed)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> Result&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
pub unsafe trait Realloc : Alloc + Free {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error>;

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        let alloc = unsafe { self.realloc_uninit(ptr, old_layout, new_layout) }?;
        if old_layout.size() < new_layout.size() {
            let all             = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), new_layout.size()) };
            let (_copied, new)  = all.split_at_mut(old_layout.size());
            new.fill(MaybeUninit::new(0u8));
        }
        Ok(alloc.cast())
    }
}
