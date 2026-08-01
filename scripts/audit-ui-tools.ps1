[CmdletBinding()]
param(
    [switch]$AsJson
)

$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$uiToolsRoot = Join-Path $repositoryRoot 'corpus/lib/ui-tools'
$sourceRoot = Join-Path $uiToolsRoot 'src'
$corpusRoot = Join-Path $repositoryRoot 'corpus'

function Count-Matches {
    param(
        [string]$Path,
        [string]$Pattern
    )

    if (-not (Test-Path $Path)) {
        return 0
    }

    if ((Get-Item -LiteralPath $Path).PSIsContainer -eq $false) {
        return @(Select-String -Path $Path -Pattern $Pattern).Count
    }

    return @(
        Get-ChildItem -Path $Path -Recurse -File -Filter '*.rs' |
            Select-String -Pattern $Pattern
    ).Count
}

$capabilities = @(
    [pscustomobject]@{ Name = 'SVG'; Path = 'svg' }
    [pscustomobject]@{ Name = 'Vector'; Path = 'vector' }
    [pscustomobject]@{ Name = 'Font outline'; Path = 'font_outline' }
    [pscustomobject]@{ Name = 'Layout'; Path = 'tests/layout.rs' }
    [pscustomobject]@{ Name = 'Controls'; Path = 'controls' }
    [pscustomobject]@{ Name = 'Text input'; Path = 'text_input.rs' }
    [pscustomobject]@{ Name = 'Scroll'; Path = 'scroll.rs' }
)

$capabilityTests = foreach ($capability in $capabilities) {
    $path = Join-Path $sourceRoot $capability.Path
    [pscustomobject]@{
        capability = $capability.Name
        tests = Count-Matches -Path $path -Pattern '^\s*#\[test\]'
    }
}

$consumerMarkers = @(
    [pscustomobject]@{ marker = 'UiRect construction'; pattern = '\bUiRect\b' }
    [pscustomobject]@{ marker = 'UiDrawer use'; pattern = '\bUiDrawer\b' }
    [pscustomobject]@{ marker = 'Surface lowering'; pattern = '\bdraw_surface\b' }
    [pscustomobject]@{ marker = 'Text lowering'; pattern = '\bdraw_text\b' }
    [pscustomobject]@{ marker = 'Renderer submission'; pattern = '\bsubmit\b' }
)

$consumerInventory = foreach ($marker in $consumerMarkers) {
    [pscustomobject]@{
        marker = $marker.marker
        occurrences = Count-Matches -Path $corpusRoot -Pattern $marker.pattern
    }
}

$migratedConsumers = @(
    [pscustomobject]@{ Name = 'hello-runtime-inspector'; Path = 'hello-runtime-inspector' }
    [pscustomobject]@{ Name = 'hello-cgm'; Path = 'hello-cgm' }
    [pscustomobject]@{ Name = 'hello-ui-layout'; Path = 'ui/hello-ui-layout' }
    [pscustomobject]@{
        Name = 'runtime-observation-workbench-engine'
        Path = 'consumers/runtime-observation-workbench/engine'
    }
)

$migratedConsumerInventory = foreach ($consumer in $migratedConsumers) {
    $path = Join-Path $corpusRoot $consumer.Path
    $markers = foreach ($marker in $consumerMarkers) {
        [pscustomobject]@{
            marker = $marker.marker
            occurrences = Count-Matches -Path $path -Pattern $marker.pattern
        }
    }
    [pscustomobject]@{
        consumer = $consumer.Name
        markers = $markers
    }
}

$report = [pscustomobject]@{
    schema = 1
    generated_utc = [DateTime]::UtcNow.ToString('o')
    ui_tools = [pscustomobject]@{
        root_public_export_statements = Count-Matches -Path (Join-Path $sourceRoot 'lib.rs') -Pattern '^\s*pub\s+'
        source_public_declarations = Count-Matches -Path $sourceRoot -Pattern '^\s*pub\s+'
        total_tests = Count-Matches -Path $sourceRoot -Pattern '^\s*#\[test\]'
        capability_tests = $capabilityTests
    }
    corpus_consumer_markers = $consumerInventory
    migrated_consumer_markers = $migratedConsumerInventory
}

if ($AsJson) {
    $report | ConvertTo-Json -Depth 5
    return
}

Write-Output 'UI tools audit'
Write-Output ("  Root public export statements: {0}" -f $report.ui_tools.root_public_export_statements)
Write-Output ("  Source public declarations: {0}" -f $report.ui_tools.source_public_declarations)
Write-Output ("  Total unit tests: {0}" -f $report.ui_tools.total_tests)
Write-Output '  Tests by capability:'
foreach ($entry in $report.ui_tools.capability_tests) {
    Write-Output ("    {0}: {1}" -f $entry.capability, $entry.tests)
}
Write-Output '  Corpus consumer markers:'
foreach ($entry in $report.corpus_consumer_markers) {
    Write-Output ("    {0}: {1}" -f $entry.marker, $entry.occurrences)
}
Write-Output '  Migrated consumer markers:'
foreach ($consumer in $report.migrated_consumer_markers) {
    Write-Output ("    {0}:" -f $consumer.consumer)
    foreach ($entry in $consumer.markers) {
        Write-Output ("      {0}: {1}" -f $entry.marker, $entry.occurrences)
    }
}
