@pushd "%~dp0.." && setlocal

rustc -V
cargo -V
ver

@popd && endlocal && exit /b %ERRORLEVEL%
