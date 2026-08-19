$ErrorActionPreference = "Stop"

$workspace = Resolve-Path (Join-Path $PSScriptRoot "../../..")
$wasm = Join-Path $workspace "target/wasm32-unknown-unknown/debug/doom_ts_boundary_workbench_engine.wasm"
$output = Join-Path $PSScriptRoot "web/pkg"

Push-Location $PSScriptRoot
try {
    if (-not (Test-Path (Join-Path $PSScriptRoot "node_modules/.bin/tsc.cmd"))) {
        throw "TypeScript dependencies are missing. Run npm install in $PSScriptRoot first."
    }
    & cargo build -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw "WASM engine compilation failed." }
    & wasm-bindgen $wasm --target web --out-dir $output --out-name doom_ts_boundary_workbench_engine
    if ($LASTEXITCODE -ne 0) { throw "wasm-bindgen output generation failed." }
    & npm run build
    if ($LASTEXITCODE -ne 0) { throw "TypeScript compilation failed." }
}
finally {
    Pop-Location
}
