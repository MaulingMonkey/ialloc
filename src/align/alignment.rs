use crate::*;

#[cfg(doc)] use core::alloc::*;
use core::alloc::Layout;
use core::fmt::{self, Debug, Formatter};
use core::mem::{align_of, size_of};
use core::num::{NonZeroUsize, TryFromIntError};



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
/// | `ALIGN_1_EiB` | `ALIGN_8192_EiB`  | Exbibyte(s)   | 1 EiB = 2<sup>50</sup> bytes = 1024<sup>5</sup> bytes
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(transparent)] pub struct Alignment(AlignImpl);
const _ : () = assert!(align_of::<Alignment>() == align_of::<NonZeroUsize>());
const _ : () = assert!(size_of ::<Alignment>() == size_of ::<NonZeroUsize>());

impl Alignment {
    #[track_caller] pub(crate) const fn constant(align: usize) -> Self { match Self::new(align) { Some(a) => a, None => panic!("Alignment::constant(align): invalid constant") } }

    /// Returns [`None`] unless `align` is a valid power of 2 (which also implies nonzero)
    pub const fn new(align: usize) -> Option<Self> { Self::try_from_usize(align) }

    /// Returns the [`Alignment`] of `T`.
    pub const fn of<T>() -> Self { unsafe { Self::new_unchecked(core::mem::align_of::<T>()) } }

    /// **Undefined behavior** unless `align` is a valid power of 2 (which also implies nonzero)
    pub const unsafe fn new_unchecked(align: usize) -> Self { unsafe { core::mem::transmute(align) } }

    /// Returns the alignment as a [`usize`] (the nicheless underlying type)
    pub const fn as_usize   (self) -> usize         { self.0 as usize }

    /// Returns the alignment as a [`NonZeroUsize`]
    pub const fn as_nonzero (self) -> NonZeroUsize  { unsafe { NonZeroUsize::new_unchecked(self.as_usize()) } }

    /// Minimum representable alignment (e.g. `1`)
    pub const MIN : Alignment = ALIGN_1;

    /// Maximum representable alignment
    ///
    /// | Bits  | MAX                           |
    /// | ------| ------------------------------|
    /// | 16    | 2<sup>15</sup> B = 32 KiB     |
    /// | 32    | 2<sup>31</sup> B = 2 GiB      |
    /// | 64    | 2<sup>63</sup> B = 8 EiB      |
    /// | 128   | 2<sup>127</sup> B = ???       |
    pub const MAX : Alignment = Alignment::constant(usize::MAX/2+1);

    #[allow(dead_code)]
    const fn try_from_nzusize(align: NonZeroUsize   ) -> Option<Self> { if align.is_power_of_two() { Some(unsafe { Self::new_unchecked(align.get()) }) } else { None } }
    const fn try_from_usize  (align: usize          ) -> Option<Self> { if align.is_power_of_two() { Some(unsafe { Self::new_unchecked(align      ) }) } else { None } }

}

impl From<Layout   > for Alignment      { fn from(value: Layout   ) -> Self { unsafe { Self::new_unchecked(value.align()) } } }
impl From<Alignment> for usize          { fn from(align: Alignment) -> Self { align.as_usize()   } }
impl From<Alignment> for NonZeroUsize   { fn from(align: Alignment) -> Self { align.as_nonzero() } }

fn try_from_int_error() -> TryFromIntError { unsafe { NonZeroUsize::try_from(0).unwrap_err_unchecked() } }
impl TryFrom<usize          > for Alignment { fn try_from(align: usize          ) -> Result<Self, Self::Error> { Self::try_from_usize  (align).ok_or_else(|| try_from_int_error()) } type Error = TryFromIntError; }
impl TryFrom<NonZeroUsize   > for Alignment { fn try_from(align: NonZeroUsize   ) -> Result<Self, Self::Error> { Self::try_from_nzusize(align).ok_or_else(|| try_from_int_error()) } type Error = TryFromIntError; }

impl Debug for Alignment { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { util::bytes::pretty(f, self.as_usize()) } }



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
