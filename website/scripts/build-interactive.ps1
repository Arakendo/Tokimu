$ErrorActionPreference = "Stop"

$website = Resolve-Path (Join-Path $PSScriptRoot "..")
$root = Resolve-Path (git -C $website rev-parse --show-toplevel)
$consumer = Join-Path $root "corpus/consumers/aspnet-wasm-asset-workbench"
$engine = Join-Path $consumer "engine"
$wasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_asset_workbench_engine.wasm"
$output = Join-Path $website "docs/assets/islands/asset-observation"
$fixture = Join-Path $root "third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/derived/shapes-rect-01-geometry.svg"

Push-Location $website
try {
    npm run build
    cargo build --manifest-path (Join-Path $engine "Cargo.toml") --target wasm32-unknown-unknown --release
    New-Item -ItemType Directory -Force $output | Out-Null
    wasm-bindgen $wasm --target web --out-dir $output --out-name tokimu_asset_workbench_engine
    Copy-Item -LiteralPath $fixture -Destination $output -Force
}
finally {
    Pop-Location
}

Write-Host "Prepared website interactive assets at $output"
