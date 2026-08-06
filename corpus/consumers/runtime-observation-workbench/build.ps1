$ErrorActionPreference = "Stop"

$project = Resolve-Path $PSScriptRoot
$root = Resolve-Path (git -C $project rev-parse --show-toplevel)
$manifest = Join-Path $project "engine/Cargo.toml"
$wasm = Join-Path $root "target/wasm32-unknown-unknown/release/runtime_observation_workbench_engine.wasm"
$dist = Join-Path $project "dist"

Push-Location $project
try {
    cargo build --manifest-path $manifest --target wasm32-unknown-unknown --release
    New-Item -ItemType Directory -Force $dist | Out-Null
    wasm-bindgen $wasm --target web --out-dir $dist --out-name runtime_observation_workbench_engine
    npm run build
    # TypeScript preserves the source-relative WASM wrapper import. Once the
    # compiled app is staged in dist, that wrapper lives beside app.js.
    $compiledApp = Join-Path $dist "app.js"
    (Get-Content -LiteralPath $compiledApp -Raw).Replace(
        'from "../dist/runtime_observation_workbench_engine.js"',
        'from "./runtime_observation_workbench_engine.js"'
    ) | Set-Content -LiteralPath $compiledApp -Encoding utf8
    Copy-Item -LiteralPath (Join-Path $project "web/index.html") -Destination $dist -Force
    Copy-Item -LiteralPath (Join-Path $project "web/styles.css") -Destination $dist -Force
}
finally {
    Pop-Location
}

Write-Host "Prepared runtime observation workbench at $dist"
