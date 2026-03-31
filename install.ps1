param(
    [string]$Repo = "syn7xx/scoria",
    [string]$InstallDir = "$env:USERPROFILE\Tools\scoria"
)

$ErrorActionPreference = "Stop"

function Get-LatestTag {
    param([string]$Repository)
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest"
    if (-not $release.tag_name) {
        throw "Could not fetch latest release tag."
    }
    return [string]$release.tag_name
}

function Add-ToUserPath {
    param([string]$Dir)

    $fullDir = [System.IO.Path]::GetFullPath($Dir)
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @($userPath -split ';' | Where-Object { $_ -and $_.Trim() -ne "" })

    if ($parts -contains $fullDir) {
        Write-Host "Already in user PATH: $fullDir"
        return
    }

    $newPath = if ($parts.Count -gt 0) {
        ($parts + $fullDir) -join ';'
    } else {
        $fullDir
    }

    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "Added to user PATH: $fullDir"
}

$tag = Get-LatestTag -Repository $Repo
$asset = "scoria-windows-x86_64.zip"
$checksumAsset = "scoria-windows-x86_64.sha256"
$url = "https://github.com/$Repo/releases/download/$tag/$asset"
$checksumUrl = "https://github.com/$Repo/releases/download/$tag/$checksumAsset"

Write-Host "Installing scoria $tag (windows/x86_64)..."
Write-Host "  from: $url"
Write-Host "  to:   $InstallDir\scoria.exe"

$msiInstallDir = Join-Path ${env:ProgramFiles} "Scoria"
if (Test-Path $msiInstallDir) {
    Write-Warning "Detected MSI-style install directory at '$msiInstallDir'. Portable install in '$InstallDir' may create PATH/version ambiguity."
}

$zipPath = Join-Path $env:TEMP "scoria-windows-x86_64.zip"
$shaPath = Join-Path $env:TEMP "scoria-windows-x86_64.zip.sha256"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Invoke-WebRequest -Uri $url -OutFile $zipPath
Invoke-WebRequest -Uri $checksumUrl -OutFile $shaPath

$expectedSha = (Get-Content -Path $shaPath -Raw).Trim().Split(" ")[0].ToLowerInvariant()
$actualSha = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expectedSha -ne $actualSha) {
    throw "Checksum mismatch for downloaded archive."
}

Expand-Archive -Path $zipPath -DestinationPath $InstallDir -Force

if (-not (Test-Path (Join-Path $InstallDir "scoria.exe"))) {
    throw "Installation failed: scoria.exe was not found after extraction."
}

Add-ToUserPath -Dir $InstallDir

Write-Host ""
Write-Host "Done. Restart your terminal and run:"
Write-Host "  scoria --version"
