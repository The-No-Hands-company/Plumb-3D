@echo off
echo =========================================
echo  Building Plumb 3D Suite (Release Mode)  
echo =========================================

cargo build --release --workspace

if %ERRORLEVEL% equ 0 (
    echo.
    echo [SUCCESS] Build completed successfully.
    echo Binary available at: target\release\app.exe
) else (
    echo.
    echo [ERROR] Build failed with code %ERRORLEVEL%
    exit /b %ERRORLEVEL%
)
