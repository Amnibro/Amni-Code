@echo off
setlocal
title Amni-Code Local Server
set OLLAMA_PORT=11434
set LOCAL_PORT=8787

echo =======================================================
echo   Amni-Code - Local Model Server
echo =======================================================
echo.
echo Ollama uses :%OLLAMA_PORT%. llama-cpp local provider uses :%LOCAL_PORT%.
echo Adam / Amni-Ai is :7700. Amni-Code UI is :3030.
echo.

:MENU
echo [1] Start with Ollama (Recommended)
echo [2] Start with python (llama-cpp-python)
echo [3] Exit
echo.
set /p CHOICE="Select an option (1-3): "

if "%CHOICE%"=="1" goto OLLAMA
if "%CHOICE%"=="2" goto PYTHON
if "%CHOICE%"=="3" exit /b 0
echo Invalid choice.
goto MENU

:OLLAMA
echo.
where ollama >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo.
    echo Ollama is not installed or not in PATH! 
    echo Please download it from https://ollama.com/
    echo.
    pause
    cls
    goto MENU
)
echo.
echo Starting Ollama on 127.0.0.1:%OLLAMA_PORT% ...
echo (Note: Make sure your desired model is pulled, e.g., 'ollama pull qwen2.5-coder:7b')
echo.
set OLLAMA_HOST=127.0.0.1:%OLLAMA_PORT%
ollama serve
pause
cls
goto MENU

:PYTHON
echo.
where python >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo.
    echo Python is not installed or not in PATH!
    echo.
    pause
    cls
    goto MENU
)
if not exist "models" mkdir "models"
echo.
echo Checking for .gguf files in the 'models' folder...
echo.
dir /b "models\*.gguf" 2>nul
if %ERRORLEVEL% neq 0 (
    echo No .gguf files found in %CD%\models
    echo Please download a GGUF model (e.g. from HuggingFace) and place it there.
    echo.
    pause
    cls
    goto MENU
)
echo.
set /p MODEL_FILE="Type the exact filename of the .gguf you want to run: "
if not exist "models\%MODEL_FILE%" (
    echo File models\%MODEL_FILE% not found!
    pause
    cls
    goto MENU
)

echo.
echo Installing llama-cpp-python server package if missing...
pip install "llama-cpp-python[server]" -q

echo.
echo Starting llama-cpp on port %LOCAL_PORT% with models\%MODEL_FILE% ...
python -m llama_cpp.server --host 127.0.0.1 --port %LOCAL_PORT% --model "models\%MODEL_FILE%"
pause
cls
goto MENU
