$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "../../..")
$engine = Join-Path $PSScriptRoot "engine"
$wasm = Join-Path $root "target/wasm32-unknown-unknown/debug/tokimu_presentation_workbench_engine.wasm"
$output = Join-Path $PSScriptRoot "wwwroot/tokimu"

Push-Location $PSScriptRoot
try {
    if (-not (Test-Path (Join-Path $PSScriptRoot "node_modules/.bin/tsc.cmd"))) {
        throw "TypeScript dependencies are missing. Run 'npm install' in $PSScriptRoot first."
    }

    & npm run build
    if ($LASTEXITCODE -ne 0) { throw "TypeScript compilation failed." }
    & cargo build --manifest-path (Join-Path $engine "Cargo.toml") --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw "WASM engine compilation failed." }
    & wasm-bindgen $wasm --target web --out-dir $output --out-name tokimu_presentation_workbench_engine
    if ($LASTEXITCODE -ne 0) { throw "wasm-bindgen output generation failed." }
    & dotnet build
    if ($LASTEXITCODE -ne 0) { throw "ASP.NET host compilation failed." }
}
finally {
    Pop-Location
}
