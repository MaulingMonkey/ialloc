// `cfg`s set by build script:
// • `msvc`                             if feature + target_env
// • `c89` ..= `c23`                    if feature + cc feature test
// • `cpp98` ..= `cpp23`                if feature + cc feature test
// • `allocator_api` = `"1.50"`         if allocator_api has the same shape it had in 1.50
// • `allocator_api` = `"unstable"`     if allocator_api requires #![feature(allocator_api)]

use std::env::var_os;
use std::ffi::OsStr;
use std::io;
use std::process::{Command, Stdio};

#[cfg(feature = "cc")] const C : &[&str] = &[
    //"src/allocator/c/ffi.c",
];

#[cfg(feature = "cc")] const CPP : &[&str] = &[
    "src/allocator/cpp/ffi.cpp",
];



fn main() {
    if feature_test("allocator_api_1_50_stable") {
        println!("cargo:rustc-cfg=allocator_api=\"*\"");
        println!("cargo:rustc-cfg=allocator_api=\"1.50\"");
    } else if feature_test("allocator_api_1_50_unstable") {
        println!("cargo:rustc-cfg=allocator_api=\"*\"");
        println!("cargo:rustc-cfg=allocator_api=\"1.50\"");
        println!("cargo:rustc-cfg=allocator_api=\"unstable\"");
    }
    if var_os("CARGO_CFG_TARGET_ENV").as_deref() == Some(OsStr::new("msvc")) && var_os("CARGO_FEATURE_MSVC").is_some() {
        println!("cargo:rustc-cfg=msvc");
    }
    use_cc();
}

fn use_cc() {
    #[cfg(feature = "cc")] {
        for src in CPP { println!("cargo:rerun-if-changed={src}") }

        let mut c   = cc::Build::new(); c   .cpp(false);
        let mut cpp = cc::Build::new(); cpp .cpp(true);

        let msvc        = cpp.get_compiler().is_like_msvc();
        //let clang       = cpp.get_compiler().is_like_clang();
        //let gnu         = cpp.get_compiler().is_like_gnu();

        macro_rules! flag { ($cc:expr, $($tt:tt)*) => {{
            let cc : &mut cc::Build = &mut $cc;
            let flag = format!($($tt)*);
            let supported = cc.is_flag_supported(&flag).unwrap_or_default();
            if supported { cc.flag(&flag); }
            supported
        }}}

        fn skip_until<I: Iterator>(peek: &mut core::iter::Peekable<I>, mut cond: impl FnMut(&I::Item) -> bool) {
            while peek.next_if(|i| !cond(i)).is_some() {}
        }

        // define standards
        let mut c_standards     = "23 17 11 99 89".split(' ').peekable();
        let mut cpp_standards   = "23 20 17 14 11 03 98".split(' ').peekable();

        // skip unconfigured standards
        skip_until(&mut   c_standards, |yy| var_os(format!("CARGO_FEATURE_C{yy}"  )).is_some());
        skip_until(&mut cpp_standards, |yy| var_os(format!("CARGO_FEATURE_C++{yy}")).is_some());

        let skip_cc = var_os("CARGO_CFG_SKIP_CC").is_some();
        if !skip_cc {
            // skip unsupported standards
            if msvc {
                skip_until(&mut   c_standards, |yy| flag!(c,   "/std:c{yy}"));
                skip_until(&mut cpp_standards, |yy| flag!(cpp, "/std:c++{yy}"));
            } else {
                skip_until(&mut   c_standards, |yy| flag!(c,   "-std=c{yy}"));
                skip_until(&mut cpp_standards, |yy| flag!(cpp, "-std=c++{yy}") || match *yy {
                    "23" => flag!(cpp, "-std=c++2b"),
                    "20" => flag!(cpp, "-std=c++2a"),
                    _yy  => false,
                });
            }
        }

        for yy in cpp_standards { println!("cargo:rustc-cfg=cpp{yy}") }
        for yy in c_standards   { println!("cargo:rustc-cfg=c{yy}") }

        let version = env!("CARGO_PKG_VERSION").replace('.', "_").replace('-', "_");
        let prefix  = format!("ialloc_{version}_");
        let cpplib  = format!("ialloc_{version}_cpp");
        let clib    = format!("ialloc_{version}_c");

        if !skip_cc {
            cpp.define("IALLOC_PREFIX", &*prefix);
            for src in CPP { cpp.file(src); }
            for src in C   { c  .file(src); }
            if !CPP.is_empty() { cpp.compile(&cpplib); println!("cargo:rustc_link_lib=static={cpplib}"); }
            if !C  .is_empty() { c.compile(&clib);     println!("cargo:rustc_link_lib=static={clib}"); }
        }

        println!("cargo:rustc-env=IALLOC_PREFIX={prefix}");
    }
}

fn feature_test(feature: &str) -> bool {
    feature_test_impl(feature).unwrap_or_else(|err| panic!("error testing feature {feature:?}: {err:?}"))
}

fn feature_test_impl(feature: &str) -> Result<bool, io::Error> {
    let mut rustc = Command::new("rustc");
    rustc
        .arg("--crate-name").arg(format!("feature_test_{feature}"))
        .arg("--crate-type=lib")
        .arg("--out-dir").arg(var_os("OUT_DIR").unwrap())
        .arg("--target").arg(var_os("TARGET").unwrap())
        .arg("--emit=llvm-ir")
        .arg(format!("build/feature/test/{feature}.rs"))
        .stderr(Stdio::null()).stdout(Stdio::null()) // XXX: these mostly just clutter the build log
        ;
    Ok(rustc.status()?.success())
}
