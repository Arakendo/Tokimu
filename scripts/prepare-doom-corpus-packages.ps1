[CmdletBinding()]
param(
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$assetRoot = Join-Path $RepositoryRoot "corpus/assets/DOOM"
$archiveRoot = Join-Path $RepositoryRoot "corpus/assets/archive/DOOM"
$outputRoot = Join-Path $assetRoot "packages"
$fixedTimestamp = [DateTimeOffset]::new(1980, 1, 1, 0, 0, 0, [TimeSpan]::Zero)

function Assert-SourceArchive {
    param(
        [Parameter(Mandatory)] [string]$Path,
        [Parameter(Mandatory)] [string]$ExpectedSha256
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Source archive not found: $Path"
    }

    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $ExpectedSha256) {
        throw "Source archive hash mismatch for $Path. Expected $ExpectedSha256, observed $actual."
    }
}

function New-CompactCorpusArchive {
    param(
        [Parameter(Mandatory)] [string]$SourcePath,
        [Parameter(Mandatory)] [string]$OutputPath,
        [Parameter(Mandatory)] [string[]]$Members,
        [Parameter(Mandatory)] [string]$ProvenancePath
    )

    $source = [System.IO.Compression.ZipFile]::OpenRead($SourcePath)
    $outputStream = $null
    $output = $null

    try {
        $outputStream = [System.IO.File]::Open(
            $OutputPath,
            [System.IO.FileMode]::Create,
            [System.IO.FileAccess]::Write,
            [System.IO.FileShare]::None
        )
        $output = [System.IO.Compression.ZipArchive]::new(
            $outputStream,
            [System.IO.Compression.ZipArchiveMode]::Create,
            $false
        )

        foreach ($memberName in $Members) {
            $sourceEntry = $source.Entries |
                Where-Object { $_.FullName -ceq $memberName } |
                Select-Object -First 1
            if ($null -eq $sourceEntry) {
                throw "Required member '$memberName' is absent from $SourcePath."
            }

            $targetEntry = $output.CreateEntry(
                $memberName,
                [System.IO.Compression.CompressionLevel]::Optimal
            )
            $targetEntry.LastWriteTime = $fixedTimestamp

            $input = $sourceEntry.Open()
            $target = $targetEntry.Open()
            try {
                $input.CopyTo($target)
            }
            finally {
                $target.Dispose()
                $input.Dispose()
            }
        }

        $provenanceEntry = $output.CreateEntry(
            "PROVENANCE.txt",
            [System.IO.Compression.CompressionLevel]::Optimal
        )
        $provenanceEntry.LastWriteTime = $fixedTimestamp
        $provenanceInput = [System.IO.File]::OpenRead($ProvenancePath)
        $provenanceOutput = $provenanceEntry.Open()
        try {
            $provenanceInput.CopyTo($provenanceOutput)
        }
        finally {
            $provenanceOutput.Dispose()
            $provenanceInput.Dispose()
        }
    }
    finally {
        if ($null -ne $output) {
            $output.Dispose()
        }
        if ($null -ne $outputStream) {
            $outputStream.Dispose()
        }
        $source.Dispose()
    }
}

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null

$packages = @(
    @{
        Source = Join-Path $archiveRoot "DOSBOX_DOOM.ZIP"
        SourceSha256 = "9ed3172e728d403962f874eaba93b4b973af1e57a8608bd803fc6e02d137fbc6"
        Output = Join-Path $outputRoot "doom-shareware-corpus-v1.zip"
        Members = @("DOOM1.WAD", "README.TXT", "HELPME.TXT")
        Provenance = Join-Path $assetRoot "curated/doom-shareware-v1.9/PROVENANCE.txt"
    },
    @{
        Source = Join-Path $archiveRoot "DOSBOX_HERETIC.ZIP"
        SourceSha256 = "f4ca7bffd27ab3e671beb3cadee7a39c3b7b8c330e5e0591f4a42c2f7b6bb944"
        Output = Join-Path $outputRoot "heretic-shareware-corpus-v1.zip"
        Members = @("HERETIC1.WAD", "LICENSE.DOC", "VENDOR.DOC", "README.TXT")
        Provenance = Join-Path $assetRoot "curated/heretic-shareware-v1.2/PROVENANCE.txt"
    }
)

foreach ($package in $packages) {
    Assert-SourceArchive -Path $package.Source -ExpectedSha256 $package.SourceSha256
    New-CompactCorpusArchive `
        -SourcePath $package.Source `
        -OutputPath $package.Output `
        -Members $package.Members `
        -ProvenancePath $package.Provenance

    $item = Get-Item -LiteralPath $package.Output
    $hash = (Get-FileHash -LiteralPath $package.Output -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Host "Prepared $($item.FullName)"
    Write-Host "  Bytes: $($item.Length)"
    Write-Host "  SHA-256: $hash"
    Write-Host "  Members: $($package.Members -join ', '), PROVENANCE.txt"
}
