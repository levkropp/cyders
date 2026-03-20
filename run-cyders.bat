@echo off
REM Cyders Launcher - Runs the emulator with firmware
REM Usage: run-cyders.bat [firmware.bin] [firmware.elf]

title Cyders - CYD Emulator

REM Check if VCRUNTIME140.dll is available
where VCRUNTIME140.dll >nul 2>&1
if errorlevel 1 (
    echo.
    echo ========================================
    echo WARNING: VCRUNTIME140.dll not found!
    echo ========================================
    echo.
    echo This is the Visual C++ 2015-2022 Redistributable.
    echo.
    echo Download from:
    echo https://aka.ms/vs/17/release/vc_redist.x64.exe
    echo.
    echo Or install via: winget install Microsoft.VCRedist.2015+.x64
    echo.
    echo Press any key to try running anyway...
    pause >nul
    echo.
)

REM Default firmware paths if not specified
if "%~1"=="" (
    set FIRMWARE=..\cyd-emulator\test-firmware\60-lvgl-demo\build\lvgl-demo.bin
    set ELF=..\cyd-emulator\test-firmware\60-lvgl-demo\build\lvgl-demo.elf
    set FIRMWARE_NAME=LVGL Comprehensive Demo
) else (
    set FIRMWARE=%~1
    set ELF=%~2
    set FIRMWARE_NAME=Custom Firmware
)

REM Check if firmware exists
if not exist "%FIRMWARE%" (
    echo.
    echo ========================================
    echo Firmware not found!
    echo ========================================
    echo %FIRMWARE%
    echo.
    echo Building firmware automatically...
    echo This will take 2-3 minutes on first build.
    echo.

    REM Try to build automatically with UV (isolated Python)
    if exist "..\cyd-emulator\test-firmware\60-lvgl-demo\build-uv.bat" (
        echo Running UV build script...
        echo This will use an isolated Python environment.
        echo.
        call "..\cyd-emulator\test-firmware\60-lvgl-demo\build-uv.bat"

        if errorlevel 1 (
            echo.
            echo ========================================
            echo Build failed!
            echo ========================================
            echo.
            echo Please follow the instructions above.
            echo.
            pause
            exit /b 1
        )

        REM Check if build actually created the files
        if not exist "%FIRMWARE%" (
            echo.
            echo ERROR: Build completed but firmware not found!
            echo %FIRMWARE%
            echo.
            pause
            exit /b 1
        )

        echo.
        echo ========================================
        echo Build complete! Starting emulator...
        echo ========================================
        echo.
        set FIRMWARE_NAME=LVGL Comprehensive Demo
        timeout /t 2 /nobreak >nul
    ) else (
        echo ERROR: Build script not found!
        echo.
        echo Please ensure the project structure is intact:
        echo   ..\cyd-emulator\test-firmware\60-lvgl-demo\build-uv.bat
        echo.
        pause
        exit /b 1
    )
)

echo.
echo ========================================
echo Cyders - CYD Emulator
echo ========================================
if defined FIRMWARE_NAME echo Loading: %FIRMWARE_NAME%
echo Firmware: %FIRMWARE%
if "%ELF%" NEQ "" echo ELF:      %ELF%
echo.
echo Controls:
echo   ESC - Quit
echo   P   - Pause/Resume
echo   F10 - Toggle info panel
echo   Mouse Click - Touch input
echo.
echo LVGL Demo Features:
echo   - 5 interactive screens
echo   - Functional buttons and widgets
echo   - Smooth animations
echo   - Full touch input support
echo   - Click navigation buttons on home!
echo.
echo ========================================
echo.

REM Run the emulator
if "%ELF%" NEQ "" (
    target\release\cyders.exe --firmware "%FIRMWARE%" --elf "%ELF%"
) else (
    target\release\cyders.exe --firmware "%FIRMWARE%"
)

set EXIT_CODE=%errorlevel%

if %EXIT_CODE% NEQ 0 (
    echo.
    echo ========================================
    echo Emulator exited with code: %EXIT_CODE%
    echo ========================================
    pause
)

exit /b %EXIT_CODE%
