use crate::boxed::ABox;
use crate::fat::Free;

use core::fmt::{self, Debug, Display, Pointer, Formatter};



impl<T: Debug,   A: Free + Debug> Debug   for ABox<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { f.debug_struct("ABox").field("data", &**self).field("allocator", Self::allocator(self)).finish() } }
impl<T: Display, A: Free        > Display for ABox<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { T::fmt(self, f) } }
impl<T: ?Sized,  A: Free        > Pointer for ABox<T, A> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Pointer::fmt(&self.data(), f) } }



#[cfg(feature = "std")]
#[allow(deprecated)]
impl<T: std::error::Error, A: Free> std::error::Error for ABox<T, A> where Self : Debug + Display {
    fn description(&self)   -> &str                                         { (**self).description() }
    fn cause(&self)         -> Option<&dyn std::error::Error>               { (**self).cause() }
    fn source(&self)        -> Option<&(dyn std::error::Error + 'static)>   { (**self).source() }
}
