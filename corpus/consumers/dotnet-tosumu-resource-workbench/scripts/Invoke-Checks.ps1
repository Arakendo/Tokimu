[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$consumerRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$project = Join-Path $consumerRoot 'src\Tokimu.ResourceWorkbench\Tokimu.ResourceWorkbench.csproj'
$contractProject = Join-Path $consumerRoot 'src\Tokimu.ResourceWorkbench.ContractTests\Tokimu.ResourceWorkbench.ContractTests.csproj'
$repositoryRoot = (Resolve-Path (Join-Path $consumerRoot '..\..\..')).Path
$tosumuSubmodule = Join-Path $repositoryRoot 'third-party\tosumu'
$forbiddenPattern = 'ClassLibrary|WebViewTools|WpfBlazorTools|HelperClient\.Wpf|MonacoTools\.WebView|F:\\LocalSource\\ClassLibrary'
$sourceFiles = Get-ChildItem -LiteralPath $consumerRoot -Recurse -File -Include '*.csproj', '*.cs', '*.xaml'
$matches = $sourceFiles | Select-String -Pattern $forbiddenPattern

if ($matches) {
    $details = ($matches | ForEach-Object { "$($_.Path):$($_.LineNumber)" }) -join [Environment]::NewLine
    throw "Forbidden historical dependency reference found:`n$details"
}

if (-not (Test-Path -LiteralPath $tosumuSubmodule -PathType Container)) {
    throw "Pinned Tosumu submodule is missing: $tosumuSubmodule"
}

& dotnet build $project --nologo
if ($LASTEXITCODE -ne 0) {
    throw "dotnet build failed with exit code $LASTEXITCODE"
}

Push-Location $repositoryRoot
try {
    & cargo test --offline -p resource-space -p tokimu-resource-workbench-bridge
    if ($LASTEXITCODE -ne 0) {
        throw "cargo test failed with exit code $LASTEXITCODE"
    }

    & cargo build --offline -p tokimu-resource-workbench-bridge
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

$bridge = Join-Path $repositoryRoot 'target\debug\tokimu-resource-workbench-bridge.exe'
$env:TOKIMU_RESOURCE_BRIDGE = $bridge
& dotnet run --project $contractProject --no-restore --nologo
if ($LASTEXITCODE -ne 0) {
    throw "bridge contract checks failed with exit code $LASTEXITCODE"
}

Write-Host 'Tokimu Resource Workbench checks completed.'
