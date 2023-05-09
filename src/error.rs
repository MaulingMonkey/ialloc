//! [`ExcessiveAlignmentRequestedError`], [`ExcessiveSliceRequestedError`] (and any future error types)

use crate::Alignment;

#[cfg(doc)] use core::alloc::LayoutError;
use core::fmt::{self, Debug, Display, Formatter};



/// More alignment was requested than the allocator could support.
#[derive(Clone, Copy, Debug)] pub struct ExcessiveAlignmentRequestedError {
    pub requested: Alignment,
    pub supported: Alignment,
}

/// A slice large enough to result in a [`LayoutError`] was requested.
#[derive(Clone, Copy, Debug)] pub struct ExcessiveSliceRequestedError {
    pub requested: usize,
}

impl Display for ExcessiveAlignmentRequestedError { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "requested {:?} alignment, but a maximum of {:?} is supported", self.requested, self.supported) } }
impl Display for ExcessiveSliceRequestedError     { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "requested {} elements, but that would result in a LayoutError", self.requested) } }
impl From<ExcessiveAlignmentRequestedError> for () { fn from(_: ExcessiveAlignmentRequestedError) -> Self { () } }
impl From<ExcessiveSliceRequestedError    > for () { fn from(_: ExcessiveSliceRequestedError) -> Self { () } }
#[cfg(feature = "std")] impl std::error::Error for ExcessiveAlignmentRequestedError { fn description(&self) -> &str { "requested more alignment than was supported" } }
#[cfg(feature = "std")] impl std::error::Error for ExcessiveSliceRequestedError     { fn description(&self) -> &str { "requested too many elements" } }
#[cfg(allocator_api = "*")] impl From<ExcessiveAlignmentRequestedError> for core::alloc::AllocError { fn from(_: ExcessiveAlignmentRequestedError) -> Self { core::alloc::AllocError } }
#[cfg(allocator_api = "*")] impl From<ExcessiveSliceRequestedError    > for core::alloc::AllocError { fn from(_: ExcessiveSliceRequestedError    ) -> Self { core::alloc::AllocError } }
