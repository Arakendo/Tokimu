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

$root = Resolve-Path (Join-Path $PSScriptRoot "../../..")
$engine = Join-Path $PSScriptRoot "engine"
$wasm = Join-Path $root "target/wasm32-unknown-unknown/debug/tokimu_asset_workbench_engine.wasm"
$output = Join-Path $PSScriptRoot "wwwroot/tokimu"

Push-Location $PSScriptRoot
try {
    Invoke-CheckedCommand { npm run build } "TypeScript build"
    Invoke-CheckedCommand {
        cargo build --manifest-path (Join-Path $engine "Cargo.toml") --target wasm32-unknown-unknown
    } "WASM engine build"
    Invoke-CheckedCommand {
        wasm-bindgen $wasm --target web --out-dir $output --out-name tokimu_asset_workbench_engine
    } "wasm-bindgen generation"
    Invoke-CheckedCommand { dotnet build } ".NET build"
}
finally {
    Pop-Location
}
