@pushd "%~dp0.." && setlocal

"%WINDIR%\System32\bash" --login -c 'cargo test'
@if ERRORLEVEL 1 goto :die

"%WINDIR%\System32\bash" --login -c 'cargo build --all-targets --release'
@if ERRORLEVEL 1 goto :die

:die
@popd && endlocal && exit /b %ERRORLEVEL%
