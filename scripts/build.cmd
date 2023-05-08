@pushd "%~dp0.." && setlocal

:: Nightly
@cargo +nightly >NUL 2>NUL || ver>NUL && goto :skip-nightly
@call :run-windows cargo +nightly test                                          || goto :die
@call :run-windows cargo +nightly build --all-targets                           || goto :die
@call :run-windows cargo +nightly doc                                           || goto :die
:skip-nightly

:: Stable
@call :run-windows cargo test                                                   || goto :die
@call :run-windows cargo build --all-targets --release                          || goto :die

:: Stable (Linux)
@"%WINDIR%\System32\bash" --version >NUL 2>NUL || ver>NUL && goto :skip-stable-linux
@call :run-linux cargo test                                                     || goto :die
@call :run-linux cargo build --all-targets --release                            || goto :die
:skip-stable-linux

:die
@popd && endlocal && exit /b %ERRORLEVEL%



:run-windows
@echo %*
@%*
@exit /b %ERRORLEVEL%

:run-linux
@echo bash -c '%*'
@"%WINDIR%\System32\bash" --login -c '%*'
@exit /b %ERRORLEVEL%
