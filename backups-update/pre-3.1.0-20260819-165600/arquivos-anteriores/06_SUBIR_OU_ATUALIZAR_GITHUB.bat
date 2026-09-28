@echo off
setlocal EnableExtensions DisableDelayedExpansion
chcp 65001 >nul
cd /d "%~dp0"
title SOS Financa - GitHub

where git >nul 2>&1
if errorlevel 1 goto git_missing

if exist ".git" goto repo_ready
git init
if errorlevel 1 goto error

:repo_ready
git branch -M main

for /f "delims=" %%U in ('git remote get-url origin 2^>nul') do set "CURRENT_REMOTE=%%U"
if defined CURRENT_REMOTE goto remote_ready

echo.
set "REPO_URL="
set /p "REPO_URL=Cole a URL do repositorio do SOS Financa (.git): "
if not defined REPO_URL goto no_url
git remote add origin "%REPO_URL%"
if errorlevel 1 goto error

:remote_ready
git add .
if errorlevel 1 goto error

git diff --cached --quiet
if not errorlevel 1 goto push

set "MSG="
set /p "MSG=Mensagem do commit [SOS Financa V3.0.0 FINAL]: "
if not defined MSG set "MSG=SOS Financa V3.0.0 FINAL - Windows e Android"
git commit -m "%MSG%"
if errorlevel 1 goto error

:push
git push -u origin main
if errorlevel 1 goto push_error

echo.
echo SOS Financa enviado/atualizado no GitHub com sucesso.
pause
exit /b 0

:no_url
echo URL nao informada. Nenhum remote foi criado.
pause
exit /b 1

:git_missing
echo Git nao foi encontrado no Windows.
pause
exit /b 1

:push_error
echo.
echo O commit local esta seguro, mas o GitHub recusou o push.
echo Confira login/permissao e execute este BAT novamente.
pause
exit /b 1

:error
echo.
echo O processo parou por causa do erro acima. Nenhum banco foi apagado.
pause
exit /b 1
