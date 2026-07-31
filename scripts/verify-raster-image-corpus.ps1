[CmdletBinding()]
param(
    [string]$FixtureRoot = (Join-Path $PSScriptRoot "..\third-party\fixtures\raster-images")
)

$ErrorActionPreference = "Stop"

$fixtureRoot = (Resolve-Path -LiteralPath $FixtureRoot).Path
$provenancePath = Join-Path $fixtureRoot "provenance.json"
$inventoryPath = Join-Path $fixtureRoot "inventory.json"
$manifestPath = Join-Path $fixtureRoot "selected\selection-v1.toml"
$jpegManifestPath = Join-Path $fixtureRoot "selected\jpeg-selection-v1.toml"
$jpegDecoderManifestPath = Join-Path $fixtureRoot "selected\jpeg-decoder-selection-v1.toml"
$bmpManifestPath = Join-Path $fixtureRoot "selected\bmp-selection-v1.toml"
$matrixPath = Join-Path $fixtureRoot "selected\feature-matrix.md"
$libjpegRoot = Join-Path $fixtureRoot "upstream\libjpeg-turbo"
$libjpegImageRoot = Join-Path $libjpegRoot "testimages"
$jpegDecoderRoot = Join-Path $fixtureRoot "upstream\jpeg-decoder"

foreach ($required in @(
    $provenancePath,
    $inventoryPath,
    $manifestPath,
    $jpegManifestPath,
    $jpegDecoderManifestPath,
    $bmpManifestPath,
    $matrixPath,
    (Join-Path $fixtureRoot "upstream\PngSuite.LICENSE"),
    (Join-Path $libjpegRoot "LICENSE.md"),
    (Join-Path $libjpegRoot "README.ijg"),
    (Join-Path $libjpegImageRoot "LICENSE.txt"),
    (Join-Path $jpegDecoderRoot "LICENSE-APACHE"),
    (Join-Path $jpegDecoderRoot "LICENSE-MIT")
)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Missing raster fixture input: $required"
    }
}

$provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json
$inventory = Get-Content -LiteralPath $inventoryPath -Raw | ConvertFrom-Json
$sourceRoot = (Resolve-Path -LiteralPath (Join-Path $fixtureRoot $provenance.layout.source_root)).Path
$sourceDocument = (Resolve-Path -LiteralPath (Join-Path $fixtureRoot $provenance.layout.source_document)).Path

$documentHash = (Get-FileHash -LiteralPath $sourceDocument -Algorithm SHA256).Hash.ToLowerInvariant()
$expectedDocumentHash = $provenance.layout.source_document_sha256.ToLowerInvariant()
if ($documentHash -ne $expectedDocumentHash) {
    throw "PNG Suite documentation checksum mismatch. Expected $expectedDocumentHash, got $documentHash"
}

$requiredFields = @(
    "id",
    "source",
    "bytes",
    "sha256",
    "format",
    "capability",
    "reason",
    "expected_stage",
    "expected"
)

function Read-RasterSelection {
    param([string]$Path)

    $selectionCases = @()
    $current = $null

    foreach ($line in Get-Content -LiteralPath $Path) {
        if ($line -match '^\[\[case\]\]') {
            if ($null -ne $current) {
                $selectionCases += [pscustomobject]$current
            }
            $current = @{}
            continue
        }

        if ($null -eq $current) {
            continue
        }

        if ($line -match '^([a-z0-9_]+)\s*=\s*"([^"]*)"') {
            $current[$Matches[1]] = $Matches[2]
            continue
        }

        if ($line -match '^bytes\s*=\s*(\d+)') {
            $current["bytes"] = [int64]$Matches[1]
            continue
        }

        if ($line -match '^capability\s*=\s*\[(.*)\]') {
            $current["capability"] = @(
                [regex]::Matches($Matches[1], '"([^"]+)"') |
                    ForEach-Object { $_.Groups[1].Value }
            )
        }
    }

    if ($null -ne $current) {
        $selectionCases += [pscustomobject]$current
    }

    return @($selectionCases)
}

function Test-RasterSelection {
    param(
        [string]$Path,
        [string]$SelectionSourceRoot,
        [int]$ExpectedCount
    )

    $selectionCases = @(Read-RasterSelection -Path $Path)
    if ($selectionCases.Count -eq 0) {
        throw "Raster selection manifest contains no cases: $Path"
    }

    foreach ($case in $selectionCases) {
        foreach ($field in $requiredFields) {
            if ($case.PSObject.Properties.Name -notcontains $field) {
                throw "Raster case '$($case.id)' is missing required field '$field'"
            }
        }
    }

    $duplicateIds = $selectionCases | Group-Object id | Where-Object Count -gt 1
    if ($duplicateIds) {
        throw "Raster selection contains duplicate IDs: $(($duplicateIds.Name | Sort-Object) -join ', ')"
    }

    $duplicateSources = $selectionCases | Group-Object source | Where-Object Count -gt 1
    if ($duplicateSources) {
        throw "Raster selection contains duplicate source paths: $(($duplicateSources.Name | Sort-Object) -join ', ')"
    }

    foreach ($case in $selectionCases) {
        $sourcePath = Join-Path $SelectionSourceRoot $case.source
        if (-not (Test-Path -LiteralPath $sourcePath)) {
            throw "Raster case '$($case.id)' references missing source '$($case.source)'"
        }

        $source = Get-Item -LiteralPath $sourcePath
        if ($source.Length -ne $case.bytes) {
            throw "Raster case '$($case.id)' size mismatch. Expected $($case.bytes), got $($source.Length)"
        }

        $actualHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actualHash -ne $case.sha256.ToLowerInvariant()) {
            throw "Raster case '$($case.id)' checksum mismatch. Expected $($case.sha256), got $actualHash"
        }
    }

    if ($selectionCases.Count -ne $ExpectedCount) {
        throw "Raster selection count mismatch for '$Path'. Expected $ExpectedCount, got $($selectionCases.Count)"
    }

    return @($selectionCases)
}

