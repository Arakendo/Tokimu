[CmdletBinding()]
param(
    [string]$FixtureRoot = (
        Join-Path $PSScriptRoot "..\third-party\fixtures\webcgm-test-suite"
    ),
    [string]$ArchivePath
)

$ErrorActionPreference = "Stop"

$archiveName = "webcgm21-ts-20100419.zip"
$archiveUri = (
    "https://docs.oasis-open.org/webcgm/test-materials/webcgm21ts/" +
    $archiveName
)
$archiveSha256 = (
    "d540a452d989091db3abd83724ab9d0d9730f57ad792f4db85a04d93103063c9"
)
$fixtureRoot = [IO.Path]::GetFullPath($FixtureRoot)
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$targetRoot = Join-Path $repositoryRoot "target\webcgm-corpus"
$workRoot = Join-Path $targetRoot ([guid]::NewGuid().ToString("N"))
$downloadPath = Join-Path $workRoot $archiveName
$extractRoot = Join-Path $workRoot "extracted"
$upstreamRoot = Join-Path $fixtureRoot "upstream"

function Remove-PreparationDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedTarget = [IO.Path]::GetFullPath($targetRoot)
    $resolvedPath = [IO.Path]::GetFullPath($Path)
    $prefix = $resolvedTarget + [IO.Path]::DirectorySeparatorChar
    if (-not $resolvedPath.StartsWith(
        $prefix,
        [StringComparison]::OrdinalIgnoreCase
    )) {
        throw "Refusing to remove preparation path outside target: $resolvedPath"
    }
    if (Test-Path -LiteralPath $resolvedPath) {
        Remove-Item -LiteralPath $resolvedPath -Recurse -Force
    }
}

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

function Get-WebCgmCaseClassification {
    param([Parameter(Mandatory)][string]$RelativePath)

    $path = $RelativePath.Replace("\\", "/").ToLowerInvariant()
    $file = [IO.Path]::GetFileNameWithoutExtension($path)

    if ($file -eq "allelm01") {
        return [ordered]@{
            category = "support"
            rule = "static-element-inventory"
            reason = "Broad element inventory retained for structural inspection, not geometry coverage."
        }
    }
    if ($path -match "/removed-from-" -or $file -match "target|old$|org$|test$") {
        return [ordered]@{
            category = "support"
            rule = "support-derivative"
            reason = "Named as an upstream target, historical, or comparison helper."
        }
    }

    if ($path.StartsWith("dynamic10/")) {
        if ($file -match "link") {
            return [ordered]@{
                category = "hyperlink"
                rule = "dynamic-linking"
                reason = "Dynamic WebCGM linking scenario."
            }
        }
        if ($file -match "interact|behavior|fragment|focus") {
            return [ordered]@{
                category = "interaction"
                rule = "dynamic-interaction"
                reason = "Dynamic WebCGM behavior or interaction scenario."
            }
        }
        return [ordered]@{
            category = "profile"
            rule = "dynamic-profile"
            reason = "Dynamic WebCGM profile scenario outside current geometry scope."
        }
    }

    if ($path.StartsWith("20tests/")) {
        if ($file -match "appstructure|node|nodelist|webcgm_event|object_behaviors|picture-highlight|region_clip|stylepropert|xcf") {
            return [ordered]@{
                category = "dom"
                rule = "webcgm-dom"
                reason = "WebCGM application structure, DOM, or XCF scenario."
            }
        }
        if ($file -match "text|utf") {
            return [ordered]@{
                category = "text"
                rule = "webcgm-text"
                reason = "Text encoding, text-on-path, or text content scenario."
            }
        }
        if ($file -match "nurbs|nubs|interpolated|transp") {
            return [ordered]@{
                category = "geometry"
                rule = "webcgm-geometry"
                reason = "Geometry or presentation-geometry scenario."
            }
        }
        return [ordered]@{
            category = "profile"
            rule = "webcgm-profile"
            reason = "WebCGM profile scenario outside current geometry scope."
        }
    }

    if ($path.StartsWith("21tests/")) {
        if ($file -match "animation|aps|setview|setredraw|getobject|alphaescape") {
            return [ordered]@{
                category = "interaction"
                rule = "webcgm-interaction"
                reason = "WebCGM animation, view, redraw, or object interaction scenario."
            }
        }
        if ($file -match "font|text|substring") {
            return [ordered]@{
                category = "text"
                rule = "webcgm-text"
                reason = "Text, font, or text-substring scenario."
            }
        }
        if ($file -match "cell") {
            return [ordered]@{
                category = "raster"
                rule = "webcgm-raster"
                reason = "Cell-array or raster-color scenario."
            }
        }
        return [ordered]@{
            category = "profile"
            rule = "webcgm-profile"
            reason = "WebCGM profile scenario outside current geometry scope."
        }
    }

    if ($file -match "celary") {
        return [ordered]@{
            category = "raster"
            rule = "static-cell-array"
            reason = "Static CGM cell-array scenario."
        }
    }
    if ($file -match "txt|chr|font|apntxt") {
        return [ordered]@{
            category = "text"
            rule = "static-text"
            reason = "Static CGM text or character-attribute scenario."
        }
    }
    if ($file -match "escape|schema|specmd|mfd|mfell|defval|desord|dgntst|realpr|vdc|col|tab|patt") {
        return [ordered]@{
            category = "profile"
            rule = "static-profile"
            reason = "Static CGM descriptor, precision, color, or profile scenario."
        }
    }
    return [ordered]@{
        category = "geometry"
        rule = "static-geometry-default"
        reason = "Static CGM primitive or presentation-geometry scenario."
    }
}

