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
$manifestLines = Get-Content -LiteralPath $manifestPath
$entries = @()
$kind = $null

foreach ($line in $manifestLines) {
    if ($line -match '^\[\[(case|derived_case|local_case)\]\]') {
        $kind = $Matches[1]
        continue
    }

    if ($line -match '^id\s*=\s*"([^"]+)"') {
        if ($null -eq $kind) {
            throw "Selection case ID appears before a case section: $line"
        }

        $entries += [pscustomobject]@{
            Kind = $kind
            Id = $Matches[1]
            Upstream = $null
        }
        continue
    }

    if ($line -match '^upstream\s*=\s*"([^"]+)"') {
        if ($entries.Count -eq 0) {
            throw "Selection upstream reference appears before a case ID: $line"
        }

        $entries[-1].Upstream = $Matches[1]
    }
}

if ($entries.Count -eq 0) {
    throw "Selection manifest contains no cases: $manifestPath"
}

$derivedRoot = Join-Path $fixtureRoot "selected\derived"
$missingSources = @()
$missingDerived = @()
$representedSources = @()

foreach ($entry in $entries) {
    if ($entry.Kind -eq "case") {
        $representedSources += $entry.Id
        if (-not (Test-Path -LiteralPath (Join-Path $sourceRoot $entry.Id))) {
            $missingSources += $entry.Id
        }
        continue
    }

    if ($entry.Kind -eq "derived_case") {
        if ([string]::IsNullOrWhiteSpace($entry.Upstream)) {
            throw "Derived case '$($entry.Id)' does not record an upstream source"
        }
        if ($entry.Upstream -ne "derived-local") {
            $representedSources += $entry.Upstream
            if (-not (Test-Path -LiteralPath (Join-Path $sourceRoot $entry.Upstream))) {
                $missingSources += $entry.Upstream
            }
        }
        if (-not (Test-Path -LiteralPath (Join-Path $derivedRoot $entry.Id))) {
            $missingDerived += $entry.Id
        }
    }
}

if ($missingSources.Count -gt 0) {
    $uniqueMissing = $missingSources | Sort-Object -Unique
    throw "Selection references missing upstream SVG files: $($uniqueMissing -join ', ')"
}

if ($missingDerived.Count -gt 0) {
    throw "Selection references missing derived SVG files: $($missingDerived -join ', ')"
}

$svgCount = @(Get-ChildItem -LiteralPath $sourceRoot -Recurse -File -Filter "*.svg").Count
$pngRoot = Join-Path $fixtureRoot "upstream\png"
$pngCount = if (Test-Path -LiteralPath $pngRoot) {
    @(Get-ChildItem -LiteralPath $pngRoot -Recurse -File -Filter "*.png").Count
} else {
    0
}

Write-Host "W3C SVG fixture verification passed"
Write-Host "  fixture: $fixtureRoot"
Write-Host "  archive sha256: $actualHash"
Write-Host "  upstream SVG files: $svgCount"
Write-Host "  upstream PNG files: $pngCount"
Write-Host "  selected manifest entries: $($entries.Count)"
Write-Host "  source cases: $(@($entries | Where-Object Kind -eq 'case').Count)"
Write-Host "  derived cases: $(@($entries | Where-Object Kind -eq 'derived_case').Count)"
Write-Host "  unique represented conformance SVGs: $(@($representedSources | Sort-Object -Unique).Count)"
