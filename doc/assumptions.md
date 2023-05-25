Assumptions [`ialloc`](crate) makes, likely resulting in unsoundness if broken.

Because I'm evil and wish to acclimatize and desensitize the reader into condoning my evil,
I start with the mild, uncontroversial assumptions first before working my way up.



# 1. "Uncontroversial"
These assumptions should be relatively uncontroversial.

#### Bug Free Dependencies
Dependencies are generally assumed to be bug-free &mdash; unless proven otherwise &mdash; including:
*   The Rust Standard Library ([`core`], [`alloc`], [`std`])
*   Various crates ([`bytemuck`], [`libc`])
*   "System" Libraries (`kernel32.dll`, `glibc`, `msvcrt`, `libc++`, etc.)

Attempting to reason about any of these things possibly being broken in any way
quickly becomes impossible without marking all code everywhere `unsafe`, which
would defeat the purpouse of `unsafe` and be counterproductive.

On the other hand, specific known bugs - such as `aligned_alloc` on OS X not
honoring alignment requests above 2 GiB, or perhaps a specific platform not zeroing
memory returned by `calloc` correctly, may (or may not) be worth guarding against.

Finally, improving test coverage to hopefully catch/prove bugs - including bugs
in what I use from dependencies - is outright encouraged.

#### Bug Free `unsafe` traits
*   Implementations of `unsafe` traits are assumed to uphold their documented safety invariants.
*   In some cases, unstated-but-obvious assumptions might be considered documentation bugs.
*   These assumptions do *not* extend to *safe* traits, which are merely assumed to be *sound* (a *very* low bar.)



# 2. "Mildly Controversial"

#### C and C++ Standard Libraries have Thread Safe Allocators
The following make thread safety assumptions about their underlying standard libraries:
*   <code>[allocator]::[c](allocator::c)::\*</code>
*   <code>[allocator]::[cpp](allocator::cpp)::\*</code>

The C and C++ standards predate the widespread adoption of multithreading, and early standards do not document thread safety requirements for allocation functions.
Worse still, explicitly "single threaded" variants of standard libraries existed before e.g. Visual Studio 2005.
While modern codebases and toolchain defaults will no longer pick these by default, there's no technical reason these would remain impossible to select.
It would be foolish to do so &mdash; with third party middleware and system libraries routinely spawning their own threads, breaking more than just [`ialloc`](crate) &mdash; but not impossible.
As such, this assumption is arguably a little unsound.

An effort is made to detect and reject single threaded stdlibs at build time (see `ffi.cpp`'s checks for `_MT`).

Additional checks are welcome.

#### Win32 Allocators are Thread Safe
The following make thread safety assumptions about Windows APIs:
*   <code>[allocator]::[win32](allocator::win32)::\*</code>

Microsoft's embrace of backwards compatability and stability despite the outrageous and foolish assumptions of userspace applications, ensure these are &mdash; effectively &mdash; guaranteed to be thread safe in practice.
However, Microsoft's documentation often lacks explicit guarantees about thread safety, merely implying it via documentation that states what *isn't* thread safe
&mdash; e.g. [`HeapCreate`](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapcreate)'s [Remarks](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapcreate#remarks)
note that using `HEAP_NO_SERIALIZE` eliminates the mutexes required for thread safety, implying *not* using it *is* thread safe.

Some of these functions *may not* have been thread safe pre-NT4 (e.g. Windows 95/98/ME?, or in the cooperative multiprocessing environment of Windows 3.1).
I'm willing to consider the introduction of new features (`"assume-windows-nt4"`?) that would gate [`Send`]/[`Sync`] for these allocators upon request.
However, as rust-lang itself only [officially supports Windows 7+](https://doc.rust-lang.org/rustc/platform-support.html), I'd rather not do that work for an incredibly niche need without at least one user.



# 3. "Moderately Controversial"

#### `OsStr` and `Path` Layout

I assume both [`std::ffi::OsStr`] and [`std::path::Path`] are layout compatible with `[u8]`, `[u16]`, or `[u32]`.

In practice, I believe the existence of APIs such as [`std::ffi::OsStr::to_str`] have ossified their internals as ≈`[u8]`s.

What makes this rather naughty is the fact that the stdlib has taken pains to avoid explicitly documenting such a thing:

```rust,ignore
// FIXME:
// `OsStr::from_inner` current implementation relies
// on `OsStr` being layout-compatible with `Slice`.
// When attribute privacy is implemented, `OsStr` should be annotated as `#[repr(transparent)]`.
// Anyway, `OsStr` representation and layout are considered implementation details, are
// not documented and must not be relied upon.
pub struct OsStr {
    inner: Slice,
}
```
[`std/src/ffi/os_str.rs` lines 113-121](https://github.com/rust-lang/rust/blob/eb9da7bfa375ad58bcb946115f3191a2756785e5/library/std/src/ffi/os_str.rs#L113-L121)

```rust,ignore
// FIXME:
// `Path::new` current implementation relies
// on `Path` being layout-compatible with `OsStr`.
// When attribute privacy is implemented, `Path` should be annotated as `#[repr(transparent)]`.
// Anyway, `Path` representation and layout are considered implementation detail, are
// not documented and must not be relied upon.
pub struct Path {
    inner: OsStr,
}
```
[`std/src/path.rs` lines 1165-1173](https://github.com/rust-lang/rust/blob/eb9da7bfa375ad58bcb946115f3191a2756785e5/library/std/src/path.rs#L1165-L1173)

If pointer metadata APIs stabilize, I'll attempt to use those instead, as this assumption is currently *only* needed when creating boxed copies of these types.
Alternatively, I may attempt to limit such APIs to using [`Global`](crate::allocator::alloc::Global)-compatible allocators and instead using [`std`].
