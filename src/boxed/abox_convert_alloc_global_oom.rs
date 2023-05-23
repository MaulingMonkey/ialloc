#![cfg(global_oom_handling)]

use crate::boxed::ABox;
use crate::meta::*;
use crate::fat::*;
use crate::util;
use crate::vec::AVec;

use core::ffi::CStr;
use core::ptr::NonNull;



impl<T: Copy, A: Alloc + Free + Default + ZstSupported> From<&[T]> for ABox<[T], A> {
    fn from(value: &[T]) -> Self {
        let len : usize = value.len();
        let mut b = ABox::<T, A>::new_uninit_slice(len);
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        unsafe { b.assume_init() }
    }
}

impl<A: Alloc + Free + Default + ZstSupported> From<&CStr> for ABox<CStr, A> {
    fn from(value: &CStr) -> Self {
        let bytes = value.to_bytes_with_nul();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::new_uninit_slice(len);
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut CStr), allocator) }
    }
}

impl<A: Alloc + Free + Default + ZstSupported> From<&str> for ABox<str, A> {
    fn from(value: &str) -> Self {
        let bytes = value.as_bytes();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::new_uninit_slice(len);
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut str), allocator) }
    }
}


impl<T, A: Alloc + Free + Default + ZstSupported, const N : usize> From<[T; N]> for ABox<[T], A> {
    fn from(value: [T; N]) -> Self {
        let mut b = ABox::<T, A>::new_uninit_slice(N);
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), N) };
        core::mem::forget(value);
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        let data = util::nn::slice_assume_init(data);
        unsafe { ABox::from_raw_in(data, allocator) }
    }
}

impl<T, A: Alloc + Free + Default   > From   <T             > for ABox<T,   A> { fn from(value: T)          -> Self { Self::new(value) } }
impl<T, A: Realloc                  > From   <AVec<T, A>    > for ABox<[T], A> { fn from(value: AVec<T, A>) -> Self { value.into_boxed_slice() } }
// TODO: impl<T, A: Free, const N: usize>                     TryFrom<ABox<[T], A>    > for ABox<[T; N], A>                           !Alloc
// TODO: impl<T, A: Free, const N: usize>                     TryFrom<AVec<T, A>      > for ABox<[T; N], A>                           Shrink to fit

#[cfg(feature = "alloc")] mod alloc {
    use crate::allocator::alloc::Global;
    use super::*;

    use ::alloc::borrow::Cow;
    use ::alloc::boxed::Box;
    use ::alloc::string::String;
    use ::alloc::vec::Vec;

    impl<T: Copy,  A: Free + From<Global>> From<Cow<'_, [T]  >> for ABox<[T],  A> { fn from(value: Cow<'_, [T]  >) -> Self { Self::from(Box::<[T]  >::from(value)) } }
    impl<          A: Free + From<Global>> From<Cow<'_, CStr >> for ABox<CStr, A> { fn from(value: Cow<'_, CStr >) -> Self { Self::from(Box::<CStr >::from(value)) } }
    impl<          A: Free + From<Global>> From<Cow<'_, str  >> for ABox<str,  A> { fn from(value: Cow<'_, str  >) -> Self { Self::from(Box::<str  >::from(value)) } }
    impl<          A: Free + From<Global>> From<String        > for ABox<str,  A> { fn from(value: String        ) -> Self { Self::from(Box::<str  >::from(value)) } }
    impl<T,        A: Free + From<Global>> From<Vec<T>        > for ABox<[T],  A> { fn from(value: Vec<T>        ) -> Self { Self::from(Box::<[T]  >::from(value)) } }

    // TODO: impl<T, A: Free + From<Global>, const N: usize> impl TryFrom<Box<[T]>    > for ABox<[T; N], A>                           !Alloc
    // TODO: impl<T, A: Free + From<Global>, const N: usize> impl TryFrom<Vec<T>      > for ABox<[T; N], A>                           Shrink to fit
}

#[cfg(feature = "std")] mod std {
    use crate::allocator::alloc::Global;
    use super::*;

    use ::std::borrow::Cow;
    use ::std::boxed::Box;
    use ::std::ffi::{OsStr, OsString};
    use ::std::path::{Path, PathBuf};

    impl<A: Free + From<Global>> From<&OsStr> for ABox<OsStr, A> { fn from(value: &OsStr) -> Self { Self::from(Box::<OsStr>::from(value)) } }
    impl<A: Free + From<Global>> From<&Path>  for ABox<Path,  A> { fn from(value: &Path ) -> Self { Self::from(Box::<Path >::from(value)) } }

    // TODO: impl From<&str> for ABox<dyn Error + ..., A>

    impl<A: Free + From<Global>> From<Cow<'_, OsStr>> for ABox<OsStr, A> { fn from(value: Cow<'_, OsStr>) -> Self { Self::from(Box::<OsStr>::from(value)) } }
    impl<A: Free + From<Global>> From<Cow<'_, Path >> for ABox<Path,  A> { fn from(value: Cow<'_, Path >) -> Self { Self::from(Box::<Path >::from(value)) } }

    // TODO: impl From<Cow<'_, str>     > for ABox<dyn Error + ..., A>
    // TODO: impl From<impl Error       > for ABox<dyn Error + ..., A>

    impl<A: Free + From<Global>> From<OsString> for ABox<OsStr, A> { fn from(value: OsString) -> Self { Self::from(Box::<OsStr>::from(value)) } }
    impl<A: Free + From<Global>> From<PathBuf > for ABox<Path,  A> { fn from(value: PathBuf ) -> Self { Self::from(Box::<Path >::from(value)) } }
}
