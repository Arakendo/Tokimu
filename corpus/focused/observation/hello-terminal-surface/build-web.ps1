param(
    [switch]$RatatuiProducer
)

$ErrorActionPreference = "Stop"

$packageRoot = Split-Path -Parent $PSCommandPath
$workspaceRoot = Resolve-Path (Join-Path $packageRoot "..\..\..\..")
$outputRoot = Join-Path $packageRoot "web\dist"
$wasmPath = Join-Path $workspaceRoot "target\wasm32-unknown-unknown\release\hello_terminal_surface.wasm"

$cargoArguments = @(
    "build",
    "--manifest-path", (Join-Path $packageRoot "Cargo.toml"),
    "--lib",
    "--target", "wasm32-unknown-unknown",
    "--release"
)
if ($RatatuiProducer) {
    $cargoArguments += @("--features", "ratatui-producer")
}

& cargo @cargoArguments
if ($LASTEXITCODE -ne 0) {
    throw "hello-terminal-surface WASM build failed with exit code $LASTEXITCODE."
}

New-Item -ItemType Directory -Force $outputRoot | Out-Null
wasm-bindgen $wasmPath --target web --out-dir $outputRoot --out-name hello_terminal_surface
if ($LASTEXITCODE -ne 0) {
    throw "wasm-bindgen generation failed with exit code $LASTEXITCODE."
}

Copy-Item (Join-Path $packageRoot "web\index.html") $outputRoot -Force
Copy-Item (Join-Path $packageRoot "web\terminal-surface.js") $outputRoot -Force

if ($RatatuiProducer) {
    Write-Host "Built hello-terminal-surface Ratatui browser evidence at $outputRoot"
} else {
    Write-Host "Built hello-terminal-surface independent browser evidence at $outputRoot"
}
