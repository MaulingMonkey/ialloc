@pushd "%~dp0.." && setlocal

cargo test
@if ERRORLEVEL 1 goto :die

cargo build --all-targets --release
@if ERRORLEVEL 1 goto :die

:die
@popd && endlocal && exit /b %ERRORLEVEL%
