use crate::*;
use crate::align::alignn::_align_impl::ValidAlignLessThan1GiB;

use core::cell::*;
use core::fmt::{self, Debug, Formatter};
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr::NonNull;



/// Pool-allocate fixed size elements from a fixed region of memory
///
/// ## Generic Parameters
/// *   `A` - alignment of pool elements in bytes
/// *   `B` - size of pool elements in bytes
/// *   `N` - number of elements in pool
///
/// ## Allocation Scaling Behavior
///
/// Worst case performance scales by `N` (will check every slot before giving up.)
/// Average case performance scales by what percentage of the pool remains free -
/// e.g. if the pool is kept 50% empty, 2 slots will be checked on average.
/// If the pool is kept 90% empty, 10 slots will be checked on average.
///
/// This further assumes the filled slots are kept permanently allocated -
/// the cyclic nature of linear allocation means that if you always free the
/// oldest allocation before creating a new allocation, performance will be `O(1)`
/// even if the pool is kept mostly full.
///
/// Freeing and reallocating memory is constant time.
///
/// ## Alternatives
/// *   I should write a `FixedPoolFreeList` allocator that (ab)uses unallocated slots as a linked list of free entries.<br>
///     This will be `O(1)` at the expense of additional complexity and minimum element size.
/// *   I should write non-`Fixed*` variants which heap allocate additional pools as necessary.
pub struct FixedPoolLinearProbe<const A: usize, const B: usize, const N: usize> where [(); A] : ValidAlignLessThan1GiB {
    buffer: [Element<A, B>; N],
    state:  [Cell<State>; N], // Lame: wastes 7 bits per element at the moment.  Would be nice to instead use e.g. `[u32; N/32]` but const generics aren't that awesome yet.
    next:   Cell<usize>,
}

impl<const A: usize, const B: usize, const N: usize> FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    pub fn new() -> Self {
        Self {
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
            state:  [(); N].map(|_| Cell::new(State::Free)), // XXX: prevents `new` from being `const`
            //state:  unsafe { MaybeUninit::zeroed().assume_init() }, // XXX: not yet stable
            next:   Cell::new(0),
        }
    }
}

impl<const A: usize, const B: usize, const N: usize> Default for FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    fn default() -> Self { Self::new() }
}

impl<const A: usize, const B: usize, const N: usize> Debug for FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "FixedBool<{A}, {B}, {N}> {{ ... }}") }
}

unsafe impl<const A: usize, const B: usize, const N: usize> thin::Alloc for &'_ FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    const MAX_ALIGN : Alignment = crate::util::align::constant(A);

    type Error = ();

    fn alloc_uninit(&self, size: core::num::NonZeroUsize) -> Result<AllocNN, Self::Error> {
        if size.get() > B { return Err(()) }

        let start = self.next.get();
        let mut pos = start;
        for states in [&self.state[..], &self.state[..start]] {
            while let Some(state) = states.get(pos) {
                match state.get() {
                    State::Free => {
                        state.set(State::Allocated);
                        self.next.set(pos+1);
                        return NonNull::new(self.buffer[pos].get().cast()).ok_or(())
                    },
                    State::Allocated => pos += 1,
                }
            }
            pos = 0; // wraparound
        }

        Err(())
    }
}

unsafe impl<const A: usize, const B: usize, const N: usize> thin::Free for &'_ FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    unsafe fn free(&self, ptr: AllocNN) {
        let index = unsafe { ptr.as_ptr().cast::<Element<A, B>>().offset_from(self.buffer.as_ptr()) } as usize;
        let state = self.state.get(index).expect("undefined behavior: free called on a ptr that wasn't part of the FixedPoolLinearProbe allocation");
        debug_assert_ne!(state.get(), State::Free, "undefined behavior: free called on a ptr that was already free");
        state.set(State::Free);
    }
}

unsafe impl<const A: usize, const B: usize, const N: usize> thin::Realloc for &'_ FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        if new_size.get() > B { return Err(()) }
        Ok(ptr)
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: NonZeroUsize) -> Result<AllocNN, Self::Error> {
        let _ = (ptr, new_size);
        Err(())
    }
}

#[no_implicit_prelude] mod cleanroom {
    use crate::impls;
    use super::{FixedPoolLinearProbe, ValidAlignLessThan1GiB};

    impls! {
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::nzst::Alloc     for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::thin::Alloc;
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::nzst::Realloc   for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::thin::Realloc;
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::nzst::Free      for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::thin::Free;

        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::zsty::Alloc     for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::nzst::Alloc;
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::zsty::Realloc   for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::nzst::Realloc;
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::zsty::Free      for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::nzst::Free;
    }
}



type Element<const A : usize, const B : usize> = UnsafeCell<MaybeUninit<AlignN<A, [u8; B]>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(u8)] enum State { Free = 0, Allocated } // TODO: more states?
impl Default for State { fn default() -> Self { State::Free } }
//unsafe impl bytemuck::Zeroable for State {}



#[test] fn test_quick() {
    use crate::boxed::ABox;

    let pool = FixedPoolLinearProbe::<4, 4, 1024>::new();
    let mut next = 0_u32;
    for _ in 0 .. 10 {
        assert!(ABox::try_new_in([0u8; 8], &pool).is_err(), "element too big to fit in pool");
        let _integers = [(); 1024].map(|_| {
            next += 1;
            ABox::new_in(next, &pool)
        });
        assert!(ABox::try_new_in(0u32, &pool).is_err(), "pool out of elements");
        std::dbg!(&_integers[0]);
    }
}
