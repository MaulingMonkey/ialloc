// #![feature(allocator_api)] // ≈ in `../malloc.rs` that `include!(...)`s this file

use ialloc::allocator::{adapt::PanicOverAlign, c::Malloc};

fn main() {
    let mut v = Vec::new_in(PanicOverAlign(Malloc));
    v.push(1);
    v.push(2);
    v.push(3);
    let v2 = v.clone();
    dbg!((v, v2));
}