$cases = @(Test-RasterSelection `
    -Path $manifestPath `
    -SelectionSourceRoot $sourceRoot `
    -ExpectedCount $inventory.formats.png.selected_files)
$jpegCases = @(Test-RasterSelection `
    -Path $jpegManifestPath `
    -SelectionSourceRoot $libjpegImageRoot `
    -ExpectedCount $inventory.formats.jpeg.libjpeg_turbo_selected_files)
$jpegDecoderCases = @(Test-RasterSelection `
    -Path $jpegDecoderManifestPath `
    -SelectionSourceRoot $jpegDecoderRoot `
    -ExpectedCount $inventory.formats.jpeg.jpeg_decoder_selected_files)
$bmpCases = @(Test-RasterSelection `
    -Path $bmpManifestPath `
    -SelectionSourceRoot $libjpegImageRoot `
    -ExpectedCount $inventory.formats.bmp.selected_files)

$allCases = @($cases) + @($jpegCases) + @($jpegDecoderCases) + @($bmpCases)
$duplicateAllIds = $allCases | Group-Object id | Where-Object Count -gt 1
if ($duplicateAllIds) {
    throw "Raster selections contain duplicate IDs: $(($duplicateAllIds.Name | Sort-Object) -join ', ')"
}

foreach ($license in $provenance.additional_sources.libjpeg_turbo.license_files) {
    $licensePath = Join-Path $fixtureRoot $license.path
    $actualHash = (Get-FileHash -LiteralPath $licensePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $license.sha256.ToLowerInvariant()) {
        throw "libjpeg-turbo license checksum mismatch for '$($license.path)'"
    }
}

foreach ($license in $provenance.additional_sources.jpeg_decoder.license_files) {
    $licensePath = Join-Path $fixtureRoot $license.path
    $actualHash = (Get-FileHash -LiteralPath $licensePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $license.sha256.ToLowerInvariant()) {
        throw "jpeg-decoder license checksum mismatch for '$($license.path)'"
    }
}

foreach ($license in $provenance.source.license_files) {
    $licensePath = Join-Path $fixtureRoot $license.path
    $actualHash = (Get-FileHash -LiteralPath $licensePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $license.sha256.ToLowerInvariant()) {
        throw "PNG Suite license checksum mismatch for '$($license.path)'"
    }
}

$upstreamCount = @(Get-ChildItem -LiteralPath $sourceRoot -File -Filter "*.png").Count
if ($upstreamCount -ne $inventory.upstream_png_files) {
    throw "Raster inventory count mismatch. Expected $($inventory.upstream_png_files), got $upstreamCount"
}

if ($cases.Count -ne $inventory.selected_candidate_files) {
    throw "PNG candidate count mismatch. Expected $($inventory.selected_candidate_files), got $($cases.Count)"
}

if ($cases.Count -ne $inventory.executed_png_candidate_files) {
    throw "Executed PNG candidate count mismatch. Expected $($inventory.executed_png_candidate_files), got $($cases.Count)"
}

$rejectionCount = @($cases | Where-Object expected -eq "candidate-rejection").Count
$jpegRejectionCount = @($jpegCases | Where-Object expected -eq "candidate-rejection").Count
$expectedExecutableExternal = $cases.Count + $jpegCases.Count + $jpegDecoderCases.Count + $bmpCases.Count
if ($inventory.executable_external_files -ne $expectedExecutableExternal) {
    throw "Executable external count mismatch. Expected $expectedExecutableExternal from admitted selections, got $($inventory.executable_external_files)"
}

Write-Host "Raster image fixture verification passed"
Write-Host "  fixture: $fixtureRoot"
Write-Host "  source root: $sourceRoot"
Write-Host "  PNG Suite document sha256: $documentHash"
Write-Host "  upstream PNG files: $upstreamCount"
Write-Host "  selected PNG fixture files: $($cases.Count)"
Write-Host "  executed PNG fixture references: $($inventory.executed_png_candidate_files)"
Write-Host "  selected rejection candidates: $rejectionCount"
Write-Host "  admitted JPEG source files: $($jpegCases.Count)"
Write-Host "  admitted grayscale JPEG source files: $($jpegDecoderCases.Count)"
Write-Host "  JPEG unsupported-profile candidates: $jpegRejectionCount"
Write-Host "  admitted BMP source files: $($bmpCases.Count)"
Write-Host "  executable external files: $($inventory.executable_external_files)"
Write-Host "  PNG redistribution status: $($provenance.source.redistribution_status)"
Write-Host "  libjpeg-turbo redistribution status: $($provenance.additional_sources.libjpeg_turbo.redistribution_status)"
Write-Host "  jpeg-decoder redistribution status: $($provenance.additional_sources.jpeg_decoder.redistribution_status)"
