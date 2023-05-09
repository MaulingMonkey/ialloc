@pushd "%~dp0.." && setlocal

@set CARGO_TARGET_DIR=target/help
@set RUSTFLAGS=--cfg skip_cc
cargo +nightly doc %*

@popd && endlocal && exit /b %ERRORLEVEL%
