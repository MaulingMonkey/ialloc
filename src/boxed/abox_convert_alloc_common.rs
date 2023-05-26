use crate::boxed::ABox;
use crate::meta::*;
use crate::fat::*;
use crate::util;

use core::alloc::Layout;
use core::ffi::CStr;
use core::mem::MaybeUninit;
use core::ptr::NonNull;



impl<T: Copy, A: Alloc + Free + ZstSupported> ABox<[T], A> {
    pub(crate) fn try_from_slice(value: &[T]) -> Result<Self, A::Error> where A : Default { Self::try_from_slice_in(value, A::default()) }
    pub(crate) fn try_from_slice_in(value: &[T], allocator: A) -> Result<Self, A::Error> {
        let len : usize = value.len();
        let mut b = ABox::<T, A>::try_new_uninit_slice_in(len, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        Ok(unsafe { b.assume_init() })
    }
}

impl<T: Clone, A: Alloc + Free + ZstSupported> ABox<[T], A> {
    #[allow(dead_code)]
    pub(crate) fn try_from_clone_slice(value: &[T]) -> Result<Self, A::Error> where A : Default { Self::try_from_clone_slice_in(value, A::default()) }
    pub(crate) fn try_from_clone_slice_in(value: &[T], allocator: A) -> Result<Self, A::Error> {
        let len : usize = value.len();
        let mut abox = ABox::<T, A>::try_new_uninit_slice_in(len, allocator)?;
        let mut drop = util::drop::InPlaceOnDrop::default();
        for i in 0 .. len {
            unsafe { drop.set(core::ptr::slice_from_raw_parts_mut(abox.as_mut_ptr().cast::<T>(), i)) };
            abox[i] = MaybeUninit::new(value[i].clone());
        }
        drop.forget();
        Ok(unsafe { abox.assume_init() })
    }
}

// TODO: remove A : ZstSupported bound?  CStr is never a ZST as it always has at least a `\0`
impl<A: Alloc + Free + ZstSupported> ABox<CStr, A> {
    pub(crate) fn try_from_cstr(value: &CStr) -> Result<Self, A::Error> where A : Default { Self::try_from_cstr_in(value, A::default()) }
    pub(crate) fn try_from_cstr_in(value: &CStr, allocator: A) -> Result<Self, A::Error> {
        let bytes = value.to_bytes_with_nul();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::try_new_uninit_slice_in(len, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut CStr), allocator) })
    }
}

impl<A: Alloc + Free + ZstSupported> ABox<str, A> {
    pub(crate) fn try_from_str(value: &str) -> Result<Self, A::Error> where A : Default { Self::try_from_str_in(value, A::default()) }
    pub(crate) fn try_from_str_in(value: &str, allocator: A) -> Result<Self, A::Error> {
        let bytes = value.as_bytes();
        let len = bytes.len();
        let mut b = ABox::<u8, A>::try_new_uninit_slice_in(len, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), len) };
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(data.as_ptr() as *mut str), allocator) })
    }
}

#[cfg(feature = "std")] impl<A: Alloc + Free + ZstSupported> ABox<std::ffi::OsStr, A> {
    pub(crate) fn try_from_osstr(value: &std::ffi::OsStr) -> Result<Self, A::Error> where A : Default { Self::try_from_osstr_in(value, A::default()) }
    pub(crate) fn try_from_osstr_in(value: &std::ffi::OsStr, allocator: A) -> Result<Self, A::Error> {
        use std::ffi::*;

        let layout = Layout::for_value(value);
        let n = layout.size() / layout.align();
        assert_eq!(layout.size(), n * layout.align());

        let data = if layout.size() != 0 { allocator.alloc_uninit(layout)? } else { util::nn::dangling(layout) };
        {
            let dst = unsafe { core::slice::from_raw_parts_mut(data.as_ptr(), layout.size()) };
            let src = unsafe { core::slice::from_raw_parts(value as *const OsStr as *const MaybeUninit<u8>, layout.size()) };
            dst.copy_from_slice(src);
        }

        match layout.align() {
            1 => Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data.as_ptr().cast::<MaybeUninit<u8 >>(), n) as *mut OsStr), allocator) }),
            2 => Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data.as_ptr().cast::<MaybeUninit<u16>>(), n) as *mut OsStr), allocator) }),
            4 => Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data.as_ptr().cast::<MaybeUninit<u32>>(), n) as *mut OsStr), allocator) }),
            _ => panic!("absurd alignment for OsStr"),
        }
    }
}

#[cfg(feature = "std")] impl<A: Alloc + Free + ZstSupported> ABox<std::path::Path, A> {
    pub(crate) fn try_from_path(value: &std::path::Path) -> Result<Self, A::Error> where A : Default { Self::try_from_path_in(value, A::default()) }
    pub(crate) fn try_from_path_in(value: &std::path::Path, allocator: A) -> Result<Self, A::Error> {
        use std::path::*;

        let layout = Layout::for_value(value);
        let n = layout.size() / layout.align();
        assert_eq!(layout.size(), n * layout.align());

        let data = if layout.size() != 0 { allocator.alloc_uninit(layout)? } else { util::nn::dangling(layout) };
        {
            let dst = unsafe { core::slice::from_raw_parts_mut(data.as_ptr(), layout.size()) };
            let src = unsafe { core::slice::from_raw_parts(value as *const Path as *const MaybeUninit<u8>, layout.size()) };
            dst.copy_from_slice(src);
        }

        match layout.align() {
            1 => Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data.as_ptr().cast::<MaybeUninit<u8 >>(), n) as *mut Path), allocator) }),
            2 => Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data.as_ptr().cast::<MaybeUninit<u16>>(), n) as *mut Path), allocator) }),
            4 => Ok(unsafe { ABox::from_raw_in(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(data.as_ptr().cast::<MaybeUninit<u32>>(), n) as *mut Path), allocator) }),
            _ => panic!("absurd alignment for Path"),
        }
    }
}

impl<T, A: Alloc + Free + ZstSupported> ABox<[T], A> {
    pub(crate) fn try_from_array<const N : usize>(value: [T; N]) -> Result<Self, A::Error> where A : Default { Self::try_from_array_in(value, A::default()) }
    pub(crate) fn try_from_array_in<const N : usize>(value: [T; N], allocator: A) -> Result<Self, A::Error> {
        let mut b = ABox::<T, A>::try_new_uninit_slice_in(N, allocator)?;
        unsafe { core::ptr::copy_nonoverlapping(value.as_ptr(), b.as_mut_ptr().cast(), N) };
        core::mem::forget(value);
        let (data, allocator) = ABox::into_raw_with_allocator(b);
        let data = util::nn::slice_assume_init(data);
        Ok(unsafe { ABox::from_raw_in(data, allocator) })
    }
}
