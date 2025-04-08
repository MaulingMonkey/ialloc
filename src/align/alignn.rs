use core::mem::MaybeUninit;

/// Wrap a value in constant-driven alignment.
///
/// `A` must generally be a power of 2 less than 1 GiB (`rustc` doesn't like `#[repr(align(1 GiB+))]` at this time.)
///
/// Note that the size will typically be rounded up to the nearest multiple of `A` (which might be `0` if `T` is a ZST.)
#[derive(Clone, Copy, Debug)] #[repr(C)] pub struct AlignN<const A : usize, T> where [(); A] : ValidAlignLessThan1GiB {
    align: MaybeUninit<<[(); A] as ValidAlignLessThan1GiB>::Align>,
    value: T,
}

#[allow(dead_code)] impl<const A : usize, T> AlignN<A, T> where [(); A] : _align_impl::ValidAlignLessThan1GiB {
    pub const fn new(value: T) -> Self { Self { value, align: MaybeUninit::uninit() } }
    pub fn into_inner(self) -> T { self.value }
}

impl<const A : usize, T> core::ops::Deref for AlignN<A, T> where [(); A] : _align_impl::ValidAlignLessThan1GiB {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.value }
}

impl<const A : usize, T> core::ops::DerefMut for AlignN<A, T> where [(); A] : _align_impl::ValidAlignLessThan1GiB {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.value }
}

impl<const A : usize, T: Default> Default for AlignN<A, T> where [(); A] : _align_impl::ValidAlignLessThan1GiB {
    fn default() -> Self { Self { value: Default::default(), align: MaybeUninit::uninit() } }
}

impl<const A : usize, T> From<T> for AlignN<A, T> where [(); A] : _align_impl::ValidAlignLessThan1GiB {
    fn from(value: T) -> Self { Self { value, align: MaybeUninit::uninit() } }
}



