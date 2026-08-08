[CmdletBinding()]
param(
    [string]$ConfigurationPath = (Join-Path $PSScriptRoot 'ring-zero-dependencies.json'),
    [string]$RepositoryRoot,
    [switch]$AllowViolations
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-FullPath([string]$Path) {
    return [System.IO.Path]::GetFullPath($Path)
}

function Test-ChildPath([string]$Path, [string]$Parent) {
    $candidate = (Get-FullPath $Path).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    $container = (Get-FullPath $Parent).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    return $candidate -eq $container -or $candidate.StartsWith($container + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)
}

function Get-SubmoduleStates([string]$RepositoryRoot) {
    $states = @{}
    $lines = & git -C $RepositoryRoot submodule status --recursive
    if ($LASTEXITCODE -ne 0) {
        throw 'Unable to read Git submodule status.'
    }

    foreach ($line in $lines) {
        if ($line -notmatch '^(.)([0-9a-f]+)\s+([^\s]+)') {
            continue
        }

        $states[$Matches[3]] = $Matches[1]
    }

    return $states
}

$repositoryRoot = if ([string]::IsNullOrWhiteSpace($RepositoryRoot)) {
    Get-FullPath (Join-Path $PSScriptRoot '..')
}
else {
    Get-FullPath $RepositoryRoot
}
$configPath = Get-FullPath $ConfigurationPath
if (-not (Test-Path -LiteralPath $configPath -PathType Leaf)) {
    throw "Ring 0 dependency configuration was not found: $configPath"
}

$config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
if ($null -eq $config.roots -or $config.roots.Count -eq 0) {
    throw 'Ring 0 dependency configuration must declare at least one root package.'
}

$approvedSources = @($config.approvedRingZeroSources)
$approvedSourcePaths = @()
foreach ($source in $approvedSources) {
    if ([string]::IsNullOrWhiteSpace($source.path)) {
        throw 'Each approved Ring 0 source must declare a path.'
    }
    $approvedSourcePaths += [pscustomobject]@{
        Name = if ([string]::IsNullOrWhiteSpace($source.name)) { $source.path } else { $source.name }
        RelativePath = $source.path.Replace('/', [System.IO.Path]::DirectorySeparatorChar)
        SubmodulePath = $source.path.Replace('\\', '/')
        FullPath = Get-FullPath (Join-Path $repositoryRoot $source.path)
    }
}

$submoduleStates = Get-SubmoduleStates $repositoryRoot
$violations = [System.Collections.Generic.List[string]]::new()
foreach ($source in $approvedSourcePaths) {
    if (-not (Test-Path -LiteralPath $source.FullPath -PathType Container)) {
        $violations.Add("approved source '$($source.Name)' is missing at $($source.RelativePath)")
        continue
    }

    if (-not $submoduleStates.ContainsKey($source.SubmodulePath)) {
        $violations.Add("approved source '$($source.Name)' is not a Git submodule at $($source.RelativePath)")
        continue
    }

    if ($submoduleStates[$source.SubmodulePath] -ne ' ') {
        $violations.Add("approved source '$($source.Name)' is not clean and parent-pinned at $($source.RelativePath)")
    }

    $ignoreSetting = & git -C $repositoryRoot config -f .gitmodules --get "submodule.$($source.SubmodulePath).ignore"
    if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace(($ignoreSetting -join "`n"))) {
        $violations.Add("approved source '$($source.Name)' uses .gitmodules ignore=$($ignoreSetting -join ',') at $($source.RelativePath)")
    }
    elseif ($LASTEXITCODE -gt 1) {
        $violations.Add("approved source '$($source.Name)' could not read its .gitmodules ignore setting at $($source.RelativePath)")
    }

    $safeDirectory = "safe.directory=$($source.FullPath.Replace('\\', '/'))"
    $submoduleChanges = & git -c $safeDirectory -C $source.FullPath status --porcelain
    if ($LASTEXITCODE -ne 0) {
        $violations.Add("approved source '$($source.Name)' could not be inspected for local changes at $($source.RelativePath)")
    }
    elseif (-not [string]::IsNullOrWhiteSpace(($submoduleChanges -join "`n"))) {
        $violations.Add("approved source '$($source.Name)' has uncommitted changes at $($source.RelativePath)")
    }
}

$metadataManifest = Join-Path $repositoryRoot 'Cargo.toml'
if (-not (Test-Path -LiteralPath $metadataManifest -PathType Leaf)) {
    throw "Ring 0 repository root does not contain Cargo.toml: $repositoryRoot"
}
$metadataText = & cargo metadata --manifest-path $metadataManifest --format-version 1 --locked
if ($LASTEXITCODE -ne 0) {
    throw 'cargo metadata failed.'
}
$metadata = $metadataText | ConvertFrom-Json

$packagesById = @{}
foreach ($package in $metadata.packages) {
    $packagesById[$package.id] = $package
}

$nodesById = @{}
foreach ($node in $metadata.resolve.nodes) {
    $nodesById[$node.id] = $node
}

