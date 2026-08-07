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
$ratatuiLabConsumer = Join-Path $root "corpus/consumers/tokimu-website-ratatui-lab"
$ratatuiLabEngine = Join-Path $ratatuiLabConsumer "engine"
$ratatuiLabWasm = Join-Path $root "target/wasm32-unknown-unknown/release/tokimu_website_ratatui_lab_engine.wasm"
$ratatuiLabOutput = Join-Path $website "docs/assets/islands/ratatui-lab"
$runtimeObservationConsumer = Join-Path $root "corpus/consumers/runtime-observation-workbench"
$runtimeObservationOutput = Join-Path $website "docs/assets/islands/runtime-observation"
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
    # Website owns the shared TypeScript toolchain; the island consumer only
    # owns its source and TypeScript configuration.
    Invoke-Checked {
        npm exec -- tsc --project (Join-Path $ratatuiLabConsumer "tsconfig.json")
    } "Ratatui template lab TypeScript build"
    Invoke-Checked {
        cargo build --manifest-path (Join-Path $ratatuiLabEngine "Cargo.toml") --target wasm32-unknown-unknown --release
    } "Ratatui template lab WASM build"
    Invoke-Checked {
        pwsh -NoProfile -ExecutionPolicy Bypass -File (Join-Path $runtimeObservationConsumer "build.ps1")
    } "Runtime observation workbench WASM and TypeScript build"

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

    New-Item -ItemType Directory -Force $ratatuiLabOutput | Out-Null
    Invoke-Checked {
        wasm-bindgen $ratatuiLabWasm --target web --out-dir $ratatuiLabOutput --out-name tokimu_website_ratatui_lab_engine
    } "Ratatui template lab binding generation"
    Copy-Item -LiteralPath (Join-Path $ratatuiLabConsumer "web/index.html") -Destination $ratatuiLabOutput -Force
    Copy-Item -LiteralPath (Join-Path $ratatuiLabConsumer "web/styles.css") -Destination $ratatuiLabOutput -Force
    Copy-Item -LiteralPath (Join-Path $ratatuiLabConsumer "dist/ratatui-lab.js") -Destination $ratatuiLabOutput -Force

    New-Item -ItemType Directory -Force $runtimeObservationOutput | Out-Null
    Copy-Item -Path (Join-Path $runtimeObservationConsumer "dist/*") -Destination $runtimeObservationOutput -Force
    $runtimeObservationSourceOutput = Join-Path $runtimeObservationOutput "src"
    New-Item -ItemType Directory -Force $runtimeObservationSourceOutput | Out-Null
    Copy-Item -LiteralPath @(
        (Join-Path $runtimeObservationConsumer "dist/src/ratatui-input.js"),
        (Join-Path $runtimeObservationConsumer "dist/src/runtime-observation.js")
    ) -Destination $runtimeObservationSourceOutput -Force
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
Write-Host "  Ratatui templates: $ratatuiLabOutput"
Write-Host "  Runtime observation: $runtimeObservationOutput"
