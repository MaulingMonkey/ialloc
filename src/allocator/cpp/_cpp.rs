#![cfg(cpp98)]
#![cfg_attr(doc_cfg = "*", doc(cfg(feature = "c++98")))]
//! [`StdAllocator<c_char>`] (C++98), <code>[NewDelete]{,[Array](NewDeleteArray)}</code> (C++98), <code>NewDelete{[Aligned](NewDeleteAligned),[ArrayAligned](NewDeleteArrayAligned)}</code> (C++17)



mod ffi;

#[cfg(cpp98)] mod new_delete;
#[cfg(cpp98)] pub use new_delete::*;

#[cfg(cpp98)] mod new_delete_array;
#[cfg(cpp98)] pub use new_delete_array::*;

#[cfg(cpp98)] mod std_allocator;
#[cfg(cpp98)] pub use std_allocator::*;

#[cfg(cpp17)] mod new_delete_aligned;
#[cfg(cpp17)] mod new_delete_array_aligned;

#[cfg(cpp17)] pub use new_delete_aligned::*;
#[cfg(cpp17)] pub use new_delete_array_aligned::*;
