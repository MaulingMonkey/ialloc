const CPP : &'static [&'static str] = &[
    "src/allocator/cpp/ffi.cpp",
];

fn main() {
    build_cpp();
}

fn build_cpp() {
    if std::env::var_os("CARGO_FEATURE_C++").is_none() { return }
    for src in CPP { println!("cargo:rerun-if-changed={src}") }

    let mut cc = cc::Build::new();

    let standard =
        if      std::env::var_os("CARGO_FEATURE_C++23").is_some()   { "c++23" }
        else if std::env::var_os("CARGO_FEATURE_C++20").is_some()   { "c++20" }
        else if std::env::var_os("CARGO_FEATURE_C++17").is_some()   { "c++17" }
        else if std::env::var_os("CARGO_FEATURE_C++14").is_some()   { "c++14" }
        else if std::env::var_os("CARGO_FEATURE_C++11").is_some()   { "c++11" }
        else if std::env::var_os("CARGO_FEATURE_C++03").is_some()   { "c++03" }
        else                                                        { "c++98" };


    if cc.get_compiler().is_like_msvc() {
        cc.flag(&format!("/std:{standard}"));
    } else if cc.get_compiler().is_like_clang() || cc.get_compiler().is_like_gnu() {
        cc.flag(&format!("-std={standard}"));
    } else {
        // ???
    }

    let version = env!("CARGO_PKG_VERSION").replace(".", "_").replace("-", "_");
    let prefix  = format!("ialloc_{version}_");
    let libname = format!("ialloc_{version}_cpp");

    cc.define("IALLOC_PREFIX", &*prefix);
    for src in CPP { cc.file(src); }
    cc.compile(&libname);

    println!("cargo:rustc-env=IALLOC_PREFIX={prefix}");
    println!("cargo:rustc_link_lib=static={libname}");
}
