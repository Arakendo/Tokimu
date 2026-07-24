[CmdletBinding()]
param(
    [string]$FixtureRoot = (Join-Path $PSScriptRoot "..\third-party\fixtures\w3c-xml-20130923")
)

$ErrorActionPreference = "Stop"

$fixtureRoot = (Resolve-Path -LiteralPath $FixtureRoot).Path
$provenancePath = Join-Path $fixtureRoot "provenance.json"
$manifestPath = Join-Path $fixtureRoot "selected\selection-v1.toml"
$archivePath = Join-Path $fixtureRoot "xmlts20130923.tar.gz"
$sourceRoot = Join-Path $fixtureRoot "upstream\xmlconf"

foreach ($required in @($provenancePath, $manifestPath, $archivePath, $sourceRoot)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Missing W3C XML fixture input: $required"
    }
}

$provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json
$expectedHash = $provenance.source.sha256.ToLowerInvariant()
$actualHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualHash -ne $expectedHash) {
    throw "Archive checksum mismatch. Expected $expectedHash, got $actualHash"
}

$paths = Select-String -LiteralPath $manifestPath -Pattern '^path\s*=\s*"([^"]+)"' |
    ForEach-Object { $_.Matches[0].Groups[1].Value }
if ($paths.Count -eq 0) {
    throw "Selection manifest contains no cases: $manifestPath"
}

$missing = @($paths | Where-Object { -not (Test-Path -LiteralPath (Join-Path $sourceRoot $_)) })
if ($missing.Count -gt 0) {
    throw "Selection references missing upstream XML files: $($missing -join ', ')"
}

$xmlCount = @(Get-ChildItem -LiteralPath $sourceRoot -Recurse -File -Filter "*.xml").Count

Write-Host "W3C XML fixture verification passed"
Write-Host "  fixture: $fixtureRoot"
Write-Host "  archive sha256: $actualHash"
Write-Host "  upstream XML files: $xmlCount"
Write-Host "  selected XML cases: $($paths.Count)"
