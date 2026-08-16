use super::super::super::{prepare_doom_seg_classic_context_presentation, SceneInput};
use tokimu::PlatformResult;

pub(crate) fn apply(scene: &mut SceneInput) -> PlatformResult<()> {
    let presentation = prepare_doom_seg_classic_context_presentation(scene)?;
    eprintln!(
        "E1M1 AR-0025 Stage 3B classic-context presentation: plane-meshes={}; plane-triangles={}; wall-meshes={}; omitted-wall-triangles={}; total-draws={}; meaning=fixed-source-spawn-context-control-with-whole-seg-wall-tiers-not-visplane-or-historic-pixel-parity",
        presentation.plane_meshes,
        presentation.plane_triangles,
        presentation.wall_meshes,
        presentation.omitted_wall_triangles,
        presentation.draws.len(),
    );
    scene.opaque_draws = presentation.draws;
    scene.cutout_draws.clear();
    Ok(())
}
