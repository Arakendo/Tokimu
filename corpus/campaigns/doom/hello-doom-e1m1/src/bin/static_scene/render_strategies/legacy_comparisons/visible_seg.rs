use super::super::super::{prepare_doom_seg_clip_presentation, SceneInput};
use tokimu::PlatformResult;

pub(crate) fn apply(scene: &mut SceneInput) -> PlatformResult<()> {
    let presentation = prepare_doom_seg_clip_presentation(scene, false)?;
    eprintln!(
        "E1M1 AR-0025 Stage 3B visible-SEG presentation: visible_intervals={}; source_triangles={}; submitted_draws={}; meaning=diagnostic-source-space-screen-span-comparison-not-historic-doom-parity",
        presentation.visible_intervals,
        presentation.source_triangles,
        presentation.draws.len(),
    );
    scene.opaque_draws = presentation.draws;
    scene.cutout_draws.clear();
    Ok(())
}
