use ialloc::{allocator::c::Malloc, boxed::ABox};
use core::mem::MaybeUninit;

fn main() {
    let a : ABox<[MaybeUninit<u32>], Malloc> = ABox::new_uninit_slice(0);
    assert_eq!(a.len(), 0);

    let a : ABox<[MaybeUninit<u32>], Malloc> = ABox::new_uninit_slice(32);
    assert_eq!(a.len(), 32);

    #[cfg(nope)]
    {
        #[repr(C, align(4096))] struct Page([u8; 4096]);
        let _ = ABox::new_in(Page([0u8; 4096]), Malloc);
    }
}
