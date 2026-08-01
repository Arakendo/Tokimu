use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiPresentationInputs {
    pub semantics: u64,
    pub measurement: u64,
    pub layout: u64,
    pub geometry: u64,
    pub draw_list: u64,
}

impl UiPresentationInputs {
    /// Records text or semantic-content change, which can affect every stage.
    pub const fn with_text_revision(mut self, revision: u64) -> Self {
        self.semantics = revision;
        self
    }

    /// Records visual-theme change without claiming text metric changes.
    ///
    /// Surface radius, border, and paint changes require geometry and draw-list
    /// work. A caller changing font metrics must also revise `measurement`.
    pub const fn with_theme_revision(mut self, revision: u64) -> Self {
        self.geometry = revision;
        self
    }

    /// Records available-bounds or scale change beginning at layout.
    pub const fn with_viewport_revision(mut self, revision: u64) -> Self {
        self.layout = revision;
        self
    }

    /// Records hover, focus, selection, or pressed-state presentation change.
    pub const fn with_interaction_revision(mut self, revision: u64) -> Self {
        self.draw_list = revision;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiPresentationInvalidation {
    pub semantics: bool,
    pub measurement: bool,
    pub layout: bool,
    pub geometry: bool,
    pub draw_list: bool,
}

impl UiPresentationInvalidation {
    pub const fn none(self) -> bool {
        !self.semantics && !self.measurement && !self.layout && !self.geometry && !self.draw_list
    }

    fn between(previous: UiPresentationInputs, current: UiPresentationInputs) -> Self {
        let semantics = previous.semantics != current.semantics;
        let measurement = semantics || previous.measurement != current.measurement;
        let layout = measurement || previous.layout != current.layout;
        let geometry = layout || previous.geometry != current.geometry;
        let draw_list = geometry || previous.draw_list != current.draw_list;
        Self {
            semantics,
            measurement,
            layout,
            geometry,
            draw_list,
        }
    }

    const fn all() -> Self {
        Self {
            semantics: true,
            measurement: true,
            layout: true,
            geometry: true,
            draw_list: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiPresentationRebuildEvidence {
    pub invalidation: UiPresentationInvalidation,
    pub semantic_rebuilds: u32,
    pub measurement_rebuilds: u32,
    pub layout_rebuilds: u32,
    pub geometry_rebuilds: u32,
    pub draw_list_rebuilds: u32,
}

/// Bounded measurements spanning UI-owned and renderer-observed work.
///
/// Producers populate only the stages they own. Upload, submit, and draw counts
/// are copied from renderer observations; their policy and lifetime remain
/// renderer-owned. Applications may feed individual fields into Tokimu's
/// kernel-native sustained performance monitors.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiPresentationWorkEvidence {
    pub measurement_micros: u32,
    pub layout_micros: u32,
    pub lowering_micros: u32,
    pub draw_list_micros: u32,
    pub uploads: u32,
    pub submits: u32,
    pub draws: u32,
}

impl UiPresentationWorkEvidence {
    pub fn with_measurement_time(mut self, elapsed: Duration) -> Self {
        self.measurement_micros = bounded_micros(elapsed);
        self
    }

    pub fn with_layout_time(mut self, elapsed: Duration) -> Self {
        self.layout_micros = bounded_micros(elapsed);
        self
    }

    pub fn with_lowering_time(mut self, elapsed: Duration) -> Self {
        self.lowering_micros = bounded_micros(elapsed);
        self
    }

    pub fn with_draw_list_time(mut self, elapsed: Duration) -> Self {
        self.draw_list_micros = bounded_micros(elapsed);
        self
    }

    pub const fn with_renderer_counts(mut self, uploads: u32, submits: u32, draws: u32) -> Self {
        self.uploads = uploads;
        self.submits = submits;
        self.draws = draws;
        self
    }
}

fn bounded_micros(elapsed: Duration) -> u32 {
    elapsed.as_micros().min(u128::from(u32::MAX)) as u32
}

impl UiPresentationRebuildEvidence {
    fn from_invalidation(invalidation: UiPresentationInvalidation) -> Self {
        Self {
            invalidation,
            semantic_rebuilds: u32::from(invalidation.semantics),
            measurement_rebuilds: u32::from(invalidation.measurement),
            layout_rebuilds: u32::from(invalidation.layout),
            geometry_rebuilds: u32::from(invalidation.geometry),
            draw_list_rebuilds: u32::from(invalidation.draw_list),
        }
    }
}

/// Tracks renderer-neutral UI stage invalidation without owning GPU caches.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiPresentationRevisionTracker {
    previous: Option<UiPresentationInputs>,
}

impl UiPresentationRevisionTracker {
    pub fn observe(&mut self, inputs: UiPresentationInputs) -> UiPresentationRebuildEvidence {
        let invalidation = self
            .previous
            .map(|previous| UiPresentationInvalidation::between(previous, inputs))
            .unwrap_or_else(UiPresentationInvalidation::all);
        self.previous = Some(inputs);
        UiPresentationRebuildEvidence::from_invalidation(invalidation)
    }

    pub const fn previous(&self) -> Option<UiPresentationInputs> {
        self.previous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_inputs_produce_zero_rebuilds_after_initial_observation() {
        let inputs = UiPresentationInputs::default();
        let mut tracker = UiPresentationRevisionTracker::default();
        assert_eq!(tracker.observe(inputs).semantic_rebuilds, 1);
        assert_eq!(
            tracker.observe(inputs),
            UiPresentationRebuildEvidence::default()
        );
    }

    #[test]
    fn invalidation_cascades_only_to_dependent_stages() {
        let mut tracker = UiPresentationRevisionTracker::default();
        tracker.observe(UiPresentationInputs::default());
        let evidence = tracker.observe(UiPresentationInputs {
            geometry: 1,
            ..UiPresentationInputs::default()
        });
        assert_eq!(evidence.layout_rebuilds, 0);
        assert_eq!(evidence.geometry_rebuilds, 1);
        assert_eq!(evidence.draw_list_rebuilds, 1);
    }

    #[test]
    fn common_mutations_begin_at_their_declared_stage() {
        let baseline = UiPresentationInputs::default();

        assert_mutation(
            baseline.with_text_revision(1),
            UiPresentationInvalidation::all(),
        );
        assert_mutation(
            baseline.with_theme_revision(1),
            UiPresentationInvalidation {
                geometry: true,
                draw_list: true,
                ..UiPresentationInvalidation::default()
            },
        );
        assert_mutation(
            baseline.with_viewport_revision(1),
            UiPresentationInvalidation {
                layout: true,
                geometry: true,
                draw_list: true,
                ..UiPresentationInvalidation::default()
            },
        );
        assert_mutation(
            baseline.with_interaction_revision(1),
            UiPresentationInvalidation {
                draw_list: true,
                ..UiPresentationInvalidation::default()
            },
        );
    }

    #[test]
    fn work_evidence_preserves_owned_measurements_and_saturates_time() {
        let evidence = UiPresentationWorkEvidence::default()
            .with_measurement_time(Duration::from_micros(12))
            .with_layout_time(Duration::from_micros(34))
            .with_lowering_time(Duration::from_micros(56))
            .with_draw_list_time(Duration::from_secs(u64::MAX))
            .with_renderer_counts(2, 3, 40);

        assert_eq!(evidence.measurement_micros, 12);
        assert_eq!(evidence.layout_micros, 34);
        assert_eq!(evidence.lowering_micros, 56);
        assert_eq!(evidence.draw_list_micros, u32::MAX);
        assert_eq!(
            (evidence.uploads, evidence.submits, evidence.draws),
            (2, 3, 40)
        );
    }

    fn assert_mutation(inputs: UiPresentationInputs, expected: UiPresentationInvalidation) {
        let mut tracker = UiPresentationRevisionTracker::default();
        tracker.observe(UiPresentationInputs::default());
        assert_eq!(tracker.observe(inputs).invalidation, expected);
    }
}
