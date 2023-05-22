use crate::fat::*;
use crate::meta::*;
use crate::vec::AVec;



#[cfg(    global_oom_handling )] impl<T, A: Free + Alloc + Default + ZstSupported  > Default for AVec<T, A> { fn default() -> Self { Self::new() } }
#[cfg(not(global_oom_handling))] impl<T, A: Free + Alloc + Default + ZstInfalliable> Default for AVec<T, A> { fn default() -> Self { Self::new() } }

// Don't bother with `try_default` / `try_default_in` / `default_in`: these would just alias `try_new` / `try_new_in` / `new_in`
