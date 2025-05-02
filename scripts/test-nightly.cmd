@pushd "%~dp0.." && setlocal

cargo +nightly test
@if ERRORLEVEL 1 goto :die

cargo +nightly build --all-targets
@if ERRORLEVEL 1 goto :die

@call "scripts\doc.cmd"
@if ERRORLEVEL 1 goto :die

:die
@popd && endlocal && exit /b %ERRORLEVEL%
