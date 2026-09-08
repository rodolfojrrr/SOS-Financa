@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - Testes da V4.0.0

echo ==============================================================
echo          SOS FINANCA V4.0.0 - TESTES AUTOMATICOS
echo ==============================================================
echo.

where node >nul 2>&1
if errorlevel 1 goto node_missing

echo [1/5] Regras financeiras...
node tests\finance.test.js
if errorlevel 1 goto failed

echo.
echo [2/5] Armazenamento da previa...
node tests\storage.test.js
if errorlevel 1 goto failed

where python >nul 2>&1
if errorlevel 1 goto python_optional

echo.
echo [3/5] Estrutura e migracoes SQLite...
python tests\schema_test.py
if errorlevel 1 goto failed

echo.
echo [4/5] Mesclagem bidirecional...
python tests\sync_merge_test.py
if errorlevel 1 goto failed

echo.
echo [5/5] Configuracao de release PC + Android...
python tests\release_test.py
if errorlevel 1 goto failed

echo.
echo TODOS OS TESTES LOCAIS DA V4 PASSARAM.
pause
exit /b 0

:python_optional
echo [AVISO] Python nao foi encontrado. Os testes restantes rodam no GitHub Actions.
pause
exit /b 0

:node_missing
echo [AVISO] Node.js nao foi encontrado. O GitHub Actions executara os testes.
pause
exit /b 0

:failed
echo Um ou mais testes falharam. Nao gere builds antes de corrigir.
pause
exit /b 1
