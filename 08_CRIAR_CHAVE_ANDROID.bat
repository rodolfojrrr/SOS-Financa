@echo off
setlocal EnableExtensions
chcp 65001 >nul
title SOS Financa - Chave Android

set "KEYDIR=%USERPROFILE%\SOS-Financa-Chave-Android"
set "KEYFILE=%KEYDIR%\sos-financa-release.jks"
set "B64FILE=%KEYDIR%\ANDROID_KEY_BASE64.txt"

if not exist "%KEYDIR%" mkdir "%KEYDIR%"

where keytool >nul 2>&1
if errorlevel 1 (
  echo keytool nao foi encontrado. Instale o Android Studio/JDK primeiro.
  pause
  exit /b 1
)

if exist "%KEYFILE%" (
  echo Ja existe uma chave em:
  echo %KEYFILE%
  echo Ela NAO sera substituida.
) else (
  echo ==============================================================
  echo     CRIAR CHAVE PRIVADA DO SOS FINANCA PARA ANDROID
  echo ==============================================================
  echo.
  echo Escolha uma senha e GUARDE ESSA SENHA.
echo Quando o keytool perguntar a senha da chave, pressione ENTER para
echo usar a mesma senha do arquivo JKS. O workflow usa uma senha unica.
echo.
  echo A chave fica fora da pasta do Git para nao ser enviada ao GitHub.
  echo.
  keytool -genkeypair -v -keystore "%KEYFILE%" -storetype JKS -keyalg RSA -keysize 2048 -validity 10000 -alias sosfinanca
  if errorlevel 1 exit /b 1
)

powershell -NoProfile -Command "[Convert]::ToBase64String([IO.File]::ReadAllBytes('%KEYFILE%'))" > "%B64FILE%"

echo.
echo ==============================================================
echo CHAVE CRIADA.
echo ==============================================================
echo Arquivo privado: %KEYFILE%
echo Base64 para GitHub Secret: %B64FILE%
echo Alias: sosfinanca
echo.
echo No GitHub, crie estes Secrets do repositorio:
echo ANDROID_KEY_BASE64 = conteudo de ANDROID_KEY_BASE64.txt
echo ANDROID_KEY_PASSWORD = a senha que voce escolheu
echo ANDROID_KEY_ALIAS = sosfinanca
echo.
echo NAO apague a chave JKS. Sem ela, futuras atualizacoes do APK
echo nao poderao ser instaladas por cima da versao anterior.
pause
