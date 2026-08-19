@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Preparar Android

echo ==============================================================
echo                 SOS FINANCA - PREPARAR ANDROID
echo ==============================================================
echo.

where cargo >nul 2>&1
if errorlevel 1 goto missing
cargo tauri --version >nul 2>&1
if errorlevel 1 goto missing

if defined JAVA_HOME echo [OK] JAVA_HOME=%JAVA_HOME%
if not defined JAVA_HOME echo [ATENCAO] JAVA_HOME nao esta definido.
if defined ANDROID_HOME echo [OK] ANDROID_HOME=%ANDROID_HOME%
if not defined ANDROID_HOME echo [ATENCAO] ANDROID_HOME nao esta definido.
if defined NDK_HOME echo [OK] NDK_HOME=%NDK_HOME%
if not defined NDK_HOME echo [ATENCAO] NDK_HOME nao esta definido.

echo.
echo Inicializando o projeto Android do Tauri...
cargo tauri android init
if errorlevel 1 goto android_error

echo.
echo Android inicializado. O proximo passo e 05_GERAR_APK_ANDROID.bat.
pause
exit /b 0

:missing
echo Rust/Cargo ou Tauri CLI nao foi encontrado.
echo Execute 01_PREPARAR_WINDOWS.bat primeiro.
pause
exit /b 1

:android_error
echo.
echo O Tauri nao conseguiu inicializar o Android.
echo Confira Android Studio, SDK, NDK, JDK e as variaveis exibidas acima.
pause
exit /b 1
