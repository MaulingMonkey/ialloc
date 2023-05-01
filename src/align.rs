#![allow(non_upper_case_globals)] // "KiB", "MiB", "GiB", "EiB", etc.

#[cfg(doc)] use core::alloc::*;
use core::fmt::{self, Debug, Formatter};
use core::mem::{align_of, size_of};
use core::num::NonZeroUsize;



/// A valid [`Layout`] alignment (a power of 2)
///
/// Available in named constant forms, which have been `#[doc(hidden)]` to avoid spam:
///
/// | Min           | Max               | Name          | Equivalences |
/// | --------------| ------------------| --------------| -------------|
/// | `ALIGN_1`     | `ALIGN_8192`      | Byte(s)       | 8 bits = 1 octect ≈ 1 byte
/// | `ALIGN_1_B`   | `ALIGN_8192_B`    | Byte(s)       | 8 bits = 1 octect ≈ 1 byte
/// | `ALIGN_1_KiB` | `ALIGN_8192_KiB`  | Kibibyte(s)   | 1 KiB = 2<sup>10</sup> bytes = 1024<sup>1</sup> bytes
/// | `ALIGN_1_MiB` | `ALIGN_8192_MiB`  | Mebibyte(s)   | 1 MiB = 2<sup>20</sup> bytes = 1024<sup>2</sup> bytes
/// | `ALIGN_1_GiB` | `ALIGN_8192_GiB`  | Gibibyte(s)   | 1 GiB = 2<sup>30</sup> bytes = 1024<sup>3</sup> bytes
/// | `ALIGN_1_TiB` | `ALIGN_8192_TiB`  | Tebibyte(s)   | 1 TiB = 2<sup>40</sup> bytes = 1024<sup>4</sup> bytes
/// | `ALIGN_1_EiB` | `ALIGN_64_EiB`    | Exbibyte(s)   | 1 EiB = 2<sup>50</sup> bytes = 1024<sup>5</sup> bytes
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Alignment(AlignImpl);
const _ : () = assert!(align_of::<Alignment>() == align_of::<NonZeroUsize>());
const _ : () = assert!(size_of ::<Alignment>() == size_of ::<NonZeroUsize>());

impl Alignment {
    /// Returns [`None`] unless `align` is a valid power of 2 (which also implies nonzero)
    pub const fn new(align: usize) -> Option<Self> { Self::try_from_usize(align) }

    /// Returns the [`Alignment`] of `T`.
    pub const fn of<T>() -> Self { unsafe { Self::new_unchecked(core::mem::align_of::<T>()) } }

    /// Get the alignment as a [`usize`] (the nicheless underlying type)
    pub const fn get(self) -> usize { self.to_usize() }

    /// **Undefined behavior** unless `align` is a valid power of 2 (which also implies nonzero)
    pub const unsafe fn new_unchecked(align: usize) -> Self { unsafe { core::mem::transmute(align) } }

    #[allow(dead_code)]
    const fn try_from_nzusize(align: NonZeroUsize   ) -> Option<Self> { if align.is_power_of_two() { Some(unsafe { Self::new_unchecked(align.get()) }) } else { None } }
    const fn try_from_usize  (align: usize          ) -> Option<Self> { if align.is_power_of_two() { Some(unsafe { Self::new_unchecked(align      ) }) } else { None } }

    const fn to_nzusize (self) -> NonZeroUsize  { unsafe { NonZeroUsize::new_unchecked(self.to_usize()) } }
    const fn to_usize   (self) -> usize         { self.0 as usize }
}

impl From<Alignment> for usize          { fn from(align: Alignment) -> Self { align.to_usize()   } }
impl From<Alignment> for NonZeroUsize   { fn from(align: Alignment) -> Self { align.to_nzusize() } }

// XXX: u8::try_from(0xFFFF_u16) isn't a const expr
//type TryFromIntError = core::num::TryFromIntError;
//const TRY_FROM_INT_ERROR : TryFromIntError = if let Err(e) = u8::try_from(0xFFFFu16) { e } else { panic!("unable to construct TRY_FROM_INT_ERROR") };
//impl TryFrom<usize          > for Alignment { fn try_from(align: usize          ) -> Result<Self, Self::Error> { Self::try_from_usize  (align).ok_or(TRY_FROM_INT_ERROR) } type Error = TryFromIntError; }
//impl TryFrom<NonZeroUsize   > for Alignment { fn try_from(align: NonZeroUsize   ) -> Result<Self, Self::Error> { Self::try_from_nzusize(align).ok_or(TRY_FROM_INT_ERROR) } type Error = TryFromIntError; }

impl Debug for Alignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "Alignment({})", self.to_usize()) }
}



