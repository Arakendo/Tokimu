param(
    [int]$Count = 100,
    [string]$Source = (Join-Path $PSScriptRoot "..\target\glyph-corpus\icons\icons"),
    [string]$Destination = (Join-Path $PSScriptRoot "..\target\lucide-corpus-100")
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $Source)) {
    throw "Lucide corpus not found. Run prepare-glyph-corpus.ps1 first."
}

$files = @(Get-ChildItem -LiteralPath $Source -Recurse -File -Filter "*.svg" |
    Sort-Object FullName |
    Select-Object -First $Count)

if ($files.Count -lt $Count) {
    throw "Expected $Count Lucide SVG files, found $($files.Count)."
}

Remove-Item -LiteralPath $Destination -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $Destination -Force | Out-Null

$manifest = foreach ($file in $files) {
    $relative = [IO.Path]::GetRelativePath($Source, $file.FullName)
    $target = Join-Path $Destination $relative
    New-Item -ItemType Directory -Path (Split-Path $target) -Force | Out-Null
    Copy-Item -LiteralPath $file.FullName -Destination $target
    $svg = Get-Content -LiteralPath $file.FullName -Raw
    $elements = @("path", "circle", "rect", "line", "polyline", "polygon") |
        Where-Object { $svg -match "<$($_)(\s|>)" }
    $hasCurve = $svg -match 'd="[^"]*[CcQqSsTt]'
    $hasArc = $svg -match 'd="[^"]*[Aa]'
    "{0}`t{1}`t{2}`t{3}" -f $relative.Replace('\', '/'), ($elements -join ','), $hasCurve, $hasArc
}

$manifest | Set-Content -LiteralPath (Join-Path $Destination "manifest.tsv") -Encoding utf8
$manifest | ForEach-Object { ($_ -split "`t")[0] } |
    Set-Content -LiteralPath (Join-Path $Destination "manifest.txt") -Encoding utf8
Write-Output "Prepared Lucide sample at $Destination"
Write-Output "  Icons: $($files.Count)"
