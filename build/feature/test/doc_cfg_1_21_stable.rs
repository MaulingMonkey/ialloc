// #![feature(doc_cfg)] // https://github.com/rust-lang/rust/issues/43781

#[doc(cfg(windows))]
pub fn foo() {}
