setlocal
@rem turbojpegƒrƒ‹ƒh‚É¸”s‚·‚éê‡‚Éİ’è‚·‚é
@rem set CMAKE_GENERATOR=NMake Makefiles
cargo build --target=i686-pc-windows-msvc --release
@rem cargo build --target=x86_64-pc-windows-msvc --release
endlocal
pause
