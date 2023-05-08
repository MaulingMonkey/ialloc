use crate::zsty;

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

unsafe impl<'a> zsty::Alloc for Bump<'a> {
    type Error = ();

    fn alloc_uninit(&self, layout: Layout) -> Result<crate::AllocNN, Self::Error> {
        let align = layout.align();
        let size = layout.size();

        if align == 0 { unsafe { core::hint::unreachable_unchecked() } } // violation of Layout
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

unsafe impl<'a> zsty::Free for Bump<'a> {
    unsafe fn free(&self, _ptr: crate::AllocNN, _layout: Layout) {
        #[cfg(debug_assertions)] self.outstanding_allocs.set(self.outstanding_allocs.get() - 1);
    }
}

unsafe impl<'a> zsty::Realloc for Bump<'a> {
    // TODO: if an allocation was the last allocation, add a in-place fast path?
}

#[no_implicit_prelude] mod cleanroom {
    use crate::impls;
    use super::Bump;

    impls! {
        unsafe impl     core::alloc::GlobalAlloc for     Bump<'static> => ialloc::zsty::Realloc;
        unsafe impl['o] core::alloc::GlobalAlloc for &'o Bump<'static> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl['a] core::alloc::Allocator(unstable 1.50) for Bump<'a> => ialloc::zsty::Realloc;
        //unsafe impl['o, 'i: 'o] core::alloc::Allocator(unstable 1.50) for &'o Bump<'i>  => core::ops::Deref; // XXX: already auto-implemented
    }
}



#[test] fn test_quick() {
    use crate::boxed::ABox;
    use std::dbg;

    let mut buffer : MaybeUninit<[_; 4096]> = MaybeUninit::uninit();
    let alloc = Bump::from_array(&mut buffer);
    let _u      = ABox::new_in(1u32, &alloc);
    let _8_1    = ABox::new_in(2u8,  &alloc);
    let _8_2    = ABox::new_in(3u8,  &alloc);
    let _8_3    = ABox::new_in(4u8,  &alloc);
    let _i      = ABox::new_in(5i32, &alloc);
    let _b      = ABox::new_in(true, &alloc);

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
