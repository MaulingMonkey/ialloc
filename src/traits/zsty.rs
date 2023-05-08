//! Rusty [ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts)-friendly allocator traits operating on [`Layout`]s
//!
//! These traits are meant to generally be auto-implemented in terms of [`nzst`], but ZST/[`Layout`]-friendly for ease of consumption.
//!
//! Mixing [`thin::Free`] and [`zsty`] is likely a bug - the former won't handle the dangling pointers the latter uses for 0-sized allocs.

use crate::*;

use core::alloc::Layout;
use core::fmt::Debug;
use core::mem::MaybeUninit;
#[cfg(doc)] use core::ptr::NonNull;



/// Allocation functions:<br>
/// <code>[alloc_uninit](Self::alloc_uninit)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[alloc_zeroed](Self::alloc_zeroed)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
pub unsafe trait Alloc {
    /// The maximum alignment this allocator is guaranteed to support.
    ///
    /// Allocators that also implement [`thin::Alloc`] are likely to have low values for this such as<br>
    /// <code>[Alignment]::[of](Alignment::of)::&lt;[usize]&gt;()</code> or
    /// <code>[Alignment]::[of](Alignment::of)::&lt;[max_align_t](https://en.cppreference.com/w/cpp/types/max_align_t)&gt;()</code>
    /// in the `4 ..= 16` range.
    ///
    /// While it should be *safe* to call [`alloc_uninit`](Self::alloc_uninit) or [`alloc_zeroed`](Self::alloc_zeroed)
    /// requesting an alignment larger than this, such calls are unlikely to return anything other than
    /// <code>[Err]\(...\)</code>.
    const MAX_ALIGN : Alignment = Alignment::MAX;

    type Error : Debug;

    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error>;

    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        let alloc = self.alloc_uninit(layout)?;
        let all = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), layout.size()) };
        all.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}

/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]&lt;\_&gt;, layout: [Layout])</code><br>
/// <br>
pub unsafe trait Free {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout);
}

/// Reallocation function:<br>
/// <code>[realloc_uninit](Self::realloc_uninit)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[realloc_zeroed](Self::realloc_zeroed)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
pub unsafe trait Realloc : Alloc + Free {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        if old_layout == new_layout { return Ok(ptr) }
        let alloc = self.alloc_uninit(new_layout)?;
        {
            let old : &    [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts    (ptr.as_ptr().cast(), old_layout.size()) };
            let new : &mut [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(),      new_layout.size()) };
            let n = old.len().min(new.len());
            new[..n].copy_from_slice(&old[..n]);
        }
        unsafe { self.free(ptr, old_layout) };
        Ok(alloc)
    }

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
