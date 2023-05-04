## Features
| Feature   | Description                       | Additional Requirements |
| ----------| ----------------------------------| ------------------------|
| `"alloc"` | [`alloc`] crate allocators        |
| `"c"`     | C standard library allocators     |
| `"c++98"` | C++98 standard library allocators |
| `"c++17"` | C++17 aligned standard library allocators
| `"msvc"`  | MSVC-specific allocators          | <code>[target_env](https://doc.rust-lang.org/reference/conditional-compilation.html#target_env) = "msvc"</code>
| `"win32"` | Windows-specific allocators       | <code>[target_os](https://doc.rust-lang.org/reference/conditional-compilation.html#target_os) = "windows"</code>