use _align_impl::*;
#[doc(hidden)] pub mod _align_impl {
    pub trait ValidAlignLessThan1GiB {
        #[doc(hidden)] type Align : Clone + Copy + Default + core::fmt::Debug + PartialEq + Eq + PartialOrd + Ord + core::hash::Hash;
    }

    impl ValidAlignLessThan1GiB for [();         1] { type Align = Align1; }
    impl ValidAlignLessThan1GiB for [();         2] { type Align = Align2; }
    impl ValidAlignLessThan1GiB for [();         4] { type Align = Align4; }
    impl ValidAlignLessThan1GiB for [();         8] { type Align = Align8; }
    impl ValidAlignLessThan1GiB for [();        16] { type Align = Align16; }
    impl ValidAlignLessThan1GiB for [();        32] { type Align = Align32; }
    impl ValidAlignLessThan1GiB for [();        64] { type Align = Align64; }
    impl ValidAlignLessThan1GiB for [();       128] { type Align = Align128; }
    impl ValidAlignLessThan1GiB for [();       256] { type Align = Align256; }
    impl ValidAlignLessThan1GiB for [();       512] { type Align = Align512; }
    impl ValidAlignLessThan1GiB for [();      1024] { type Align = Align1024; }
    impl ValidAlignLessThan1GiB for [();      2048] { type Align = Align2048; }
    impl ValidAlignLessThan1GiB for [();      4096] { type Align = Align4096; }
    impl ValidAlignLessThan1GiB for [();      8192] { type Align = Align8192; }
    impl ValidAlignLessThan1GiB for [();     16384] { type Align = Align16384; }
    impl ValidAlignLessThan1GiB for [();     32768] { type Align = Align32768; }
    impl ValidAlignLessThan1GiB for [();     65536] { type Align = Align65536; }
    impl ValidAlignLessThan1GiB for [();    131072] { type Align = Align131072; }
    impl ValidAlignLessThan1GiB for [();    262144] { type Align = Align262144; }
    impl ValidAlignLessThan1GiB for [();    524288] { type Align = Align524288; }
    impl ValidAlignLessThan1GiB for [();   1048576] { type Align = Align1048576; }
    impl ValidAlignLessThan1GiB for [();   2097152] { type Align = Align2097152; }
    impl ValidAlignLessThan1GiB for [();   4194304] { type Align = Align4194304; }
    impl ValidAlignLessThan1GiB for [();   8388608] { type Align = Align8388608; }
    impl ValidAlignLessThan1GiB for [();  16777216] { type Align = Align16777216; }
    impl ValidAlignLessThan1GiB for [();  33554432] { type Align = Align33554432; }
    impl ValidAlignLessThan1GiB for [();  67108864] { type Align = Align67108864; }
    impl ValidAlignLessThan1GiB for [(); 134217728] { type Align = Align134217728; }
    impl ValidAlignLessThan1GiB for [(); 268435456] { type Align = Align268435456; }
    impl ValidAlignLessThan1GiB for [(); 536870912] { type Align = Align536870912; }

    #[allow(dead_code)] pub trait ByAlign<const A : usize> where Self : Sized {
        #[doc(hidden)] type Align : Clone + Copy + Default + core::fmt::Debug + PartialEq + Eq + PartialOrd + Ord + core::hash::Hash;
    }

    impl ByAlign<        1> for () { type Align = Align1; }
    impl ByAlign<        2> for () { type Align = Align2; }
    impl ByAlign<        4> for () { type Align = Align4; }
    impl ByAlign<        8> for () { type Align = Align8; }
    impl ByAlign<       16> for () { type Align = Align16; }
    impl ByAlign<       32> for () { type Align = Align32; }
    impl ByAlign<       64> for () { type Align = Align64; }
    impl ByAlign<      128> for () { type Align = Align128; }
    impl ByAlign<      256> for () { type Align = Align256; }
    impl ByAlign<      512> for () { type Align = Align512; }
    impl ByAlign<     1024> for () { type Align = Align1024; }
    impl ByAlign<     2048> for () { type Align = Align2048; }
    impl ByAlign<     4096> for () { type Align = Align4096; }
    impl ByAlign<     8192> for () { type Align = Align8192; }
    impl ByAlign<    16384> for () { type Align = Align16384; }
    impl ByAlign<    32768> for () { type Align = Align32768; }
    impl ByAlign<    65536> for () { type Align = Align65536; }
    impl ByAlign<   131072> for () { type Align = Align131072; }
    impl ByAlign<   262144> for () { type Align = Align262144; }
    impl ByAlign<   524288> for () { type Align = Align524288; }
    impl ByAlign<  1048576> for () { type Align = Align1048576; }
    impl ByAlign<  2097152> for () { type Align = Align2097152; }
    impl ByAlign<  4194304> for () { type Align = Align4194304; }
    impl ByAlign<  8388608> for () { type Align = Align8388608; }
    impl ByAlign< 16777216> for () { type Align = Align16777216; }
    impl ByAlign< 33554432> for () { type Align = Align33554432; }
    impl ByAlign< 67108864> for () { type Align = Align67108864; }
    impl ByAlign<134217728> for () { type Align = Align134217728; }
    impl ByAlign<268435456> for () { type Align = Align268435456; }
    impl ByAlign<536870912> for () { type Align = Align536870912; }

    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(        1))] pub struct Align1; // 1 B
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(        2))] pub struct Align2;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(        4))] pub struct Align4;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(        8))] pub struct Align8;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(       16))] pub struct Align16;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(       32))] pub struct Align32;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(       64))] pub struct Align64;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(      128))] pub struct Align128;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(      256))] pub struct Align256;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(      512))] pub struct Align512;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(     1024))] pub struct Align1024; // 1 KiB
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(     2048))] pub struct Align2048;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(     4096))] pub struct Align4096;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(     8192))] pub struct Align8192;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(    16384))] pub struct Align16384;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(    32768))] pub struct Align32768;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(    65536))] pub struct Align65536;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(   131072))] pub struct Align131072;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(   262144))] pub struct Align262144;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(   524288))] pub struct Align524288;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(  1048576))] pub struct Align1048576; // 1 MiB
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(  2097152))] pub struct Align2097152;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(  4194304))] pub struct Align4194304;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(  8388608))] pub struct Align8388608;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align( 16777216))] pub struct Align16777216;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align( 33554432))] pub struct Align33554432;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align( 67108864))] pub struct Align67108864;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(134217728))] pub struct Align134217728;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(268435456))] pub struct Align268435456;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(C, align(536870912))] pub struct Align536870912;
    // error[E0589]: invalid `repr(align)` attribute: larger than 2^29
    //#[derive(Clone, Copy, Default)] #[repr(C, align(1073741824))] pub struct Align1073741824; // 1 GiB
}
