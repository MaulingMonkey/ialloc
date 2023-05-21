#[cfg(doc)] use crate as ialloc;
use crate::*;
use fat::*;

use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit, size_of};
use core::ptr::*;



/// [`fat::Alloc`]-friendly [`alloc::boxed::Box`] alternative
///
/// ## Notable Differences
/// | Feature           | [`ialloc::boxed::ABox`]                           | [`alloc::boxed::Box`]                                 |
/// | ------------------| --------------------------------------------------| ------------------------------------------------------|
/// | `#![no_std]`      | [`core`]-only friendly!                           | requires [`core`] + [`alloc`]
/// | Allocator API     | stable lean [`fat::Free`] (+ ...)                 | nightly wide [`alloc::alloc::Allocator`]
/// | Zeroed Allocs     | stable [`bytemuck::Zeroable`]-aware               | nightly ugly [`MaybeUninit`]
/// | Panic-on-OOM APIs | `--features panicy-memory`                        | unless `-Z build-std --cfg no_global_oom_handling`
/// | Alignment         | compile time checked allocator support            | allocator must be general or fail at runtime
/// | [`NonNull`]       | in public API as appropriate                      | interior only
/// | `#[may_dangle]`   | NYI (not yet stable: [#34761](https://github.com/rust-lang/rust/issues/34761)) | yes
/// | Into Inner        | Explicit (e.g. <code>ABox::[into_inner](Self::into_inner)\(b\)</code>)  | Magic `DerefMove` (e.g. `*b`)
///
pub struct ABox<T: ?Sized, A: Free> {
    allocator:  A,
    data:       NonNull<T>,
    _phantom:   PhantomData<T>,
}

// SAFETY: ✔️ (T, A) are Send
unsafe impl<T: ?Sized + Send, A: Free + Send> Send for ABox<T, A> {}
// SAFETY: ✔️ (T, A) are Sync
unsafe impl<T: ?Sized + Sync, A: Free + Sync> Sync for ABox<T, A> {} // A: Sync is mainly required to safely clone/default allocator

impl<T: ?Sized, A: Free> Drop for ABox<T, A> {
    fn drop(&mut self) {
        let layout = self.layout();
        // SAFETY: ✔️ `self` is going out of scope, nothing else will ever access `*self.data` again
        unsafe { self.data.as_ptr().drop_in_place() };
        // SAFETY: ✔️ we previously allocated `*self.data` with `(self.allocator, self.layout)` and will never access that allocation again
        unsafe { self.allocator.free(self.data.cast(), layout) };
    }
}

impl<T: ?Sized, A: Free> ABox<T, A> {
    /// Retrieve the [`fat::Free`] (+ [`fat::Alloc`] + [`fat::Realloc`] + ...) associated with this [`ABox`].
    #[inline(always)] pub fn allocator(this: &Self) -> &A { &this.allocator }
    #[inline(always)] pub(super) fn data(&self) -> NonNull<T> { self.data }
    #[inline(always)] pub(super) unsafe fn set_data(&mut self, data: NonNull<T>) { self.data = data; }
    #[inline(always)] pub(super) fn layout(&self) -> Layout { Layout::for_value(&**self) }

    /// Construct an [`ABox`] from a pointer to raw data and an allocator that can free it.
    ///
    /// ## Safety
    /// *   `data` must point to a valid and deferencable `T` (e.g. initialized or [`MaybeUninit`])
    /// *   `data` and it's [`Layout`] must be safely freeable via `allocator`
    /// *   [`ABox`] takes exclusive ownership over `data`
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let b = ABox::new_in(42_u32, Malloc);
    /// let (data, allocator) = ABox::into_raw_with_allocator(b);
    ///
    /// let b = unsafe { ABox::from_raw_in(data, allocator) };
    /// ```
    pub unsafe fn from_raw_in(data: NonNull<T>, allocator: A) -> Self {
        Self { data, allocator, _phantom: PhantomData }
    }

    const ASSERT_A_IS_ZST_FROM_RAW : () = assert!(size_of::<A>() == 0, "A is not a ZST - it is unlikely that `data` happens to be compatible with `A::default()`.  Prefer `ABox::from_raw_in` to specify an allocator instead.");
    /// Construct an [`ABox`] from a pointer to raw data.
    ///
    /// ## Failure modes
    /// *   Fails to compile if `A` isn't a ZST (you likely need a specific allocator, not just `A::default()`)
    ///
    /// ## Safety
    /// *   `data` must point to a valid and deferencable `T` (e.g. initialized or [`MaybeUninit`])
    /// *   `data` and it's [`Layout`] must be safely freeable via `A::default()`
    /// *   [`ABox`] takes exclusive ownership over `data`
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let b = ABox::new_in(42_u32, Malloc);
    /// let data = ABox::into_raw(b);
    ///
    /// let b = unsafe { ABox::<_, Malloc>::from_raw(data) };
    /// ```
    pub unsafe fn from_raw(data: NonNull<T>) -> Self where A : Default {
        let _ = Self::ASSERT_A_IS_ZST_FROM_RAW;
        // SAFETY: ✔️ same preconditions as documented
        unsafe { Self::from_raw_in(data, A::default()) }
    }

