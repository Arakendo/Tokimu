use hello_doom_e1m1::{
    classify_static_draw_frustum_rejection, StaticDrawAabb, StaticDrawFrustumRejection,
    StaticDrawPlanEntry, StaticDrawSphere,
};
use tokimu::Camera;
use tokimu_core::math::{Mat4, Vec3};

use super::model::{
    CandidateSelection, CandidateSelectionSummary, GroupCandidateSelectionSummary,
    UniformGridAabbIndex, UniformGridSelectionSummary,
};

pub(crate) fn draw_bounds(draws: &[StaticDrawPlanEntry]) -> Vec<Option<StaticDrawAabb>> {
    draws
        .iter()
        .map(|draw| StaticDrawAabb::from_positions(&draw.mesh.positions))
        .collect()
}

pub(crate) fn draw_spheres(draws: &[StaticDrawPlanEntry]) -> Vec<Option<StaticDrawSphere>> {
    draws
        .iter()
        .map(|draw| StaticDrawSphere::from_positions(&draw.mesh.positions))
        .collect()
}

pub(crate) fn candidate_is_selected(
    policy: CandidateSelection,
    bounds: Option<StaticDrawAabb>,
    view_projection: Mat4,
    summary: &mut CandidateSelectionSummary,
    source_label: &str,
    rejection_samples: &mut Vec<String>,
    capture_sample: bool,
) -> bool {
    summary.candidates += 1;
    let rejection = match (policy, bounds) {
        (CandidateSelection::FullSubmission, _) => None,
        (CandidateSelection::UniformGrid8x4x8, _) => {
            unreachable!("uniform-grid selection must use the grid broad-phase path")
        }
        (CandidateSelection::DoomMembershipUnion, _) => {
            unreachable!("membership selection must use source-topology evidence")
        }
        (CandidateSelection::DoomSegPerColumn, _) => {
            unreachable!("SEG selection must use retained source-grid evidence")
        }
        (CandidateSelection::DoomClassicBsp, _) => {
            unreachable!("classic BSP selection must use retained Doom source evidence")
        }
        (CandidateSelection::FrustumAabb, Some(bounds)) => {
            classify_static_draw_frustum_rejection(bounds, view_projection)
        }
        (CandidateSelection::FrustumAabb, None) => {
            // Uncertain bounds fail open. This preserves correctness while
            // retaining pressure to repair the invalid candidate evidence.
            summary.uncertain_bounds += 1;
            None
        }
    };
    if let Some(rejection) = rejection {
        summary.rejected += 1;
        summary.rejected_by_plane[frustum_rejection_index(rejection)] += 1;
        if capture_sample && rejection_samples.len() < 12 {
            rejection_samples.push(format!("{source_label}:{rejection:?}"));
        }
        false
    } else {
        summary.submitted += 1;
        true
    }
}

