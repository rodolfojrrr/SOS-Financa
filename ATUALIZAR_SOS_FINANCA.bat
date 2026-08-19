@echo off
setlocal EnableExtensions
chcp 65001 >nul
title SOS Financa 3.1.0 - Atualizacao SYNC
cd /d "%~dp0"

if not exist "%~dp0ATUALIZAR_SOS_FINANCA.ps1" (
  echo [ERRO] ATUALIZAR_SOS_FINANCA.ps1 nao foi encontrado.
  echo Extraia o ZIP inteiro antes de executar.
  echo.
  pause
  exit /b 1
)

powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0ATUALIZAR_SOS_FINANCA.ps1"
set "CODIGO=%ERRORLEVEL%"

echo.
if "%CODIGO%"=="0" (
  echo Atualizacao finalizada. Pressione qualquer tecla para fechar.
) else (
  echo A atualizacao nao foi concluida. Codigo: %CODIGO%
  echo Leia a mensagem acima antes de tentar novamente.
)
echo.
pause
exit /b %CODIGO%
