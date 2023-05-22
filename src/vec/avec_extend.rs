use crate::fat::Realloc;
use crate::meta::ZstSupported;
use crate::vec::AVec;



#[cfg(global_oom_handling)] impl<T, A: Realloc + ZstSupported> Extend<T> for AVec<T, A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for item in iter { self.push(item) }
    }
}

#[cfg(global_oom_handling)] impl<'a, T: Copy + 'a, A: Realloc + ZstSupported> Extend<&'a T> for AVec<T, A> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for item in iter { self.push(*item) }
    }
    // unstable:
    // fn extend_one(&mut self, item: &'a T) { self.push(*item) }
    // fn extend_reserve(&mut self, additional: usize) { self.reserve(additional) }
}

// TODO: try_extend
// TODO: try_extend_copy ?