#[allow(clippy::too_many_arguments)]
fn select_candidates(
    policy: CandidateSelection,
    bounds: &[Option<StaticDrawAabb>],
    draws: &[StaticDrawPlanEntry],
    view_projection: Mat4,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) {
    debug_assert_eq!(bounds.len(), draws.len());
    debug_assert_eq!(selected.len(), draws.len());
    for ((selected, bounds), draw) in selected.iter_mut().zip(bounds.iter().copied()).zip(draws) {
        *selected = candidate_is_selected(
            policy,
            bounds,
            view_projection,
            summary,
            &draw.source_label,
            rejection_samples,
            capture_samples,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn select_current_candidates(
    policy: CandidateSelection,
    grid: Option<&UniformGridAabbIndex>,
    bounds: &[Option<StaticDrawAabb>],
    draws: &[StaticDrawPlanEntry],
    view_projection: Mat4,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) {
    match policy {
        CandidateSelection::FullSubmission | CandidateSelection::FrustumAabb => select_candidates(
            policy,
            bounds,
            draws,
            view_projection,
            selected,
            summary,
            rejection_samples,
            capture_samples,
        ),
        CandidateSelection::UniformGrid8x4x8 => {
            let Some(grid) = grid else {
                selected.fill(true);
                summary.candidates += bounds.len();
                summary.submitted += bounds.len();
                summary.uncertain_bounds += bounds.len();
                return;
            };
            let (grid_selected, grid_summary) = grid.select(bounds, view_projection);
            debug_assert_eq!(grid_selected.len(), draws.len());
            selected.copy_from_slice(&grid_selected);
            summary.candidates += bounds.len();
            summary.rejected += grid_summary.rejected;
            summary.submitted += grid_summary.submitted;
            summary.uncertain_bounds += grid_summary.uncertain_bounds;
            for (total, rejected) in summary
                .rejected_by_plane
                .iter_mut()
                .zip(grid_summary.rejected_by_plane)
            {
                *total += rejected;
            }
            if capture_samples {
                for (selected, draw) in grid_selected.iter().zip(draws) {
                    if !selected && rejection_samples.len() < 12 {
                        rejection_samples
                            .push(format!("{}:uniform-grid-filtered", draw.source_label));
                    }
                }
            }
        }
        CandidateSelection::DoomMembershipUnion => {
            unreachable!("membership selection must use source-topology evidence")
        }
        CandidateSelection::DoomSegPerColumn => {
            unreachable!("SEG per-column selection must use retained Doom source evidence")
        }
        CandidateSelection::DoomClassicBsp => {
            unreachable!("classic BSP selection must use retained Doom source evidence")
        }
    }
}

const fn frustum_rejection_index(rejection: StaticDrawFrustumRejection) -> usize {
    match rejection {
        StaticDrawFrustumRejection::Left => 0,
        StaticDrawFrustumRejection::Right => 1,
        StaticDrawFrustumRejection::Bottom => 2,
        StaticDrawFrustumRejection::Top => 3,
        StaticDrawFrustumRejection::Near => 4,
        StaticDrawFrustumRejection::Far => 5,
    }
}

pub(crate) fn summarize_candidate_selection<'a, Bounds: Copy>(
    draws: impl Iterator<Item = (&'a StaticDrawPlanEntry, Option<Bounds>)>,
    camera: Camera,
    classify: impl Fn(Bounds, Mat4) -> Option<StaticDrawFrustumRejection>,
) -> (CandidateSelectionSummary, Vec<String>) {
    let mut summary = CandidateSelectionSummary::default();
    let mut samples = Vec::new();
    let view_projection = camera.projection * camera.view;
    for (draw, bounds) in draws {
        summary.candidates += 1;
        let rejection = match bounds {
            Some(bounds) => classify(bounds, view_projection),
            None => {
                summary.uncertain_bounds += 1;
                None
            }
        };
        if let Some(rejection) = rejection {
            summary.rejected += 1;
            summary.rejected_by_plane[frustum_rejection_index(rejection)] += 1;
            if samples.len() < 12 {
                samples.push(format!("{}:{rejection:?}", draw.source_label));
            }
        } else {
            summary.submitted += 1;
        }
    }
    (summary, samples)
}

pub(crate) fn summarize_grouped_aabb_selection(
    bounds: &[Option<StaticDrawAabb>],
    view_projection: Mat4,
    group_size: usize,
) -> GroupCandidateSelectionSummary {
    assert!(
        group_size > 0,
        "grouped selection requires a non-zero group size"
    );
    let mut summary = GroupCandidateSelectionSummary::default();
    for group in bounds.chunks(group_size) {
        summary.groups += 1;
        let group_bounds = if group.iter().all(Option::is_some) {
            StaticDrawAabb::enclosing_iter(group.iter().flatten().copied())
        } else {
            None
        };
        let rejected = group_bounds
            .and_then(|bounds| classify_static_draw_frustum_rejection(bounds, view_projection))
            .is_some();
        if rejected {
            summary.rejected_groups += 1;
        } else {
            summary.submitted_groups += 1;
            summary.submitted_draws += group.len();
            if group_bounds.is_none() {
                summary.uncertain_groups += 1;
            }
        }
    }
    summary
}

impl UniformGridAabbIndex {
    pub(crate) fn build(bounds: &[Option<StaticDrawAabb>], dimensions: [usize; 3]) -> Option<Self> {
        if dimensions.contains(&0) {
            return None;
        }
        let scene_bounds = StaticDrawAabb::enclosing_iter(bounds.iter().flatten().copied())?;
        let cell_count = dimensions
            .into_iter()
            .try_fold(1_usize, usize::checked_mul)?;
        let mut index = Self {
            bounds: scene_bounds,
            dimensions,
            cells: (0..cell_count).map(|_| Vec::new()).collect(),
            uncertain_draws: Vec::new(),
        };
        for (draw_index, bounds) in bounds.iter().copied().enumerate() {
            let Some(bounds) = bounds else {
                index.uncertain_draws.push(draw_index);
                continue;
            };
            let minimum = index.cell_coordinates(bounds.minimum());
            let maximum = index.cell_coordinates(bounds.maximum());
            for z in minimum[2]..=maximum[2] {
                for y in minimum[1]..=maximum[1] {
                    for x in minimum[0]..=maximum[0] {
                        let cell_index = index.cell_index([x, y, z]);
                        index.cells[cell_index].push(draw_index);
                    }
                }
            }
        }
        Some(index)
    }

    fn cell_coordinates(&self, point: Vec3) -> [usize; 3] {
        let minimum = self.bounds.minimum();
        let maximum = self.bounds.maximum();
        let extent = maximum - minimum;
        [
            grid_coordinate(point.x, minimum.x, extent.x, self.dimensions[0]),
            grid_coordinate(point.y, minimum.y, extent.y, self.dimensions[1]),
            grid_coordinate(point.z, minimum.z, extent.z, self.dimensions[2]),
        ]
    }

    fn cell_index(&self, coordinates: [usize; 3]) -> usize {
        (coordinates[2] * self.dimensions[1] + coordinates[1]) * self.dimensions[0] + coordinates[0]
    }

    fn cell_bounds(&self, cell_index: usize) -> StaticDrawAabb {
        let x = cell_index % self.dimensions[0];
        let y = (cell_index / self.dimensions[0]) % self.dimensions[1];
        let z = cell_index / (self.dimensions[0] * self.dimensions[1]);
        let minimum = self.bounds.minimum();
        let extent = self.bounds.maximum() - minimum;
        let cell_minimum = minimum
            + Vec3::new(
                extent.x * x as f32 / self.dimensions[0] as f32,
                extent.y * y as f32 / self.dimensions[1] as f32,
                extent.z * z as f32 / self.dimensions[2] as f32,
            );
        let cell_maximum = minimum
            + Vec3::new(
                extent.x * (x + 1) as f32 / self.dimensions[0] as f32,
                extent.y * (y + 1) as f32 / self.dimensions[1] as f32,
                extent.z * (z + 1) as f32 / self.dimensions[2] as f32,
            );
        StaticDrawAabb::from_minimum_maximum(cell_minimum, cell_maximum)
            .expect("uniform grid construction must produce finite ordered bounds")
    }

    pub(crate) fn select(
        &self,
        bounds: &[Option<StaticDrawAabb>],
        view_projection: Mat4,
    ) -> (Vec<bool>, UniformGridSelectionSummary) {
        let mut candidates = vec![false; bounds.len()];
        let mut summary = UniformGridSelectionSummary::default();
        for &draw_index in &self.uncertain_draws {
            candidates[draw_index] = true;
            summary.uncertain_bounds += 1;
        }
        for (cell_index, draw_indices) in self.cells.iter().enumerate() {
            if draw_indices.is_empty() {
                continue;
            }
            summary.cells_tested += 1;
            if classify_static_draw_frustum_rejection(self.cell_bounds(cell_index), view_projection)
                .is_some()
            {
                summary.cells_rejected += 1;
                continue;
            }
            for &draw_index in draw_indices {
                candidates[draw_index] = true;
            }
        }
        summary.grid_candidates = candidates.iter().filter(|candidate| **candidate).count();
        for (candidate, bounds) in candidates.iter_mut().zip(bounds.iter().copied()) {
            if !*candidate {
                summary.rejected += 1;
                continue;
            }
            match bounds {
                Some(bounds) => {
                    summary.exact_tests += 1;
                    if let Some(rejection) =
                        classify_static_draw_frustum_rejection(bounds, view_projection)
                    {
                        *candidate = false;
                        summary.rejected += 1;
                        summary.rejected_by_plane[frustum_rejection_index(rejection)] += 1;
                    } else {
                        summary.submitted += 1;
                    }
                }
                None => summary.submitted += 1,
            }
        }
        (candidates, summary)
    }
}

fn grid_coordinate(value: f32, minimum: f32, extent: f32, dimension: usize) -> usize {
    if extent <= f32::EPSILON {
        return 0;
    }
    (((value - minimum) / extent * dimension as f32).floor() as isize)
        .clamp(0, dimension.saturating_sub(1) as isize) as usize
}

#[cfg(test)]
mod tests {
    use hello_doom_e1m1::StaticDrawAabb;
    use tokimu_core::math::{Mat4, Vec3};

    use super::{candidate_is_selected, summarize_grouped_aabb_selection};
    use crate::candidate_selection::{
        CandidateSelection, CandidateSelectionSummary, UniformGridAabbIndex,
    };

    #[test]
    fn frustum_selection_fails_open_for_uncertain_bounds() {
        let mut summary = CandidateSelectionSummary::default();
        let mut samples = Vec::new();

        assert!(candidate_is_selected(
            CandidateSelection::FrustumAabb,
            None,
            Mat4::IDENTITY,
            &mut summary,
            "uncertain",
            &mut samples,
            true,
        ));
        assert_eq!(summary.candidates, 1);
        assert_eq!(summary.submitted, 1);
        assert_eq!(summary.uncertain_bounds, 1);
        assert!(samples.is_empty());
    }

    #[test]
    fn frustum_selection_preserves_survivor_order() {
        let bounds = [
            bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
            bounds([2.0, -0.5, -0.5], [3.0, 0.5, 0.5]),
            bounds([-0.25, -0.25, -0.25], [0.25, 0.25, 0.25]),
        ];
        let labels = ["A", "B", "C"];
        let mut summary = CandidateSelectionSummary::default();
        let mut samples = Vec::new();
        let survivors = bounds
            .iter()
            .copied()
            .zip(labels)
            .filter_map(|(bounds, label)| {
                candidate_is_selected(
                    CandidateSelection::FrustumAabb,
                    Some(bounds),
                    Mat4::IDENTITY,
                    &mut summary,
                    label,
                    &mut samples,
                    true,
                )
                .then_some(label)
            })
            .collect::<Vec<_>>();

        assert_eq!(survivors, ["A", "C"]);
        assert_eq!(summary.candidates, 3);
        assert_eq!(summary.rejected, 1);
        assert_eq!(summary.submitted, 2);
    }

    #[test]
    fn grouped_selection_fails_open_for_crossing_or_uncertain_members() {
        let bounds = [
            Some(bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5])),
            Some(bounds([2.0, -0.5, -0.5], [3.0, 0.5, 0.5])),
            None,
        ];

        let groups = summarize_grouped_aabb_selection(&bounds, Mat4::IDENTITY, 2);
        assert_eq!(groups.groups, 2);
        assert_eq!(groups.rejected_groups, 0);
        assert_eq!(groups.submitted_draws, 3);
        assert_eq!(groups.uncertain_groups, 1);
    }

    #[test]
    fn uniform_grid_preserves_the_per_draw_conservative_survivors() {
        let bounds = [
            Some(bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5])),
            Some(bounds([-3.0, -0.5, -0.5], [-2.0, 0.5, 0.5])),
            Some(bounds([-2.0, -0.5, -0.5], [0.25, 0.5, 0.5])),
            None,
        ];
        let index =
            UniformGridAabbIndex::build(&bounds, [2, 1, 1]).expect("fixture has finite bounds");
        let (survivors, summary) = index.select(&bounds, Mat4::IDENTITY);

        assert_eq!(survivors, [true, false, true, true]);
        assert_eq!(summary.submitted, 3);
        assert_eq!(summary.rejected, 1);
        assert_eq!(summary.uncertain_bounds, 1);
    }

    fn bounds(minimum: [f32; 3], maximum: [f32; 3]) -> StaticDrawAabb {
        StaticDrawAabb::from_minimum_maximum(Vec3::from_array(minimum), Vec3::from_array(maximum))
            .expect("finite ordered fixture bounds")
    }
}
