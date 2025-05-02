@pushd "%~dp0.." && setlocal

cargo test  --target=i686-pc-windows-msvc
@if ERRORLEVEL 1 goto :die

cargo build --target=i686-pc-windows-msvc --all-targets --release
@if ERRORLEVEL 1 goto :die

:die
@popd && endlocal && exit /b %ERRORLEVEL%
