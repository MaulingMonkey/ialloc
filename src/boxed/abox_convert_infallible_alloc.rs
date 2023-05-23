#![cfg(feature = "alloc")]

use crate::allocator::alloc::Global;
use crate::boxed::ABox;
use crate::fat::*;

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use core::ffi::CStr;
use core::ptr::NonNull;



impl<A: Free + From<Global>, T: ?Sized  > From<Box<T>         > for ABox<T, A>  { fn from(value: Box<T>       ) -> Self { ABox::from_std_box_global(value) } }

impl<A: Free + Into<Global>, T          > From<ABox<[T],    A>> for Vec<T>      { fn from(value: ABox<[T],  A>) -> Self { Self::from(ABox::into_std_box_global(value)) } } // XXX: allocator_api?
impl<A: Free + Into<Global>             > From<ABox<CStr,   A>> for CString     { fn from(value: ABox<CStr, A>) -> Self { Self::from(ABox::into_std_box_global(value)) } }
impl<A: Free + Into<Global>, T: ?Sized  > From<ABox<T,      A>> for Arc<T>      { fn from(value: ABox<T,    A>) -> Self { Self::from(ABox::into_std_box_global(value)) } }
impl<A: Free + Into<Global>, T: ?Sized  > From<ABox<T,      A>> for Rc<T>       { fn from(value: ABox<T,    A>) -> Self { Self::from(ABox::into_std_box_global(value)) } }
impl<A: Free + Into<Global>             > From<ABox<str,    A>> for Box<[u8]>   { fn from(value: ABox<str,  A>) -> Self { Self::from(ABox::into_std_box_global(value)) } } // XXX: allocator_api?
impl<A: Free + Into<Global>             > From<ABox<str,    A>> for String      { fn from(value: ABox<str,  A>) -> Self { Self::from(ABox::into_std_box_global(value)) } }

// None of these work, IDK why ABox → Box fails when ABox → {Arc, Vec} succeeds:
//  • impl<A: Free + Into<Global>, T: ?Sized  > From<ABox<T,      A>> for Box<T>      { fn from(value: ABox<T,    A>) -> Self { Self::from(ABox::into_std_box_global(value)) } } // error[E0210]: type parameter `T` must be covered by another type when it appears before the first local type (`abox::ABox<T, A>`)
//  • impl<A: Free + Into<Global>, T: ?Sized  > From<ABox<T,      A>> for Pin<Box<T>> { fn from(value: ABox<T,    A>) -> Self { Self::from(ABox::into_std_box_global(value)) } } // error[E0210]: type parameter `T` must be covered by another type when it appears before the first local type (`abox::ABox<T, A>`)
//  • impl<A: Free + Into<Global>, T: ?Sized  > Into<Box<T>>          for ABox<T, A>  { fn into(self) -> Box<T> { ABox::into_std_box_global(self) } } // error[E0119]: conflicting implementations of trait `Into<Box<_>>` for type `abox::ABox<_, _>`



impl<T: ?Sized, A: Free> ABox<T, A> {
    pub(crate) fn from_std_box_global(this: Box<T>) -> Self where A : From<Global> {
        let data = Box::into_raw(this);
        let data = unsafe { NonNull::new_unchecked(data) };
        let allocator = A::from(Global);
        unsafe { ABox::from_raw_in(data, allocator) }
    }

    pub(crate) fn into_std_box_global(this: Self) -> Box<T> where A : Into<Global> {
        let (data, allocator) = ABox::into_raw_with_allocator(this);
        let _allocator : Global = allocator.into();
        unsafe { Box::from_raw(data.as_ptr()) }
    }
}
