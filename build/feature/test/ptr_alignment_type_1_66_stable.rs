const _ : () = {
    use core::ptr::Alignment;
    use core::fmt::Debug;
    use core::hash::Hash;
    use core::num::NonZeroUsize;

    let a : Alignment = Alignment::MIN;

    const fn check_impl<A: Copy + Clone + Eq + PartialEq + Debug + Ord + PartialOrd + Hash + TryFrom<NonZeroUsize> + TryFrom<usize>>(_: &A) where NonZeroUsize : From<A>, usize : From<A> {}
    check_impl(&a);

    let a : Alignment = Alignment::of::<u32>();
    let a : Option<Alignment> = Alignment::new(1);
    let a : Alignment = unsafe { Alignment::new_unchecked(1) };
    let _ : usize = a.as_usize();
    let _ : NonZeroUsize = a.as_nonzero();
    let _ : u32 = a.log2();
};
