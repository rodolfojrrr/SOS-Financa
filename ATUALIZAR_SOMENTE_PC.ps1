$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Payload = Join-Path $Root 'payload'
$Version = '3.1.2'
$CommitMessage = 'SOS Financa V3.1.2 - correcao sync somente PC'

function Test-SosRepo {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return $false }
    try {
        $full = [IO.Path]::GetFullPath($Path)
        return (
            (Test-Path -LiteralPath (Join-Path $full 'src-tauri\Cargo.toml') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $full 'src-tauri\src\db.rs') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $full 'app\app.js') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $full '.git') -PathType Container)
        )
    } catch { return $false }
}

function Select-SosRepo {
    try {
        Add-Type -AssemblyName System.Windows.Forms
        $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
        $dialog.Description = 'Selecione a pasta do repositorio SOS-Financa'
        $dialog.ShowNewFolderButton = $false
        if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
            return $dialog.SelectedPath
        }
    } catch {}
    return $null
}

function Find-SosRepo {
    $candidates = @(
        'D:\Nova pasta\SOS-Financa-V1.0.1-REVISADA\SOS-Financa-V1-REVISADA',
        (Split-Path -Parent $Root),
        (Join-Path ([Environment]::GetFolderPath('Desktop')) 'SOS-Financa'),
        (Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'SOS-Financa'),
        (Join-Path $env:USERPROFILE 'Downloads\SOS-Financa')
    )
    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-SosRepo $candidate)) { return [IO.Path]::GetFullPath($candidate) }
    }
    return $null
}