#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] #[cfg(target_pointer_width = "16")] #[repr(usize)] enum AlignImpl {
    _00 = 1 << 0x00, _01 = 1 << 0x01, _02 = 1 << 0x02, _03 = 1 << 0x03, _04 = 1 << 0x04, _05 = 1 << 0x05, _06 = 1 << 0x06, _07 = 1 << 0x07,
    _08 = 1 << 0x08, _09 = 1 << 0x09, _0A = 1 << 0x0A, _0B = 1 << 0x0B, _0C = 1 << 0x0C, _0D = 1 << 0x0D, _0E = 1 << 0x0E, _0F = 1 << 0x0F,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] #[cfg(target_pointer_width = "32")] #[repr(usize)] enum AlignImpl {
    _00 = 1 << 0x00, _01 = 1 << 0x01, _02 = 1 << 0x02, _03 = 1 << 0x03, _04 = 1 << 0x04, _05 = 1 << 0x05, _06 = 1 << 0x06, _07 = 1 << 0x07,
    _08 = 1 << 0x08, _09 = 1 << 0x09, _0A = 1 << 0x0A, _0B = 1 << 0x0B, _0C = 1 << 0x0C, _0D = 1 << 0x0D, _0E = 1 << 0x0E, _0F = 1 << 0x0F,
    _10 = 1 << 0x10, _11 = 1 << 0x11, _12 = 1 << 0x12, _13 = 1 << 0x13, _14 = 1 << 0x14, _15 = 1 << 0x15, _16 = 1 << 0x16, _17 = 1 << 0x17,
    _18 = 1 << 0x18, _19 = 1 << 0x19, _1A = 1 << 0x1A, _1B = 1 << 0x1B, _1C = 1 << 0x1C, _1D = 1 << 0x1D, _1E = 1 << 0x1E, _1F = 1 << 0x1F,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] #[cfg(target_pointer_width = "64")] #[repr(usize)] enum AlignImpl {
    _00 = 1 << 0x00, _01 = 1 << 0x01, _02 = 1 << 0x02, _03 = 1 << 0x03, _04 = 1 << 0x04, _05 = 1 << 0x05, _06 = 1 << 0x06, _07 = 1 << 0x07,
    _08 = 1 << 0x08, _09 = 1 << 0x09, _0A = 1 << 0x0A, _0B = 1 << 0x0B, _0C = 1 << 0x0C, _0D = 1 << 0x0D, _0E = 1 << 0x0E, _0F = 1 << 0x0F,
    _10 = 1 << 0x10, _11 = 1 << 0x11, _12 = 1 << 0x12, _13 = 1 << 0x13, _14 = 1 << 0x14, _15 = 1 << 0x15, _16 = 1 << 0x16, _17 = 1 << 0x17,
    _18 = 1 << 0x18, _19 = 1 << 0x19, _1A = 1 << 0x1A, _1B = 1 << 0x1B, _1C = 1 << 0x1C, _1D = 1 << 0x1D, _1E = 1 << 0x1E, _1F = 1 << 0x1F,
    _20 = 1 << 0x20, _21 = 1 << 0x21, _22 = 1 << 0x22, _23 = 1 << 0x23, _24 = 1 << 0x24, _25 = 1 << 0x25, _26 = 1 << 0x26, _27 = 1 << 0x27,
    _28 = 1 << 0x28, _29 = 1 << 0x29, _2A = 1 << 0x2A, _2B = 1 << 0x2B, _2C = 1 << 0x2C, _2D = 1 << 0x2D, _2E = 1 << 0x2E, _2F = 1 << 0x2F,
    _30 = 1 << 0x30, _31 = 1 << 0x31, _32 = 1 << 0x32, _33 = 1 << 0x33, _34 = 1 << 0x34, _35 = 1 << 0x35, _36 = 1 << 0x36, _37 = 1 << 0x37,
    _38 = 1 << 0x38, _39 = 1 << 0x39, _3A = 1 << 0x3A, _3B = 1 << 0x3B, _3C = 1 << 0x3C, _3D = 1 << 0x3D, _3E = 1 << 0x3E, _3F = 1 << 0x3F,
}



macro_rules! constants {
    ( $($id:ident = $value:expr),* $(,)? ) => {$(
        #[doc(hidden)] pub const $id : Alignment = if let Some(a) = Alignment::new($value) { a } else { panic!(concat!("invalid alignment for constant ", stringify!($id))) };
    )*};
}

