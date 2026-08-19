@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Desenvolvimento
cargo tauri dev
if errorlevel 1 (
  echo.
  echo Nao foi possivel iniciar. Execute primeiro 01_PREPARAR_WINDOWS.bat.
  pause
)
