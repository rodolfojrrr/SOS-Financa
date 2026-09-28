@echo off
net session >nul 2>&1
if not "%errorlevel%"=="0" (
  powershell -NoProfile -Command "Start-Process -Verb RunAs -FilePath '%~f0'"
  exit /b
)
netsh advfirewall firewall delete rule name="SOS Financa Sync 45454" >nul 2>&1
netsh advfirewall firewall add rule name="SOS Financa Sync 45454" dir=in action=allow protocol=TCP localport=45454 profile=private remoteip=localsubnet
if errorlevel 1 (
  echo ERRO ao criar a regra do Firewall.
) else (
  echo OK - Porta TCP 45454 liberada somente em rede privada/local.
)
pause
