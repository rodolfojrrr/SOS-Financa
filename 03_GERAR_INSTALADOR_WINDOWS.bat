@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Gerar instalador Windows

echo Gerando o SOS Financa em modo release...
cargo tauri build
if errorlevel 1 goto error

echo.
echo ==============================================================
echo BUILD CONCLUIDO.
echo Procure o instalador em:
echo src-tauri\target\release\bundle\nsis
echo ==============================================================
start "" "%CD%\src-tauri\target\release\bundle\nsis"
pause
exit /b 0

:error
echo.
echo Falha no build. Execute 01_PREPARAR_WINDOWS.bat e confira o erro acima.
pause
exit /b 1
