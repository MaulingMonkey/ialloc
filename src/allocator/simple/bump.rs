use crate::*;
use crate::meta::*;

use core::alloc::Layout;
use core::cell::Cell;
use core::fmt::{self, Debug, Formatter};
use core::mem::MaybeUninit;
use core::ptr::NonNull;



/// Bump-allocate from a slice of memory.
pub struct Bump<'a> {
    buffer: Cell<&'a mut [MaybeUninit<u8>]>,
    #[cfg(debug_assertions)] outstanding_allocs: Cell<usize>,
}

impl<'a> Debug for Bump<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[cfg(    debug_assertions )] return write!(f, "Bump {{ buffer: [...{} bytes], outstanding_allocs: {} }}", self.available(), self.outstanding_allocs.get());
        #[cfg(not(debug_assertions))] return write!(f, "Bump {{ buffer: [...{} bytes], outstanding_allocs: ?? }}", self.available());
    }
}

impl<'a> Bump<'a> {
    pub fn from_array<const N: usize>(array: &'a mut MaybeUninit<[MaybeUninit<u8>; N]>) -> Self {
        // supposedly safe per <https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element>
        let array : &mut [MaybeUninit<u8>; N] = unsafe { array.assume_init_mut() };
        Self::new(&mut array[..])
    }

    pub fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            buffer: Cell::new(buffer),
            #[cfg(debug_assertions)] outstanding_allocs: Cell::new(0),
        }
    }

    fn available(&self) -> usize {
        let b = self.buffer.take();
        let len = b.len();
        self.buffer.set(b);
        len
    }
}

impl<'a> Drop for Bump<'a> {
    fn drop(&mut self) {
        // As `'a` is likely nonstatic, this has a pretty serious chance of being a soundness bug - e.g. perhaps `ABox::into_inner` outlived `Bump`.
        // On the other hand, leaking via `ABox::leak` *is* sound... do we want a "this memory was intentionally leaked" hint perhaps?
        #[cfg(debug_assertions)] assert_eq!(0, self.outstanding_allocs.get(), "allocator::simple::Bump has outstanding allocations");
    }
}



// meta::*

impl<'a> Meta for Bump<'a> {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::MAX;
    const MAX_SIZE  : usize     = usize::MAX;
    const ZST_SUPPORTED : bool  = true;
}

impl ZstSupported for Bump<'_> {}

unsafe impl ZstInfalliable for Bump<'_> {}



// fat::*

unsafe impl<'a> fat::Alloc for Bump<'a> {
    fn alloc_uninit(&self, layout: Layout) -> Result<crate::AllocNN, Self::Error> {
        let align = layout.align();
        let size = layout.size();

        let _ = Alignment::from(layout); // XXX: possibly hint to compiler that layout is nonzero
        if size == 0 {
            #[cfg(debug_assertions)] self.outstanding_allocs.set(self.outstanding_allocs.get() + 1);
            return Ok(crate::util::nn::dangling(layout));
        }

        let buffer = self.buffer.take();
        let align_mask = align.wrapping_sub(1);
        let misalign = align_mask & (buffer.as_ptr() as usize);
        if misalign == 0 {
            if buffer.len() < size { // ≈ OOM
                self.buffer.set(buffer);
                Err(())
            } else {
                let (alloc, unalloc) = buffer.split_at_mut(size);

                #[cfg(debug_assertions)] self.outstanding_allocs.set(self.outstanding_allocs.get() + 1);
                self.buffer.set(unalloc);
                Ok(unsafe { NonNull::new_unchecked(alloc.as_mut_ptr()) })
            }
        } else {
            let skip = align - misalign;
            if skip.saturating_add(size) > buffer.len() {
                self.buffer.set(buffer);
                Err(())
            } else {
                let (alloc, unalloc) = buffer[skip..].split_at_mut(size);
                #[cfg(debug_assertions)] self.outstanding_allocs.set(self.outstanding_allocs.get() + 1);
                self.buffer.set(unalloc);
                Ok(unsafe { NonNull::new_unchecked(alloc.as_mut_ptr()) })
            }
        }
    }
}

unsafe impl<'a> fat::Free for Bump<'a> {
    unsafe fn free(&self, _ptr: crate::AllocNN, _layout: Layout) {
        #[cfg(debug_assertions)] self.outstanding_allocs.set(self.outstanding_allocs.get() - 1);
    }
}

unsafe impl<'a> fat::Realloc for Bump<'a> {
    // TODO: if an allocation was the last allocation, add a in-place fast path?
}

#[no_implicit_prelude] mod cleanroom {
    use crate::impls;
    use super::Bump;

    impls! {
        unsafe impl     core::alloc::GlobalAlloc for     Bump<'static> => ialloc::fat::Realloc;
        unsafe impl['o] core::alloc::GlobalAlloc for &'o Bump<'static> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl['a] core::alloc::Allocator(unstable 1.50) for Bump<'a> => ialloc::fat::Realloc;
    }
}



#[test] fn test_quick() {
    use crate::boxed::ABox;
    use std::dbg;

    let mut buffer : MaybeUninit<[_; 4096]> = MaybeUninit::uninit();
    let alloc = Bump::from_array(&mut buffer);
    let _u      = ABox::try_new_in(1u32, &alloc).unwrap();
    let _8_1    = ABox::try_new_in(2u8,  &alloc).unwrap();
    let _8_2    = ABox::try_new_in(3u8,  &alloc).unwrap();
    let _8_3    = ABox::try_new_in(4u8,  &alloc).unwrap();
    let _i      = ABox::try_new_in(5i32, &alloc).unwrap();
    let _b      = ABox::try_new_in(true, &alloc).unwrap();

    dbg!(&*_u as *const _);
    dbg!(&*_8_1 as *const _);
    dbg!(&*_8_2 as *const _);
    dbg!(&*_8_3 as *const _);
    dbg!(&*_i as *const _);
    dbg!(&*_b as *const _);

    dbg!(_u);
    dbg!(_8_1);
    dbg!(_8_2);
    dbg!(_8_3);
    dbg!(_i);
    dbg!(_b);

    while let Ok(_) = ABox::try_new_in(1u8, &alloc) {}
}



#[test] fn fat_alignment()          { fat::test::alignment(&Bump::from_array(&mut MaybeUninit::new([(); 131072].map(|_| MaybeUninit::<u8>::new(0xFF))))) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(&Bump::from_array(&mut MaybeUninit::new([(); 131072].map(|_| MaybeUninit::<u8>::new(0xFF))))) }
#[test] fn fat_uninit()             { unsafe { fat::test::uninit_alloc_unsound(&Bump::from_array(&mut MaybeUninit::new([(); 131072].map(|_| MaybeUninit::<u8>::new(0xFF))))) } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(&Bump::from_array(&mut MaybeUninit::new([(); 131072].map(|_| MaybeUninit::<u8>::new(0xFF))))) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(&Bump::from_array(&mut MaybeUninit::new([(); 131072].map(|_| MaybeUninit::<u8>::new(0xFF))))) }
