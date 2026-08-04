[CmdletBinding()]
param(
    [switch]$NoRestore
)

$ErrorActionPreference = "Stop"

function Invoke-CheckedCommand {
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$Command,
        [Parameter(Mandatory = $true)]
        [string]$Description
    )

    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit code $LASTEXITCODE."
    }
}

$workspaceRoot = Resolve-Path (Join-Path $PSScriptRoot "../../..")
$engineManifest = Join-Path $PSScriptRoot "engine/Cargo.toml"
$wasm = Join-Path $workspaceRoot "target/wasm32-unknown-unknown/release/tokimu_asset_workbench_engine.wasm"
$wasmOutput = Join-Path $PSScriptRoot "wwwroot/tokimu"
$publishOutput = Join-Path $PSScriptRoot "publish"
$project = Join-Path $PSScriptRoot "Tokimu.AssetWorkbench.csproj"

Push-Location $PSScriptRoot
try {
    Invoke-CheckedCommand { npm run build } "TypeScript build"
    Invoke-CheckedCommand {
        cargo build --release --manifest-path $engineManifest --target wasm32-unknown-unknown
    } "WASM engine build"
    Invoke-CheckedCommand {
        wasm-bindgen $wasm --target web --out-dir $wasmOutput --out-name tokimu_asset_workbench_engine
    } "wasm-bindgen generation"

    $publishArgs = @(
        "publish",
        $project,
        "--configuration", "Release",
        "--output", $publishOutput
    )
    if ($NoRestore) {
        $publishArgs += "--no-restore"
    }
    Invoke-CheckedCommand { & dotnet @publishArgs } ".NET publish"

    Write-Host "Published Tokimu Asset Workbench to $publishOutput"
}
finally {
    Pop-Location
}
