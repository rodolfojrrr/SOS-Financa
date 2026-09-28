$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Payload = Join-Path $Root 'payload'
$Version = '3.1.0'
$CommitMessage = 'SOS Financa V3.1.0 - sincronizacao PC para celular'

function Test-SosRepo {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return $false }
    try {
        $full = [IO.Path]::GetFullPath($Path)
        return (
            (Test-Path -LiteralPath (Join-Path $full 'src-tauri\Cargo.toml') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $full 'src-tauri\src\db.rs') -PathType Leaf) -and
            (Test-Path -LiteralPath (Join-Path $full 'app\app.js') -PathType Leaf)
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
    $candidates = New-Object 'System.Collections.Generic.List[string]'
    $parent = Split-Path -Parent $Root
    foreach ($path in @(
        $parent,
        (Join-Path $parent 'SOS-Financa'),
        (Join-Path ([Environment]::GetFolderPath('Desktop')) 'SOS-Financa'),
        (Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'SOS-Financa'),
        (Join-Path $env:USERPROFILE 'Downloads\SOS-Financa')
    )) {
        if ($path -and -not $candidates.Contains($path)) { [void]$candidates.Add($path) }
    }
    foreach ($candidate in $candidates) {
        if (Test-SosRepo $candidate) { return [IO.Path]::GetFullPath($candidate) }
    }
    return $null
}

function Copy-DatabaseBackup {
    param([string]$BackupRoot)
    $candidates = New-Object 'System.Collections.Generic.List[string]'
    foreach ($path in @(
        (Join-Path $env:APPDATA 'com.sosfinanca.app\sos_financa.db'),
        (Join-Path $env:LOCALAPPDATA 'com.sosfinanca.app\sos_financa.db'),
        (Join-Path $env:APPDATA 'SOS Financa\sos_financa.db'),
        (Join-Path $env:LOCALAPPDATA 'SOS Financa\sos_financa.db'),
        (Join-Path $env:APPDATA 'SOS Finança\sos_financa.db'),
        (Join-Path $env:LOCALAPPDATA 'SOS Finança\sos_financa.db')
    )) {
        if ($path -and (Test-Path -LiteralPath $path -PathType Leaf) -and -not $candidates.Contains($path)) {
            [void]$candidates.Add($path)
        }
    }

    if ($candidates.Count -eq 0) {
        Write-Host '      Banco local nao foi encontrado nos caminhos padrao. O atualizador nao altera banco algum.' -ForegroundColor Yellow
        return
    }

    $dbBackup = Join-Path $BackupRoot 'banco-local'
    New-Item -ItemType Directory -Force -Path $dbBackup | Out-Null
    $index = 1
    foreach ($db in $candidates) {
        $folder = Join-Path $dbBackup ("banco-$index")
        New-Item -ItemType Directory -Force -Path $folder | Out-Null
        Copy-Item -LiteralPath $db -Destination (Join-Path $folder 'sos_financa.db') -Force
        foreach ($suffix in @('-wal','-shm')) {
            $extra = "$db$suffix"
            if (Test-Path -LiteralPath $extra -PathType Leaf) {
                Copy-Item -LiteralPath $extra -Destination (Join-Path $folder ("sos_financa.db$suffix")) -Force
            }
        }
        Set-Content -LiteralPath (Join-Path $folder 'ORIGEM.txt') -Value $db -Encoding UTF8
        $index++
    }
    Write-Host "      Backup do banco criado em: $dbBackup" -ForegroundColor Green
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
        Write-Host '      Verificando JavaScript e regras financeiras...' -ForegroundColor Cyan
        Push-Location $Repo
        try {
            & $node.Source --check app\app.js
            if ($LASTEXITCODE -ne 0) { throw 'Falha de sintaxe em app\app.js' }
            & $node.Source --check app\storage.js
            if ($LASTEXITCODE -ne 0) { throw 'Falha de sintaxe em app\storage.js' }
            & $node.Source tests\finance.test.js | Out-Host
            if ($LASTEXITCODE -ne 0) { throw 'Os testes financeiros falharam.' }
            & $node.Source tests\storage.test.js | Out-Host
            if ($LASTEXITCODE -ne 0) { throw 'Os testes de armazenamento falharam.' }
        } finally { Pop-Location }
    } else {
        Write-Host '      Node.js nao encontrado. Os testes Node serao executados pelo GitHub Actions.' -ForegroundColor Yellow
    }

    Write-Host '      Testes SQLite/release serao executados pelo GitHub Actions.' -ForegroundColor Yellow
    Write-Host '      O atualizador nao depende de Python instalado neste PC.' -ForegroundColor DarkYellow
}

