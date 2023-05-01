@pushd "%~dp0.." && setlocal

:: Stable
cargo test
@if ERRORLEVEL 1 goto :die
cargo build --all-targets --release
@if ERRORLEVEL 1 goto :die

:: Nightly
@cargo +nightly >NUL 2>NUL || ver>NUL && goto :die
cargo +nightly build --features nightly --all-targets
@if ERRORLEVEL 1 goto :die
cargo +nightly run   --features nightly --example malloc
@if ERRORLEVEL 1 goto :die

:die
@popd && endlocal && exit /b %ERRORLEVEL%
