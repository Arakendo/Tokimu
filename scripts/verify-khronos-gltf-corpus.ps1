[CmdletBinding()]
param(
    [string]$FixtureRoot = (Join-Path $PSScriptRoot "..\third-party\fixtures\khronos-gltf-sample-assets")
)

$ErrorActionPreference = "Stop"

$fixtureRoot = (Resolve-Path -LiteralPath $FixtureRoot).Path
$provenancePath = Join-Path $fixtureRoot "provenance.json"
$manifestPath = Join-Path $fixtureRoot "selected\selection-v1.toml"

foreach ($required in @($provenancePath, $manifestPath)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Missing Khronos glTF corpus input: $required"
    }
}

$provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json
$manifest = Get-Content -LiteralPath $manifestPath -Raw
$revision = $provenance.source.revision
if ($manifest -notmatch [regex]::Escape("revision = `"$revision`"")) {
    throw "Selection revision does not match provenance revision $revision"
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
    $id = [regex]::Match($caseText, '(?m)^id\s*=\s*"([^"]+)"').Groups[1].Value
    $source = [regex]::Match($caseText, '(?m)^source\s*=\s*"([^"]+)"').Groups[1].Value
    $expectedHash = [regex]::Match(
        $caseText,
        '(?m)^source_sha256\s*=\s*"([0-9a-f]{64})"'
    ).Groups[1].Value
    if (-not $id -or -not $source -or -not $expectedHash) {
        throw "Case is missing id, source, or source_sha256: $caseText"
    }

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
    $dependencies = @(
        [regex]::Matches($dependencyLine, '"([^"]+)"') |
            ForEach-Object { $_.Groups[1].Value }
    )
    $dependencyHashes = @(
        [regex]::Matches($dependencyHashLine, '"([0-9a-f]{64})"') |
            ForEach-Object { $_.Groups[1].Value }
    )
    if ($dependencies.Count -ne $dependencyHashes.Count) {
        throw "Case $id dependency and dependency_sha256 counts differ"
    }
    for ($index = 0; $index -lt $dependencies.Count; $index++) {
        $dependencyPath = Join-Path (Join-Path $fixtureRoot "upstream") $dependencies[$index]
        if (-not (Test-Path -LiteralPath $dependencyPath)) {
            throw "Case $id references missing dependency: $dependencyPath"
        }
        $actualDependencyHash = (
            Get-FileHash -LiteralPath $dependencyPath -Algorithm SHA256
        ).Hash.ToLowerInvariant()
        if ($actualDependencyHash -ne $dependencyHashes[$index]) {
            throw "Case $id dependency checksum mismatch for $($dependencies[$index])"
        }
    }
}

$logicalModels = @(
    $caseMatches |
        ForEach-Object {
            [regex]::Match(
                $_.Groups[1].Value,
                '(?m)^logical_model\s*=\s*"([^"]+)"'
            ).Groups[1].Value
        } |
        Where-Object { $_ } |
        Sort-Object -Unique
)

$triangleBuffer = Join-Path $fixtureRoot "upstream\Models\Triangle\glTF\Triangle.bin"
if (-not (Test-Path -LiteralPath $triangleBuffer)) {
    throw "Missing Triangle external buffer: $triangleBuffer"
}

Write-Host "Khronos glTF corpus verification passed"
Write-Host "  fixture: $fixtureRoot"
Write-Host "  revision: $revision"
Write-Host "  selected cases: $($caseMatches.Count)"
Write-Host "  logical models: $($logicalModels.Count)"
