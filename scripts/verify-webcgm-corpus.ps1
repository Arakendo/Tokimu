[CmdletBinding()]
param(
    [string]$FixtureRoot = (
        Join-Path $PSScriptRoot "..\third-party\fixtures\webcgm-test-suite"
    )
)

$ErrorActionPreference = "Stop"

function Get-TreeHash {
    param([Parameter(Mandatory)][string]$Root)

    $lines = Get-ChildItem -LiteralPath $Root -Recurse -File |
        ForEach-Object {
            $relative = (
                [IO.Path]::GetRelativePath($Root, $_.FullName)
            ).Replace("\", "/")
            $hash = (
                Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256
            ).Hash.ToLowerInvariant()
            "$relative`t$hash`n"
        } |
        Sort-Object
    $bytes = [Text.Encoding]::UTF8.GetBytes(($lines -join ""))
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        return [Convert]::ToHexString($sha.ComputeHash($bytes)).
            ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

$fixtureRoot = (Resolve-Path -LiteralPath $FixtureRoot).Path
$upstreamRoot = Join-Path $fixtureRoot "upstream"
$provenancePath = Join-Path $fixtureRoot "provenance.json"
$inventoryPath = Join-Path $fixtureRoot "inventory.json"
$manifestPath = Join-Path $fixtureRoot "selected\selection-v1.toml"
$featureMatrixPath = Join-Path $fixtureRoot "selected\feature-matrix.md"

foreach ($required in @(
    $upstreamRoot,
    $provenancePath,
    $inventoryPath,
    $manifestPath,
    $featureMatrixPath,
    (Join-Path $upstreamRoot "copyright-license.html")
)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Missing WebCGM corpus input: $required"
    }
}

$provenance = Get-Content -LiteralPath $provenancePath -Raw |
    ConvertFrom-Json
$inventory = Get-Content -LiteralPath $inventoryPath -Raw |
    ConvertFrom-Json
$manifest = Get-Content -LiteralPath $manifestPath -Raw

if ($inventory.generated_from_archive_sha256 -ne $provenance.source.sha256) {
    throw "Inventory archive identity does not match provenance"
}

$actualTreeHash = Get-TreeHash -Root $upstreamRoot
if ($actualTreeHash -ne $inventory.upstream_tree_sha256) {
    throw (
        "Upstream tree checksum mismatch. Expected " +
        "$($inventory.upstream_tree_sha256), got $actualTreeHash"
    )
}

$files = @(Get-ChildItem -LiteralPath $upstreamRoot -Recurse -File)
$cgmFiles = @($files | Where-Object Extension -EQ ".cgm")
$pngFiles = @($files | Where-Object Extension -EQ ".png")
if ($files.Count -ne $inventory.totals.files) {
    throw "Inventory file count does not match upstream"
}
if ($cgmFiles.Count -ne $inventory.totals.cgm_files) {
    throw "Inventory CGM count does not match upstream"
}
if ($pngFiles.Count -ne $inventory.totals.reference_png_files) {
    throw "Inventory PNG count does not match upstream"
}

$classification = $inventory.classification
if (-not $classification -or $classification.schema -ne 1) {
    throw "Inventory is missing classification schema 1"
}
$expectedCategories = @(
    "geometry",
    "text",
    "raster",
    "dom",
    "hyperlink",
    "interaction",
    "profile",
    "support"
)
foreach ($category in $expectedCategories) {
    if ($null -eq $classification.categories.$category) {
        throw "Inventory classification is missing category: $category"
    }
}
$classifiedCases = @($classification.cases)
if ($classifiedCases.Count -ne $cgmFiles.Count) {
    throw "Inventory classification count does not match upstream CGM files"
}
$classifiedSources = @($classifiedCases | ForEach-Object source | Sort-Object -Unique)
if ($classifiedSources.Count -ne $cgmFiles.Count) {
    throw "Inventory classification does not assign every CGM source exactly once"
}
$classifiedTotal = @($expectedCategories | ForEach-Object {
    [int]$classification.categories.$_
} | Measure-Object -Sum).Sum
if ($classifiedTotal -ne $cgmFiles.Count) {
    throw "Inventory category totals do not match upstream CGM files"
}

$caseMatches = [regex]::Matches(
    $manifest,
    '(?ms)^\[\[case\]\]\s*(.*?)(?=^\[\[case\]\]|\z)'
)
if ($caseMatches.Count -eq 0) {
    throw "Selection manifest contains no cases: $manifestPath"
}

foreach ($caseMatch in $caseMatches) {
    $caseText = $caseMatch.Groups[1].Value
    $id = [regex]::Match(
        $caseText,
        '(?m)^id\s*=\s*"([^"]+)"'
    ).Groups[1].Value
    $source = [regex]::Match(
        $caseText,
        '(?m)^source\s*=\s*"([^"]+)"'
    ).Groups[1].Value
    $sourceHash = [regex]::Match(
        $caseText,
        '(?m)^source_sha256\s*=\s*"([0-9a-f]{64})"'
    ).Groups[1].Value
    $reference = [regex]::Match(
        $caseText,
        '(?m)^reference_image\s*=\s*"([^"]+)"'
    ).Groups[1].Value
    $referenceHash = [regex]::Match(
        $caseText,
        '(?m)^reference_sha256\s*=\s*"([0-9a-f]{64})"'
    ).Groups[1].Value

    if (-not $id -or -not $source -or -not $sourceHash) {
        throw "Case is missing id, source, or source_sha256: $caseText"
    }

    $sourcePath = Join-Path $upstreamRoot $source
    if (-not (Test-Path -LiteralPath $sourcePath)) {
        throw "Case $id references missing source: $sourcePath"
    }
    $actualSourceHash = (
        Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256
    ).Hash.ToLowerInvariant()
    if ($actualSourceHash -ne $sourceHash) {
        throw "Case $id source checksum mismatch"
    }

    if ($reference) {
        if (-not $referenceHash) {
            throw "Case $id has a reference image without a checksum"
        }
        $referencePath = Join-Path $upstreamRoot $reference
        if (-not (Test-Path -LiteralPath $referencePath)) {
            throw "Case $id references missing image: $referencePath"
        }
        $actualReferenceHash = (
            Get-FileHash -LiteralPath $referencePath -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        if ($actualReferenceHash -ne $referenceHash) {
            throw "Case $id reference image checksum mismatch"
        }
    }
}

Write-Host "WebCGM corpus verification passed"
Write-Host "  fixture: $fixtureRoot"
Write-Host "  release: $($provenance.source.release)"
Write-Host "  upstream tree sha256: $actualTreeHash"
Write-Host "  files: $($files.Count)"
Write-Host "  CGM files: $($cgmFiles.Count)"
Write-Host "  geometry-classified CGM files: $($classification.categories.geometry)"
Write-Host "  reference PNG files: $($pngFiles.Count)"
Write-Host "  selected cases: $($caseMatches.Count)"