function Backup-ChangedFiles {
    param([string]$Repo, [string]$BackupRoot)
    $sourceBackup = Join-Path $BackupRoot 'arquivos-anteriores'
    New-Item -ItemType Directory -Force -Path $sourceBackup | Out-Null
    Get-ChildItem -LiteralPath $Payload -File -Recurse | ForEach-Object {
        $relative = $_.FullName.Substring($Payload.Length).TrimStart([char[]]@('\','/'))
        $current = Join-Path $Repo $relative
        if (Test-Path -LiteralPath $current -PathType Leaf) {
            $target = Join-Path $sourceBackup $relative
            New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
            Copy-Item -LiteralPath $current -Destination $target -Force
        }
    }
    return $sourceBackup
}

function Apply-Payload {
    param([string]$Repo)
    Get-ChildItem -LiteralPath $Payload -File -Recurse | ForEach-Object {
        $relative = $_.FullName.Substring($Payload.Length).TrimStart([char[]]@('\','/'))
        $target = Join-Path $Repo $relative
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
        Copy-Item -LiteralPath $_.FullName -Destination $target -Force
    }
}

function Restore-ChangedFiles {
    param([string]$Repo, [string]$SourceBackup)
    if (-not (Test-Path -LiteralPath $SourceBackup -PathType Container)) { return }
    Get-ChildItem -LiteralPath $SourceBackup -File -Recurse | ForEach-Object {
        $relative = $_.FullName.Substring($SourceBackup.Length).TrimStart([char[]]@('\','/'))
        $target = Join-Path $Repo $relative
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
        Copy-Item -LiteralPath $_.FullName -Destination $target -Force
    }
}

function Run-Checks {
    param([string]$Repo)
    $node = Get-Command node.exe -ErrorAction SilentlyContinue
    if (-not $node) { $node = Get-Command node -ErrorAction SilentlyContinue }
    if ($node) {
        Write-Host '      Verificando JavaScript...' -ForegroundColor Cyan
        Push-Location $Repo
        try {
            & $node.Source --check app\app.js
            if ($LASTEXITCODE -ne 0) { throw 'Falha de sintaxe em app\app.js' }
            & $node.Source --check app\storage.js
            if ($LASTEXITCODE -ne 0) { throw 'Falha de sintaxe em app\storage.js' }
        } finally { Pop-Location }
    } else {
        Write-Host '      Node.js nao encontrado. O GitHub Actions fara as verificacoes.' -ForegroundColor Yellow
    }
    Write-Host '      Testes completos serao executados no Build Windows do GitHub.' -ForegroundColor Yellow
}

function Push-GitHub {
    param([string]$Repo)
    $git = Get-Command git.exe -ErrorAction SilentlyContinue
    if (-not $git) { $git = Get-Command git -ErrorAction SilentlyContinue }
    if (-not $git) { throw 'Git nao foi encontrado neste PC.' }

    Push-Location $Repo
    try {
        & $git.Source add -A
        if ($LASTEXITCODE -ne 0) { throw 'git add falhou.' }
        $status = (& $git.Source status --porcelain | Out-String).Trim()
        if (-not $status) {
            Write-Host '[OK] O repositorio ja possui esta atualizacao.' -ForegroundColor Green
            return
        }
        & $git.Source commit -m $CommitMessage | Out-Host
        if ($LASTEXITCODE -ne 0) { throw 'git commit falhou.' }
        & $git.Source push | Out-Host
        if ($LASTEXITCODE -ne 0) { throw 'git push falhou. Os arquivos continuam atualizados localmente.' }
        Write-Host '[OK] Atualizacao enviada ao GitHub.' -ForegroundColor Green
    } finally { Pop-Location }
}

Clear-Host
Write-Host '============================================================'
Write-Host ' SOS FINANCA 3.1.2 - CORRECAO SOMENTE PC'
Write-Host '============================================================'
Write-Host ''
Write-Host 'O APK atual do celular NAO precisa ser reinstalado.' -ForegroundColor Green
Write-Host 'O banco sos_financa.db NAO faz parte deste pacote.' -ForegroundColor Green
Write-Host 'Depois do push, somente Build Windows inicia automaticamente.' -ForegroundColor Green
Write-Host ''

$Repo = Find-SosRepo
if (-not $Repo) {
    $picked = Select-SosRepo
    if ($picked -and (Test-SosRepo $picked)) { $Repo = [IO.Path]::GetFullPath($picked) }
}
while (-not $Repo) {
    $typed = Read-Host 'Cole o caminho da pasta do repositorio SOS-Financa ou digite CANCELAR'
    if ($typed.Trim().ToUpperInvariant() -eq 'CANCELAR') { exit 2 }
    $typed = $typed.Trim().Trim('"')
    if (Test-SosRepo $typed) { $Repo = [IO.Path]::GetFullPath($typed) }
    else { Write-Host '[ERRO] Essa pasta nao parece ser o repositorio SOS-Financa.' -ForegroundColor Red }
}

Write-Host "[OK] Repositorio: $Repo" -ForegroundColor Green
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$BackupRoot = Join-Path $Repo "backups-update\pre-PC-$Version-$stamp"
New-Item -ItemType Directory -Force -Path $BackupRoot | Out-Null
$SourceBackup = $null

try {
    Write-Host '[1/4] Salvando os arquivos que serao alterados...'
    $SourceBackup = Backup-ChangedFiles -Repo $Repo -BackupRoot $BackupRoot

    Write-Host '[2/4] Aplicando a correcao do Windows...'
    Apply-Payload -Repo $Repo

    Write-Host '[3/4] Fazendo verificacoes locais...'
    Run-Checks -Repo $Repo

    Write-Host '[4/4] Enviando para o GitHub...'
    Push-GitHub -Repo $Repo

    Write-Host ''
    Write-Host '============================================================' -ForegroundColor Green
    Write-Host '      ATUALIZACAO PC 3.1.2 ENVIADA COM SUCESSO' -ForegroundColor Green
    Write-Host '============================================================' -ForegroundColor Green
    Write-Host ''
    Write-Host 'No GitHub, aguarde SOMENTE o Build Windows.' -ForegroundColor Cyan
    Write-Host 'Instale o novo Windows e mantenha o APK atual no celular.' -ForegroundColor Cyan
    Write-Host ''
    Write-Host 'Para sincronizar com o APK atual:' -ForegroundColor Yellow
    Write-Host '  Porta: 45454'
    Write-Host '  Codigo fixo: 534F-5346-494E-414E'
    Write-Host '  O IP continua sendo o IP mostrado pelo PC.'
    exit 0
}
catch {
    Write-Host "[ERRO] $($_.Exception.Message)" -ForegroundColor Red
    if ($SourceBackup) {
        Write-Host 'Restaurando os arquivos anteriores...' -ForegroundColor Yellow
        Restore-ChangedFiles -Repo $Repo -SourceBackup $SourceBackup
        Write-Host '[OK] Arquivos anteriores restaurados.' -ForegroundColor Green
    }
    exit 1
}
