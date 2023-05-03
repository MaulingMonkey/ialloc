use ialloc::*;

use core::alloc::AllocError;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



fn main() {
    let mut v = Vec::new_in(Malloc);
    v.push(1);
    v.push(2);
    v.push(3);
    let v2 = v.clone();
    dbg!((v, v2));
}



// TODO: exile into a crate
/// malloc / realloc / free
#[derive(Default, Clone, Copy)] struct Malloc;
impl Malloc {
    // TODO: more accurately determine malloc's alignment/size guarantees
    pub const MAX_ALIGN : Alignment     = ALIGN_4;
    pub const MAX_SIZE  : NonZeroUsize  = NonZeroUsize::new(usize::MAX/2).unwrap();
}
unsafe impl nzst::Alloc for Malloc {
    type Error = AllocError;
    fn alloc_uninit(&self, layout: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        assert!(layout.size()  <= Self::MAX_SIZE,  "requested allocation beyond malloc's capabilities");
        assert!(layout.align() <= Self::MAX_ALIGN, "requested allocation beyond malloc's capabilities");

        let size = layout.size().get().next_multiple_of(layout.align().as_usize());
        let alloc = unsafe { libc::malloc(size) };
        NonNull::new(alloc.cast()).ok_or(AllocError)
    }
}
unsafe impl nzst::Realloc for Malloc {
    unsafe fn realloc_uninit(&self, ptr: NonNull<MaybeUninit<u8>>, old: LayoutNZ, new: LayoutNZ) -> Result<NonNull<MaybeUninit<u8>>, Self::Error> {
        assert!(old.size()  <= Self::MAX_SIZE,  "this allocation couldn't have belonged to this allocator, has too much alignment");
        assert!(old.align() <= Self::MAX_ALIGN, "this allocation couldn't have belonged to this allocator, has too much alignment");
        assert!(new.size()  <= Self::MAX_SIZE,  "requested reallocation beyond malloc's capabilities");
        assert!(new.align() <= Self::MAX_ALIGN, "requested reallocation beyond malloc's capabilities");

        let size = new.size().get().next_multiple_of(new.align().as_usize());
        let alloc = unsafe { libc::realloc(ptr.as_ptr().cast(), size) };
        NonNull::new(alloc.cast()).ok_or(AllocError)
    }
}
unsafe impl nzst::Free for Malloc {
    unsafe fn free(&self, ptr: NonNull<MaybeUninit<u8>>, _layout: LayoutNZ) {
        assert!(_layout.size()  <= Self::MAX_SIZE,  "this allocation couldn't have belonged to this allocator, has too much alignment");
        assert!(_layout.align() <= Self::MAX_ALIGN, "this allocation couldn't have belonged to this allocator, has too much alignment");

        unsafe { libc::free(ptr.as_ptr().cast()) }
    }
}

#[no_implicit_prelude] mod cleanroom { // verify `impls!` is hygenic
    ::ialloc::impls! {
        unsafe impl core::alloc::GlobalAlloc         for super::Malloc => ialloc::nzst::Realloc;
        unsafe impl core::alloc::Allocator(unstable) for super::Malloc => ialloc::zsty::Realloc;
    }
}
