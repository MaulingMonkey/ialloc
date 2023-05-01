# ialloc

Allocator interface traits for Rust

<!--
[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/ialloc.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/ialloc)
[![crates.io](https://img.shields.io/crates/v/ialloc.svg)](https://crates.io/crates/ialloc)
[![docs.rs](https://docs.rs/ialloc/badge.svg)](https://docs.rs/ialloc)
[![License](https://img.shields.io/crates/l/ialloc.svg)](https://github.com/MaulingMonkey/ialloc)
[![Build Status](https://github.com/MaulingMonkey/ialloc/workflows/Rust/badge.svg)](https://github.com/MaulingMonkey/ialloc/actions?query=workflow%3Arust)
-->

## Raison d'être
*   Why not [`core::alloc::Allocator`](https://doc.rust-lang.org/core/alloc/trait.Allocator.html) / [`allocator_api`](https://github.com/rust-lang/rust/issues/32838)?
    *   7+ years unstabilized and counting.
    *   I want container allocations this decade, thanks!
    *   We can aim to be compatible and interopable with it, if/when it stabilizes, or via separate nightly crates.
*   Why not [`core::alloc::GlobalAlloc`](https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html)?
    *   Win32 [`FreeSid`](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-freesid)
        has no equivalent arbitrary allocation function to implement
        [`GlobalAlloc::alloc`](https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html#tymethod.alloc) with.
    *   <code>[bgfx](https://github.com/bkaradzic/bgfx#readme)::[alloc](https://bkaradzic.github.io/bgfx/bgfx.html#bgfx::alloc__uint32_t)</code>
        has no equivalent arbitrary free function to implement
        [`GlobalAlloc::dealloc`](https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html#tymethod.dealloc) with.
    *   Split alloc/free/realloc traits will help avoid bugs and misuse!
    *   That said, we can aim to be compatible and interopable with it.
*   Rust-style traits are annoying to adapt to C-style allocators.  This provides more C-friendly traits as options.

## Out of scope
*   Actual implementations of these traits, including for `core`/`alloc`/`std`.
*   NUMA?  Although ask again later.
*   Physical GPU memory allocation, probably.  Might warrant a related crate?
*   ID/handle allocation, perhaps.  Might warrant a related crate?



## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.



## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
