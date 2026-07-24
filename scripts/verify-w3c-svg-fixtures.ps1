[CmdletBinding()]
param(
    [string]$FixtureRoot = (Join-Path $PSScriptRoot "..\third-party\fixtures\w3c-svg-1.1-2nd-edition")
)

$ErrorActionPreference = "Stop"

$fixtureRoot = (Resolve-Path -LiteralPath $FixtureRoot).Path
$provenancePath = Join-Path $fixtureRoot "provenance.json"
$manifestPath = Join-Path $fixtureRoot "selected\selection-v1.toml"
$archivePath = Join-Path $fixtureRoot "W3C_SVG_11_TestSuite.tar.gz"
$upstreamPath = Join-Path $fixtureRoot "upstream"

foreach ($required in @($provenancePath, $manifestPath, $archivePath, $upstreamPath)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Missing W3C fixture input: $required"
    }
}

$provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json
$expectedHash = $provenance.source.sha256.ToLowerInvariant()
$actualHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualHash -ne $expectedHash) {
    throw "Archive checksum mismatch. Expected $expectedHash, got $actualHash"
}

$sourceRoot = Join-Path $fixtureRoot "upstream\svg"
$ids = Select-String -LiteralPath $manifestPath -Pattern '^id\s*=\s*"([^"]+)"' |
    ForEach-Object { $_.Matches[0].Groups[1].Value }
if ($ids.Count -eq 0) {
    throw "Selection manifest contains no cases: $manifestPath"
}

$missing = @($ids | Where-Object { -not (Test-Path -LiteralPath (Join-Path $sourceRoot $_)) })
if ($missing.Count -gt 0) {
    throw "Selection references missing upstream SVG files: $($missing -join ', ')"
}

$svgCount = @(Get-ChildItem -LiteralPath $sourceRoot -File -Filter "*.svg").Count
$pngRoot = Join-Path $fixtureRoot "upstream\png"
$pngCount = if (Test-Path -LiteralPath $pngRoot) {
    @(Get-ChildItem -LiteralPath $pngRoot -File -Filter "*.png").Count
} else {
    0
}

Write-Host "W3C SVG fixture verification passed"
Write-Host "  fixture: $fixtureRoot"
Write-Host "  archive sha256: $actualHash"
Write-Host "  upstream SVG files: $svgCount"
Write-Host "  upstream PNG files: $pngCount"
Write-Host "  selected SVG cases: $($ids.Count)"
