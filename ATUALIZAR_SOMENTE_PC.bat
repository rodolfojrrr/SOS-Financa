@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0ATUALIZAR_SOMENTE_PC.ps1"
set "RC=%ERRORLEVEL%"
echo.
if not "%RC%"=="0" echo A atualizacao terminou com erro. Codigo: %RC%
pause
exit /b %RC%
