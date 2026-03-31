param(
    [string]$Bin = "scoria.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $Bin)) {
    throw "Binary not found: $Bin"
}

$BinExe = (Resolve-Path -LiteralPath $Bin).Path

Write-Host "[windows] smoke: version"
& $BinExe --version *> $null

Write-Host "[windows] smoke: help"
& $BinExe --help *> $null

Write-Host "[windows] smoke: command help"
& $BinExe save --help *> $null
& $BinExe settings-gui --help *> $null

# Do not run `scoria save` here: headless GitHub Windows runners often crash in native
# clipboard / arboard (e.g. 0xC0000139) before Rust can report an error. Validate full
# save on a real desktop session (see docs/smoke-tests.md).

Write-Host "[windows] smoke: OK"