function Get-WebCgmInventory {
    param([Parameter(Mandatory)][string]$Root)

    $files = @(Get-ChildItem -LiteralPath $Root -Recurse -File)
    $extensionCounts = [ordered]@{}
    foreach ($group in ($files | Group-Object Extension | Sort-Object Name)) {
        $name = if ($group.Name) {
            $group.Name.ToLowerInvariant()
        } else {
            "(none)"
        }
        $extensionCounts[$name] = $group.Count
    }

    $moduleCounts = [ordered]@{}
    foreach ($module in @("static10", "dynamic10", "20tests", "21tests")) {
        $moduleRoot = Join-Path $Root $module
        $moduleCounts[$module] = [ordered]@{
            files = @(
                Get-ChildItem -LiteralPath $moduleRoot -Recurse -File
            ).Count
            cgm_files = @(
                Get-ChildItem -LiteralPath $moduleRoot -Recurse -File `
                    -Filter "*.cgm"
            ).Count
            png_files = @(
                Get-ChildItem -LiteralPath $moduleRoot -Recurse -File `
                    -Filter "*.png"
            ).Count
        }
    }

    $cgmFiles = @($files | Where-Object Extension -EQ ".cgm")
    $binaryHeaderCount = 0
    $profileCounts = [ordered]@{}
    foreach ($file in $cgmFiles) {
        $bytes = [IO.File]::ReadAllBytes($file.FullName)
        if ($bytes.Length -ge 2) {
            $word = ($bytes[0] -shl 8) -bor $bytes[1]
            $elementClass = ($word -shr 12) -band 0x0f
            $elementId = ($word -shr 5) -band 0x7f
            if ($elementClass -eq 0 -and $elementId -eq 1) {
                $binaryHeaderCount++
            }
        }

        $text = [Text.Encoding]::Latin1.GetString($bytes)
        $profile = [regex]::Match(
            $text,
            'ProfileEd:([0-9.]+)'
        ).Groups[1].Value
        if (-not $profile) {
            $profile = "not-observed"
        }
        if (-not $profileCounts.Contains($profile)) {
            $profileCounts[$profile] = 0
        }
        $profileCounts[$profile]++
    }

    $staticList = Join-Path $Root "static10\allStatic10.lst"
    $staticCaseIds = @(
        Get-Content -LiteralPath $staticList |
            ForEach-Object { $_.Trim() } |
            Where-Object { $_ }
    )
    $sourceCgmPaths = @(
        $cgmFiles |
            ForEach-Object {
                (
                    [IO.Path]::GetRelativePath($Root, $_.FullName)
                ).Replace("\", "/")
            } |
            Sort-Object
    )
    $categoryOrder = @(
        "geometry",
        "text",
        "raster",
        "dom",
        "hyperlink",
        "interaction",
        "profile",
        "support"
    )
    $categoryCounts = [ordered]@{}
    foreach ($category in $categoryOrder) {
        $categoryCounts[$category] = 0
    }
    $classifiedCases = @(
        foreach ($sourcePath in $sourceCgmPaths) {
            $classification = Get-WebCgmCaseClassification -RelativePath $sourcePath
            $categoryCounts[$classification.category]++
            [ordered]@{
                source = $sourcePath
                category = $classification.category
                rule = $classification.rule
                reason = $classification.reason
            }
        }
    )

    return [ordered]@{
        schema = 1
        suite = "WebCGM 2.1 Test Suite"
        release = "1.2"
        release_date = "2010-04-19"
        generated_from_archive_sha256 = $archiveSha256
        upstream_tree_sha256 = Get-TreeHash -Root $Root
        totals = [ordered]@{
            files = $files.Count
            bytes = ($files | Measure-Object Length -Sum).Sum
            cgm_files = $cgmFiles.Count
            reference_png_files = @(
                $files | Where-Object Extension -EQ ".png"
            ).Count
            static_case_ids = $staticCaseIds.Count
        }
        encoding_evidence = [ordered]@{
            begin_metafile_binary_headers = $binaryHeaderCount
            other_or_unclassified_headers = (
                $cgmFiles.Count - $binaryHeaderCount
            )
            policy = (
                "Header classification is inventory evidence, not a complete " +
                "encoding conformance check."
            )
        }
        profile_editions_observed = $profileCounts
        files_by_extension = $extensionCounts
        modules = $moduleCounts
        static_case_ids = $staticCaseIds
        source_cgm_paths = $sourceCgmPaths
        classification = [ordered]@{
            schema = 1
            policy = (
                "Every CGM source receives one conservative category from its " +
                "upstream module and stable filename evidence. Categories guide " +
                "corpus scope; they are not a conformance classification."
            )
            categories = $categoryCounts
            cases = $classifiedCases
        }
    }
}

New-Item -ItemType Directory -Force -Path $workRoot | Out-Null

try {
    if ($ArchivePath) {
        $resolvedArchive = (Resolve-Path -LiteralPath $ArchivePath).Path
        Copy-Item -LiteralPath $resolvedArchive -Destination $downloadPath
    } else {
        Write-Host "Fetching $archiveUri"
        Invoke-WebRequest -Uri $archiveUri -OutFile $downloadPath
    }

    $actualHash = (
        Get-FileHash -LiteralPath $downloadPath -Algorithm SHA256
    ).Hash.ToLowerInvariant()
    if ($actualHash -ne $archiveSha256) {
        throw "Archive checksum mismatch. Expected $archiveSha256, got $actualHash"
    }

    Expand-Archive -LiteralPath $downloadPath -DestinationPath $extractRoot
    $licensePath = Join-Path $extractRoot "copyright-license.html"
    if (-not (Test-Path -LiteralPath $licensePath)) {
        throw "Extracted suite is missing copyright-license.html"
    }

    if (Test-Path -LiteralPath $upstreamRoot) {
        $existingHash = Get-TreeHash -Root $upstreamRoot
        $extractedHash = Get-TreeHash -Root $extractRoot
        if ($existingHash -ne $extractedHash) {
            throw (
                "Existing upstream fixture differs from the pinned archive. " +
                "Refusing to replace reviewed source files implicitly."
            )
        }
    } else {
        New-Item -ItemType Directory -Force -Path $fixtureRoot | Out-Null
        Copy-Item -LiteralPath $extractRoot -Destination $upstreamRoot `
            -Recurse
    }

    $inventory = Get-WebCgmInventory -Root $upstreamRoot
    $inventoryPath = Join-Path $fixtureRoot "inventory.json"
    # Keep generated metadata stable across PowerShell hosts and Git settings.
    # The upstream fixture tree itself remains a verbatim extraction.
    $inventoryJson = (($inventory | ConvertTo-Json -Depth 8) -replace "`r`n", "`n") + "`n"
    [System.IO.File]::WriteAllText(
        $inventoryPath,
        $inventoryJson,
        [System.Text.UTF8Encoding]::new($false)
    )

    & (Join-Path $PSScriptRoot "verify-webcgm-corpus.ps1") `
        -FixtureRoot $fixtureRoot
} finally {
    Remove-PreparationDirectory -Path $workRoot
}
