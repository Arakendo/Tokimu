$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "../../..")
$engine = Join-Path $PSScriptRoot "engine"
$wasm = Join-Path $root "target/wasm32-unknown-unknown/debug/tokimu_asset_workbench_engine.wasm"
$output = Join-Path $PSScriptRoot "wwwroot/tokimu"

Push-Location $PSScriptRoot
try {
    npm run build
    cargo build --manifest-path (Join-Path $engine "Cargo.toml") --target wasm32-unknown-unknown
    wasm-bindgen $wasm --target web --out-dir $output --out-name tokimu_asset_workbench_engine
    dotnet build
}
finally {
    Pop-Location
}
