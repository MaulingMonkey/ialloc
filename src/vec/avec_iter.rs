use crate::fat::*;
use crate::meta::*;
use crate::vec::AVec;



#[cfg(global_oom_handling)] impl<T, A: Realloc + Default + ZstSupported> FromIterator<T> for AVec<T, A> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v = Self::new();
        v.extend(iter);
        v
    }
}

// TODO:
//  • [ ] IntoIterator

// UNNECESSARY:
//  • Iterator/ExactSizeIterator/... - see slice instead
