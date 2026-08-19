@echo off
setlocal EnableExtensions
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Preparar Android

echo ==============================================================
echo            SOS FINANCA V3 - PREPARAR ANDROID
echo ==============================================================
echo.

where cargo >nul 2>&1
if errorlevel 1 goto missing
cargo tauri --version >nul 2>&1
if errorlevel 1 goto missing

if not defined JAVA_HOME (
  echo [ERRO] JAVA_HOME nao esta definido.
  goto android_error
)
if not defined ANDROID_HOME (
  echo [ERRO] ANDROID_HOME nao esta definido.
  goto android_error
)
if not defined NDK_HOME (
  echo [ERRO] NDK_HOME nao esta definido.
  goto android_error
)

echo [OK] JAVA_HOME=%JAVA_HOME%
echo [OK] ANDROID_HOME=%ANDROID_HOME%
echo [OK] NDK_HOME=%NDK_HOME%
echo.

echo Adicionando alvos Rust para Android...
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
if errorlevel 1 goto android_error

echo.
echo Inicializando o projeto Android do Tauri...
cargo tauri android init --ci
if errorlevel 1 goto android_error

echo.
echo ==============================================================
echo ANDROID PREPARADO.
echo Agora execute 05_GERAR_APK_ANDROID.bat.
echo ==============================================================
pause
exit /b 0

:missing
echo Rust/Cargo ou Tauri CLI nao foi encontrado.
echo Execute 01_PREPARAR_WINDOWS.bat primeiro.
pause
exit /b 1

:android_error
echo.
echo Nao foi possivel preparar o Android.
echo Confira Android Studio, SDK, NDK, JDK e as variaveis acima.
pause
exit /b 1
