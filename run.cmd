@echo off
setlocal enabledelayedexpansion

set CMD_ARG=%~1

if "%CMD_ARG%"=="" (
  echo OpenCode Proxy CLI Management Tool
  echo.
  echo Usage: .\run.cmd ^<command^> [args...]
  echo.
  echo Available commands:
  echo   start          - Start proxy server (port 3001)
  echo   dev            - Start in development mode (watch mode)
  echo   build          - Build release binary
  echo   test           - Run all tests
  echo   health         - Check proxy health
  echo   status         - Show proxy metrics
  echo   setup-factory  - Configure Factory Droid integration
  echo   doctor         - Run diagnostics
  echo.
  exit /b 0
)

if "%CMD_ARG%"=="start" (
  echo Starting OpenCode Proxy on port 3001...
  .\target\release\opencode-proxy.exe
  exit /b %ERRORLEVEL%
)

if "%CMD_ARG%"=="dev" (
  echo Starting in development mode...
  cargo run
  exit /b %ERRORLEVEL%
)

if "%CMD_ARG%"=="build" (
  echo Building release binary...
  cargo build --release
  if %ERRORLEVEL% equ 0 (
    echo Binary ready: .\target\release\opencode-proxy.exe
  )
  exit /b %ERRORLEVEL%
)

if "%CMD_ARG%"=="test" (
  echo Running tests...
  cargo test
  exit /b %ERRORLEVEL%
)

if "%CMD_ARG%"=="health" (
  echo Checking proxy health...
  curl -s http://127.0.0.1:3001/health
  exit /b 0
)

if "%CMD_ARG%"=="status" (
  echo Proxy Status:
  for /f %%i in ('type .env ^| find "MANAGEMENT_TOKEN=" ^| cut -d= -f2') do set TOKEN=%%i
  if "%TOKEN%"=="" (
    curl -s http://127.0.0.1:3001/metrics
  ) else (
    curl -s -H "Authorization: Bearer %TOKEN%" http://127.0.0.1:3001/metrics
  )
  exit /b 0
)

if "%CMD_ARG%"=="setup-factory" (
  echo Setting up Factory Droid integration...
  echo.
  echo Add this to ~/.factory/settings.json under customModels:
  echo.
  echo {
  echo   "model": "deepseek-v4-flash-free",
  echo   "id": "custom:opencode-deepseek-v4-flash-free",
  echo   "displayName": "DeepSeek V4 Flash [OpenCode Proxy]",
  echo   "baseUrl": "http://127.0.0.1:3001/v1",
  echo   "apiKey": "public",
  echo   "provider": "generic-chat-completion-api"
  echo }
  exit /b 0
)

if "%CMD_ARG%"=="doctor" (
  echo OpenCode Proxy Diagnostics
  echo ==========================
  echo.
  echo 1. Proxy Status:
  curl -s http://127.0.0.1:3001/health >nul 2>&1
  if %ERRORLEVEL% equ 0 (
    echo    ✓ Proxy is running
    curl -s http://127.0.0.1:3001/health
  ) else (
    echo    ✗ Proxy is NOT running
    echo    Start with: .\run.cmd start
  )
  echo.
  echo 2. Configuration:
  for /f "tokens=2 delims==" %%i in ('type .env ^| find "PORT="') do echo    PORT: %%i
  for /f "tokens=2 delims==" %%i in ('type .env ^| find "UPSTREAM_URL="') do echo    UPSTREAM: %%i
  for /f "tokens=2 delims==" %%i in ('type .env ^| find "MODELS="') do echo    MODELS: %%i
  exit /b 0
)

echo Unknown command: %CMD_ARG%
exit /b 1
