## Features
| Feature   | Description                       | Additional Requirements |
| ----------| ----------------------------------| ------------------------|
| `"alloc"` | [`alloc`] crate support           |
| `"c89"`   | C89 standard library support      | [`cc`](https://github.com/rust-lang/cc-rs) finds a C89+ compatible compiler
| `"c11"`   | C11 standard library support      | [`cc`](https://github.com/rust-lang/cc-rs) finds a C11+ compatible compiler
| `"c23"`   | C23 standard library support      | [`cc`](https://github.com/rust-lang/cc-rs) finds a C23+ compatible compiler
| `"c++98"` | C++98 standard library support    | [`cc`](https://github.com/rust-lang/cc-rs) finds a C++98+ compatible compiler
| `"c++17"` | C++17 standard library support    | [`cc`](https://github.com/rust-lang/cc-rs) finds a C++17+ compatible compiler
| `"msvc"`  | MSVC-specific library support     | <code>[target_env](https://doc.rust-lang.org/reference/conditional-compilation.html#target_env) = `"msvc"`</code>
| `"win32"` | Windows-specific allocators       | <code>[target_os](https://doc.rust-lang.org/reference/conditional-compilation.html#target_os) = `"windows"`</code>
