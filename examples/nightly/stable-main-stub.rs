#[cfg(not(feature = "nightly"))] fn main() {
    eprintln!(concat!(
        "\u{001B}[31;1merror\u{001B}[37m:\u{001B}[0m this example requires nightly features and is stubbed out by default.  To run, use e.g.:\n",
        "  cargo +nightly run --features nightly --example ", env!("CARGO_BIN_NAME"),
    ));
}
