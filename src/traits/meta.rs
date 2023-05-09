//! Metadata traits common to all (de)allocators

use crate::*;
use crate::error::*;

use core::fmt::Debug;



/// Allocator metadata (are ZSTs supported, max allocation size/alignment, error type, etc.)
pub trait Meta {
    /// The error type returned from failed (re)allocation.
    type Error : Debug + From<ExcessiveAlignmentRequestedError> + From<ExcessiveSliceRequestedError>;

    #[doc(hidden)] // XXX: not sure if I want to actually support this.
    /// Indicates the minimum alignment this allocator should be expected to succeed in allocating.
    const MIN_ALIGN : Alignment = Alignment::MIN;

    /// Indicates the maximum alignment this allocator should be expected to succeed in allocating.
    /// Requesting an allocation with more alignment than this is almost certainly a bug.
    ///
    /// ## Safety
    /// *   It should be "safe" to attempt an allocation with larger alignment than this - however, such calls are unlikely to return anything other than <code>[Err]\(...\)</code>.
    /// *   [`thin`] style allocations don't parameterize alignment, and will simply return allocations with at most this much alignment.  The caller is responsible for ensuring that's sufficient.
    ///
    /// ## Use Cases
    /// *   A consistent threshhold for a wrapper to use a fallback allocator.
    /// *   Compile time assertions to prevent using e.g. <code>[ABox](crate::boxed::ABox)&lt;T&gt;</code> with allocators that could never support `T`.
    ///
    /// ## In practice
    /// | Platform                      | Common Values     |
    /// | ------------------------------| ------------------|
    /// | 32&zwj;-&zwj;bit&nbsp;Windows | `MEMORY_ALLOCATION_ALIGNMENT == 8`,  most [`thin`] allocators (including default `malloc`/`free`, `new`/`delete`, etc.) provide this much alignment.
    /// | 64&zwj;-&zwj;bit&nbsp;Windows | `MEMORY_ALLOCATION_ALIGNMENT == 16`, most [`thin`] allocators (including default `malloc`/`free`, `new`/`delete`, etc.) provide this much alignment.
    /// | C stdlib                      | <code>[Alignment]::[of](Alignment::of)::&lt;[max_align_t](https://en.cppreference.com/w/cpp/types/max_align_t)&gt;()</code>... at least in theory.<br><code>[Alignment]::[of](Alignment::of)::&lt;[f64]&gt;()</code> in practice?
    /// | C++ stdlib                    | <code>[Alignment]::[new](Alignment::new)\(\_\_STDCPP_DEFAULT_NEW_ALIGNMENT\_\_\)</code>... at least in theory
    /// | Rust (0+ bytes)               | [`Alignment::MAX`] (≈ <code>[usize::MAX]/2+1</code>, or 2 GiB on 32-bit)
    /// | Rust (1+ bytes)               | <code>[Alignment::MAX]/2</code> (≈ <code>[usize::MAX]/4+1</code>, or 1 GiB on 32-bit)
    const MAX_ALIGN : Alignment;

    /// Indicates the maximum size this allocator should be expected to succeed in allocating.
    /// Requesting an allocation larger than this is almost certainly a bug.
    ///
    /// ## Safety
    /// *   It should be "safe" to attempt an allocation with larger size than this - however, such calls are unlikely to return anything other than <code>[Err]\(...\)</code>.
    ///
    /// ## Use Cases
    /// *   A consistent threshhold for a wrapper to use a fallback allocator.
    /// *   Compile time assertions to prevent using e.g. <code>[ABox](crate::boxed::ABox)&lt;T&gt;</code> with allocators that could never support `T`.
    const MAX_SIZE : usize;

    /// Indicates if this allocator supports zero-sized allocations.
    /// This generally goes hand in hand with implementing [`zsty`].
    const ZST_SUPPORTED : bool;
}

impl<'a, A: Meta> Meta for &'a A {
    type Error                      = A::Error;
    const MAX_ALIGN     : Alignment = A::MAX_ALIGN;
    const MAX_SIZE      : usize     = A::MAX_SIZE;
    const ZST_SUPPORTED : bool      = A::ZST_SUPPORTED;
}
