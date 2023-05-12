/// Like [`panic!`], but meant for undefined behavior which might be worth using [`core::hint::unreachable_unchecked`] on in the future.
macro_rules! ub {
    ( $($tt:tt)* ) => {{
        $crate::_macros::maybe_eventually_sometimes_unreachable();
        panic!($($tt)*);
    }};
}



/// ### Safety
///
/// In the future, this may be equivalent to [`core::hint::unreachable_unchecked`] in some builds.
pub unsafe fn maybe_eventually_sometimes_unreachable() {}
