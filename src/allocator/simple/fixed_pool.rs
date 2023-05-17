use crate::*;
use crate::align::alignn::_align_impl::ValidAlignLessThan1GiB;

use core::cell::*;
use core::fmt::{self, Debug, Formatter};
use core::mem::MaybeUninit;
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
            #[allow(clippy::uninit_assumed_init)] // the generics confuse clippy, but [UnsafeCell<MaybeUninit<T>>; N] should be safe... probably!
            // SAFETY: ⚠️ UnsafeCell<MaybeUninit<T>> is always "init" even if T isn't, ergo this should be safe?
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
            state:  [(); N].map(|_| Cell::new(State::Free)), // XXX: prevents `new` from being `const`
            //state:  unsafe { MaybeUninit::zeroed().assume_init() }, // XXX: not yet stable
            next:   Cell::new(0),
        }
    }

    /// Get the index into `buffer` and `state` of `ptr`, an allocation of `buffer`.
    /// The returned index is guaranteed valid for either (if undefined behavior hasn't already been invoked.)
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self` or this is straight up undefined behavior.  It need not be a *live* allocation - dead or never-allocated is permitted as well.
    unsafe fn live_or_dead_index_of(&self, ptr: AllocNN) -> usize {
        // SAFETY: ✔️ `ptr` should belong to `self.buffer` per documented safety precondition, ergo `offset_from` should be safe.
        let index = unsafe { ptr.as_ptr().cast::<Element<A, B>>().offset_from(self.buffer.as_ptr()) } as usize;
        if cfg!(debug_assertions) && index >= self.state.len() { bug::ub::invalid_ptr_for_allocator(ptr) }
        index
    }

    /// Get a reference to the `state` corresponding to `ptr`.
    ///
    /// ### Safety
    /// *   `ptr` must belong to `self` or this is straight up undefined behavior.  It need not be a *live* allocation - dead or never-allocated is permitted as well.
    unsafe fn state_of(&self, ptr: AllocNN) -> &Cell<State> {
        // SAFETY: ✔️ `ptr` should belong to `self` per `thin::Free::free`'s safety precondition
        let index = unsafe { self.live_or_dead_index_of(ptr) };

        // SAFETY: ✔️ `index` should be a valid index into `self.{buffer,state}`.  Since it was when `ptr` was allocated, and these buffers are fixed in size, this should be trivially true.
        let state = unsafe { self.state.get_unchecked(index) };
        state
    }
}

impl<const A: usize, const B: usize, const N: usize> Default for FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    fn default() -> Self { Self::new() }
}

impl<const A: usize, const B: usize, const N: usize> Debug for FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "FixedBool<{A}, {B}, {N}> {{ ... }}") }
}

impl<const A: usize, const B: usize, const N: usize> meta::Meta for &'_ FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    type Error                  = ();
    const MAX_ALIGN : Alignment = Alignment::constant(A);
    const MAX_SIZE  : usize     = B;
    const ZST_SUPPORTED : bool  = true;
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
// SAFETY: ✔️ if `alloc_uninit` succeeds:
//  • We must return a pointer to at least `size` bytes.  As all elements of `self.buffer` are only `B` bytes long, we reject `size` above that [1].
//  • Allocations should have at least alignment `min(MAX_ALIGN, ...)`.  All elements of `self.buffer` have alignment `A` = `MAX_ALIGN` via `Element`'s definition including `AlignN` [2].
//  • The bucket should not have already been allocated [3].
//  • The allocations should remain valid for the lifetime of `Self`.  Ergo, we must implement this on a *reference* to FixedPoolLinearProbe, not a value of it.
//
unsafe impl<const A: usize, const B: usize, const N: usize> thin::Alloc for &'_ FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    fn alloc_uninit(&self, size: usize) -> Result<AllocNN, Self::Error> {
        if size > B { return Err(()) } // [1]

        let start = self.next.get();
        let mut pos = start;
        for states in [&self.state[..], &self.state[..start]] {
            while let Some(state) = states.get(pos) {
                match state.get() {
                    State::Free => { // [3]
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

// SAFETY: ✔️ all thin::* impls intercompatible with each other
// SAFETY: ✔️ `free` should be safe to call on any allocated `ptr` belonging to `self`
//
unsafe impl<const A: usize, const B: usize, const N: usize> thin::Free for &'_ FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    unsafe fn free(&self, ptr: AllocNN) {
        // SAFETY: ✔️ `ptr` should belong to `self` per `thin::Free::free`'s documented safety precondition
        let state = unsafe { self.state_of(ptr) };
        if cfg!(debug_assertions) && state.get() == State::Free { bug::ub::freed_ptr_for_allocator(ptr) }
        state.set(State::Free);
    }
}

// SAFETY: ✔️ all thin::* impls intercompatible with each other
// SAFETY: ✔️ if `realloc_uninit` succeeds with defined behavior:
//  • We must return a pointer to at least `size` bytes.  As all elements of `self.buffer` are only `B` bytes long, we reject `size` above that [4].
//  • Allocations should have at least alignment `min(MAX_ALIGN, ...)`.  All elements of `self.buffer` have the same alignment, so returning `ptr` again provides the same guarantee [5].
//  • The bucket should have already been allocated [6].
//  • The allocations should remain valid for the lifetime of `Self`.  Ergo, we must implement this on a *reference* to FixedPoolLinearProbe, not a value of it.
//
unsafe impl<const A: usize, const B: usize, const N: usize> thin::Realloc for &'_ FixedPoolLinearProbe<A, B, N> where [(); A] : ValidAlignLessThan1GiB {
    const CAN_REALLOC_ZEROED : bool = false;

    unsafe fn realloc_uninit(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ `ptr` should belong to `self` per `thin::Realloc::realloc_uninit`'s documented safety precondition
        if cfg!(debug_assertions) && unsafe { self.state_of(ptr) }.get() == State::Free { bug::ub::freed_ptr_for_allocator(ptr) } // [6]
        if new_size > B { return Err(()) } // [4]
        Ok(ptr) // [5]
    }

    unsafe fn realloc_zeroed(&self, ptr: AllocNN, new_size: usize) -> Result<AllocNN, Self::Error> {
        // SAFETY: ✔️ trivially safe - always failing is allowed
        let _ = (ptr, new_size);
        Err(())
    }
}

// I could implement thin::SizeOf{,Debug}. I'd either need to:
//  • Track the requested sizes
//  • Zero the full slot for `thin::Alloc::alloc_zeroed` (which I don't currently do) and always return the slot size.
// If I take the latter route, I could also:
//  • Turn `thin::Realloc::realloc_zeroed` into a trivial success
//    (the entire slot would be initially zeroed, ergo no new zeroing necessary)

#[no_implicit_prelude] mod cleanroom {
    use crate::impls;
    use super::{FixedPoolLinearProbe, ValidAlignLessThan1GiB};

    impls! {
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::fat::Alloc      for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::thin::Alloc;
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::fat::Realloc    for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::thin::Realloc;
        unsafe impl['a, const A: usize, const B: usize, const N: usize] ialloc::fat::Free       for &'a FixedPoolLinearProbe<A, B, N> where [[(); A] : ValidAlignLessThan1GiB] => ialloc::thin::Free;
    }
}



type Element<const A : usize, const B : usize> = UnsafeCell<MaybeUninit<AlignN<A, [u8; B]>>>; // [2]

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)] #[repr(u8)] enum State { #[default] Free = 0, Allocated } // TODO: more states?
//unsafe impl bytemuck::Zeroable for State {}



#[test] fn test_quick() {
    use crate::boxed::ABox;

    let pool = FixedPoolLinearProbe::<4, 4, 1024>::new();
    let mut next = 0_u32;
    for _ in 0 .. 10 {
        assert!(ABox::try_new_in([0u8; 8], &pool).is_err(), "element too big to fit in pool");
        let _integers = [(); 1024].map(|_| {
            next += 1;
            ABox::try_new_in(next, &pool).unwrap()
        });
        assert!(ABox::try_new_in(0u32, &pool).is_err(), "pool out of elements");
        std::dbg!(&_integers[0]);
    }
}

#[test] fn thin_alignment()         { thin::test::alignment(&FixedPoolLinearProbe::<4, 4, 1024>::new()) }
#[test] fn thin_edge_case_sizes()   { thin::test::edge_case_sizes(&FixedPoolLinearProbe::<4, 4, 1024>::new()) }
#[test] fn thin_nullable()          { thin::test::nullable(&FixedPoolLinearProbe::<4, 4, 1024>::new()) }
//#[test] fn thin_size()              { thin::test::size_over_alloc(&FixedPoolLinearProbe::<4, 4, 1024>::new()) } // NYI
#[test] fn thin_uninit()            { unsafe { thin::test::uninit_alloc_unsound(&FixedPoolLinearProbe::<4, 4, 128>::new()) } }
#[test] fn thin_zeroed()            { thin::test::zeroed_alloc(&FixedPoolLinearProbe::<4, 4, 128>::new()) }
#[test] fn thin_zst_support()       { thin::test::zst_supported_accurate(&FixedPoolLinearProbe::<4, 4, 1024>::new()) }

#[test] fn fat_alignment()          { fat::test::alignment(&FixedPoolLinearProbe::<4, 4, 1024>::new()) }
#[test] fn fat_edge_case_sizes()    { fat::test::edge_case_sizes(&FixedPoolLinearProbe::<4, 4, 1024>::new()) }
#[test] fn fat_uninit()             { unsafe { fat::test::uninit_alloc_unsound(&FixedPoolLinearProbe::<4, 4, 128>::new()) } }
#[test] fn fat_zeroed()             { fat::test::zeroed_alloc(&FixedPoolLinearProbe::<4, 4, 128>::new()) }
#[test] fn fat_zst_support()        { fat::test::zst_supported_accurate(&FixedPoolLinearProbe::<4, 4, 1024>::new()) }
