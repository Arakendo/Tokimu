[CmdletBinding()]
param(
    [string]$FixtureRoot = (Join-Path $PSScriptRoot "..\third-party\fixtures\fbx-corpus"),
    [string]$CacheRoot = (Join-Path $PSScriptRoot "..\target\ufbx-corpus-source")
)

$ErrorActionPreference = "Stop"

$repository = "https://github.com/ufbx/ufbx.git"
$revision = "fcc5d6ba444cfd3eb80677dba5e37e493941abe5"
$fixtureRoot = [IO.Path]::GetFullPath($FixtureRoot)
$cacheRoot = [IO.Path]::GetFullPath($CacheRoot)
$upstreamRoot = Join-Path $fixtureRoot "upstream"

$files = @(
    "LICENSE",
    "data/maya_cube_6100_ascii.fbx",
    "data/maya_cube_6100_binary.fbx",
    "data/maya_cube_7500_ascii.fbx",
    "data/maya_cube_7500_binary.fbx",
    "data/maya_cube_big_endian_7500_binary.fbx",
    "data/maya_cube.obj",
    "data/maya_cube_big_endian.obj",
    "data/blender_279_uv_sets_6100_ascii.fbx",
    "data/blender_279_uv_sets_7400_binary.fbx",
    "data/blender_293_instancing_7400_binary.fbx",
    "data/blender_293_instancing.obj",
    "data/blender_340_y_up_7400_binary.fbx",
    "data/blender_340_y_up.obj",
    "data/blender_340_z_up_7400_binary.fbx",
    "data/blender_340_z_up.obj",
    "data/max_gltf_material_7700_ascii.fbx",
    "data/max_gltf_material_7700_binary.fbx",
    "data/max_unicode_7500_ascii.fbx",
    "data/max_unicode_7500_binary.fbx",
    "data/max2009_cube_anim_5800_ascii.fbx",
    "data/max2009_cube_anim_5800_binary.fbx",
    "data/max2009_cube_anim_15.obj",
    "data/max2009_cube_anim_45.obj",
    "data/max_transformed_skin_7500_ascii.fbx",
    "data/max_transformed_skin_7500_binary.fbx",
    "data/max_transformed_skin_5.obj",
    "data/max_transformed_skin_15.obj",
    "data/blender_331_static_blend_shape_7400_binary.fbx",
    "data/blender_331_static_blend_shape.obj",
    "data/blender440_shape_weight_anim_7400_binary.fbx",
    "data/blender440_shape_weight_anim_5.obj",
    "data/blender440_shape_weight_anim_15.obj",
    "data/blender440_shape_weight_anim_25.obj",
    "data/synthetic_truncated_quot_fail_7500_ascii.fbx",
    "data/synthetic_bad_inf_nan_fail_7700_ascii.fbx",
    "data/synthetic_broken_cluster_7500_ascii.fbx"
)

if (-not (Test-Path -LiteralPath (Join-Path $cacheRoot ".git"))) {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $cacheRoot) | Out-Null
    git clone --no-checkout $repository $cacheRoot
}

$safeDirectory = $cacheRoot.Replace("\", "/")
$actualRevision = git -c "safe.directory=$safeDirectory" -C $cacheRoot rev-parse HEAD
if ($actualRevision -ne $revision) {
    git -c "safe.directory=$safeDirectory" -C $cacheRoot fetch --depth 1 origin $revision
    git -c "safe.directory=$safeDirectory" -C $cacheRoot checkout --detach $revision
    $actualRevision = git -c "safe.directory=$safeDirectory" -C $cacheRoot rev-parse HEAD
}
if ($actualRevision -ne $revision) {
    throw "Expected ufbx revision $revision, got $actualRevision"
}

foreach ($relativePath in $files) {
    $source = Join-Path $cacheRoot $relativePath
    if (-not (Test-Path -LiteralPath $source)) {
        throw "Pinned ufbx revision is missing selected file: $relativePath"
    }
    $destination = Join-Path $upstreamRoot $relativePath
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
    Copy-Item -LiteralPath $source -Destination $destination -Force
}

& (Join-Path $PSScriptRoot "verify-ufbx-fbx-corpus.ps1") -FixtureRoot $fixtureRoot
