use crate::boxed::ABox;
use crate::fat::Free;



#[cfg(feature = "alloc")]
#[cfg(global_oom_handling)]
impl<A: Free> Extend<ABox<str, A>> for alloc::string::String {
    fn extend<I: IntoIterator<Item = ABox<str, A>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}
