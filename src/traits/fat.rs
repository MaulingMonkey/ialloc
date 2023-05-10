//! Rusty [ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts)-friendly allocator traits operating on [`Layout`]s

use crate::*;

use core::alloc::Layout;
use core::mem::MaybeUninit;
#[cfg(doc)] use core::ptr::NonNull;



/// Allocation functions:<br>
/// <code>[alloc_uninit](Self::alloc_uninit)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[alloc_zeroed](Self::alloc_zeroed)(layout: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
///
/// ## Safety
/// *   Allocations created by this trait must be compatible with any other [`fat`] traits implemented on this allocator type.
/// *   Returned allocations must obey `layout` alignment and size.
pub unsafe trait Alloc : meta::Meta {
    /// Allocate at least `layout.size()` bytes of uninitialized memory aligned to `layout.align()`.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN, Self::Error>;

    /// Allocate at least `layout.size()` bytes of zeroed memory aligned to `layout.align()`.
    ///
    /// The resulting allocation can typically be freed with <code>[Free]::[free](Free::free)</code>
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> {
        let alloc = self.alloc_uninit(layout)?;
        // SAFETY: ⚠️ `alloc` is non-null by type, `align` is 1/trivial, `layout.size()` was just allocated, size <= isize::MAX by Layout
        let all = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), layout.size()) };
        all.fill(MaybeUninit::new(0u8));
        Ok(alloc.cast())
    }
}

/// Deallocation function:<br>
/// <code>[free](Self::free)(ptr: [NonNull]&lt;\_&gt;, layout: [Layout])</code><br>
/// <br>
///
/// ## Safety
/// *   This trait must be able to free allocations made by any other [`fat`] traits implemented on this allocator type.
pub unsafe trait Free : meta::Meta {
    /// Deallocate an allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after free
    /// *   `layout` must exactly match the [`Layout`] last used to successfully (re)allocate `ptr`
    unsafe fn free(&self, ptr: AllocNN, layout: Layout);
}

/// Reallocation function:<br>
/// <code>[realloc_uninit](Self::realloc_uninit)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <code>[realloc_zeroed](Self::realloc_zeroed)(ptr: [NonNull]&lt;\_&gt;, old: [Layout], new: [Layout]) -> [Result]&lt;[NonNull]&lt;\_&gt;, \_&gt;</code><br>
/// <br>
///
/// ## Safety
/// *   This trait must be able to reallocate allocations made by any other [`fat`] traits implemented on this allocator type.
/// *   Returned allocations must obey `new_layout` alignment and size.
pub unsafe trait Realloc : Alloc + Free {
    /// Reallocate an existing allocation, `ptr`, belonging to `self`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after a succesful realloc (`realloc_uninit` returns <code>[Ok]\(...\)</code>)
    /// *   `old_layout` must exactly match the [`Layout`] last used to successfully (re)allocate `ptr`
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        if old_layout == new_layout { return Ok(ptr) }
        let alloc = self.alloc_uninit(new_layout)?;
        {
            // SAFETY: ⚠️ `ptr` is non-null by type, `align` is 1/trivial, `old_layout.size()` was previously allocated by fn precondition, size <= isize::MAX implied by Layout
            let old : &    [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts    (ptr.as_ptr().cast(), old_layout.size()) };
            // SAFETY: ⚠️ `alloc` is non-null by type, `align` is 1/trivial, `new_layout.size()` was just allocated, size <= isize::MAX implied by Layout
            let new : &mut [MaybeUninit<u8>] = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(),      new_layout.size()) };
            let n = old.len().min(new.len());
            new[..n].copy_from_slice(&old[..n]);
        }
        // SAFETY: ✔️ (ptr, old_layout) was a previous valid alloc by fn safety precondition
        unsafe { self.free(ptr, old_layout) };
        Ok(alloc)
    }

    /// Reallocate an existing allocation, `ptr`, belonging to `self`.
    ///
    /// Any newly allocated memory will be zeroed.
    /// Any memory beyond `old_layout.size()` **may or may not** be zeroed.
    /// This might be *no* new memory, even if `new_layout.size()` > `old_layout.size()` - the implementation could've rounded both the old and new sizes up to the same value.
    /// As such, it's almost certainly a bug to use this to (re)allocate memory that came from <code>{,[re](Self::realloc_uninit)}[alloc_uninit](Alloc::alloc_uninit)</code>.
    /// Reallocate memory that came from <code>{,[re](Self::realloc_zeroed)}[alloc_zeroed](Alloc::alloc_zeroed)</code> instead.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self`
    /// *   `ptr` will no longer be accessible after a succesful realloc (`realloc_uninit` returns <code>[Ok]\(...\)</code>)
    /// *   `old_layout` must exactly match the [`Layout`] last used to successfully (re)allocate `ptr`
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ realloc_uninit has same prereqs as realloc_zeroed
        let alloc = unsafe { self.realloc_uninit(ptr, old_layout, new_layout) }?;
        if old_layout.size() < new_layout.size() {
            // SAFETY: ⚠️ `alloc` is non-null by type, `align` is 1/trivial, `new_layout.size()` was just allocated and implied <= isize::MAX by Layout
            let all             = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), new_layout.size()) };
            let (_copied, new)  = all.split_at_mut(old_layout.size());
            new.fill(MaybeUninit::new(0u8));
        }
        Ok(alloc.cast())
    }
}



#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Alloc> Alloc for &'a A {
    fn alloc_uninit(&self, layout: Layout) -> Result<AllocNN,  Self::Error> { A::alloc_uninit(self, layout) }
    fn alloc_zeroed(&self, layout: Layout) -> Result<AllocNN0, Self::Error> { A::alloc_zeroed(self, layout) }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Free> Free for &'a A {
    unsafe fn free(&self, ptr: AllocNN, layout: Layout) { unsafe { A::free(self, ptr, layout) } }
}

#[allow(clippy::undocumented_unsafe_blocks)] // SAFETY: ✔️ same trait, same prereqs
unsafe impl<'a, A: Realloc> Realloc for &'a A {
    unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { unsafe { A::realloc_uninit(self, ptr, old_layout, new_layout) } }
    unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: Layout, new_layout: Layout) -> Result<AllocNN, Self::Error> { unsafe { A::realloc_zeroed(self, ptr, old_layout, new_layout) } }
}
