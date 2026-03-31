param(
    [string]$Bin = "scoria.exe"
)

$ErrorActionPreference = "Stop"

Write-Host "[windows] smoke: version"
& $Bin --version *> $null

Write-Host "[windows] smoke: help"
& $Bin --help *> $null

Write-Host "[windows] smoke: command help"
& $Bin save --help *> $null
& $Bin settings-gui --help *> $null

Write-Host "[windows] smoke: expected failure path (save without clipboard)"
$saveOutput = & $Bin save 2>&1
if ($LASTEXITCODE -eq 0) {
    throw "Expected non-zero exit for save without clipboard context"
}
$text = ($saveOutput | Out-String)
if (($text -notmatch "nothing to save") -and ($text -notmatch "Нечего сохранять")) {
    throw "Expected user-facing empty clipboard error. Got:`n$text"
}

Write-Host "[windows] smoke: OK"