$rootIds = @()
$workspacePackageIds = [System.Collections.Generic.HashSet[string]]::new()
foreach ($workspaceMember in $metadata.workspace_members) {
    [void]$workspacePackageIds.Add($workspaceMember)
}
foreach ($rootName in $config.roots) {
    $matches = @($metadata.workspace_members | Where-Object { $packagesById[$_].name -eq $rootName })
    if ($matches.Count -ne 1) {
        throw "Ring 0 root '$rootName' must resolve to exactly one workspace package; found $($matches.Count)."
    }
    $rootIds += $matches[0]
}

$visited = [System.Collections.Generic.HashSet[string]]::new()
$pending = [System.Collections.Generic.Queue[string]]::new()
$inboundEdges = @{}
foreach ($rootId in $rootIds) {
    $pending.Enqueue($rootId)
    $inboundEdges[$rootId] = [System.Collections.Generic.List[string]]::new()
    $inboundEdges[$rootId].Add('root@all-targets')
}

while ($pending.Count -gt 0) {
    $packageId = $pending.Dequeue()
    if (-not $visited.Add($packageId)) {
        continue
    }

    $node = $nodesById[$packageId]
    if ($null -eq $node) {
        throw "Cargo metadata did not include a resolve node for '$packageId'."
    }

    foreach ($dependency in $node.deps) {
        $kinds = @($dependency.dep_kinds)
        $includeDependency = $kinds.Count -eq 0
        $includedKinds = [System.Collections.Generic.List[string]]::new()
        foreach ($kind in $kinds) {
            if ($kind.kind -ne 'dev') {
                $includeDependency = $true
                $kindName = if ($null -eq $kind.kind) { 'normal' } else { $kind.kind }
                $targetName = if ($null -eq $kind.target) { 'all-targets' } else { $kind.target }
                $includedKinds.Add("$kindName@$targetName")
            }
        }
        if ($includeDependency) {
            $pending.Enqueue($dependency.pkg)
            if (-not $inboundEdges.ContainsKey($dependency.pkg)) {
                $inboundEdges[$dependency.pkg] = [System.Collections.Generic.List[string]]::new()
            }
            if ($includedKinds.Count -eq 0) {
                $includedKinds.Add('normal@all-targets')
            }
            foreach ($edge in $includedKinds) {
                if (-not $inboundEdges[$dependency.pkg].Contains($edge)) {
                    $inboundEdges[$dependency.pkg].Add($edge)
                }
            }
        }
    }
}

$rows = foreach ($packageId in $visited) {
    $package = $packagesById[$packageId]
    $manifestDirectory = Split-Path -Parent $package.manifest_path
    $classification = ''
    $detail = ''

    if ($null -ne $package.source) {
        if ($package.source.StartsWith('registry+')) {
            $classification = 'registry (rejected)'
            $detail = $package.source
            $violations.Add("$($package.name) $($package.version) resolves from registry source $($package.source)")
        }
        elseif ($package.source.StartsWith('git+')) {
            $classification = 'remote Git (rejected)'
            $detail = $package.source
            $violations.Add("$($package.name) $($package.version) resolves from remote Git source $($package.source)")
        }
        else {
            $classification = 'unknown external source (rejected)'
            $detail = $package.source
            $violations.Add("$($package.name) $($package.version) resolves from unapproved source $($package.source)")
        }
    }
    else {
        $approvedSource = @($approvedSourcePaths | Where-Object { Test-ChildPath $manifestDirectory $_.FullPath }) | Select-Object -First 1
        if ($null -ne $approvedSource) {
            $classification = 'approved Ring 0 submodule'
            $detail = $approvedSource.RelativePath
        }
        elseif ($workspacePackageIds.Contains($packageId)) {
            $classification = 'Tokimu workspace path'
            $detail = $manifestDirectory.Substring($repositoryRoot.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
        }
        else {
            $classification = 'unapproved local path (rejected)'
            $detail = $manifestDirectory
            $violations.Add("$($package.name) $($package.version) resolves from unapproved local path $manifestDirectory")
        }
    }

    $node = $nodesById[$packageId]
    [pscustomobject]@{
        Package = $package.name
        Version = $package.version
        Source = $classification
        Detail = $detail
        Features = (@($node.features) | Sort-Object) -join ','
        DependencyKinds = (@($inboundEdges[$packageId]) | Sort-Object) -join ','
        TargetConditions = ((@($inboundEdges[$packageId]) | ForEach-Object { ($_ -split '@', 2)[1] } | Sort-Object -Unique) -join ',')
    }
}

Write-Output 'Ring 0 dependency closure:'
$rows | Sort-Object Package, Version | Format-Table -AutoSize | Out-String | Write-Output

if ($violations.Count -gt 0) {
    [Console]::Error.WriteLine(('Ring 0 source audit found {0} violation(s):' -f $violations.Count))
    foreach ($violation in ($violations | Sort-Object -Unique)) {
        [Console]::Error.WriteLine("- $violation")
    }
    if (-not $AllowViolations) {
        exit 1
    }
}

if ($violations.Count -eq 0) {
    Write-Output 'Ring 0 source audit passed.'
}
