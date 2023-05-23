#![cfg(feature = "std")]

use crate::allocator::alloc::Global;
use crate::boxed::ABox;
use crate::fat::*;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};



impl<A: Free + Into<Global>> From<ABox<OsStr,  A>> for OsString { fn from(value: ABox<OsStr,A>) -> Self { Self::from(ABox::into_std_box_global(value)) } }
impl<A: Free + Into<Global>> From<ABox<Path,   A>> for PathBuf  { fn from(value: ABox<Path, A>) -> Self { Self::from(ABox::into_std_box_global(value)) } }
