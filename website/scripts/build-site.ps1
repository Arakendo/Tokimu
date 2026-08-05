[CmdletBinding()]
param(
    [switch]$Serve,
    [switch]$Strict
)

$ErrorActionPreference = 'Stop'

$websiteRoot = Split-Path -Parent $PSScriptRoot
$python = Join-Path $websiteRoot '.venv\Scripts\python.exe'

if (-not (Test-Path -LiteralPath $python)) {
    throw "Website virtual environment is missing: $python. Run `python -m venv .venv` and `.\.venv\Scripts\python.exe -m pip install -r requirements.txt` from the website directory."
}

$arguments = @('-m', 'mkdocs')
if ($Serve) {
    $arguments += @('serve', '-f', 'mkdocs.yml')
} else {
    $arguments += @('build', '-f', 'mkdocs.yml')
    if ($Strict) {
        $arguments += '--strict'
    }
}

Push-Location $websiteRoot
try {
    & $python @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "MkDocs exited with code $LASTEXITCODE."
    }
} finally {
    Pop-Location
}
