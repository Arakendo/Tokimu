param(
    [string] $OutputDirectory = (Join-Path $PSScriptRoot "dist\hello-hole-punch-windows")
)

$ErrorActionPreference = "Stop"

$project = Resolve-Path $PSScriptRoot
$workspace = Resolve-Path (Join-Path $project "..\..")
$asset = Join-Path $workspace "corpus\assets\CheckLicense\hole_punch1.glb"
$binary = Join-Path $workspace "target\release\hello-hole-punch.exe"
$output = [System.IO.Path]::GetFullPath($OutputDirectory)
$stagedAsset = Join-Path $output "assets\CheckLicense\hole_punch1.glb"

Push-Location $workspace
try {
    cargo build --package hello-hole-punch --release
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed with exit code $LASTEXITCODE."
    }

    Remove-Item -LiteralPath $output -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force (Split-Path -Parent $stagedAsset) | Out-Null
    Copy-Item -LiteralPath $binary -Destination $output -Force
    Copy-Item -LiteralPath $asset -Destination $stagedAsset -Force
    Set-Content -LiteralPath (Join-Path $output "Run Hello Hole Punch.cmd") -Encoding ascii -Value @(
        "@echo off"
        '"%~dp0hello-hole-punch.exe"'
    )

    Push-Location $output
    try {
        & .\hello-hole-punch.exe --verify-assets
        if ($LASTEXITCODE -ne 0) {
            throw "Staged asset verification failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }
}
finally {
    Pop-Location
}

Write-Host "Portable demo prepared at $output"