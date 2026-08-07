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
    $compiledSource = (Get-Content -LiteralPath $compiledApp -Raw).Replace(
        '../dist/runtime_observation_workbench_engine.js',
        './runtime_observation_workbench_engine.js'
    )
    [System.IO.File]::WriteAllText(
        $compiledApp,
        $compiledSource,
        [System.Text.UTF8Encoding]::new($false)
    )
    Copy-Item -LiteralPath (Join-Path $project "web/index.html") -Destination $dist -Force
    Copy-Item -LiteralPath (Join-Path $project "web/styles.css") -Destination $dist -Force
}
finally {
    Pop-Location
}

Write-Host "Prepared runtime observation workbench at $dist"
