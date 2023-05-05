use std::alloc::{AllocError, Global, Layout};
use std::ptr::NonNull;

struct Min;
struct Max;



unsafe impl std::alloc::Allocator for Min {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> { todo!() }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) { todo!() }
}

unsafe impl std::alloc::Allocator for Max {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> { todo!() }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) { todo!() }

    // Provided methods
    fn allocate_zeroed( &self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> { todo!() }
    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> { todo!() }
    unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> { todo!() }
    unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> { todo!() }
    fn by_ref(&self) -> &Self where Self: Sized { todo!() }
}

fn a() {
    let _ : AllocError = AllocError;
    let _ : Global = Global;
}
