use core::fmt::{self, Debug, Display, Formatter};
use core::ops::ShrAssign;



/// Pretty print `v` as e.g. "16 KiB" or similar
pub fn pretty<T: Copy + Display + From<u16> + Ord + ShrAssign>(f: &mut Formatter<'_>, mut v: T) -> fmt::Result {
    let limit = T::from(8192);
    let shr   = T::from(10);

    for unit in ["B", "KiB", "MiB", "GiB", "TiB", "PiB"] {
        if v <= limit { return write!(f, "{v} {unit}"); }
        v >>= shr;
    }
    write!(f, "{v} EiB")
}

/// Pretty print `self.0`
#[derive(Clone, Copy)] pub struct Pretty<T: Copy + Display + From<u16> + Ord + ShrAssign>(pub T);
impl<T: Copy + Display + From<u16> + Ord + ShrAssign> Debug   for Pretty<T> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { pretty(f, self.0) } }
impl<T: Copy + Display + From<u16> + Ord + ShrAssign> Display for Pretty<T> { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { pretty(f, self.0) } }
