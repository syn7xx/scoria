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

Write-Host "[windows] smoke: expected failure path (save without clipboard)"
# Native stderr from Rust is often surfaced as ErrorRecord objects; piping to Out-String can yield
# an empty string. Redirect to temp files for reliable capture.
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

if ($exitCode -eq 0) {
    throw "Expected non-zero exit for save without clipboard context (exit $exitCode)"
}
# EN: "Nothing to save — copy something first."  RU: "Нечего сохранять — ..."
if (($text -notmatch "nothing to save") -and ($text -notmatch "Нечего сохранять")) {
    throw "Expected user-facing empty clipboard error. Exit $exitCode. Got:`n$text"
}

Write-Host "[windows] smoke: OK"
