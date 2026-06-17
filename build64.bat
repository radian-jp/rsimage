setlocal
@rem turbojpegƒrƒ‹ƒh‚É¸”s‚·‚éê‡‚Éİ’è‚·‚é
@rem set CMAKE_GENERATOR=NMake Makefiles
@rem cargo build --target=i686-pc-windows-msvc --release
cargo build --target=x86_64-pc-windows-msvc --release
endlocal
pause
