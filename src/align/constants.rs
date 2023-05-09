#![allow(non_upper_case_globals)] // "KiB", "MiB", "GiB", "EiB", etc.

use crate::Alignment;



macro_rules! constants {
    ( $($id:ident = $value:expr),* $(,)? ) => {$(
        #[doc(hidden)] pub const $id : Alignment = crate::util::align::constant($value);
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

/// Maximum representable alignment
///
/// | Bits  | MAX                           |
/// | ------| ------------------------------|
/// | 16    | 2<sup>15</sup> B = 32 KiB     |
/// | 32    | 2<sup>31</sup> B = 2 GiB      |
/// | 64    | 2<sup>63</sup> B = 8 EiB      |
/// | 128   | 2<sup>127</sup> B = ???       |
pub const ALIGN_MAX : Alignment = crate::util::align::constant(usize::MAX/2+1);



#[test] fn alignment_debug() {
    use crate::*;
    macro_rules! pp { ($expr:expr) => { alloc::format!("{}", crate::util::bytes::Pretty($expr.as_usize())) }; }

    assert_eq!("16 B",   pp!(ALIGN_16_B));
    assert_eq!("16 KiB", pp!(ALIGN_16_KiB));
    #[cfg(not(target_pointer_width = "16"))] {
        assert_eq!("16 MiB", pp!(ALIGN_16_MiB));
        #[cfg(not(target_pointer_width = "32"))] {
            assert_eq!("16 GiB", pp!(ALIGN_16_GiB));
            assert_eq!("16 TiB", pp!(ALIGN_16_TiB));
            assert_eq!("16 PiB", pp!(ALIGN_16_PiB));
            #[cfg(not(target_pointer_width = "64"))] {
                assert_eq!("16 EiB", pp!(ALIGN_16_EiB));
            }
        }
    }
}