constants! { // 16+-bit
    ALIGN_1 = 1, ALIGN_2 = 2, ALIGN_4 = 4, ALIGN_8 = 8, ALIGN_16 = 16, ALIGN_32 = 32, ALIGN_64 = 64, ALIGN_128 = 128, ALIGN_256 = 256, ALIGN_512 = 512, ALIGN_1024 = 1024, ALIGN_2048 = 2048, ALIGN_4096 = 4096, ALIGN_8192 = 8192,
    ALIGN_1_B = 1, ALIGN_2_B = 2, ALIGN_4_B = 4, ALIGN_8_B = 8, ALIGN_16_B = 16, ALIGN_32_B = 32, ALIGN_64_B = 64, ALIGN_128_B = 128, ALIGN_256_B = 256, ALIGN_512_B = 512, ALIGN_1024_B = 1024, ALIGN_2048_B = 2048, ALIGN_4096_B = 4096, ALIGN_8192_B = 8192,
    ALIGN_1_KiB = 1 << 10, ALIGN_2_KiB = 2 << 10, ALIGN_4_KiB = 4 << 10, ALIGN_8_KiB = 8 << 10, ALIGN_16_KiB = 16 << 10, // 32+ KiB overflows
}

#[cfg(not(target_pointer_width = "16"))] constants! { // 32+ bit
    ALIGN_32_KiB = 32 << 10, ALIGN_64_KiB = 64 << 10, ALIGN_128_KiB = 128 << 10, ALIGN_256_KiB = 256 << 10, ALIGN_512_KiB = 512 << 10, ALIGN_1024_KiB = 1024 << 10, ALIGN_2048_KiB = 2048 << 10, ALIGN_4096_KiB = 4096 << 10, ALIGN_8192_KiB = 8192 << 10,
    ALIGN_1_MiB = 1 << 20, ALIGN_2_MiB = 2 << 20, ALIGN_4_MiB = 4 << 20, ALIGN_8_MiB = 8 << 20, ALIGN_16_MiB = 16 << 20, ALIGN_32_MiB = 32 << 20, ALIGN_64_MiB = 64 << 20, ALIGN_128_MiB = 128 << 20, ALIGN_256_MiB = 256 << 20, ALIGN_512_MiB = 512 << 20, ALIGN_1024_MiB = 1024 << 20, ALIGN_2048_MiB = 2048 << 20, // 4096+ MiB overflows
    ALIGN_1_GiB = 1 << 30, ALIGN_2_GiB = 2 << 30, // 4+ GiB overflows
}

#[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32")))] constants! { // 64+ bit
    ALIGN_4096_MiB = 4096 << 20, ALIGN_8192_MiB = 8192 << 20,
    ALIGN_4_GiB = 4 << 30, ALIGN_8_GiB = 8 << 30, ALIGN_16_GiB = 16 << 30, ALIGN_32_GiB = 32 << 30, ALIGN_64_GiB = 64 << 30, ALIGN_128_GiB = 128 << 30, ALIGN_256_GiB = 256 << 30, ALIGN_512_GiB = 512 << 30, ALIGN_1024_GiB = 1024 << 30, ALIGN_2048_GiB = 2048 << 30, ALIGN_4096_GiB = 4096 << 30, ALIGN_8192_GiB = 8192 << 30,
    ALIGN_1_TiB = 1 << 40, ALIGN_2_TiB = 2 << 40, ALIGN_4_TiB = 4 << 40, ALIGN_8_TiB = 8 << 40, ALIGN_16_TiB = 16 << 40, ALIGN_32_TiB = 32 << 40, ALIGN_64_TiB = 64 << 40, ALIGN_128_TiB = 128 << 40, ALIGN_256_TiB = 256 << 40, ALIGN_512_TiB = 512 << 40, ALIGN_1024_TiB = 1024 << 40, ALIGN_2048_TiB = 2048 << 40, ALIGN_4096_TiB = 4096 << 40, ALIGN_8192_TiB = 8192 << 40,
    ALIGN_1_PiB = 1 << 50, ALIGN_2_PiB = 2 << 50, ALIGN_4_PiB = 4 << 50, ALIGN_8_PiB = 8 << 50, ALIGN_16_PiB = 16 << 50, ALIGN_32_PiB = 32 << 50, ALIGN_64_PiB = 64 << 50, ALIGN_128_PiB = 128 << 50, ALIGN_256_PiB = 256 << 50, ALIGN_512_PiB = 512 << 50, ALIGN_1024_PiB = 1024 << 50, ALIGN_2048_PiB = 2048 << 50, ALIGN_4096_PiB = 4096 << 50, ALIGN_8192_PiB = 8192 << 50,
    ALIGN_1_EiB = 1 << 60, ALIGN_2_EiB = 2 << 60, ALIGN_4_EiB = 4 << 60, ALIGN_8_EiB = 8 << 60, // 16+ EiB overflows
}

#[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32", target_pointer_width = "64")))] constants! { // 128+ bit
    ALIGN_16_EiB = 16 << 60, ALIGN_32_EiB = 32 << 60, ALIGN_64_EiB = 64 << 60, // TODO: the rest of the owl
}
