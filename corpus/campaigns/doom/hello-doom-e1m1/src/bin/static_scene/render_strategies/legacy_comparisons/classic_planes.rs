use super::super::super::{prepare_doom_seg_classic_plane_presentation, SceneInput};
use tokimu::PlatformResult;

pub(crate) fn apply(scene: &mut SceneInput) -> PlatformResult<()> {
    let presentation = prepare_doom_seg_classic_plane_presentation(scene)?;
    eprintln!(
        "E1M1 AR-0025 Stage 3B classic-plane presentation: source-cells={}; grouped-meshes={}; triangles={}; meaning=fixed-source-spawn-doom-plane-comparison-not-visplane-parity-or-renderer-visibility",
        presentation.source_cells,
        presentation.grouped_meshes,
        presentation.triangles,
    );
    scene.opaque_draws = presentation.draws;
    scene.cutout_draws.clear();
    Ok(())
}
