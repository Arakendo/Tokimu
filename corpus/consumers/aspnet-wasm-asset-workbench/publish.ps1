[CmdletBinding()]
param(
    [switch]$NoRestore
)

$ErrorActionPreference = "Stop"

$workspaceRoot = Resolve-Path (Join-Path $PSScriptRoot "../../..")
$engineManifest = Join-Path $PSScriptRoot "engine/Cargo.toml"
$wasm = Join-Path $workspaceRoot "target/wasm32-unknown-unknown/release/tokimu_asset_workbench_engine.wasm"
$wasmOutput = Join-Path $PSScriptRoot "wwwroot/tokimu"
$publishOutput = Join-Path $PSScriptRoot "publish"
$project = Join-Path $PSScriptRoot "Tokimu.AssetWorkbench.csproj"

Push-Location $PSScriptRoot
try {
    npm run build
    cargo build --release --manifest-path $engineManifest --target wasm32-unknown-unknown
    wasm-bindgen $wasm --target web --out-dir $wasmOutput --out-name tokimu_asset_workbench_engine

    $publishArgs = @(
        "publish",
        $project,
        "--configuration", "Release",
        "--output", $publishOutput
    )
    if ($NoRestore) {
        $publishArgs += "--no-restore"
    }
    & dotnet @publishArgs

    Write-Host "Published Tokimu Asset Workbench to $publishOutput"
}
finally {
    Pop-Location
}
