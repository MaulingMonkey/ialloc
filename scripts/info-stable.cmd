@pushd "%~dp0.." && setlocal

rustc +stable -V
cargo +stable -V
ver

@popd && endlocal && exit /b %ERRORLEVEL%