    /// Decompose an [`ABox`] into a pointer to raw data and an allocator that can free it.
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let b = ABox::new_in(42_u32, Malloc);
    ///
    /// let (data, allocator) = ABox::into_raw_with_allocator(b);
    ///
    /// let b = unsafe { ABox::from_raw_in(data, allocator) };
    /// ```
    pub fn into_raw_with_allocator(this: Self) -> (NonNull<T>, A) {
        let this        = ManuallyDrop::new(this);
        let data        = this.data;
        // SAFETY: ✔️ `this.allocator` will never be read again, including for Drop
        let allocator   = unsafe { core::ptr::read(&this.allocator) };
        (data, allocator)
    }

    const ASSERT_A_IS_ZST_INTO_RAW : () = assert!(size_of::<A>() == 0, "A is not a ZST - it is unlikely that `data` can be freed with anything but the discarded allocator.  Prefer `ABox::into_raw_with_allocator` to acquire `data`'s allocator as well.");
    /// Decompose an [`ABox`] into a pointer to raw data.
    ///
    /// ## Failure modes
    /// *   Fails to compile if `A` isn't a ZST (you likely need a specific allocator, not just `A::default()`, to free the returned data)
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let b = ABox::new_in(42_u32, Malloc);
    ///
    /// let data = ABox::into_raw(b);
    ///
    /// let b = unsafe { ABox::<_, Malloc>::from_raw(data) };
    /// ```
    pub fn into_raw(this: Self) -> NonNull<T> {
        let _ = Self::ASSERT_A_IS_ZST_INTO_RAW;
        Self::into_raw_with_allocator(this).0
    }

    /// Leak an [`ABox`] into an exclusive reference to it's data.
    ///
    /// As the reference may narrow the [spatial provenance](https://doc.rust-lang.org/std/ptr/index.html#provenance) to
    /// not include the full memory allocation provided by `A`, I'd argue it's likely unsound to ever pass said reference
    /// back into [`ABox::from_raw`].  This also doesn't guard against non-zero sized allocators like [`ABox::into_raw`]
    /// does.  If you wish to ever "unleak" the box, strongly consider using [`ABox::into_raw_with_allocator`] instead.
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let b = ABox::new_in(42_u32, Malloc);
    ///
    /// let v : &'static u32 = ABox::leak(b);
    /// ```
    pub fn leak<'a>(this: Self) -> &'a mut T where A: 'a {
        let mut raw = ABox::into_raw_with_allocator(this).0;
        // SAFETY: ✔️ `raw` is guaranteed to point to a valid allocated `T`.  We just:
        // • Threw out the last means of deallocating it (`.1`)
        // • Threw out the last means of accessing it (consumed `this`)
        unsafe { raw.as_mut() }
    }
}

// TODO:
//  • [ ] downcast
//  • [ ] downcast_unchecked
//  • [ ] into_boxed_slice
//  • [ ] into_pin
//  • [ ] pin
//  • [ ] pin_in

impl<T, A: Free> ABox<T, A> {
    // Sized

    /// Move the value out of the [`ABox`] and onto the stack.  `A`'s allocation is freed.
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let b = ABox::new_in(42_u32, Malloc);
    ///
    /// let v : u32 = ABox::into_inner(b);
    /// ```
    pub fn into_inner(this: Self) -> T { Self::into_inner_with_allocator(this).0 }

    /// Move the value out of the [`ABox`] and onto the stack.  `A`'s allocation is freed.
    /// `A` is also returned, if you have use for it.
    ///
    /// ## Examples
    /// ```
    /// use ialloc::{allocator::c::Malloc, boxed::ABox};
    /// let b = ABox::new_in(42_u32, Malloc);
    ///
    /// let (v, a) : (u32, Malloc) = ABox::into_inner_with_allocator(b);
    ///
    /// let b = ABox::new_in(v, a);
    /// ```
    pub fn into_inner_with_allocator(this: Self) -> (T, A) {
        let layout = this.layout();
        let (ptr, allocator) = ABox::into_raw_with_allocator(this);
        // SAFETY: ✔️ ptr is guaranteed to point at a valid allocation of T
        let data = unsafe { ptr.as_ptr().read() };
        // SAFETY: ✔️ ptr is guaranteed to point at a valid allocation of T belonging to allocator (were decomposed from the same box) with the box-known layout
        unsafe { allocator.free(ptr.cast(), layout) };
        (data, allocator)
    }
}

impl<T, A: Free> ABox<MaybeUninit<T>, A> {
    // MaybeUninit<T>

    // XXX: make pub?
    pub(super) unsafe fn assume_init(self) -> ABox<T, A> {
        let (data, allocator) = ABox::into_raw_with_allocator(self);
        // SAFETY: ✔️ we just decomposed (data, allocator) from a compatible-layout box
        unsafe { ABox::from_raw_in(data.cast(), allocator) }
    }

    // XXX: make pub?
    pub(super) fn write(boxed: Self, value: T) -> ABox<T, A> {
        // SAFETY: ✔️ boxed.data is guaranteed to point at a valid allocation of T
        unsafe { boxed.data.as_ptr().write(MaybeUninit::new(value)) };
        // SAFETY: ✔️ we just wrote to `boxed`
        unsafe { boxed.assume_init() }
    }
}

impl<T, A: Free> ABox<[MaybeUninit<T>], A> {
    // [MaybeUninit<T>]

    // XXX: make pub?
    pub(crate) unsafe fn assume_init(self) -> ABox<[T], A> {
        let (data, allocator) = ABox::into_raw_with_allocator(self);
        let data = util::nn::slice_from_raw_parts(data.cast(), data.len());
        // SAFETY: ✔️ we just decomposed (data, allocator) from a compatible-layout box
        unsafe { ABox::from_raw_in(data, allocator) }
    }
}
