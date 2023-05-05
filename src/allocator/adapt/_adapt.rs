//! [`PanicOverAlign`]

use crate::{*, Alignment};

use core::mem::MaybeUninit;
use core::num::NonZeroUsize;



/// Adapt a [`thin`] allocator to a wider interface, `panic!`ing if more than [`thin::Alloc::MAX_ALIGN`] is requested.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct PanicOverAlign<A>(pub A);

impl<A> core::ops::Deref for PanicOverAlign<A> {
    type Target = A;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<A: thin::Alloc> PanicOverAlign<A> {
    #[track_caller] fn layout_to_size(layout: LayoutNZ) -> NonZeroUsize {
        let align = layout.align();
        if align > A::MAX_ALIGN { Self::invalid_alignment(layout.align()) }
        layout.size().max(align.as_nonzero())
    }

    #[inline(never)] #[track_caller] fn invalid_alignment(align: Alignment) -> ! {
        panic!("alignment {align:?} > Self::MAX_ALIGN ({:?})", A::MAX_ALIGN)
    }
}



unsafe impl<A: thin::Alloc> nzst::Alloc for PanicOverAlign<A> {
    type Error = A::Error;
    #[track_caller] fn alloc_uninit(&self, layout: LayoutNZ) -> Result<AllocNN,  Self::Error> { self.0.alloc_uninit(Self::layout_to_size(layout)) }
    #[track_caller] fn alloc_zeroed(&self, layout: LayoutNZ) -> Result<AllocNN0, Self::Error> { self.0.alloc_zeroed(Self::layout_to_size(layout)) }
}

unsafe impl<A: thin::Alloc + thin::Free> nzst::Free for PanicOverAlign<A> {
    #[track_caller] unsafe fn free(&self, ptr: AllocNN, layout: LayoutNZ) {
        let _ = Self::layout_to_size(layout); // if this fails, we never could've allocated this allocation
        unsafe { self.0.free(ptr) }
    }
}

unsafe impl<A: thin::Realloc> nzst::Realloc for PanicOverAlign<A> {
    #[track_caller] unsafe fn realloc_uninit(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        let _ = Self::layout_to_size(old_layout);
        let new_size = Self::layout_to_size(new_layout);
        unsafe { self.0.realloc_uninit(ptr, new_size) }
    }

    #[track_caller] unsafe fn realloc_zeroed(&self, ptr: AllocNN, old_layout: LayoutNZ, new_layout: LayoutNZ) -> Result<AllocNN, Self::Error> {
        if A::CAN_REALLOC_ZEROED {
            let _ = Self::layout_to_size(old_layout);
            let new_size = Self::layout_to_size(new_layout);
            unsafe { self.0.realloc_zeroed(ptr, new_size) }
        } else {
            let alloc = unsafe { self.realloc_uninit(ptr, old_layout, new_layout) }?;
            if old_layout.size() < new_layout.size() {
                let all             = unsafe { core::slice::from_raw_parts_mut(alloc.as_ptr(), new_layout.size().get()) };
                let (_copied, new)  = all.split_at_mut(old_layout.size().get());
                new.fill(MaybeUninit::new(0u8));
            }
            Ok(alloc.cast())
        }
    }
}

#[no_implicit_prelude] mod cleanroom {
    use super::{impls, PanicOverAlign};

    impls! {
        unsafe impl[A: thin::Realloc        ] core::alloc::GlobalAlloc  for PanicOverAlign<A> => ialloc::zsty::Realloc;

        unsafe impl[A: thin::Alloc          ] ialloc::thin::Alloc       for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::Free           ] ialloc::thin::Free        for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::Realloc        ] ialloc::thin::Realloc     for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::SizeOf         ] ialloc::thin::SizeOf      for PanicOverAlign<A> => core::ops::Deref;
        unsafe impl[A: thin::SizeOfDebug    ] ialloc::thin::SizeOfDebug for PanicOverAlign<A> => core::ops::Deref;
    }

    #[cfg(allocator_api = "1.50")] impls! {
        unsafe impl[A: thin::Realloc        ] core::alloc::Allocator(unstable 1.50) for PanicOverAlign<A> => ialloc::zsty::Realloc;
    }
}
