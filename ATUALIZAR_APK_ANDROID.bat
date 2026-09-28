@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0ATUALIZAR_APK_ANDROID.ps1"
set EXITCODE=%ERRORLEVEL%
echo.
if not "%EXITCODE%"=="0" (
  echo A atualizacao nao foi concluida. Codigo: %EXITCODE%
) else (
  echo Atualizacao enviada. Confira o Build Android no GitHub.
)
echo.
pause
exit /b %EXITCODE%
