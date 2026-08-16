use super::super::super::{prepare_doom_seg_per_column_presentation, SceneInput, StaticDrawSource};
use tokimu::PlatformResult;

pub(crate) fn apply(scene: &mut SceneInput) -> PlatformResult<()> {
    let presentation = prepare_doom_seg_per_column_presentation(scene)?;
    eprintln!(
        "E1M1 AR-0025 Stage 3B per-column SEG comparison: selected_segs={}; submitted_wall_draws={}; meaning=diagnostic-source-space-grid-comparison-not-historic-doom-parity",
        presentation.selected_segs,
        presentation.wall_draws.len(),
    );
    scene
        .opaque_draws
        .retain(|draw| !matches!(draw.source, StaticDrawSource::Wall { .. }));
    scene.opaque_draws.extend(presentation.wall_draws);
    Ok(())
}
