#![cfg(not(global_oom_handling))]

use crate::boxed::ABox;
use crate::meta::*;
use crate::fat::*;
use crate::vec::AVec;

use core::ffi::CStr;



impl<T: Copy, A: Alloc + Free + Default + ZstSupported                  > TryFrom<&[T] > for ABox<[T],  A> { type Error = A::Error; fn try_from(value: &[T] ) -> Result<Self, Self::Error> { Self::try_from_slice(value) } }
impl<         A: Alloc + Free + Default + ZstSupported                  > TryFrom<&CStr> for ABox<CStr, A> { type Error = A::Error; fn try_from(value: &CStr) -> Result<Self, Self::Error> { Self::try_from_cstr (value) } }
impl<         A: Alloc + Free + Default + ZstSupported                  > TryFrom<&str > for ABox<str,  A> { type Error = A::Error; fn try_from(value: &str ) -> Result<Self, Self::Error> { Self::try_from_str  (value) } }
impl<T,       A: Alloc + Free + Default + ZstSupported, const N : usize > TryFrom<[T;N]> for ABox<[T],  A> { type Error = A::Error; fn try_from(value: [T;N]) -> Result<Self, Self::Error> { Self::try_from_array(value) } }
