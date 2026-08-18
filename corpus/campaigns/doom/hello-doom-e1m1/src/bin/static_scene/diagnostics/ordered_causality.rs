//! Target-specific causal observations over the retained ordered Doom replay.
//!
//! This module freezes the six exact BVH/source cases and records observation-
//! only stage evidence. It does not change source decisions or renderer work.

use super::super::*;
use super::tokimu_spatial_bake::SpatialRayShadow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OrderedSixRayExpectedTarget {
    RejectedWallSegs {
        source_linedef: u32,
        source_segs: &'static [u32],
    },
    RejectedPlane {
        subsector: u32,
        kind: OrderedPlaneKind,
    },
    PartialPlane {
        subsector: u32,
        kind: OrderedPlaneKind,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct OrderedSixRayCase {
    pub(super) name: &'static str,
    pub(super) origin: [f64; 3],
    pub(super) direction: [f64; 3],
    pub(super) expected_global_label: &'static str,
    pub(super) expected: OrderedSixRayExpectedTarget,
}

const WALL_230_SEGS: &[u32] = &[415, 423];
const WALL_247_SEGS: &[u32] = &[559, 567];

const ORDERED_SIX_RAY_CASES: [OrderedSixRayCase; 6] = [
    OrderedSixRayCase {
        name: "hut-east-wall-230",
        origin: [2076.0, -3560.0, 36.0],
        direction: [0.905568898, -0.424199343, 0.0],
        expected_global_label: "wall:230:BROWN1",
        expected: OrderedSixRayExpectedTarget::RejectedWallSegs {
            source_linedef: 230,
            source_segs: WALL_230_SEGS,
        },
    },
    OrderedSixRayCase {
        name: "wall-247-east",
        origin: [1306.508666992, -3272.168457031, 21.432840347],
        direction: [0.939651787, -0.338751376, 0.047981590],
        expected_global_label: "wall:247:BROWN96",
        expected: OrderedSixRayExpectedTarget::RejectedWallSegs {
            source_linedef: 247,
            source_segs: WALL_247_SEGS,
        },
    },
    OrderedSixRayCase {
        name: "ceiling-104-reached",
        origin: [1477.330444336, -3594.213134766, 8.994521141],
        direction: [-0.792175531, -0.565008104, 0.230702817],
        expected_global_label: "flat:40:CEIL3_5",
        expected: OrderedSixRayExpectedTarget::PartialPlane {
            subsector: 104,
            kind: OrderedPlaneKind::Ceiling,
        },
    },
    OrderedSixRayCase {
        name: "wall-247-west",
        origin: [2115.047851562, -3569.925048828, 8.994521141],
        direction: [0.928815067, -0.358562857, 0.093463443],
        expected_global_label: "wall:247:BROWN96",
        expected: OrderedSixRayExpectedTarget::RejectedWallSegs {
            source_linedef: 247,
            source_segs: WALL_247_SEGS,
        },
    },
    OrderedSixRayCase {
        name: "ceiling-149-rejected",
        origin: [2139.683349609, -3196.036376953, 8.994521141],
        direction: [0.180356100, 0.780082107, 0.599119186],
        expected_global_label: "flat:7:CEIL3_5",
        expected: OrderedSixRayExpectedTarget::RejectedPlane {
            subsector: 149,
            kind: OrderedPlaneKind::Ceiling,
        },
    },
    OrderedSixRayCase {
        name: "ceiling-104-rejected",
        origin: [2902.150878906, -3206.857421875, 8.994521141],
        direction: [-0.952072978, -0.304107845, 0.032795019],
        expected_global_label: "flat:40:CEIL3_5",
        expected: OrderedSixRayExpectedTarget::RejectedPlane {
            subsector: 104,
            kind: OrderedPlaneKind::Ceiling,
        },
    },
];

pub(super) const fn ordered_six_ray_cases() -> &'static [OrderedSixRayCase; 6] {
    &ORDERED_SIX_RAY_CASES
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CausalEvidenceLevel {
    ExactGeometryControl,
    ClassicProtocolReplay,
    OrderedOccurrenceReplay,
}

impl CausalEvidenceLevel {
    const fn label(self) -> &'static str {
        match self {
            Self::ExactGeometryControl => "exact-geometry-control",
            Self::ClassicProtocolReplay => "classic-protocol-replay",
            Self::OrderedOccurrenceReplay => "ordered-occurrence-replay",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OrderedCausalEvent {
    ordinal: usize,
    stage: &'static str,
    evidence: CausalEvidenceLevel,
    detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PairedCeilingSummary {
    name: &'static str,
    reached: bool,
    associations: usize,
    destinations: usize,
    first_elision_node: Option<u16>,
    covering_provenance: String,
    paired_sky_causal_source_segs: BTreeSet<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FocusedCoveringProvenance {
    detail: String,
    resolved: bool,
    paired_sky_source_segs: BTreeSet<u32>,
    causal_source_segs: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExactCounterfactualSummary {
    detail: String,
    tested_source_segs: usize,
    target_reached_source_segs: Vec<u32>,
}

const POSITIVE_WALL_CONTROL_ORIGIN: [f64; 3] = [-29.114_915_848, -3_236.915_527_344, 140.0];
const POSITIVE_WALL_CONTROL_DIRECTION: [f64; 3] = [-0.494_670_182, -0.868_812_501, -0.021_598_173];
const POSITIVE_WALL_CONTROL_SEG: u32 = 270;
const POSITIVE_WALL_CONTROL_LINEDEF: u32 = 135;
const POSITIVE_WALL_CONTROL_LABEL: &str = "wall:135:SUPPORT2";

pub(super) const fn positive_wall_support_control() -> ([f64; 3], [f64; 3], u32, &'static str) {
    (
        POSITIVE_WALL_CONTROL_ORIGIN,
        POSITIVE_WALL_CONTROL_DIRECTION,
        POSITIVE_WALL_CONTROL_LINEDEF,
        POSITIVE_WALL_CONTROL_LABEL,
    )
}

impl OrderedCausalEvent {
    fn format(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.ordinal,
            self.stage,
            self.evidence.label(),
            self.detail,
        )
    }
}

/// Starts the source-ordered non-presentation study with an immutable six-case
/// contract. Covering provenance is deliberately pending until Slice 1/2; the
/// report does not pretend final dispositions are already causal explanations.
pub(crate) fn report_ordered_non_presentation_causality(scene: &SceneInput) -> PlatformResult<()> {
    let map = &scene.door_geometry_source.map;
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let spatial_shadow = SpatialRayShadow::build(scene)?;
    let mut matrix_fingerprint = fnv_offset();
    let mut reports = Vec::new();
    let mut resolved_covering_provenance = 0usize;
    let mut absent_targets_reached_without_solid_pruning = 0usize;
    let mut absent_cases_with_paired_sky_causal_source = 0usize;
    let mut exact_counterfactuals_tested = 0usize;
    let mut exact_counterfactuals_reopening_target = 0usize;
    let mut absent_cases_with_individually_necessary_event = 0usize;
    let mut paired_ceiling = Vec::new();

    for case in ordered_six_ray_cases() {
        let viewer = [case.origin[0].round() as i16, case.origin[1].round() as i16];
        let heading = case.direction[1].atan2(case.direction[0]);
        let eye_height = case.origin[2].round() as i16;
        let target_subsectors = target_subsectors(map, case.expected)?;
        let spatial_hit = spatial_shadow
            .query_source_ray(
                DoomComparativeEmbedding::CurrentReflected,
                case.origin,
                case.direction,
            )?
            .ok_or_else(|| {
                io::Error::other(format!(
                    "causality case {} expected an exact prepared-triangle hit",
                    case.name,
                ))
            })?;
        if spatial_hit.source_label != case.expected_global_label {
            return Err(io::Error::other(format!(
                "causality case {} expected exact source {} but hit {}",
                case.name, case.expected_global_label, spatial_hit.source_label,
            ))
            .into());
        }

        let classic = observe_doom_seg_classic_bsp(map, viewer, heading, &target_subsectors)?;
        let plane_marks = observe_doom_seg_plane_marks(map, eye_height)?
            .into_iter()
            .map(|mark| (mark.source_seg.record_index, mark))
            .collect::<BTreeMap<_, _>>();
        let no_solid_pruning = observe_doom_classic_bsp_without_solid_range_pruning(
            map,
            viewer,
            heading,
            &target_subsectors,
        )?;
        let prepared = prepare_ordered_occurrence_submission(
            map,
            viewer,
            heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        prepared.verify_conservation().map_err(io::Error::other)?;

        // A second uninstrumented call is an explicit Slice 0 observation-only
        // guard. The causal report must not perturb the result it describes.
        let replay = prepare_ordered_occurrence_submission(
            map,
            viewer,
            heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        if prepared != replay {
            return Err(io::Error::other(format!(
                "causality observation changed ordered result for {}",
                case.name,
            ))
            .into());
        }

        let reached = target_subsectors
            .intersection(&classic.visited_subsectors)
            .copied()
            .collect::<BTreeSet<_>>();
        let counterfactual_reached = target_subsectors
            .intersection(&no_solid_pruning.visited_subsectors)
            .copied()
            .collect::<BTreeSet<_>>();
        let mut events = vec![OrderedCausalEvent {
            ordinal: 0,
            stage: "exact-target",
            evidence: CausalEvidenceLevel::ExactGeometryControl,
            detail: format!(
                "source={}:member={}:distance={:.3}",
                spatial_hit.source_label, spatial_hit.member_identity, spatial_hit.distance,
            ),
        }];
        events.push(OrderedCausalEvent {
            ordinal: 1,
            stage: "classic-traversal",
            evidence: CausalEvidenceLevel::ClassicProtocolReplay,
            detail: format!(
                "targets={target_subsectors:?}:reached={reached:?}:elisions=[{}]",
                classic.watched_subsector_elisions.join("|")
            ),
        });

        let covering = focused_covering_provenance(&classic, reached.is_empty(), &plane_marks);
        let covering_provenance = covering.detail.clone();
        resolved_covering_provenance += usize::from(covering.resolved);
        absent_cases_with_paired_sky_causal_source +=
            usize::from(!covering.paired_sky_source_segs.is_empty());
        if reached.is_empty() {
            events.push(OrderedCausalEvent {
                ordinal: events.len(),
                stage: "covering-provenance",
                evidence: CausalEvidenceLevel::ClassicProtocolReplay,
                detail: covering_provenance.clone(),
            });
            absent_targets_reached_without_solid_pruning +=
                usize::from(counterfactual_reached == target_subsectors);

            let counterfactuals = exact_covering_event_counterfactuals(
                map,
                viewer,
                heading,
                &target_subsectors,
                &classic,
                &covering.causal_source_segs,
            )?;
            exact_counterfactuals_tested += counterfactuals.tested_source_segs;
            exact_counterfactuals_reopening_target +=
                counterfactuals.target_reached_source_segs.len();
            absent_cases_with_individually_necessary_event +=
                usize::from(!counterfactuals.target_reached_source_segs.is_empty());
            events.push(OrderedCausalEvent {
                ordinal: events.len(),
                stage: "exact-covering-event-counterfactuals",
                evidence: CausalEvidenceLevel::ClassicProtocolReplay,
                detail: counterfactuals.detail,
            });
        }
        events.push(OrderedCausalEvent {
            ordinal: events.len(),
            stage: "broad-counterfactual-control",
            evidence: CausalEvidenceLevel::ClassicProtocolReplay,
            detail: format!(
                "solid-range-bsp-pruning=disabled:reached={counterfactual_reached:?}:interpretation=corroborating-class-level-control-not-single-event-proof"
            ),
        });

        let (target, final_outcome, first_decisive, ordered_detail) =
            target_outcome(case.expected, &prepared, &classic, &reached)?;
        events.push(OrderedCausalEvent {
            ordinal: events.len(),
            stage: "ordered-outcome",
            evidence: CausalEvidenceLevel::OrderedOccurrenceReplay,
            detail: ordered_detail,
        });

        if matches!(
            case.expected,
            OrderedSixRayExpectedTarget::RejectedPlane {
                subsector: 104,
                kind: OrderedPlaneKind::Ceiling,
            } | OrderedSixRayExpectedTarget::PartialPlane {
                subsector: 104,
                kind: OrderedPlaneKind::Ceiling,
            }
        ) {
            let associations = prepared
                .planes
                .associations
                .iter()
                .filter(|association| {
                    association.source_subsector == 104
                        && association.kind == OrderedPlaneKind::Ceiling
                })
                .count();
            let destinations = prepared
                .planes
                .plane_destinations
                .iter()
                .filter(|destination| {
                    destination.source_subsector == 104
                        && destination.kind == OrderedPlaneKind::Ceiling
                })
                .count();
            paired_ceiling.push(PairedCeilingSummary {
                name: case.name,
                reached: reached.contains(&104),
                associations,
                destinations,
                first_elision_node: classic
                    .watched_elision_provenance
                    .first()
                    .map(|elision| elision.node),
                covering_provenance: covering_provenance.clone(),
                paired_sky_causal_source_segs: covering.paired_sky_source_segs,
            });
        }
        events.push(OrderedCausalEvent {
            ordinal: events.len(),
            stage: "causal-boundary",
            evidence: CausalEvidenceLevel::OrderedOccurrenceReplay,
            detail: format!(
                "first-decisive={first_decisive}:covering-provenance={covering_provenance}"
            ),
        });

        hash_text(&mut matrix_fingerprint, case.name);
        hash_text(&mut matrix_fingerprint, &target);
        hash_text(&mut matrix_fingerprint, final_outcome);
        for event in &events {
            hash_text(&mut matrix_fingerprint, &event.format());
        }
        reports.push(format!(
            "case={}:target={target}:outcome={final_outcome}:first-decisive={first_decisive}:covering-provenance={covering_provenance}:events=[{}]",
            case.name,
            events
                .iter()
                .map(OrderedCausalEvent::format)
                .collect::<Vec<_>>()
                .join(" -> "),
        ));
    }

    let retained_ceiling = paired_ceiling
        .iter()
        .find(|summary| summary.name == "ceiling-104-reached")
        .ok_or_else(|| io::Error::other("paired ceiling retained control is missing"))?;
    let rejected_ceiling = paired_ceiling
        .iter()
        .find(|summary| summary.name == "ceiling-104-rejected")
        .ok_or_else(|| io::Error::other("paired ceiling rejected control is missing"))?;
    if !retained_ceiling.reached
        || retained_ceiling.associations == 0
        || retained_ceiling.destinations == 0
        || rejected_ceiling.reached
        || rejected_ceiling.associations != 0
        || rejected_ceiling.destinations != 0
        || rejected_ceiling.first_elision_node.is_none()
    {
        return Err(io::Error::other(format!(
            "paired ceiling causal contract changed: retained={retained_ceiling:?} rejected={rejected_ceiling:?}"
        ))
        .into());
    }

    let positive_wall = positive_wall_control(scene, &spatial_shadow, &cutout_materials)?;
    let source_covered_walkabout = source_covered_walkabout_controls(scene)?;
    println!(
        "E1M1 ordered non-presentation causality Slice 0-4: cases={}; absent-covering-provenance-resolved={resolved_covering_provenance}/5; absent-targets-reached-without-solid-pruning={absent_targets_reached_without_solid_pruning}/5; exact-covering-event-counterfactuals-tested={exact_counterfactuals_tested}; exact-counterfactuals-reopening-target={exact_counterfactuals_reopening_target}/{exact_counterfactuals_tested}; absent-cases-with-individually-necessary-covering-event={absent_cases_with_individually_necessary_event}/5; absent-cases-with-paired-sky-causal-source={absent_cases_with_paired_sky_causal_source}/5; positive-wall-control=[{positive_wall}]; case-inventory-fingerprint={:016x}; matrix-fingerprint={matrix_fingerprint:016x}; replay-identical=true; conservation=balanced; submission-changes=none; evidence-levels=[exact-geometry-control,classic-protocol-replay,ordered-occurrence-replay]; results=[{}]",
        ordered_six_ray_cases().len(),
        ordered_six_ray_case_inventory_fingerprint(),
        reports.join(" | "),
    );
    println!(
        "E1M1 subsector 104 ceiling causal comparison: retained=[reached:{},associations:{},destinations:{}]; rejected=[reached:{},associations:{},destinations:{},first-elision-node:{:?},covering-provenance:{},paired-sky-causal-source-segs:{:?}]; first-material-divergence=rejected-view-solid-range-prunes-target-child-before-plane-eligibility; rejected-vertical-clip-stage=not-entered; sky-causality=not-in-covering-chain; submission-changes=none",
        retained_ceiling.reached,
        retained_ceiling.associations,
        retained_ceiling.destinations,
        rejected_ceiling.reached,
        rejected_ceiling.associations,
        rejected_ceiling.destinations,
        rejected_ceiling.first_elision_node,
        rejected_ceiling.covering_provenance,
        rejected_ceiling.paired_sky_causal_source_segs,
    );
    println!("{source_covered_walkabout}");
    Ok(())
}

fn source_covered_walkabout_controls(scene: &SceneInput) -> PlatformResult<String> {
    let mut passed = 0usize;
    let mut reports = Vec::new();
    for case in ordered_six_ray_cases() {
        let prepared = crate::render_strategies::source_covered_global_shell::prepare(
            scene,
            &scene.door_geometry_source.map,
            [case.origin[0].round() as i16, case.origin[1].round() as i16],
            case.direction[1].atan2(case.direction[0]),
        )?;
        prepared
            .observation
            .verify_conservation()
            .map_err(io::Error::other)?;
        let retained = source_covered_target_draw_count(&prepared, case.expected);
        let expected_retained = matches!(
            case.expected,
            OrderedSixRayExpectedTarget::PartialPlane { .. }
        );
        let case_passed = (retained > 0) == expected_retained;
        passed += usize::from(case_passed);
        reports.push(format!(
            "case={}:expected={}:retained-draws={retained}:visited-subsectors={}:result={}",
            case.name,
            if expected_retained {
                "retained"
            } else {
                "absent"
            },
            prepared.observation.visited_subsectors.len(),
            if case_passed { "pass" } else { "fail" },
        ));
    }

    let positive = crate::render_strategies::source_covered_global_shell::prepare(
        scene,
        &scene.door_geometry_source.map,
        [
            POSITIVE_WALL_CONTROL_ORIGIN[0].round() as i16,
            POSITIVE_WALL_CONTROL_ORIGIN[1].round() as i16,
        ],
        POSITIVE_WALL_CONTROL_DIRECTION[1].atan2(POSITIVE_WALL_CONTROL_DIRECTION[0]),
    )?;
    positive
        .observation
        .verify_conservation()
        .map_err(io::Error::other)?;
    let positive_retained = positive
        .opaque_draws
        .iter()
        .chain(&positive.cutout_draws)
        .filter(|draw| {
            matches!(
                draw.source,
                StaticDrawSource::Wall { source_linedef, .. }
                    if source_linedef.record_index == POSITIVE_WALL_CONTROL_LINEDEF
            )
        })
        .count();
    let positive_passed = positive_retained > 0;
    passed += usize::from(positive_passed);
    reports.push(format!(
        "case=positive-wall-135:expected=retained:retained-draws={positive_retained}:visited-subsectors={}:result={}",
        positive.observation.visited_subsectors.len(),
        if positive_passed { "pass" } else { "fail" },
    ));

    const CONTROL_COUNT: usize = 7;
    if passed != CONTROL_COUNT {
        return Err(io::Error::other(format!(
            "source-covered walkabout controls failed: passed={passed}/{CONTROL_COUNT}; results=[{}]",
            reports.join(" | "),
        ))
        .into());
    }
    Ok(format!(
        "E1M1 source-covered global-shell walkabout controls: passed={passed}/{CONTROL_COUNT}; whole-source-owner-geometry=true; unresolved-policy=fail-open; renderer-vocabulary=unchanged; results=[{}]",
        reports.join(" | "),
    ))
}

fn source_covered_target_draw_count(
    prepared: &crate::render_strategies::source_covered_global_shell::SourceCoveredGlobalShellPreparation,
    target: OrderedSixRayExpectedTarget,
) -> usize {
    prepared
        .opaque_draws
        .iter()
        .chain(&prepared.cutout_draws)
        .filter(|draw| match (draw.source, target) {
            (
                StaticDrawSource::Wall { source_linedef, .. },
                OrderedSixRayExpectedTarget::RejectedWallSegs {
                    source_linedef: expected,
                    ..
                },
            ) => source_linedef.record_index == expected,
            (
                StaticDrawSource::Flat {
                    source_subsector,
                    plane,
                    ..
                },
                OrderedSixRayExpectedTarget::RejectedPlane { subsector, kind }
                | OrderedSixRayExpectedTarget::PartialPlane { subsector, kind },
            ) => {
                source_subsector.record_index == subsector
                    && matches!(
                        (plane, kind),
                        (DoomSurfacePlane::Floor, OrderedPlaneKind::Floor)
                            | (DoomSurfacePlane::Ceiling, OrderedPlaneKind::Ceiling)
                    )
            }
            _ => false,
        })
        .count()
}

fn focused_covering_provenance(
    classic: &DoomSegClassicBspObservation,
    target_not_reached: bool,
    plane_marks: &BTreeMap<u32, DoomSegPlaneMarkObservation>,
) -> FocusedCoveringProvenance {
    if !target_not_reached {
        return FocusedCoveringProvenance {
            detail: "not-applicable-target-reached".to_owned(),
            resolved: false,
            paired_sky_source_segs: BTreeSet::new(),
            causal_source_segs: Vec::new(),
        };
    }
    let relevant = classic
        .watched_elision_provenance
        .iter()
        .filter(|elision| elision.reason == "solid-range")
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        return FocusedCoveringProvenance {
            detail: "unresolved-no-structured-solid-elision".to_owned(),
            resolved: false,
            paired_sky_source_segs: BTreeSet::new(),
            causal_source_segs: Vec::new(),
        };
    }
    let mut rows = Vec::new();
    let mut resolved = true;
    let mut all_paired_sky_causal_source_segs = BTreeSet::new();
    let mut all_causal_source_segs = Vec::new();
    for elision in relevant {
        let Some(target_interval) = elision.projected_interval else {
            resolved = false;
            rows.push(format!(
                "node{}:targets{:?}:unresolved-no-target-interval",
                elision.node, elision.subsectors,
            ));
            continue;
        };
        let mut target_coverage = vec![false; target_interval[1] - target_interval[0] + 1];
        let mut source_events = Vec::new();
        let mut causal_source_segs = BTreeSet::new();
        let mut causal_source_linedefs = BTreeSet::new();
        for event in &classic.solid_range_events {
            let first = event.input_interval[0].max(target_interval[0]);
            let last = event.input_interval[1].min(target_interval[1]);
            if first > last {
                continue;
            }
            let newly_covered = (first..=last)
                .filter(|column| !target_coverage[column - target_interval[0]])
                .collect::<Vec<_>>();
            if newly_covered.is_empty() {
                continue;
            }
            for column in &newly_covered {
                target_coverage[column - target_interval[0]] = true;
            }
            causal_source_segs.insert(event.source_seg);
            if !all_causal_source_segs.contains(&event.source_seg) {
                all_causal_source_segs.push(event.source_seg);
            }
            causal_source_linedefs.insert(event.source_linedef);
            source_events.push(format!(
                "event{}:seg{}:line{}:input{:?}:new{:?}",
                event.event_ordinal,
                event.source_seg,
                event.source_linedef,
                event.input_interval,
                inclusive_column_runs(&newly_covered),
            ));
        }
        let target_fully_covered = target_coverage.iter().all(|covered| *covered);
        if !target_fully_covered || causal_source_segs.is_empty() {
            resolved = false;
        }
        let paired_sky_causal_source_segs = causal_source_segs
            .iter()
            .filter(|source_seg| {
                plane_marks
                    .get(source_seg)
                    .is_some_and(|mark| mark.paired_sky_ceiling_adjustment)
            })
            .copied()
            .collect::<BTreeSet<_>>();
        all_paired_sky_causal_source_segs.extend(&paired_sky_causal_source_segs);
        rows.push(format!(
            "node{}:targets{:?}:target-interval{:?}:covering-range{:?}:causal-source-segs{:?}:causal-source-lines{:?}:paired-sky-causal-source-segs{:?}:target-fully-covered{}:source-events[{}]",
            elision.node,
            elision.subsectors,
            elision.projected_interval,
            elision.covering_range,
            causal_source_segs,
            causal_source_linedefs,
            paired_sky_causal_source_segs,
            target_fully_covered,
            source_events.join(","),
        ));
    }
    FocusedCoveringProvenance {
        detail: rows.join("+"),
        resolved,
        paired_sky_source_segs: all_paired_sky_causal_source_segs,
        causal_source_segs: all_causal_source_segs,
    }
}

fn exact_covering_event_counterfactuals(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    target_subsectors: &BTreeSet<u16>,
    normal: &DoomSegClassicBspObservation,
    causal_source_segs: &[u32],
) -> PlatformResult<ExactCounterfactualSummary> {
    let mut rows = Vec::new();
    let mut target_reached_source_segs = Vec::new();
    for source_seg in causal_source_segs {
        let shadow = observe_doom_classic_bsp_suppressing_solid_range_source_seg(
            map,
            viewer,
            heading,
            target_subsectors,
            *source_seg,
        )?;
        let suppressed = &shadow.suppressed_solid_range_mutations;
        if suppressed.len() != 1 || suppressed[0].source_seg != *source_seg {
            return Err(io::Error::other(format!(
                "exact counterfactual expected one suppressed mutation for SEG {source_seg}, observed {suppressed:?}"
            ))
            .into());
        }
        let reached = target_subsectors
            .intersection(&shadow.visited_subsectors)
            .copied()
            .collect::<BTreeSet<_>>();
        if !reached.is_empty() {
            target_reached_source_segs.push(*source_seg);
        }
        let newly_visited_subsectors = shadow
            .visited_subsectors
            .difference(&normal.visited_subsectors)
            .count();
        let no_longer_visited_subsectors = normal
            .visited_subsectors
            .difference(&shadow.visited_subsectors)
            .count();
        let newly_admitted_segs = shadow
            .admitted_seg_records
            .difference(&normal.admitted_seg_records)
            .count();
        let no_longer_admitted_segs = normal
            .admitted_seg_records
            .difference(&shadow.admitted_seg_records)
            .count();
        let target_reach = if reached.is_empty() {
            "none"
        } else if reached == *target_subsectors {
            "all"
        } else {
            "partial"
        };
        rows.push(format!(
            "seg{}:line{}:interval{:?}:target-reach={target_reach}:reached={reached:?}:cascade=[new-subsectors:{newly_visited_subsectors},lost-subsectors:{no_longer_visited_subsectors},new-admitted-segs:{newly_admitted_segs},lost-admitted-segs:{no_longer_admitted_segs},far-pruned:{}->{}]",
            suppressed[0].source_seg,
            suppressed[0].source_linedef,
            suppressed[0].input_interval,
            normal.far_children_pruned,
            shadow.far_children_pruned,
        ));
    }
    Ok(ExactCounterfactualSummary {
        detail: format!(
            "tested={}:target-reached-source-segs={target_reached_source_segs:?}:rows=[{}]",
            causal_source_segs.len(),
            rows.join("|"),
        ),
        tested_source_segs: causal_source_segs.len(),
        target_reached_source_segs,
    })
}

fn positive_wall_control(
    scene: &SceneInput,
    spatial_shadow: &SpatialRayShadow,
    cutout_materials: &BTreeMap<String, MaterialHandle>,
) -> PlatformResult<String> {
    let map = &scene.door_geometry_source.map;
    let hit = spatial_shadow
        .query_source_ray(
            DoomComparativeEmbedding::CurrentReflected,
            POSITIVE_WALL_CONTROL_ORIGIN,
            POSITIVE_WALL_CONTROL_DIRECTION,
        )?
        .ok_or_else(|| io::Error::other("positive wall control expected an exact geometry hit"))?;
    if hit.source_label != POSITIVE_WALL_CONTROL_LABEL {
        return Err(io::Error::other(format!(
            "positive wall control expected {POSITIVE_WALL_CONTROL_LABEL}, hit {}",
            hit.source_label,
        ))
        .into());
    }
    let target_subsectors = target_subsectors_for_wall_segs(map, &[POSITIVE_WALL_CONTROL_SEG])?;
    let viewer = [
        POSITIVE_WALL_CONTROL_ORIGIN[0].round() as i16,
        POSITIVE_WALL_CONTROL_ORIGIN[1].round() as i16,
    ];
    let heading = POSITIVE_WALL_CONTROL_DIRECTION[1].atan2(POSITIVE_WALL_CONTROL_DIRECTION[0]);
    let classic = observe_doom_classic_bsp(map, viewer, heading, &target_subsectors)?;
    let range_event = classic
        .solid_range_events
        .iter()
        .find(|event| event.source_seg == POSITIVE_WALL_CONTROL_SEG)
        .ok_or_else(|| io::Error::other("positive wall SEG has no admitted solid-range event"))?;
    let prepared = prepare_ordered_occurrence_submission(
        map,
        viewer,
        heading,
        POSITIVE_WALL_CONTROL_ORIGIN[2].round() as i16,
        &scene.door_geometry_source.wall_extents,
        &scene.door_geometry_source.wall_materials,
        cutout_materials,
        &scene.opaque_uploads,
    )
    .map_err(io::Error::other)?;
    prepared.verify_conservation().map_err(io::Error::other)?;
    let declarations = prepared
        .walls
        .prepared_declarations
        .iter()
        .filter(|declaration| declaration.occurrence.source_seg == POSITIVE_WALL_CONTROL_SEG)
        .collect::<Vec<_>>();
    let view_intervals = declarations
        .iter()
        .map(|declaration| declaration.occurrence.view_interval)
        .collect::<Vec<_>>();
    if !target_subsectors.is_subset(&classic.visited_subsectors)
        || !classic
            .admitted_seg_records
            .contains(&POSITIVE_WALL_CONTROL_SEG)
        || range_event.fully_covered_before
        || declarations.is_empty()
    {
        return Err(io::Error::other(format!(
            "positive wall control failed: targets={target_subsectors:?}:visited={:?}:admitted={}:fully-covered-before={}:declarations={}",
            classic.visited_subsectors,
            classic.admitted_seg_records.contains(&POSITIVE_WALL_CONTROL_SEG),
            range_event.fully_covered_before,
            declarations.len(),
        ))
        .into());
    }
    Ok(format!(
        "label={POSITIVE_WALL_CONTROL_LABEL}:seg={POSITIVE_WALL_CONTROL_SEG}:linedef={POSITIVE_WALL_CONTROL_LINEDEF}:targets={target_subsectors:?}:reached=true:admitted=true:projected-interval={:?}:fully-covered-before=false:declarations={}:view-intervals={view_intervals:?}",
        range_event.input_interval,
        declarations.len(),
    ))
}

fn inclusive_column_runs(columns: &[usize]) -> Vec<[usize; 2]> {
    let mut runs = Vec::<[usize; 2]>::new();
    for column in columns {
        match runs.last_mut() {
            Some([_, last]) if last.saturating_add(1) == *column => *last = *column,
            _ => runs.push([*column, *column]),
        }
    }
    runs
}

fn target_outcome(
    expected: OrderedSixRayExpectedTarget,
    prepared: &OrderedPreparedSubmissionObservation,
    classic: &DoomSegClassicBspObservation,
    reached: &BTreeSet<u16>,
) -> PlatformResult<(String, &'static str, &'static str, String)> {
    match expected {
        OrderedSixRayExpectedTarget::RejectedWallSegs {
            source_linedef,
            source_segs,
        } => {
            let dispositions = prepared
                .source
                .dispositions
                .iter()
                .filter(|disposition| source_segs.contains(&disposition.source_seg))
                .collect::<Vec<_>>();
            let declarations = prepared
                .walls
                .prepared_declarations
                .iter()
                .filter(|declaration| source_segs.contains(&declaration.occurrence.source_seg))
                .count();
            if dispositions.len() != source_segs.len()
                || declarations != 0
                || dispositions.iter().any(|disposition| {
                    disposition.kind != OrderedSourceDispositionKind::TerminalRejected
                })
            {
                return Err(io::Error::other(format!(
                    "wall causality contract changed for linedef {source_linedef}: segs={source_segs:?} dispositions={} declarations={declarations}",
                    dispositions.len(),
                ))
                .into());
            }
            let admitted = source_segs
                .iter()
                .filter(|source_seg| classic.admitted_seg_records.contains(source_seg))
                .copied()
                .collect::<Vec<_>>();
            let first_decisive = if reached.is_empty() {
                "classic-target-subsector-not-reached"
            } else if admitted.is_empty() {
                "classic-target-segs-not-admitted"
            } else {
                "ordered-target-segs-terminally-covered"
            };
            let reasons = dispositions
                .iter()
                .map(|disposition| format!("{}:{}", disposition.source_seg, disposition.reason))
                .collect::<Vec<_>>()
                .join(",");
            Ok((
                format!("wall:linedef={source_linedef}:segs={source_segs:?}"),
                "absent",
                first_decisive,
                format!(
                    "terminal-dispositions={}:declarations=0:classic-admitted-target-segs={admitted:?}:reasons=[{reasons}]",
                    dispositions.len(),
                ),
            ))
        }
        OrderedSixRayExpectedTarget::RejectedPlane { subsector, kind } => {
            let associations = prepared
                .planes
                .associations
                .iter()
                .filter(|association| {
                    association.source_subsector == subsector && association.kind == kind
                })
                .count();
            let destinations = prepared
                .planes
                .plane_destinations
                .iter()
                .filter(|destination| {
                    destination.source_subsector == subsector && destination.kind == kind
                })
                .count();
            if associations != 0 || destinations != 0 {
                return Err(io::Error::other(format!(
                    "rejected plane causality contract changed for subsector {subsector} {kind:?}: associations={associations} destinations={destinations}",
                ))
                .into());
            }
            Ok((
                format!("plane:subsector={subsector}:kind={kind:?}"),
                "absent",
                if reached.is_empty() {
                    "classic-target-subsector-not-reached"
                } else {
                    "ordered-target-plane-association-absent"
                },
                "associations=0:destinations=0:declarations=0".to_owned(),
            ))
        }
        OrderedSixRayExpectedTarget::PartialPlane { subsector, kind } => {
            let associations = prepared
                .planes
                .associations
                .iter()
                .filter(|association| {
                    association.source_subsector == subsector && association.kind == kind
                })
                .count();
            let destinations = prepared
                .planes
                .plane_destinations
                .iter()
                .filter(|destination| {
                    destination.source_subsector == subsector && destination.kind == kind
                })
                .count();
            if associations == 0 || destinations == 0 {
                return Err(io::Error::other(format!(
                    "partial plane causality control changed for subsector {subsector} {kind:?}: associations={associations} destinations={destinations}",
                ))
                .into());
            }
            Ok((
                format!("plane:subsector={subsector}:kind={kind:?}"),
                "partial",
                "ordered-target-plane-occurrence-produced",
                format!("associations={associations}:destinations={destinations}"),
            ))
        }
    }
}

fn target_subsectors(
    map: &DoomMapCore,
    expected: OrderedSixRayExpectedTarget,
) -> PlatformResult<BTreeSet<u16>> {
    match expected {
        OrderedSixRayExpectedTarget::RejectedWallSegs { source_segs, .. } => {
            target_subsectors_for_wall_segs(map, source_segs)
        }
        OrderedSixRayExpectedTarget::RejectedPlane { subsector, .. }
        | OrderedSixRayExpectedTarget::PartialPlane { subsector, .. } => {
            Ok(BTreeSet::from([u16::try_from(subsector).map_err(
                |_| io::Error::other("plane target subsector exceeds the Classic BSP domain"),
            )?]))
        }
    }
}

fn target_subsectors_for_wall_segs(
    map: &DoomMapCore,
    source_segs: &[u32],
) -> PlatformResult<BTreeSet<u16>> {
    let mut result = BTreeSet::new();
    for (index, subsector) in map.subsectors.iter().enumerate() {
        let first = usize::from(subsector.first_seg);
        let end = first + usize::from(subsector.seg_count);
        if map.segs[first..end]
            .iter()
            .any(|seg| source_segs.contains(&seg.source.record_index))
        {
            result.insert(
                u16::try_from(index).map_err(|_| {
                    io::Error::other("subsector index exceeds the Classic BSP domain")
                })?,
            );
        }
    }
    if result.is_empty() {
        return Err(io::Error::other(format!(
            "wall target SEGs {source_segs:?} have no owning subsectors"
        ))
        .into());
    }
    Ok(result)
}

fn ordered_six_ray_case_inventory_fingerprint() -> u64 {
    let mut fingerprint = fnv_offset();
    for case in ordered_six_ray_cases() {
        hash_text(&mut fingerprint, case.name);
        hash_text(&mut fingerprint, case.expected_global_label);
        for value in case.origin.into_iter().chain(case.direction) {
            hash_u64(&mut fingerprint, value.to_bits());
        }
        hash_text(&mut fingerprint, &format!("{:?}", case.expected));
    }
    fingerprint
}

const fn fnv_offset() -> u64 {
    0xcbf2_9ce4_8422_2325
}

fn hash_text(fingerprint: &mut u64, value: &str) {
    for byte in value.bytes() {
        *fingerprint ^= u64::from(byte);
        *fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn hash_u64(fingerprint: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *fingerprint ^= u64::from(byte);
        *fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_ray_causal_contract_has_unique_names_and_one_positive_control() {
        let cases = ordered_six_ray_cases();
        assert_eq!(cases.len(), 6);
        assert_eq!(
            cases
                .iter()
                .map(|case| case.name)
                .collect::<BTreeSet<_>>()
                .len(),
            cases.len(),
        );
        assert_eq!(
            cases
                .iter()
                .filter(|case| matches!(
                    case.expected,
                    OrderedSixRayExpectedTarget::PartialPlane { .. }
                ))
                .count(),
            1,
        );
        assert_ne!(ordered_six_ray_case_inventory_fingerprint(), fnv_offset());
    }
}
