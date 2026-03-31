param(
    [string]$Bin = "scoria.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $Bin)) {
    throw "Binary not found: $Bin"
}

Write-Host "[windows] smoke: version"
& $Bin --version *> $null

Write-Host "[windows] smoke: help"
& $Bin --help *> $null

Write-Host "[windows] smoke: command help"
& $Bin save --help *> $null
& $Bin settings-gui --help *> $null

Write-Host "[windows] smoke: deterministic save path (temp vault + clipboard)"
$vaultDir = Join-Path $env:TEMP ("scoria-smoke-vault-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $vaultDir -Force | Out-Null

$cfgDir = Join-Path $env:APPDATA "scoria"
$cfgPath = Join-Path $cfgDir "config.toml"
$backupPath = Join-Path $env:TEMP ("scoria-smoke-config-backup-" + [guid]::NewGuid().ToString("N") + ".toml")
$hadConfig = Test-Path -LiteralPath $cfgPath
if ($hadConfig) {
    Copy-Item -LiteralPath $cfgPath -Destination $backupPath -Force
}

try {
    New-Item -ItemType Directory -Path $cfgDir -Force | Out-Null
    $cfg = @"
vault_path = '$vaultDir'
target = 'append_to_file'
folder = 'scoria'
append_file = 'Scoria.md'
filename_template = 'clip-%Y-%m-%d-%H%M%S.md'
prepend_timestamp_header = true
autostart = false
auto_update = false
language = 'en'
"@
    Set-Content -LiteralPath $cfgPath -Value $cfg -Encoding UTF8

    Set-Clipboard -Value "scoria smoke clipboard text"

    $tempOut = Join-Path $env:TEMP "scoria-smoke-save-out.txt"
    $tempErr = Join-Path $env:TEMP "scoria-smoke-save-err.txt"
    Remove-Item $tempOut, $tempErr -ErrorAction SilentlyContinue
    $p = Start-Process -FilePath $Bin -ArgumentList @("save") -Wait -PassThru -NoNewWindow `
        -RedirectStandardOutput $tempOut -RedirectStandardError $tempErr
    $exitCode = $p.ExitCode
    $text = ""
    if (Test-Path -LiteralPath $tempOut) {
        $text += Get-Content -LiteralPath $tempOut -Raw -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $tempErr) {
        $text += Get-Content -LiteralPath $tempErr -Raw -ErrorAction SilentlyContinue
    }
    Remove-Item $tempOut, $tempErr -ErrorAction SilentlyContinue

    if ($exitCode -ne 0) {
        throw "Expected successful save in smoke test. Exit $exitCode. Output:`n$text"
    }

    $savedPath = ($text.Trim() -split "`r?`n")[-1].Trim()
    if ([string]::IsNullOrWhiteSpace($savedPath) -or -not (Test-Path -LiteralPath $savedPath)) {
        throw "Expected saved file path in output and existing file. Output:`n$text"
    }
}
finally {
    if ($hadConfig) {
        Copy-Item -LiteralPath $backupPath -Destination $cfgPath -Force
    }
    else {
        Remove-Item -LiteralPath $cfgPath -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath $backupPath -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $vaultDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "[windows] smoke: OK"
