@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Testes da V2.0.0

where node >nul 2>&1
if errorlevel 1 goto node_missing

echo ==============================================================
echo             SOS FINANCA - TESTES AUTOMATICOS
echo ==============================================================
echo.
echo [1/2] Regras financeiras...
node tests\finance.test.js
if errorlevel 1 goto failed

echo.
echo [2/2] Armazenamento da previa...
node tests\storage.test.js
if errorlevel 1 goto failed

echo.
echo ==============================================================
echo 54 TESTES AUTOMATICOS PASSARAM.
echo ==============================================================
echo.
echo Os testes estruturais do SQLite e de interface usados na revisao
echo ficam na pasta tests e exigem Python/Playwright para repetir.
pause
exit /b 0

:node_missing
echo.
echo Node.js nao foi encontrado. O app nao depende dele para funcionar,
echo mas este script usa Node para validar as regras antes do build.
pause
exit /b 1

:failed
echo.
echo Um ou mais testes falharam. Nao gere builds antes de corrigir o erro.
pause
exit /b 1
