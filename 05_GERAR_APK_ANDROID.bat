@echo off
setlocal EnableExtensions
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Gerar APK Android

echo ==============================================================
echo               SOS FINANCA V3 - BUILD ANDROID
echo ==============================================================
echo.

if exist "src-tauri\gen\android\keystore.properties" goto signed

echo Nenhuma configuracao local de assinatura Release foi encontrada.
echo Sera gerado um APK DEBUG instalavel para teste.
echo Para o APK Release definitivo, o caminho recomendado e o workflow
echo Build Android do GitHub com os Secrets descritos no README.
echo.
cargo tauri android build --debug --apk --ci
if errorlevel 1 goto error
goto show

:signed
echo Configuracao de assinatura encontrada. Preparando Gradle...
python scripts\configure_android_signing.py
if errorlevel 1 goto error

echo Gerando APK Android Release assinado...
cargo tauri android build --apk --ci
if errorlevel 1 goto error

:show
echo.
echo APKs encontrados:
for /r "src-tauri\gen\android\app\build\outputs\apk" %%F in (*.apk) do echo %%F

echo.
echo ==============================================================
echo BUILD ANDROID CONCLUIDO.
echo ==============================================================
pause
exit /b 0

:error
echo.
echo Falha no build Android. Execute 04_PREPARAR_ANDROID.bat e confira o erro acima.
pause
exit /b 1
