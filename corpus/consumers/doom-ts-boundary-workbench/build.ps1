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

    $startupPayloadLimit = 12 * 1024 * 1024
    $startupPayloadFiles = @(
        (Join-Path $PSScriptRoot "web/index.html"),
        (Join-Path $PSScriptRoot "web/app/main.js"),
        (Join-Path $PSScriptRoot "web/app/intake.js"),
        (Join-Path $PSScriptRoot "web/app/terminal-observer.js"),
        (Join-Path $output "doom_ts_boundary_workbench_engine.js"),
        (Join-Path $output "doom_ts_boundary_workbench_engine_bg.wasm")
    )
    $startupPayloadBytes = ($startupPayloadFiles | ForEach-Object {
        if (-not (Test-Path -LiteralPath $_)) {
            throw "Expected browser startup payload is missing: $_"
        }
        (Get-Item -LiteralPath $_).Length
    } | Measure-Object -Sum).Sum
    if ($startupPayloadBytes -gt $startupPayloadLimit) {
        throw "Browser startup payload has $startupPayloadBytes emitted bytes, exceeding the corpus limit of $startupPayloadLimit."
    }
    Write-Output "Browser startup payload: emitted-bytes=$startupPayloadBytes; limit=$startupPayloadLimit; files=$($startupPayloadFiles.Count); transfer-compression=unmeasured"
}
finally {
    Pop-Location
}
