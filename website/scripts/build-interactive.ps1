$ErrorActionPreference = "Stop"

$website = Resolve-Path (Join-Path $PSScriptRoot "..")
$root = Resolve-Path (git -C $website rev-parse --show-toplevel)
$assetConsumer = Join-Path $root "corpus/consumers/aspnet-wasm-asset-workbench"
$assetEngine = Join-Path $assetConsumer "engine"
$assetWasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_asset_workbench_engine.wasm"
$assetOutput = Join-Path $website "docs/assets/islands/asset-observation"
$asteroidsConsumer = Join-Path $root "corpus/consumers/tokimu-website-asteroids"
$asteroidsEngine = Join-Path $asteroidsConsumer "engine"
$asteroidsWasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_website_asteroids_engine.wasm"
$asteroidsOutput = Join-Path $website "docs/assets/islands/asteroids-game"
$visualizerConsumer = Join-Path $root "corpus/consumers/tokimu-website-visualizer"
$visualizerEngine = Join-Path $visualizerConsumer "engine"
$visualizerWasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_website_visualizer_engine.wasm"
$visualizerOutput = Join-Path $website "docs/assets/islands/tokimu-visualizer"
$paintConsumer = Join-Path $root "corpus/consumers/tokimu-website-paint"
$paintEngine = Join-Path $paintConsumer "engine"
$paintWasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_website_paint_engine.wasm"
$paintOutput = Join-Path $website "docs/assets/islands/tokimu-paint"
$kernelUiConsumer = Join-Path $root "corpus/consumers/tokimu-website-kernel-ui"
$kernelUiEngine = Join-Path $kernelUiConsumer "engine"
$kernelUiWasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_website_kernel_ui_engine.wasm"
$kernelUiOutput = Join-Path $website "docs/assets/islands/kernel-ui"
$fixture = Join-Path $root "third-party/fixtures/w3c-svg-1.1-2nd-edition/selected/derived/shapes-rect-01-geometry.svg"

function Invoke-Checked {
    param(
        [Parameter(Mandatory)]
        [scriptblock] $Command,
        [Parameter(Mandatory)]
        [string] $Description
    )

    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit code $LASTEXITCODE."
    }
}

Push-Location $website
try {
    Invoke-Checked { npm run build } "Website TypeScript build"
    Invoke-Checked { npm --prefix $asteroidsConsumer run build } "Asteroids TypeScript build"
    Invoke-Checked {
        cargo build --manifest-path (Join-Path $assetEngine "Cargo.toml") --target wasm32-unknown-unknown --release
    } "Asset workbench WASM build"
    Invoke-Checked {
        cargo build --manifest-path (Join-Path $asteroidsEngine "Cargo.toml") --target wasm32-unknown-unknown --release
    } "Asteroids WASM build"
    Invoke-Checked {
        cargo build --manifest-path (Join-Path $paintEngine "Cargo.toml") --target wasm32-unknown-unknown --release
    } "Tokimu Paint WASM build"
    Invoke-Checked {
        cargo build --manifest-path (Join-Path $kernelUiEngine "Cargo.toml") --target wasm32-unknown-unknown --release
    } "Kernel UI WASM build"

    New-Item -ItemType Directory -Force $assetOutput | Out-Null
    Invoke-Checked {
        wasm-bindgen $assetWasm --target web --out-dir $assetOutput --out-name tokimu_asset_workbench_engine
    } "Asset workbench binding generation"
    Copy-Item -LiteralPath $fixture -Destination $assetOutput -Force

    New-Item -ItemType Directory -Force $asteroidsOutput | Out-Null
    Invoke-Checked {
        wasm-bindgen $asteroidsWasm --target web --out-dir $asteroidsOutput --out-name tokimu_website_asteroids_engine
    } "Asteroids binding generation"
    Copy-Item -LiteralPath (Join-Path $asteroidsConsumer "web/index.html") -Destination $asteroidsOutput -Force
    Copy-Item -LiteralPath (Join-Path $asteroidsConsumer "web/styles.css") -Destination $asteroidsOutput -Force
    Copy-Item -LiteralPath (Join-Path $asteroidsConsumer "dist/asteroids.js") -Destination $asteroidsOutput -Force

    New-Item -ItemType Directory -Force $visualizerOutput | Out-Null
    Invoke-Checked {
        pwsh -NoProfile -File (Join-Path $visualizerConsumer "build.ps1")
    } "Visualizer WASM and TypeScript build"
    Copy-Item -Path (Join-Path $visualizerConsumer "dist/*") -Destination $visualizerOutput -Force

    Invoke-Checked { npm --prefix $paintConsumer run build } "Tokimu Paint TypeScript build"
    New-Item -ItemType Directory -Force $paintOutput | Out-Null
    Invoke-Checked {
        wasm-bindgen $paintWasm --target web --out-dir $paintOutput --out-name tokimu_website_paint_engine
    } "Tokimu Paint binding generation"
    Copy-Item -LiteralPath (Join-Path $paintConsumer "web/index.html") -Destination $paintOutput -Force
    Copy-Item -LiteralPath (Join-Path $paintConsumer "web/styles.css") -Destination $paintOutput -Force
    Copy-Item -LiteralPath (Join-Path $paintConsumer "web/lucide.svg") -Destination $paintOutput -Force
    Copy-Item -LiteralPath (Join-Path $paintConsumer "dist/paint.js") -Destination $paintOutput -Force

    Invoke-Checked { npm --prefix $kernelUiConsumer run build } "Kernel UI TypeScript build"
    New-Item -ItemType Directory -Force $kernelUiOutput | Out-Null
    Invoke-Checked {
        wasm-bindgen $kernelUiWasm --target web --out-dir $kernelUiOutput --out-name tokimu_website_kernel_ui_engine
    } "Kernel UI binding generation"
    Copy-Item -LiteralPath (Join-Path $kernelUiConsumer "web/index.html") -Destination $kernelUiOutput -Force
    Copy-Item -LiteralPath (Join-Path $kernelUiConsumer "web/styles.css") -Destination $kernelUiOutput -Force
    Copy-Item -LiteralPath (Join-Path $kernelUiConsumer "dist/kernel-ui.js") -Destination $kernelUiOutput -Force
}
finally {
    Pop-Location
}

Write-Host "Prepared website interactive assets:"
Write-Host "  Asset observation: $assetOutput"
Write-Host "  Asteroids game:    $asteroidsOutput"
Write-Host "  Tokimu visualizer:$visualizerOutput"
Write-Host "  Tokimu Paint:      $paintOutput"
Write-Host "  Kernel UI:         $kernelUiOutput"
