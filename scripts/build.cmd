@pushd "%~dp0.." && setlocal

:: Keep in sync with .github\workflows\rust.yml

:: Nightly
@cargo +nightly >NUL 2>NUL || ver>NUL && goto :skip-nightly
@call "scripts\test-nightly.cmd" || goto :die
:skip-nightly

:: MSRV
@call "scripts\test-msrv.cmd" || goto :die

:: MSRV (i686)
@call "scripts\test-msrv-i686.cmd" || goto :die

:: MSRV (WSL)
@"%WINDIR%\System32\bash" --version >NUL 2>NUL || ver>NUL && goto :skip-msrv-linux
@call "scripts\test-msrv-wsl.cmd" || goto :die
:skip-msrv-linux

:: Stable
@call "scripts\test-stable.cmd" || goto :die

:die
@popd && endlocal && exit /b %ERRORLEVEL%
