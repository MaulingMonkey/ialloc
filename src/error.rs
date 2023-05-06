//! [`ExcessiveAlignmentRequestedError`] (and any future error types)

use crate::Alignment;

use core::fmt::{self, Debug, Display, Formatter};



/// More alignment was requested than the allocator could support.
#[derive(Clone, Copy, Debug)] pub struct ExcessiveAlignmentRequestedError {
    pub requested: Alignment,
    pub supported: Alignment,
}

impl Display for ExcessiveAlignmentRequestedError { fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "requested {:?} alignment, but a maximum of {:?} is supported", self.requested, self.supported) } }
impl From<ExcessiveAlignmentRequestedError> for () { fn from(_: ExcessiveAlignmentRequestedError) -> Self { () } }
#[cfg(feature = "std")] impl std::error::Error for ExcessiveAlignmentRequestedError { fn description(&self) -> &str { "requested more alignment than was supported" } }
#[cfg(allocator_api = "*")] impl From<ExcessiveAlignmentRequestedError> for core::alloc::AllocError { fn from(_: ExcessiveAlignmentRequestedError) -> Self { core::alloc::AllocError } }
