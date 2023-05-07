@pushd "%~dp0.." && setlocal

:: Stable
@call :run-windows cargo test                                                                           || goto :die
@call :run-windows cargo test --all-features                                                            || goto :die
@call :run-windows cargo test --lib --no-default-features                                               || goto :die
@call :run-windows cargo test --lib --no-default-features --features alloc                              || goto :die
@call :run-windows cargo test --lib --no-default-features --features win32                              || goto :die
@call :run-windows cargo test --lib --no-default-features --features msvc                               || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc"                 || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c89"             || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c99"             || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c11"             || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c17"             || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c23"             || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c++98"           || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c++03"           || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c++11"           || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c++14"           || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c++17"           || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c++20"           || goto :die
@call :run-windows cargo test --lib --no-default-features --features "alloc win32 msvc c++23"           || goto :die
@call :run-windows cargo doc                                                                            || goto :die

:: Nightly
@call :run-windows cargo +nightly test                                                                  || goto :die
@call :run-windows cargo +nightly test --all-features                                                   || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features                                      || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features alloc                     || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features win32                     || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features msvc                      || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc"        || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c89"    || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c99"    || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c11"    || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c17"    || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c23"    || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c++98"  || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c++03"  || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c++11"  || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c++14"  || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c++17"  || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c++20"  || goto :die
@call :run-windows cargo +nightly test --lib --no-default-features --features "alloc win32 msvc c++23"  || goto :die
@call :run-windows cargo +nightly doc                                                                   || goto :die

:: Stable (Linux)
@call :run-linux   cargo test                                                                           || goto :die
@call :run-linux   cargo test --all-features                                                            || goto :die
@call :run-linux   cargo test --lib --no-default-features                                               || goto :die
@call :run-linux   cargo test --lib --no-default-features --features alloc                              || goto :die
@call :run-linux   cargo test --lib --no-default-features --features win32                              || goto :die
@call :run-linux   cargo test --lib --no-default-features --features msvc                               || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc"                 || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c89"             || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c99"             || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c11"             || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c17"             || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c23"             || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c++98"           || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c++03"           || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c++11"           || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c++14"           || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c++17"           || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c++20"           || goto :die
@call :run-linux   cargo test --lib --no-default-features --features "alloc win32 msvc c++23"           || goto :die
@call :run-linux   cargo doc                                                                            || goto :die

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
