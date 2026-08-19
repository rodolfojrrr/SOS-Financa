@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Testes da V3.1.0

echo ==============================================================
echo          SOS FINANCA V3.1.0 - TESTES AUTOMATICOS
echo ==============================================================
echo.

where node >nul 2>&1
if errorlevel 1 goto node_missing

where python >nul 2>&1
if errorlevel 1 goto python_missing

echo [1/4] Regras financeiras...
node tests\finance.test.js
if errorlevel 1 goto failed

echo.
echo [2/4] Armazenamento da previa...
node tests\storage.test.js
if errorlevel 1 goto failed

echo.
echo [3/4] Estrutura e migracoes SQLite...
python tests\schema_test.py
if errorlevel 1 goto failed

echo.
echo [4/4] Configuracao de release PC + Android...
python tests\release_test.py
if errorlevel 1 goto failed

echo.
echo ==============================================================
echo TESTES DE LOGICA, BANCO E RELEASE PASSARAM.
echo ==============================================================
echo.
echo O teste visual completo fica em tests\ui_smoke.py e exige
echo Playwright + Chromium instalados no computador.
pause
exit /b 0

:node_missing
echo.
echo Node.js nao foi encontrado. Este script precisa do Node para
echo validar as regras financeiras antes do build.
pause
exit /b 1

:python_missing
echo.
echo Python nao foi encontrado. Instale Python 3 ou use os workflows
echo do GitHub, que executam as verificacoes automaticamente.
pause
exit /b 1

:failed
echo.
echo Um ou mais testes falharam. Nao gere builds antes de corrigir.
pause
exit /b 1
