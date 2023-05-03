use crate::*;

use core::alloc::*;
use core::num::NonZeroUsize;
use core::ops::Deref;



/// Like [`Layout`], but size is nonzero ([`Layout`] already requires a nonzero power of 2 alignment.)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)] #[repr(transparent)] pub struct LayoutNZ(Layout);

impl LayoutNZ {
    pub(crate) fn from_layout(layout: Layout) -> Result<Self, LayoutError> { if layout.size() == 0 { Err(ERROR_SIZE_0) } else { Ok(Self(layout)) } }
    pub fn from_size_align(size: NonZeroUsize, align: Alignment) -> Result<Self, LayoutError> { Self::from_layout(Layout::from_size_align(size.get(), align.as_usize())?) }

    pub fn new<T>() -> Result<Self, LayoutError> { Self::from_layout(Layout::new::<T>()) }
    pub fn for_value<T: ?Sized>(t: &T) -> Result<Self, LayoutError> { Self::from_layout(Layout::for_value(t)) }

    pub const fn align(&self)   -> Alignment    { unsafe { Alignment   ::new_unchecked(self.0.align()) } }
    pub const fn size(&self)    -> NonZeroUsize { unsafe { NonZeroUsize::new_unchecked(self.0.size() ) } }
}

impl From<LayoutNZ> for Layout { fn from(layout: LayoutNZ) -> Self { layout.0 } }
impl TryFrom<Layout> for LayoutNZ { fn try_from(layout: Layout) -> Result<Self, Self::Error> { Self::from_layout(layout) } type Error = LayoutError; }

impl Deref for LayoutNZ { type Target = Layout; fn deref(&self) -> &Self::Target { &self.0 } }



const ERROR_SIZE_0 : LayoutError = if let Err(e) = Layout::from_size_align(0, 0) { e } else { panic!("failed to construct ERROR_SIZE_0") };
