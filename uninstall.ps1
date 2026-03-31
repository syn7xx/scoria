param(
    [string]$InstallDir = "$env:USERPROFILE\Tools\scoria"
)

$ErrorActionPreference = "Stop"

Write-Host "Scoria Windows uninstall helper (portable ZIP install)"
Write-Host "If Scoria was installed via MSI, uninstall it from 'Apps & Features' or run:"
Write-Host "  msiexec /x scoria-windows-x86_64.msi"
Write-Host ""

$msiInstallDir = Join-Path ${env:ProgramFiles} "Scoria"
if (Test-Path $msiInstallDir) {
    Write-Host "Note: MSI install directory detected at '$msiInstallDir'."
}

function Remove-FromUserPath {
    param([string]$Dir)

    $fullDir = [System.IO.Path]::GetFullPath($Dir)
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @($userPath -split ';' | Where-Object { $_ -and $_.Trim() -ne "" })
    $filtered = @($parts | Where-Object { $_ -ne $fullDir })

    if ($filtered.Count -eq $parts.Count) {
        Write-Host "Path entry not found in user PATH: $fullDir"
        return
    }

    [Environment]::SetEnvironmentVariable("Path", ($filtered -join ';'), "User")
    Write-Host "Removed from user PATH: $fullDir"
}

$exePath = Join-Path $InstallDir "scoria.exe"
if (Test-Path $exePath) {
    Remove-Item -Force $exePath
    Write-Host "Removed: $exePath"
}

if (Test-Path $InstallDir) {
    $remaining = Get-ChildItem -Path $InstallDir -Force
    if ($remaining.Count -eq 0) {
        Remove-Item -Force $InstallDir
        Write-Host "Removed empty directory: $InstallDir"
    }
}

Remove-FromUserPath -Dir $InstallDir

# Remove autostart entry if present.
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "Scoria" /f *> $null

Write-Host ""
Write-Host "Done. Restart your terminal to refresh PATH."
