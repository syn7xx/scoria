param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$Sha256,
    [string]$Identifier = "syn7xx.Scoria",
    [string]$Publisher = "syn7xx",
    [string]$OutputDir = "packaging/winget/manifests"
)

$ErrorActionPreference = "Stop"

if ($Version.StartsWith("v")) {
    $Version = $Version.Substring(1)
}

$manifestDir = Join-Path $OutputDir $Version
New-Item -ItemType Directory -Force -Path $manifestDir | Out-Null

$installerUrl = "https://github.com/syn7xx/scoria/releases/download/v$Version/scoria-windows-x86_64.zip"
$packageName = "Scoria"

$versionManifest = @"
PackageIdentifier: $Identifier
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.9.0
"@

$installerManifest = @"
PackageIdentifier: $Identifier
PackageVersion: $Version
MinimumOSVersion: 10.0.0.0
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: scoria.exe
    PortableCommandAlias: scoria
Installers:
  - Architecture: x64
    InstallerUrl: $installerUrl
    InstallerSha256: $Sha256
ManifestType: installer
ManifestVersion: 1.9.0
"@

$defaultLocaleManifest = @"
PackageIdentifier: $Identifier
PackageVersion: $Version
PackageLocale: en-US
Publisher: $Publisher
PackageName: $packageName
ShortDescription: Save clipboard content to an Obsidian vault.
License: MIT OR Apache-2.0
PublisherUrl: https://github.com/syn7xx
PackageUrl: https://github.com/syn7xx/scoria
Moniker: scoria
Tags:
  - obsidian
  - clipboard
  - tray
ManifestType: defaultLocale
ManifestVersion: 1.9.0
"@

$idStem = $Identifier

Set-Content -Path (Join-Path $manifestDir "$idStem.yaml") -Value $versionManifest -Encoding utf8
Set-Content -Path (Join-Path $manifestDir "$idStem.installer.yaml") -Value $installerManifest -Encoding utf8
Set-Content -Path (Join-Path $manifestDir "$idStem.locale.en-US.yaml") -Value $defaultLocaleManifest -Encoding utf8

Write-Host "Generated winget manifests in: $manifestDir"
