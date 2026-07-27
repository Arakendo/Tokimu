[CmdletBinding()]
param(
    [string]$FixtureRoot = (Join-Path $PSScriptRoot "..\third-party\fixtures\khronos-gltf-sample-assets")
)

$ErrorActionPreference = "Stop"

$revision = "2bac6f8c57bf471df0d2a1e8a8ec023c7801dddf"
$rawRoot = "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/$revision"
$fixtureRoot = [IO.Path]::GetFullPath($FixtureRoot)
$upstreamRoot = Join-Path $fixtureRoot "upstream"

$files = @(
    "LICENSES/CC0-1.0.txt",
    "LICENSES/CC-BY-4.0.txt",
    "Models/Triangle/LICENSE.md",
    "Models/Triangle/metadata.json",
    "Models/Triangle/README.body.md",
    "Models/Triangle/README.md",
    "Models/Triangle/glTF-Embedded/Triangle.gltf",
    "Models/Triangle/glTF/Triangle.bin",
    "Models/Triangle/glTF/Triangle.gltf",
    "Models/Triangle/screenshot/screenshot.png",
    "Models/Triangle/screenshot/simpleTriangle.png",
    "Models/Box/LICENSE.md",
    "Models/Box/metadata.json",
    "Models/Box/README.body.md",
    "Models/Box/README.md",
    "Models/Box/glTF-Binary/Box.glb",
    "Models/Box/glTF-Draco/Box.bin",
    "Models/Box/glTF-Draco/Box.gltf",
    "Models/Box/glTF-Embedded/Box.gltf",
    "Models/Box/glTF/Box.gltf",
    "Models/Box/glTF/Box0.bin",
    "Models/Box/screenshot/screenshot.png",
    "Models/BoxTextured/LICENSE.md",
    "Models/BoxTextured/metadata.json",
    "Models/BoxTextured/README.body.md",
    "Models/BoxTextured/README.md",
    "Models/BoxTextured/glTF/BoxTextured.gltf",
    "Models/BoxTextured/glTF/BoxTextured0.bin",
    "Models/BoxTextured/glTF/CesiumLogoFlat.png"
)

foreach ($relativePath in $files) {
    $destination = Join-Path $upstreamRoot $relativePath
    New-Item -ItemType Directory -Force -Path (Split-Path $destination) | Out-Null
    $url = "$rawRoot/$relativePath"
    Write-Host "Fetching $relativePath"
    Invoke-WebRequest -Uri $url -OutFile $destination
}

& (Join-Path $PSScriptRoot "verify-khronos-gltf-corpus.ps1") -FixtureRoot $fixtureRoot