function Push-GitHub {
    param([string]$Repo)
    $git = Get-Command git.exe -ErrorAction SilentlyContinue
    if (-not $git) { $git = Get-Command git -ErrorAction SilentlyContinue }
    if (-not $git) {
        Write-Host '[AVISO] Git nao foi encontrado. A atualizacao foi aplicada, mas voce precisara subir manualmente.' -ForegroundColor Yellow
        return
    }
    if (-not (Test-Path -LiteralPath (Join-Path $Repo '.git') -PathType Container)) {
        Write-Host '[AVISO] A pasta selecionada nao possui .git. A atualizacao foi aplicada, mas nao farei push automatico.' -ForegroundColor Yellow
        return
    }

    $answer = Read-Host 'Subir esta atualizacao agora para o GitHub? [S/n]'
    if ($answer -and $answer.Trim().ToUpperInvariant() -notin @('S','SIM','Y','YES')) {
        Write-Host 'Push ignorado. Execute 06_SUBIR_OU_ATUALIZAR_GITHUB.bat quando quiser.' -ForegroundColor Yellow
        return
    }

    Push-Location $Repo
    try {
        & $git.Source add -A
        if ($LASTEXITCODE -ne 0) { throw 'git add falhou.' }
        $status = (& $git.Source status --porcelain | Out-String).Trim()
        if (-not $status) {
            Write-Host '[OK] Os arquivos ja estavam atualizados; nao ha novo commit.' -ForegroundColor Green
            return
        }
        & $git.Source commit -m $CommitMessage | Out-Host
        if ($LASTEXITCODE -ne 0) { throw 'git commit falhou. Confira nome/e-mail configurados no Git.' }
        & $git.Source push | Out-Host
        if ($LASTEXITCODE -ne 0) { throw 'git push falhou. A atualizacao continua salva localmente.' }
        Write-Host '[OK] Atualizacao enviada ao GitHub.' -ForegroundColor Green
    } finally { Pop-Location }
}

Clear-Host
Write-Host '============================================================'
Write-Host ' SOS FINANCA 3.1.0 - ATUALIZACAO SYNC PC -> CELULAR'
Write-Host '============================================================'
Write-Host ''
Write-Host 'Este pacote contem somente os arquivos alterados.'
Write-Host 'O banco sos_financa.db nao faz parte do payload.'
Write-Host ''

if (-not (Test-Path -LiteralPath (Join-Path $Payload 'src-tauri\src\sync.rs') -PathType Leaf) -or
    -not (Test-Path -LiteralPath (Join-Path $Payload 'app\app.js') -PathType Leaf)) {
    Write-Host '[ERRO] Pacote incompleto. Extraia o ZIP inteiro antes de executar.' -ForegroundColor Red
    exit 10
}

$Repo = Find-SosRepo
if (-not $Repo) {
    Write-Host 'Nao localizei automaticamente a pasta do projeto.' -ForegroundColor Yellow
    Write-Host 'Selecione a pasta SOS-Financa que voce usa para subir ao GitHub.'
    $picked = Select-SosRepo
    if ($picked -and (Test-SosRepo $picked)) { $Repo = [IO.Path]::GetFullPath($picked) }
}

while (-not $Repo) {
    $typed = Read-Host 'Cole o caminho da pasta SOS-Financa ou digite CANCELAR'
    if ($typed.Trim().ToUpperInvariant() -eq 'CANCELAR') { exit 2 }
    $typed = $typed.Trim().Trim('"')
    if (Test-SosRepo $typed) { $Repo = [IO.Path]::GetFullPath($typed) }
    else { Write-Host '[ERRO] Essa pasta nao parece ser o projeto SOS-Financa.' -ForegroundColor Red }
}

Write-Host "[OK] Projeto localizado: $Repo" -ForegroundColor Green
Write-Host ''
Write-Host '[IMPORTANTE] Feche o SOS Financa instalado antes de continuar.' -ForegroundColor Yellow
[void](Read-Host 'Pressione ENTER quando o app estiver fechado')

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$BackupRoot = Join-Path $Repo "backups-update\pre-$Version-$stamp"
New-Item -ItemType Directory -Force -Path $BackupRoot | Out-Null

try {
    Write-Host '[1/5] Fazendo backup de seguranca do banco local encontrado...'
    Copy-DatabaseBackup -BackupRoot $BackupRoot

    Write-Host '[2/5] Salvando os arquivos atuais que serao alterados...'
    $SourceBackup = Backup-ChangedFiles -Repo $Repo -BackupRoot $BackupRoot

    Write-Host '[3/5] Aplicando somente os arquivos da V3.1.0...'
    Apply-Payload -Repo $Repo

    Write-Host '[4/5] Executando verificacoes disponiveis neste PC...'
    Run-Checks -Repo $Repo

    Write-Host '[5/5] Preparando envio para o GitHub...'
    Push-GitHub -Repo $Repo

    Write-Host ''
    Write-Host '============================================================' -ForegroundColor Green
    Write-Host '        SOS FINANCA 3.1.0 APLICADO COM SUCESSO' -ForegroundColor Green
    Write-Host '============================================================' -ForegroundColor Green
    Write-Host "[OK] Backup da atualizacao: $BackupRoot" -ForegroundColor Green
    Write-Host '[OK] Nenhum banco foi incluido no payload.' -ForegroundColor Green
    Write-Host ''
    Write-Host 'Depois do push, aguarde Build Windows e Build Android no GitHub Actions.'
    Write-Host 'Atualize os dois aparelhos para usar a sincronizacao Wi-Fi.'
    exit 0
}
catch {
    Write-Host ''
    Write-Host "[ERRO] $($_.Exception.Message)" -ForegroundColor Red
    Write-Host 'Tentando restaurar os arquivos-fonte anteriores...' -ForegroundColor Yellow
    try {
        if ($SourceBackup) { Restore-ChangedFiles -Repo $Repo -SourceBackup $SourceBackup }
        Write-Host '[OK] Arquivos anteriores restaurados.' -ForegroundColor Green
    } catch {
        Write-Host '[AVISO] A restauracao automatica nao foi completa.' -ForegroundColor Yellow
        Write-Host "Backup disponivel em: $BackupRoot"
    }
    Write-Host 'O banco financeiro nao foi substituido por este atualizador.'
    exit 1
}
