@pushd "%~dp0.." && setlocal

@set RUSTFLAGS=--cfg skip_cc
cargo +nightly doc %*

@popd && endlocal && exit /b %ERRORLEVEL%
