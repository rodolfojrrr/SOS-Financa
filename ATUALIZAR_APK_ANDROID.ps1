$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Payload = Join-Path $Root 'payload'
$Version = '3.1.4'
$CommitMessage = 'SOS Financa V3.1.4 - correcao recebimento Android'

function Test-SosRepo {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return $false }
    try {
        $full = [IO.Path]::GetFullPath($Path)
        return (
            (Test-Path -LiteralPath (Join-Path $full 'src-tauri\Cargo.toml') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $full 'src-tauri\src\db.rs') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $full 'src-tauri\src\lib.rs') -PathType Leaf) -and
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
        if ($candidate -and (Test-SosRepo $candidate)) {
            return [IO.Path]::GetFullPath($candidate)
        }
    }
    return $null
}

function Backup-ChangedFiles {
    param([string]$Repo, [string]$BackupRoot)
    New-Item -ItemType Directory -Force -Path $BackupRoot | Out-Null
    Get-ChildItem -LiteralPath $Payload -File -Recurse | ForEach-Object {
        $relative = $_.FullName.Substring($Payload.Length).TrimStart([char[]]@('\','/'))
        $current = Join-Path $Repo $relative
        if (Test-Path -LiteralPath $current -PathType Leaf) {
            $target = Join-Path $BackupRoot $relative
            New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
            Copy-Item -LiteralPath $current -Destination $target -Force
        }
    }
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
    param([string]$Repo, [string]$BackupRoot)
    if (-not (Test-Path -LiteralPath $BackupRoot -PathType Container)) { return }
    Get-ChildItem -LiteralPath $BackupRoot -File -Recurse | ForEach-Object {
        $relative = $_.FullName.Substring($BackupRoot.Length).TrimStart([char[]]@('\','/'))
        $target = Join-Path $Repo $relative
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
        Copy-Item -LiteralPath $_.FullName -Destination $target -Force
    }
}

function Assert-Contains {
    param([string]$Path, [string]$Needle, [string]$Message)
    $text = Get-Content -LiteralPath $Path -Raw
    if (-not $text.Contains($Needle)) { throw $Message }
}

function Run-Checks {
    param([string]$Repo)

    Write-Host '      Verificando correcao Android...' -ForegroundColor Cyan
    Assert-Contains (Join-Path $Repo 'src-tauri\src\lib.rs') 'async fn receive_sync_from_pc' 'Comando Android nao ficou assincrono.'
    Assert-Contains (Join-Path $Repo 'src-tauri\src\lib.rs') 'spawn_blocking' 'Recebimento bloqueante nao foi movido para thread propria.'
    Assert-Contains (Join-Path $Repo 'src-tauri\src\sync.rs') 'read_protocol_line(&mut stream, 1024)' 'Leitura robusta do cabecalho nao foi aplicada.'
    Assert-Contains (Join-Path $Repo 'src-tauri\src\sync.rs') 'A conexao terminou apos receber' 'Diagnostico de bytes recebidos nao foi aplicado.'
    Assert-Contains (Join-Path $Repo 'app\storage.js') 'port: 45454' 'Porta fixa interna nao foi aplicada.'
    Assert-Contains (Join-Path $Repo 'app\storage.js') "code: '534F-5346-494E-414E'" 'Codigo de compatibilidade interno nao foi aplicado.'
    Assert-Contains (Join-Path $Repo 'app\app.js') 'No celular voce so precisa informar o IP' 'Tela simplificada de sincronizacao nao foi aplicada.'

    $node = Get-Command node.exe -ErrorAction SilentlyContinue
    if (-not $node) { $node = Get-Command node -ErrorAction SilentlyContinue }
    if ($node) {
        Push-Location $Repo
        try {
            & $node.Source --check app\app.js
            if ($LASTEXITCODE -ne 0) { throw 'Falha de sintaxe em app\app.js.' }
            & $node.Source --check app\storage.js
            if ($LASTEXITCODE -ne 0) { throw 'Falha de sintaxe em app\storage.js.' }
            & $node.Source tests\finance.test.js
            if ($LASTEXITCODE -ne 0) { throw 'Testes financeiros falharam.' }
            & $node.Source tests\storage.test.js
            if ($LASTEXITCODE -ne 0) { throw 'Testes de armazenamento falharam.' }
        } finally {
            Pop-Location
        }
    } else {
        Write-Host '      Node.js nao encontrado; o GitHub Actions executara os testes completos.' -ForegroundColor Yellow
    }

    Write-Host '[OK] Verificacoes locais concluidas.' -ForegroundColor Green
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
        if ($LASTEXITCODE -ne 0) {
            throw 'git push falhou. Os arquivos continuam atualizados localmente.'
        }

        Write-Host '[OK] Atualizacao enviada ao GitHub.' -ForegroundColor Green
    } finally {
        Pop-Location
    }
}

Clear-Host
Write-Host '============================================================'
Write-Host ' SOS FINANCA 3.1.4 - CORRECAO DO APK / RECEBIMENTO ANDROID'
Write-Host '============================================================'
Write-Host ''
Write-Host 'Esta atualizacao corrige o lado Android da sincronizacao.' -ForegroundColor Green
Write-Host 'O PC 3.1.3 pode continuar instalado.' -ForegroundColor Green
Write-Host 'No celular, depois da atualizacao, voce informara SOMENTE o IP do PC.' -ForegroundColor Green
Write-Host ''

$Repo = Find-SosRepo

if (-not $Repo) {
    $picked = Select-SosRepo
    if ($picked -and (Test-SosRepo $picked)) {
        $Repo = [IO.Path]::GetFullPath($picked)
    }
}

while (-not $Repo) {
    $typed = Read-Host 'Cole o caminho da pasta do repositorio SOS-Financa ou digite CANCELAR'
    if ($typed.Trim().ToUpperInvariant() -eq 'CANCELAR') { exit 2 }
    $typed = $typed.Trim().Trim('"')
    if (Test-SosRepo $typed) {
        $Repo = [IO.Path]::GetFullPath($typed)
    } else {
        Write-Host '[ERRO] Essa pasta nao parece ser o repositorio SOS-Financa.' -ForegroundColor Red
    }
}

Write-Host "[OK] Repositorio: $Repo" -ForegroundColor Green

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$BackupRoot = Join-Path $env:TEMP "SOS-Financa-update-$Version-$stamp"

try {
    Write-Host '[1/4] Salvando copia dos arquivos que serao alterados...'
    Backup-ChangedFiles -Repo $Repo -BackupRoot $BackupRoot

    Write-Host '[2/4] Aplicando correcao Android...'
    Apply-Payload -Repo $Repo

    Write-Host '[3/4] Fazendo verificacoes locais...'
    Run-Checks -Repo $Repo

    Write-Host '[4/4] Enviando para o GitHub...'
    Push-GitHub -Repo $Repo

    Write-Host ''
    Write-Host '============================================================' -ForegroundColor Green
    Write-Host '        ANDROID 3.1.4 ENVIADO COM SUCESSO' -ForegroundColor Green
    Write-Host '============================================================' -ForegroundColor Green
    Write-Host ''
    Write-Host 'Abra GitHub > Actions e acompanhe Build Android.' -ForegroundColor Cyan
    Write-Host 'Quando ficar verde, baixe o APK novo e instale no celular.' -ForegroundColor Cyan
    Write-Host ''
    Write-Host 'Depois da atualizacao do APK:' -ForegroundColor Yellow
    Write-Host '  1. No PC 3.1.3 abra Enviar dados deste PC.'
    Write-Host '  2. No celular abra Receber do PC.'
    Write-Host '  3. Digite SOMENTE o IP mostrado pelo PC.'
    Write-Host ''
    Write-Host "Backup dos arquivos-fonte antigos: $BackupRoot" -ForegroundColor DarkGray
    exit 0
}
catch {
    Write-Host ''
    Write-Host "[ERRO] $($_.Exception.Message)" -ForegroundColor Red
    Write-Host 'Restaurando os arquivos-fonte anteriores...' -ForegroundColor Yellow
    Restore-ChangedFiles -Repo $Repo -BackupRoot $BackupRoot
    Write-Host '[OK] Arquivos-fonte anteriores restaurados.' -ForegroundColor Green
    Write-Host 'O banco financeiro nao faz parte desta atualizacao.' -ForegroundColor Green
    exit 1
}
