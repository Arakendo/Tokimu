[CmdletBinding()]
param(
    [string]$FixtureRoot = (Join-Path $PSScriptRoot "..\third-party\fixtures\fbx-corpus")
)

$ErrorActionPreference = "Stop"

$fixtureRoot = (Resolve-Path -LiteralPath $FixtureRoot).Path
$provenancePath = Join-Path $fixtureRoot "provenance.json"
$inventoryPath = Join-Path $fixtureRoot "inventory.json"
$manifestPath = Join-Path $fixtureRoot "selected\selection-v1.toml"

foreach ($required in @($provenancePath, $inventoryPath, $manifestPath)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Missing FBX corpus input: $required"
    }
}

$provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json
$inventory = Get-Content -LiteralPath $inventoryPath -Raw | ConvertFrom-Json
$manifest = Get-Content -LiteralPath $manifestPath -Raw
$revision = $provenance.source.revision

if ($manifest -notmatch [regex]::Escape("revision = `"$revision`"")) {
    throw "Selection revision does not match provenance revision $revision"
}
if ($inventory.source_revision -ne $revision) {
    throw "Inventory revision does not match provenance revision $revision"
}

$licensePath = Join-Path $fixtureRoot $provenance.license.upstream_notice
if (-not (Test-Path -LiteralPath $licensePath)) {
    throw "Missing upstream license notice: $licensePath"
}
$licenseHash = (Get-FileHash -LiteralPath $licensePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($licenseHash -ne $provenance.license.upstream_notice_sha256) {
    throw "Upstream license checksum mismatch"
}

$caseMatches = [regex]::Matches(
    $manifest,
    '(?ms)^\[\[case\]\]\s*(.*?)(?=^\[\[case\]\]|\z)'
)
if ($caseMatches.Count -eq 0) {
    throw "Selection manifest contains no cases: $manifestPath"
}

$ids = @{}
$sources = @{}
$logicalScenes = @{}
$encodings = @{}
$exporters = @{}
$versions = @{}
$expected = @{}
$dependencies = @{}

foreach ($caseMatch in $caseMatches) {
    $caseText = $caseMatch.Groups[1].Value
    $id = [regex]::Match($caseText, '(?m)^id\s*=\s*"([^"]+)"').Groups[1].Value
    $source = [regex]::Match($caseText, '(?m)^source\s*=\s*"([^"]+)"').Groups[1].Value
    $expectedHash = [regex]::Match(
        $caseText,
        '(?m)^source_sha256\s*=\s*"([0-9a-f]{64})"'
    ).Groups[1].Value
    $logicalScene = [regex]::Match($caseText, '(?m)^logical_scene\s*=\s*"([^"]+)"').Groups[1].Value
    $encoding = [regex]::Match($caseText, '(?m)^encoding\s*=\s*"([^"]+)"').Groups[1].Value
    $exporter = [regex]::Match($caseText, '(?m)^exporter\s*=\s*"([^"]+)"').Groups[1].Value
    $version = [regex]::Match($caseText, '(?m)^fbx_version\s*=\s*(\d+)').Groups[1].Value
    $expectedResult = [regex]::Match($caseText, '(?m)^expected\s*=\s*"([^"]+)"').Groups[1].Value
    if (-not $id -or -not $source -or -not $expectedHash -or -not $logicalScene -or
        -not $encoding -or -not $exporter -or -not $version -or -not $expectedResult) {
        throw "Case is missing required metadata: $caseText"
    }
    if ($ids.ContainsKey($id)) {
        throw "Duplicate case id: $id"
    }
    if ($sources.ContainsKey($source)) {
        throw "Duplicate authoritative source: $source"
    }
    $ids[$id] = $true
    $sources[$source] = $true
    $logicalScenes[$logicalScene] = $true
    $encodings[$encoding] = 1 + ($encodings[$encoding] ?? 0)
    $exporters[$exporter] = $true
    $versions[$version] = $true
    $expected[$expectedResult] = 1 + ($expected[$expectedResult] ?? 0)

    $sourcePath = Join-Path (Join-Path $fixtureRoot "upstream") $source
    if (-not (Test-Path -LiteralPath $sourcePath)) {
        throw "Case $id references missing source: $sourcePath"
    }
    $actualHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $expectedHash) {
        throw "Case $id checksum mismatch. Expected $expectedHash, got $actualHash"
    }

    $dependencyLine = [regex]::Match(
        $caseText,
        '(?m)^dependencies\s*=\s*\[(.*)\]'
    ).Groups[1].Value
    $dependencyHashLine = [regex]::Match(
        $caseText,
        '(?m)^dependency_sha256\s*=\s*\[(.*)\]'
    ).Groups[1].Value
    $caseDependencies = @(
        [regex]::Matches($dependencyLine, '"([^"]+)"') |
            ForEach-Object { $_.Groups[1].Value }
    )
    $dependencyHashes = @(
        [regex]::Matches($dependencyHashLine, '"([0-9a-f]{64})"') |
            ForEach-Object { $_.Groups[1].Value }
    )
    if ($caseDependencies.Count -ne $dependencyHashes.Count) {
        throw "Case $id dependency and dependency_sha256 counts differ"
    }
    for ($index = 0; $index -lt $caseDependencies.Count; $index++) {
        $dependency = $caseDependencies[$index]
        $dependencyPath = Join-Path (Join-Path $fixtureRoot "upstream") $dependency
        if (-not (Test-Path -LiteralPath $dependencyPath)) {
            throw "Case $id references missing dependency: $dependencyPath"
        }
        $actualDependencyHash = (
            Get-FileHash -LiteralPath $dependencyPath -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        if ($actualDependencyHash -ne $dependencyHashes[$index]) {
            throw "Case $id dependency checksum mismatch for $dependency"
        }
        $dependencies[$dependency] = $true
    }
}

$selected = $inventory.selection_v1
$checks = @{
    "fbx_cases" = $caseMatches.Count
    "logical_scenes" = $logicalScenes.Count
    "dependency_files" = $dependencies.Count
    "ascii_cases" = ($encodings["ascii"] ?? 0)
    "binary_cases" = ($encodings["binary"] ?? 0)
    "exporter_classes" = $exporters.Count
    "expected_valid" = ($expected["valid-source"] ?? 0)
    "expected_invalid" = ($expected["invalid-source"] ?? 0)
}
foreach ($entry in $checks.GetEnumerator()) {
    if ([int]$selected.($entry.Key) -ne [int]$entry.Value) {
        throw "Inventory $($entry.Key) expected $($selected.($entry.Key)), observed $($entry.Value)"
    }
}
if ($selected.fbx_versions.Count -ne $versions.Count) {
    throw "Inventory fbx_versions expected $($selected.fbx_versions.Count), observed $($versions.Count)"
}

Write-Host "ufbx FBX corpus verification passed"
Write-Host "  fixture: $fixtureRoot"
Write-Host "  revision: $revision"
Write-Host "  selected cases: $($caseMatches.Count)"
Write-Host "  logical scenes: $($logicalScenes.Count)"
Write-Host "  dependencies: $($dependencies.Count)"
