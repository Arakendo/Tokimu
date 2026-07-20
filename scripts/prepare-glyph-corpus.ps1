[CmdletBinding()]
param(
    [string]$Output = "target/glyph-corpus",
    [int]$MaxIcons = 0,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\")).Path
$outputRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot $Output))

$providers = @{
    lucide = Join-Path $repoRoot "third-party\glyph-providers\lucide"
    inter = Join-Path $repoRoot "third-party\fonts\inter"
    jetbrains_mono = Join-Path $repoRoot "third-party\fonts\jetbrains-mono"
    noto = Join-Path $repoRoot "third-party\fonts\noto"
}

if ($Clean -and (Test-Path -LiteralPath $outputRoot)) {
    Remove-Item -LiteralPath $outputRoot -Recurse -Force
}

New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
$iconsRoot = Join-Path $outputRoot "icons"
$fontsRoot = Join-Path $outputRoot "fonts"
New-Item -ItemType Directory -Path $iconsRoot, $fontsRoot -Force | Out-Null

function Assert-Provider([string]$Name) {
    $path = $providers[$Name]
    if (-not (Test-Path -LiteralPath $path -PathType Container)) {
        throw "Missing provider '$Name' at '$path'. Initialize submodules before preparing the corpus."
    }
}

foreach ($name in $providers.Keys) {
    Assert-Provider $name
}

$iconQuery = @(Get-ChildItem -LiteralPath $providers.lucide -Recurse -File -Filter "*.svg" |
    Where-Object { $_.FullName -notmatch "\\node_modules\\|\\dist\\|\\docs\\" } |
    Sort-Object FullName)
$iconFiles = if ($MaxIcons -gt 0) { $iconQuery | Select-Object -First $MaxIcons } else { $iconQuery }

foreach ($icon in $iconFiles) {
    $relativeIcon = $icon.FullName.Substring($providers.lucide.Length).TrimStart([char]92, [char]47)
    $iconDestination = Join-Path $iconsRoot $relativeIcon
    New-Item -ItemType Directory -Path (Split-Path $iconDestination) -Force | Out-Null
    Copy-Item -LiteralPath $icon.FullName -Destination $iconDestination -Force
}

$fontPatterns = @("*.ttf", "*.otf", "*.woff2")
$fontFiles = foreach ($name in @("inter", "jetbrains_mono", "noto")) {
    foreach ($pattern in $fontPatterns) {
        Get-ChildItem -LiteralPath $providers[$name] -Recurse -File -Filter $pattern -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -notmatch "\\node_modules\\|\\test\\|\\tests\\" } |
            ForEach-Object {
                [PSCustomObject]@{ Provider = $name; File = $_ }
            }
    }
}

foreach ($font in $fontFiles) {
    $providerDir = Join-Path $fontsRoot $font.Provider
    $relativeFont = $font.File.FullName.Substring($providers[$font.Provider].Length).TrimStart([char]92, [char]47)
    $fontDestination = Join-Path $providerDir $relativeFont
    New-Item -ItemType Directory -Path (Split-Path $fontDestination) -Force | Out-Null
    Copy-Item -LiteralPath $font.File.FullName -Destination $fontDestination -Force
}

$textFixture = @'
ASCII: ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz 0123456789
Punctuation: ! ? + - = / \ : ; , . ( ) [ ] { } @ # $ % & * _
Whitespace: one two    three
Unicode: Cafe cafe, naive naive, こんにちは, Привет, مرحبا, 中文
Emoji: ^_^ [ok] (fallback coverage depends on the selected font)
'@
Set-Content -LiteralPath (Join-Path $outputRoot "text-fixture.txt") -Value $textFixture -Encoding utf8

$manifest = [ordered]@{
    generated_at = [DateTime]::UtcNow.ToString("o")
    purpose = "Tokimu glyph and text rendering reference corpus"
    max_icons = $MaxIcons
    providers = [ordered]@{}
    counts = [ordered]@{
        icons = $iconFiles.Count
        fonts = $fontFiles.Count
    }
    fixtures = @("text-fixture.txt")
}

foreach ($name in $providers.Keys) {
    $revision = git -C $providers[$name] rev-parse HEAD 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($revision)) {
        $revision = "unknown"
    } else {
        $revision = $revision.Trim()
    }
    $relativePath = $providers[$name].Substring($repoRoot.Length).TrimStart([char]92, [char]47)
    $manifest.providers[$name] = [ordered]@{
        path = $relativePath.Replace([char]92, [char]47)
        revision = $revision
    }
}

$manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $outputRoot "manifest.json") -Encoding utf8

Write-Host "Prepared glyph corpus at $outputRoot"
Write-Host ("  Icons: {0}" -f $iconFiles.Count)
Write-Host ("  Fonts: {0}" -f $fontFiles.Count)
Write-Host "  Fixture: text-fixture.txt"
