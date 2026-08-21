@echo off
REM Script to prepare native libraries for local C# development on Windows

setlocal enabledelayedexpansion

echo Preparing native libraries for C# development...

REM Get the script directory and project root
for %%A in ("%~dp0") do set "CSHARP_DIR=%%~dpA"
for %%A in ("%CSHARP_DIR%..") do set "PROJECT_ROOT=%%~dpA"

REM Build the Rust library
echo Building Rust library with FFI...
cd /d "%PROJECT_ROOT%"
call cargo build --release --features ffi

if errorlevel 1 (
    echo Failed to build Rust library
    exit /b 1
)

REM Prepare paths
set "LIB_FILE=%PROJECT_ROOT%target\release\aam_rs.dll"
set "RUNTIME_DIR=%CSHARP_DIR%runtimes\win-x64\native"

REM Create runtime directory if it doesn't exist
if not exist "%RUNTIME_DIR%" (
    mkdir "%RUNTIME_DIR%"
)

REM Copy the library
if exist "%LIB_FILE%" (
    echo Copying %LIB_FILE% to %RUNTIME_DIR%
    copy "%LIB_FILE%" "%RUNTIME_DIR%\" >nul
    echo OK: Native library copied successfully
) else (
    echo ERROR: Native library not found at %LIB_FILE%
    exit /b 1
)

echo.
echo Setup complete! You can now run C# tests and examples:
echo.
echo   cd csharp
echo   dotnet test
echo   dotnet run --project examples/Basic/AamCsharp.Basic.csproj
echo.

endlocal

