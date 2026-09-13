@echo off
setlocal EnableDelayedExpansion
chcp 65001 >nul 2>nul
for /f %%e in ('powershell -nop -c "[char]27"') do set "E=%%e"

title Amni-Code Installer
echo.
echo   !E![95m╔════════════════════════════════════════════════╗!E![0m
echo   !E![95m║!E![97;1m           ⚡  A M N I - C O D E                !E![0;95m║!E![0m
echo   !E![95m║!E![90m              Installer  v2.8.4                  !E![0;95m║!E![0m
echo   !E![95m╚════════════════════════════════════════════════╝!E![0m
echo.

set "H=%USERPROFILE%\.amni"
set "INST_EXE=!H!\amni-installer.exe"
set "REPO=%~dp0"
set "REPO=!REPO:~0,-1!"

REM ── Step 1: Rust (needed to compile; skip if pre-built installer already exists) ─
if exist "!INST_EXE!" goto :run_installer
if exist "!REPO!\installer\target\release\amni-installer.exe" (
    copy /Y "!REPO!\installer\target\release\amni-installer.exe" "!INST_EXE!" >nul
    goto :run_installer
)

<nul set /p =  !E![96m RUST !E![90m·····  !E![97mChecking Rust toolchain...  !E![0m
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo !E![93mmissing!E![0m
    echo.
    echo   !E![93mRust is not installed. Installing now — this takes ~2 min.!E![0m
    echo   !E![90m  (Rust is only needed once to build the installer binary)!E![0m
    echo.
    powershell -nop -ExecutionPolicy Bypass -Command ^
        "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe' -UseBasicParsing; ^
         Start-Process '%TEMP%\rustup-init.exe' -ArgumentList '-y','--default-toolchain','stable','--profile','minimal' -Wait; ^
         del '%TEMP%\rustup-init.exe'"
    set "PATH=%USERPROFILE%\.cargo\bin;!PATH!"
    where cargo >nul 2>nul
    if errorlevel 1 (
        echo.
        echo   !E![91m✗ Rust install failed. Please visit https://rustup.rs and install manually, then re-run this script.!E![0m
        pause & exit /b 1
    )
    echo   !E![92m✓ Rust installed!E![0m
) else (
    echo !E![92mfound!E![0m
)

REM ── Step 2: Git (to clone/pull repo; skip if we already have sources) ──────────
<nul set /p =  !E![96m  GIT !E![90m·····  !E![97mChecking Git...             !E![0m
where git >nul 2>nul
if %errorlevel% neq 0 (
    echo !E![93mmissing!E![0m
    echo   !E![93mGit not found. Attempting install via winget...!E![0m
    winget install --id Git.Git -e --source winget --silent --accept-package-agreements --accept-source-agreements >nul 2>nul
    where git >nul 2>nul
    if errorlevel 1 (
        echo.
        echo   !E![91m✗ Git install failed. Please install from https://git-scm.com then re-run.!E![0m
        start https://git-scm.com/download/win
        pause & exit /b 1
    )
    echo   !E![92m✓ Git installed!E![0m
) else (
    echo !E![92mfound!E![0m
)

REM ── Step 3: WebView2 Runtime (required for wry/tao window) ───────────────────
<nul set /p =  !E![96m  WV2 !E![90m·····  !E![97mChecking WebView2...        !E![0m
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>nul
if %errorlevel% neq 0 (
    reg query "HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>nul
)
if %errorlevel% neq 0 (
    echo !E![93mmissing!E![0m
    echo   !E![93mInstalling WebView2 Runtime (Microsoft Edge component)...!E![0m
    powershell -nop -ExecutionPolicy Bypass -Command ^
        "Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile '%TEMP%\MicrosoftEdgeWebview2Setup.exe' -UseBasicParsing; ^
         Start-Process '%TEMP%\MicrosoftEdgeWebview2Setup.exe' -ArgumentList '/silent /install' -Wait; ^
         del '%TEMP%\MicrosoftEdgeWebview2Setup.exe'"
    echo   !E![92m✓ WebView2 installed!E![0m
) else (
    echo !E![92mfound!E![0m
)

REM ── Step 4: Build installer binary ───────────────────────────────────────────
<nul set /p =  !E![96m BUILD !E![90m·····  !E![97mBuilding installer GUI...   !E![0m
pushd "!REPO!\installer"
cargo build --release --quiet >nul 2>&1
if errorlevel 1 (
    echo !E![91m✗ Build failed!E![0m
    echo.
    cargo build --release 2>&1
    popd
    pause & exit /b 1
)
popd
echo !E![92m✓!E![0m

if not exist "!H!" mkdir "!H!"
copy /Y "!REPO!\installer\target\release\amni-installer.exe" "!INST_EXE!" >nul

:run_installer
echo.
echo   !E![92;1m  ✓ Launching Amni-Code GUI Installer...!E![0m
echo.
"!INST_EXE!"
