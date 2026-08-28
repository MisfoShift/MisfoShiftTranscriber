[CmdletBinding()]
param(
    [string]$ReleaseDirectory = "",
    [string]$OutputPath = ""
)

if ([string]::IsNullOrWhiteSpace($ReleaseDirectory)) {
    $ReleaseDirectory = Join-Path $PSScriptRoot "..\src-tauri\target\release"
}

$resolvedReleaseDirectory = (Resolve-Path -LiteralPath $ReleaseDirectory -ErrorAction Stop).Path

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $resolvedReleaseDirectory "SHA256SUMS.txt"
}

$files = @()
$applicationPath = Join-Path $resolvedReleaseDirectory "misfo-shift-transcriber.exe"
if (Test-Path -LiteralPath $applicationPath -PathType Leaf) {
    $files += Get-Item -LiteralPath $applicationPath
}

$msiDirectory = Join-Path $resolvedReleaseDirectory "bundle\msi"
if (Test-Path -LiteralPath $msiDirectory -PathType Container) {
    $files += Get-ChildItem -LiteralPath $msiDirectory -Filter "*.msi" -File
}

$nsisDirectory = Join-Path $resolvedReleaseDirectory "bundle\nsis"
if (Test-Path -LiteralPath $nsisDirectory -PathType Container) {
    $files += Get-ChildItem -LiteralPath $nsisDirectory -Filter "*.exe" -File
}

$files = @($files | Sort-Object -Property FullName -Unique)
if ($files.Count -eq 0) {
    throw "SHA-256を生成するWindows配布物が見つかりません: $resolvedReleaseDirectory"
}

$releaseRootUri = [Uri]($resolvedReleaseDirectory.TrimEnd("\") + "\")
$lines = foreach ($file in $files) {
    $fileUri = [Uri]$file.FullName
    $relativePath = [Uri]::UnescapeDataString($releaseRootUri.MakeRelativeUri($fileUri).ToString())
    $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $relativePath"
}

$outputDirectory = Split-Path -Parent $OutputPath
if ($outputDirectory -and -not (Test-Path -LiteralPath $outputDirectory -PathType Container)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

$lines | Set-Content -LiteralPath $OutputPath -Encoding ascii
Write-Output "SHA-256一覧を生成しました: $OutputPath"
Write-Output ($lines -join [Environment]::NewLine)
