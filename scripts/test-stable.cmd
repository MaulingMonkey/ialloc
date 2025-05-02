@pushd "%~dp0.." && setlocal

cargo +stable test
@if ERRORLEVEL 1 goto :die

cargo +stable build --all-targets
@if ERRORLEVEL 1 goto :die

:die
@popd && endlocal && exit /b %ERRORLEVEL%
