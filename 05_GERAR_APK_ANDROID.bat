@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Gerar APK Android

echo Gerando APK Android...
cargo tauri android build --apk
if errorlevel 1 goto error

echo.
echo ==============================================================
echo BUILD ANDROID CONCLUIDO.
echo O caminho exato do APK aparece no log do Tauri acima.
echo ==============================================================
pause
exit /b 0

:error
echo.
echo Falha no build Android. Execute 04_PREPARAR_ANDROID.bat e confira o erro acima.
pause
exit /b 1
