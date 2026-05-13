@echo off
echo =========================================
echo   Build Bieu Thue XNK 2026 for Windows
echo =========================================
echo.

:: Check Rust
where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo [LOI] Chua cai Rust! Vui long cai tai: https://rustup.rs/
    pause
    exit /b 1
)

echo [OK] Rust da san sang
rustc --version
cargo --version
echo.

echo [*] Dang build (lan dau co the mat 5-10 phut)...
cargo build --release

if %errorlevel% neq 0 (
    echo.
    echo [LOI] Build that bai! Kiem tra log o tren.
    pause
    exit /b 1
)

echo.
echo =========================================
echo   Build THANH CONG!
echo   File: target\release\bieu-thue-xnk.exe
echo =========================================
echo.
echo Nhan Enter de mo thu ngay...
pause >nul
start "" "target\release\bieu-thue-xnk.exe"
