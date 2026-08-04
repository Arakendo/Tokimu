[CmdletBinding()]
param(
    [ValidateRange(1, 3600)]
    [int]$Seconds = 60,

    [ValidateRange(1, 8192)]
    [int]$MaxInputBytes = 8192,

    [ValidateRange(128, 8192)]
    [int]$MaxRssMb = 768
)

$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$fuzzRoot = Join-Path $repositoryRoot 'corpus\lib\diff-tools\fuzz'
$visualStudioRoot = Join-Path $env:ProgramFiles 'Microsoft Visual Studio'

if (-not (Get-Command cargo-fuzz -ErrorAction SilentlyContinue)) {
    throw 'cargo-fuzz is required. Install it with: cargo install cargo-fuzz'
}

if (-not (Test-Path -LiteralPath $visualStudioRoot)) {
    throw "Visual Studio sanitizer runtime root was not found: $visualStudioRoot"
}

$asanRuntime = Get-ChildItem -Path $visualStudioRoot -Recurse -Filter 'clang_rt.asan_dynamic-x86_64.dll' -File |
    Where-Object { $_.FullName -match '\\Hostx64\\x64\\' } |
    Select-Object -First 1

if (-not $asanRuntime) {
    throw 'Visual Studio x64 ASan runtime was not found. Install the C++ build tools with the x64 sanitizer runtime.'
}

$env:PATH = "$($asanRuntime.DirectoryName);$env:PATH"
Push-Location $fuzzRoot
try {
    Write-Host "Running unified-parser-apply for $Seconds second(s) with max input $MaxInputBytes bytes and max RSS $MaxRssMb MiB."
    $fuzzArguments = @(
        '+nightly',
        'fuzz',
        'run',
        'unified-parser-apply',
        '--',
        "-max_total_time=$Seconds",
        "-max_len=$MaxInputBytes",
        "-rss_limit_mb=$MaxRssMb",
        '-print_final_stats=1'
    )
    & cargo @fuzzArguments
}
finally {
    Pop-Location
}
