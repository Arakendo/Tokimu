[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$auditScript = Join-Path $PSScriptRoot 'audit-ring-zero-dependencies.ps1'
$invalidConfiguration = Join-Path $PSScriptRoot 'tests/ring-zero-unapproved-source.json'

function Invoke-Audit([string]$ConfigurationPath) {
    $output = & pwsh -NoProfile -File $auditScript -ConfigurationPath $ConfigurationPath 2>&1
    return [pscustomobject]@{
        ExitCode = $LASTEXITCODE
        Output = ($output | Out-String)
    }
}

function Invoke-CommandChecked([string[]]$Arguments, [string]$FailureMessage) {
    & $Arguments[0] $Arguments[1..($Arguments.Count - 1)]
    if ($LASTEXITCODE -ne 0) {
        throw "$FailureMessage (exit code $LASTEXITCODE)."
    }
}

function Test-DirtySubmoduleFixture {
    $temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("tokimu-ring-zero-dirty-fixture-" + [guid]::NewGuid().ToString('N'))
    $temporaryRootFullPath = [System.IO.Path]::GetFullPath($temporaryRoot)
    $temporaryParent = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    if (-not $temporaryRootFullPath.StartsWith($temporaryParent + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to create or remove fixture outside the temporary directory: $temporaryRootFullPath"
    }

    try {
        [void](New-Item -ItemType Directory -Path $temporaryRootFullPath)
        Invoke-CommandChecked @('git', '-C', $temporaryRootFullPath, 'init') 'Unable to initialize dirty-submodule fixture repository'
        Invoke-CommandChecked @('git', '-C', $temporaryRootFullPath, 'config', 'user.email', 'audit-fixture@tokimu.invalid') 'Unable to configure dirty-submodule fixture author email'
        Invoke-CommandChecked @('git', '-C', $temporaryRootFullPath, 'config', 'user.name', 'Tokimu audit fixture') 'Unable to configure dirty-submodule fixture author name'

        $fixtureCratePath = Join-Path $temporaryRootFullPath 'crates/ring-zero-fixture'
        [void](New-Item -ItemType Directory -Path (Join-Path $fixtureCratePath 'src') -Force)
        @"
[workspace]
members = ["crates/ring-zero-fixture"]
exclude = ["third-party/ring-0/glam"]
resolver = "2"
"@ | Set-Content -LiteralPath (Join-Path $temporaryRootFullPath 'Cargo.toml') -NoNewline
        @"
[package]
name = "ring-zero-fixture"
version = "0.1.0"
edition = "2021"

[dependencies]
glam = { path = "../../third-party/ring-0/glam", default-features = false, features = ["std"] }
"@ | Set-Content -LiteralPath (Join-Path $fixtureCratePath 'Cargo.toml') -NoNewline
        'pub fn fixture() {}' | Set-Content -LiteralPath (Join-Path $fixtureCratePath 'src/lib.rs') -NoNewline
        Invoke-CommandChecked @('git', '-C', $temporaryRootFullPath, 'add', '.') 'Unable to stage dirty-submodule fixture files'
        Invoke-CommandChecked @('git', '-C', $temporaryRootFullPath, 'commit', '-m', 'fixture root') 'Unable to commit dirty-submodule fixture root'

        $glamSource = Join-Path $repositoryRoot 'third-party/ring-0/glam'
        $glamGitDirectory = & git -c "safe.directory=$($glamSource.Replace('\\', '/'))" -C $glamSource rev-parse --path-format=absolute --git-dir
        if ($LASTEXITCODE -ne 0) {
            throw 'Unable to resolve the audited source Git directory for the dirty-submodule fixture.'
        }
        $glamGitDirectory = ($glamGitDirectory | Select-Object -Last 1).Trim().Replace('\\', '/')
        Invoke-CommandChecked @(
            'git',
            '-c', 'protocol.file.allow=always',
            '-c', "safe.directory=$($glamSource.Replace('\\', '/'))",
            '-c', "safe.directory=$glamGitDirectory",
            '-C', $temporaryRootFullPath,
            'submodule', 'add', $glamSource, 'third-party/ring-0/glam'
        ) 'Unable to add fixture Ring 0 submodule'
        Invoke-CommandChecked @('git', '-C', $temporaryRootFullPath, 'add', '.gitmodules', 'third-party/ring-0/glam') 'Unable to stage fixture submodule'
        Invoke-CommandChecked @('git', '-C', $temporaryRootFullPath, 'commit', '-m', 'fixture Ring 0 source') 'Unable to commit fixture submodule'

        'deliberate dirty-source audit fixture' | Add-Content -LiteralPath (Join-Path $temporaryRootFullPath 'third-party/ring-0/glam/README.md')
        Invoke-CommandChecked @('cargo', 'generate-lockfile', '--manifest-path', (Join-Path $temporaryRootFullPath 'Cargo.toml'), '--offline') 'Unable to generate the dirty-submodule fixture lockfile offline'
        $fixtureConfiguration = Join-Path $temporaryRootFullPath 'ring-zero-dependencies.json'
        @"
{
  "roots": ["ring-zero-fixture"],
  "approvedRingZeroSources": [
    { "name": "fixture glam", "path": "third-party/ring-0/glam" }
  ]
}
"@ | Set-Content -LiteralPath $fixtureConfiguration -NoNewline

        $output = & pwsh -NoProfile -File $auditScript -RepositoryRoot $temporaryRootFullPath -ConfigurationPath $fixtureConfiguration 2>&1
        if ($LASTEXITCODE -eq 0) {
            throw 'The deliberately dirty Ring 0 source was accepted.'
        }
        $renderedOutput = $output | Out-String
        if (-not $renderedOutput.Contains("approved source 'fixture glam' has uncommitted changes", [System.StringComparison]::Ordinal)) {
            throw "The dirty fixture did not report an actionable local-change diagnostic:`n$renderedOutput"
        }
    }
    finally {
        if (Test-Path -LiteralPath $temporaryRootFullPath -PathType Container) {
            Remove-Item -LiteralPath $temporaryRootFullPath -Recurse -Force
        }
    }
}

$cleanResult = Invoke-Audit (Join-Path $PSScriptRoot 'ring-zero-dependencies.json')
if ($cleanResult.ExitCode -ne 0) {
    throw "The approved Ring 0 source configuration failed unexpectedly:`n$($cleanResult.Output)"
}

$rejectedResult = Invoke-Audit $invalidConfiguration
if ($rejectedResult.ExitCode -eq 0) {
    throw 'The deliberately unapproved Ring 0 source configuration was accepted.'
}

foreach ($expectedDiagnostic in @(
    "approved source 'deliberately missing fixture source' is missing",
    'glam 0.29.3 resolves from unapproved local path'
)) {
    if (-not $rejectedResult.Output.Contains($expectedDiagnostic, [System.StringComparison]::Ordinal)) {
        throw "The rejected configuration did not report the expected diagnostic '$expectedDiagnostic':`n$($rejectedResult.Output)"
    }
}

Test-DirtySubmoduleFixture

Write-Output 'Ring 0 provenance audit positive, unapproved-source, and dirty-source fixtures passed.'
