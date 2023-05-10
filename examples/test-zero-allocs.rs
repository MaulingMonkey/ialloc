#[cfg(not(c89))] fn main() { println!("missing feature = \"c89\"") }
#[cfg(    c89 )] fn main() {
    use ialloc::{allocator::{adapt::DangleZst, c::Malloc}, boxed::ABox};
    use core::mem::MaybeUninit;

    let a : ABox<[MaybeUninit<u32>], DangleZst<Malloc>> = ABox::new_uninit_slice(0);
    assert_eq!(a.len(), 0);

    let a : ABox<[MaybeUninit<u32>], DangleZst<Malloc>> = ABox::new_uninit_slice(32);
    assert_eq!(a.len(), 32);

    #[cfg(nope)]
    {
        #[repr(C, align(4096))] struct Page([u8; 4096]);
        let _ = ABox::new_in(Page([0u8; 4096]), DangleZst(Malloc));
    }
}
