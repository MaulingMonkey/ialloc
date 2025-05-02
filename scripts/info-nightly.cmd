@pushd "%~dp0.." && setlocal

rustc +nightly -V
cargo +nightly -V
ver

@popd && endlocal && exit /b %ERRORLEVEL%
