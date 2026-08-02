$ErrorActionPreference = "Stop"

$project = Resolve-Path $PSScriptRoot
$root = Resolve-Path (git -C $project rev-parse --show-toplevel)
$manifest = Join-Path $project "engine/Cargo.toml"
$wasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_website_visualizer_engine.wasm"
$dist = Join-Path $project "dist"

Push-Location $project
try {
    npm run build
    cargo build --manifest-path $manifest --target wasm32-unknown-unknown --release
    New-Item -ItemType Directory -Force $dist | Out-Null
    wasm-bindgen $wasm --target web --out-dir $dist --out-name tokimu_website_visualizer_engine
    Copy-Item -LiteralPath (Join-Path $project "web/index.html") -Destination $dist -Force
    Copy-Item -LiteralPath (Join-Path $project "web/styles.css") -Destination $dist -Force
}
finally {
    Pop-Location
}

Write-Host "Prepared Tokimu website visualizer consumer at $dist"
