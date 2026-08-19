@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Preparar Windows

echo ==============================================================
echo                 SOS FINANCA - PREPARAR WINDOWS
echo ==============================================================
echo.
echo Este script verifica o ambiente para compilar o aplicativo.
echo.

where cargo >nul 2>&1
if errorlevel 1 goto rust_missing

echo [OK] Rust/Cargo encontrado.

cargo tauri --version >nul 2>&1
if not errorlevel 1 goto tauri_ready

echo Instalando Tauri CLI 2...
cargo install tauri-cli --version "^2.0.0" --locked
if errorlevel 1 goto operation_error

:tauri_ready
echo [OK] Tauri CLI encontrado.
echo.
echo Verificando configuracao...
cargo tauri info
if errorlevel 1 goto operation_error

echo.
echo ==============================================================
echo Ambiente basico pronto.
echo Se o Tauri informar falta do Microsoft C++ Build Tools,
echo instale o workload "Desktop development with C++".
echo ==============================================================
pause
exit /b 0

:rust_missing
echo.
echo ERRO: Rust/Cargo nao foi encontrado.
echo Instale o Rust pelo rustup e depois execute este BAT novamente.
echo No Windows, o Tauri tambem exige Microsoft C++ Build Tools.
echo.
pause
exit /b 1

:operation_error
echo.
echo O processo parou por causa do erro acima.
pause
exit /b 1
