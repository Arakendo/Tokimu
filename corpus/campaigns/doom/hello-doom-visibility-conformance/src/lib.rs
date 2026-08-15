//! Small, explicit Doom source fixtures for viewer-relative presentation work.
//!
//! This crate deliberately constructs `DoomMapCore` below byte decoding but
//! calls the same Doom geometry provider used by the E1M1 corpus. It owns no
//! renderer policy, candidate selection, or alternate visibility algorithm.

use std::collections::BTreeSet;

use doom_geometry_provider::{
    lower_doom_seg_textured_wall_triangles, observe_doom_classic_bsp,
    observe_doom_classic_vertical_clip_state, observe_doom_seg_plane_marks,
    project_doom_sector_runtime_heights, resolve_doom_subsector_loops, DoomClassicBspObservation,
    DoomGeometryError, DoomSectorRuntimeHeightSnapshot, DoomSegClassicVerticalClipObservation,
    DoomSegPlaneMarkObservation, DoomSubsectorLoop, DoomTextureExtent,
};
use doom_map_provider::{
    DoomBlockmapObservation, DoomBspChild, DoomLinedef, DoomMapCore, DoomNode, DoomRejectMatrix,
    DoomSector, DoomSeg, DoomSidedef, DoomSourceRecord, DoomSubsector, DoomThing, DoomVertex,
};
use thiserror::Error;

const FIXTURE_LUMP_INDEX: u32 = 0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoomFixtureViewer {
    pub position: [i16; 2],
    pub heading_radians: f64,
}

#[derive(Clone, Debug)]
pub struct DoomVisibilityFixture {
    pub name: String,
    pub map: DoomMapCore,
    pub viewer: DoomFixtureViewer,
    pub watched_subsectors: BTreeSet<u16>,
}

impl DoomVisibilityFixture {
    /// Runs the shared Doom provider control. The fixture adds only explicit
    /// source data and watched identities; it never substitutes an algorithm.
    pub fn observe_classic_bsp(&self) -> Result<DoomClassicBspObservation, DoomGeometryError> {
        observe_doom_classic_bsp(
            &self.map,
            self.viewer.position,
            self.viewer.heading_radians,
            &self.watched_subsectors,
        )
    }

    /// Compact, source-labelled result evidence for the shared classic BSP
    /// control. This is intentionally narrower than the campaign's final
    /// presentation fingerprint: no projected plane instance or runtime
    /// height state exists at this seam yet.
    pub fn classic_bsp_manifest(&self) -> Result<DoomClassicBspManifest, DoomGeometryError> {
        let observation = self.observe_classic_bsp()?;
        let (fingerprint, trace) = compact_evidence_fingerprint(
            observation
                .admitted_seg_order
                .iter()
                .map(|source_seg| format!("seg:{source_seg}")),
            std::iter::empty::<String>(),
            [format!(
                "covered-columns:{}",
                observation.solid_range_covered_columns
            )],
            [
                format!("backface:{}", observation.backface_rejected),
                format!("edge-on:{}", observation.edge_on),
                format!("near-fail-open:{}", observation.near_plane_fail_open),
                format!("outside-fov:{}", observation.outside_fov_rejected),
                format!("pass:{}", observation.pass_admitted),
                format!("solid:{}", observation.solid_admitted),
                format!(
                    "watched-elisions:{:?}",
                    observation.watched_subsector_elisions
                ),
            ],
            [
                format!("fixture:{}", self.name),
                format!(
                    "viewer:{:?}:{:.12}",
                    self.viewer.position, self.viewer.heading_radians
                ),
                format!(
                    "leaves:{}:{:?}",
                    observation.leaves_visited, observation.visited_subsectors
                ),
                format!("source-segs:{}", observation.source_segs_visited),
                "runtime:static-source".to_owned(),
                format!("samples:{:?}", observation.samples),
            ],
        );
        Ok(DoomClassicBspManifest {
            admitted_seg_records: observation.admitted_seg_order,
            leaves_visited: observation.leaves_visited,
            backface_rejected: observation.backface_rejected,
            edge_on: observation.edge_on,
            outside_fov_rejected: observation.outside_fov_rejected,
            near_plane_fail_open: observation.near_plane_fail_open,
            solid_range_covered_columns: observation.solid_range_covered_columns,
            fingerprint,
            trace,
        })
    }

    /// Returns the provider's source-only plane-mark facts. This deliberately
    /// does not claim that a plane has been projected or presented.
    pub fn observe_plane_marks(
        &self,
        source_view_height: i16,
    ) -> Result<Vec<DoomSegPlaneMarkObservation>, DoomGeometryError> {
        observe_doom_seg_plane_marks(&self.map, source_view_height)
    }

    /// Exercises the same source-only wall-tier/vertical-clip observation as
    /// E1M1. Texture extents are deliberately fixture inputs: this seam still
    /// creates no material, raster, mesh resource, or presentation claim.
    pub fn observe_classic_vertical_clips(
        &self,
        source_view_height: i16,
        wall_extents: &[DoomTextureExtent],
    ) -> Result<DoomSegClassicVerticalClipObservation, DoomGeometryError> {
        let triangles = lower_doom_seg_textured_wall_triangles(&self.map, wall_extents)?;
        let plane_marks = self.observe_plane_marks(source_view_height)?;
        let traversal = self.observe_classic_bsp()?;
        Ok(observe_doom_classic_vertical_clip_state(
            &self.map,
            &triangles,
            &plane_marks,
            &traversal,
            self.viewer.position,
            self.viewer.heading_radians,
            f64::from(source_view_height),
        ))
    }

    /// Projects caller-supplied, temporary Doom sector-height facts over this
    /// fixture's immutable decoded source. This provides a shared preparation
    /// seam for dynamic controls without embedding a door or platform state
    /// machine in the fixture builder.
    pub fn with_runtime_height_snapshots(
        &self,
        snapshots: &[DoomSectorRuntimeHeightSnapshot],
    ) -> Result<Self, DoomGeometryError> {
        Ok(Self {
            name: self.name.clone(),
            map: project_doom_sector_runtime_heights(&self.map, snapshots)?,
            viewer: self.viewer,
            watched_subsectors: self.watched_subsectors.clone(),
        })
    }

    /// Resolves actual decoded-style SEG loops; fixture code does not invent
    /// an expected polygon from its vertices.
    pub fn resolve_subsector_loops(&self) -> Result<Vec<DoomSubsectorLoop>, DoomGeometryError> {
        resolve_doom_subsector_loops(&self.map)
    }

    pub fn structural_manifest(&self) -> DoomFixtureManifest {
        let normalized = format!(
            "name={};viewer={:?}:{:.12};watched={:?};vertices={:?};linedefs={:?};sidedefs={:?};sectors={:?};segs={:?};subsectors={:?};nodes={:?}",
            self.name,
            self.viewer.position,
            self.viewer.heading_radians,
            self.watched_subsectors,
            self.map.vertices,
            self.map.linedefs,
            self.map.sidedefs,
            self.map.sectors,
            self.map.segs,
            self.map.subsectors,
            self.map.nodes,
        );
        DoomFixtureManifest {
            name: self.name.clone(),
            vertices: self.map.vertices.len(),
            linedefs: self.map.linedefs.len(),
            sidedefs: self.map.sidedefs.len(),
            sectors: self.map.sectors.len(),
            segs: self.map.segs.len(),
            subsectors: self.map.subsectors.len(),
            nodes: self.map.nodes.len(),
            fingerprint: blake3::hash(normalized.as_bytes()).to_hex().to_string(),
            trace: normalized,
        }
    }
}

/// Produces the campaign's compact evidence format. Each bucket remains
/// explicit because a source identity, a presentation instance, an interval,
/// a classifier reason, and temporary runtime state have different authority.
/// Empty buckets are retained in the trace so an early source-only fixture
/// cannot accidentally claim it observed presentation or dynamic state.
fn compact_evidence_fingerprint(
    source_identities: impl IntoIterator<Item = String>,
    presentation_instances: impl IntoIterator<Item = String>,
    intervals: impl IntoIterator<Item = String>,
    reasons: impl IntoIterator<Item = String>,
    runtime_state: impl IntoIterator<Item = String>,
) -> (String, String) {
    fn normalized(values: impl IntoIterator<Item = String>) -> Vec<String> {
        values
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    let source_identities = normalized(source_identities);
    let presentation_instances = normalized(presentation_instances);
    let intervals = normalized(intervals);
    let reasons = normalized(reasons);
    let runtime_state = normalized(runtime_state);
    let trace = format!(
        "source={source_identities:?};presentation={presentation_instances:?};intervals={intervals:?};reasons={reasons:?};runtime={runtime_state:?}"
    );
    (blake3::hash(trace.as_bytes()).to_hex().to_string(), trace)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomFixtureManifest {
    pub name: String,
    pub vertices: usize,
    pub linedefs: usize,
    pub sidedefs: usize,
    pub sectors: usize,
    pub segs: usize,
    pub subsectors: usize,
    pub nodes: usize,
    pub fingerprint: String,
    pub trace: String,
}

/// Bounded pose/result evidence for a source-only classic BSP observation.
/// It deliberately retains source SEG identity and classification counters,
/// rather than reducing the result to a draw count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomClassicBspManifest {
    pub admitted_seg_records: Vec<u32>,
    pub leaves_visited: usize,
    pub backface_rejected: usize,
    pub edge_on: usize,
    pub outside_fov_rejected: usize,
    pub near_plane_fail_open: usize,
    pub solid_range_covered_columns: usize,
    pub fingerprint: String,
    pub trace: String,
}

/// Explicit BSP source record used by the fixture builder. Keeping the node
/// fields grouped makes fixture topology visible at each call site without
/// smuggling traversal decisions into the builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomFixtureNode {
    pub point: [i16; 2],
    pub delta: [i16; 2],
    pub right_bbox: [i16; 4],
    pub left_bbox: [i16; 4],
    pub right_child: DoomBspChild,
    pub left_child: DoomBspChild,
}

/// Explicit source-record builder. It intentionally offers append-only record
/// construction, not topology inference or visibility conveniences.
#[derive(Clone, Debug)]
pub struct DoomFixtureBuilder {
    name: String,
    viewer: DoomFixtureViewer,
    watched_subsectors: BTreeSet<u16>,
    things: Vec<DoomThing>,
    vertices: Vec<DoomVertex>,
    linedefs: Vec<DoomLinedef>,
    sidedefs: Vec<DoomSidedef>,
    sectors: Vec<DoomSector>,
    segs: Vec<DoomSeg>,
    subsectors: Vec<DoomSubsector>,
    nodes: Vec<DoomNode>,
}

impl DoomFixtureBuilder {
    pub fn new(name: impl Into<String>, viewer: DoomFixtureViewer) -> Self {
        Self {
            name: name.into(),
            viewer,
            watched_subsectors: BTreeSet::new(),
            things: Vec::new(),
            vertices: Vec::new(),
            linedefs: Vec::new(),
            sidedefs: Vec::new(),
            sectors: Vec::new(),
            segs: Vec::new(),
            subsectors: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn watch_subsector(&mut self, index: u16) -> &mut Self {
        self.watched_subsectors.insert(index);
        self
    }

    pub fn thing(&mut self, x: i16, y: i16, angle: u16, kind: u16) -> u16 {
        let index = u16::try_from(self.things.len()).expect("fixture Thing count fits u16");
        self.things.push(DoomThing {
            source: source(index),
            x,
            y,
            angle,
            kind,
            flags: 0,
        });
        index
    }

    pub fn vertex(&mut self, x: i16, y: i16) -> u16 {
        let index = u16::try_from(self.vertices.len()).expect("fixture vertex count fits u16");
        self.vertices.push(DoomVertex {
            source: source(index),
            x,
            y,
        });
        index
    }

    pub fn sector(&mut self, floor_height: i16, ceiling_height: i16) -> u16 {
        let index = u16::try_from(self.sectors.len()).expect("fixture sector count fits u16");
        self.sectors.push(DoomSector {
            source: source(index),
            floor_height,
            ceiling_height,
            floor_texture: "FLOOR".to_owned(),
            ceiling_texture: "CEILING".to_owned(),
            light_level: 160,
            special: 0,
            tag: 0,
        });
        index
    }

    pub fn sidedef(&mut self, sector: u16, middle_texture: impl Into<String>) -> u16 {
        let index = u16::try_from(self.sidedefs.len()).expect("fixture sidedef count fits u16");
        self.sidedefs.push(DoomSidedef {
            source: source(index),
            x_offset: 0,
            y_offset: 0,
            upper_texture: "-".to_owned(),
            lower_texture: "-".to_owned(),
            middle_texture: middle_texture.into(),
            sector,
        });
        index
    }

    pub fn linedef(
        &mut self,
        start_vertex: u16,
        end_vertex: u16,
        right_sidedef: Option<u16>,
        left_sidedef: Option<u16>,
    ) -> u16 {
        let index = u16::try_from(self.linedefs.len()).expect("fixture linedef count fits u16");
        self.linedefs.push(DoomLinedef {
            source: source(index),
            start_vertex,
            end_vertex,
            flags: 0,
            special: 0,
            tag: 0,
            right_sidedef,
            left_sidedef,
        });
        index
    }

    pub fn seg(&mut self, start_vertex: u16, end_vertex: u16, linedef: u16, direction: u16) -> u16 {
        let index = u16::try_from(self.segs.len()).expect("fixture seg count fits u16");
        self.segs.push(DoomSeg {
            source: source(index),
            start_vertex,
            end_vertex,
            angle: 0,
            linedef,
            direction,
            offset: 0,
        });
        index
    }

    pub fn subsector(&mut self, first_seg: u16, seg_count: u16) -> u16 {
        let index = u16::try_from(self.subsectors.len()).expect("fixture subsector count fits u16");
        self.subsectors.push(DoomSubsector {
            source: source(index),
            first_seg,
            seg_count,
        });
        index
    }

    pub fn node(&mut self, record: DoomFixtureNode) -> u16 {
        let index = u16::try_from(self.nodes.len()).expect("fixture node count fits u16");
        self.nodes.push(DoomNode {
            source: source(index),
            x: record.point[0],
            y: record.point[1],
            delta_x: record.delta[0],
            delta_y: record.delta[1],
            right_bbox: record.right_bbox,
            left_bbox: record.left_bbox,
            right_child: record.right_child,
            left_child: record.left_child,
        });
        index
    }

    pub fn build(self) -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
        validate_fixture(&self)?;
        Ok(DoomVisibilityFixture {
            name: self.name,
            map: DoomMapCore {
                map_name: "SYNTHETIC".to_owned(),
                things: self.things,
                vertices: self.vertices,
                linedefs: self.linedefs,
                sidedefs: self.sidedefs,
                sectors: self.sectors,
                segs: self.segs,
                subsectors: self.subsectors,
                nodes: self.nodes,
                reject: DoomRejectMatrix::default(),
                blockmap: empty_blockmap(),
            },
            viewer: self.viewer,
            watched_subsectors: self.watched_subsectors,
        })
    }
}

/// Small source-only control for the paired-sky presentation experiment.
///
/// The near two-sided boundary has unequal `F_SKY1` ceilings; the distant
/// one-sided wall is deliberately retained as a separate candidate. The
/// fixture does not decide final draw order: consumers must make that
/// Doom-local presentation choice explicit and retain it as evidence.
pub fn paired_sky_far_control_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut builder = DoomFixtureBuilder::new(
        "paired-sky-far-control",
        DoomFixtureViewer {
            position: [0, -96],
            heading_radians: std::f64::consts::FRAC_PI_2,
        },
    );
    let lower_sky = builder.sector(0, 96);
    let higher_sky = builder.sector(0, 128);
    let far_sector = builder.sector(0, 128);
    builder.sectors[usize::from(lower_sky)].ceiling_texture = "F_SKY1".to_owned();
    builder.sectors[usize::from(higher_sky)].ceiling_texture = "F_SKY1".to_owned();
    let near_right = builder.sidedef(lower_sky, "-");
    let near_left = builder.sidedef(higher_sky, "-");
    let far_right = builder.sidedef(far_sector, "WALL");
    let near_start = builder.vertex(-48, 0);
    let near_end = builder.vertex(48, 0);
    let far_start = builder.vertex(-24, 64);
    let far_end = builder.vertex(24, 64);
    let near = builder.linedef(near_start, near_end, Some(near_right), Some(near_left));
    let far = builder.linedef(far_start, far_end, Some(far_right), None);
    builder.seg(near_start, near_end, near, 0);
    builder.seg(far_start, far_end, far, 0);
    builder.subsector(0, 1);
    builder.subsector(1, 1);
    builder.node(DoomFixtureNode {
        point: [0, 0],
        delta: [0, 64],
        right_bbox: [64, 64, 24, -24],
        left_bbox: [0, 0, 48, -48],
        right_child: DoomBspChild::Subsector(1),
        left_child: DoomBspChild::Subsector(0),
    });
    builder.watch_subsector(1);
    builder.build()
}

/// Partial-coverage variant of [`paired_sky_far_control_fixture`].
///
/// The paired-sky boundary covers only the middle of a wider far wall. This
/// makes the whole-candidate question falsifiable: the far source contribution
/// must remain available outside the paired-sky columns while the overlapping
/// columns remain governed by the ordered Doom source protocol. The fixture
/// does not prescribe meshes, scissors, or renderer visibility vocabulary.
pub fn partial_paired_sky_far_control_fixture(
) -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut fixture = paired_sky_far_control_fixture()?;
    fixture.name = "partial-paired-sky-far-control".to_owned();
    fixture.map.vertices[0].x = -24;
    fixture.map.vertices[1].x = 24;
    fixture.map.vertices[2].x = -48;
    fixture.map.vertices[3].x = 48;
    Ok(fixture)
}

/// Corpus-only evidence that a single far source contribution can require
/// different dispositions in different viewer-relative columns.
///
/// This is deliberately an expressiveness observation, not an implementation
/// proposal. `requires_source_fragments` means only that a Boolean decision at
/// whole-source-SEG granularity cannot preserve both required regions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialCoverageExpressivenessObservation {
    pub paired_sky_source_seg: u32,
    pub far_wall_source_seg: u32,
    pub paired_sky_columns: usize,
    pub far_wall_columns: usize,
    pub overlapping_columns: usize,
    pub far_only_columns: usize,
    /// Contiguous diagnostic-column intervals where the far contribution is
    /// governed by the nearer paired-sky source interval. These remain
    /// Doom-corpus evidence, not renderer scissors or public pixel spans.
    pub overlapping_runs: Vec<DiagnosticColumnRun>,
    /// Contiguous diagnostic-column intervals where the same far source SEG
    /// remains required.
    pub surviving_runs: Vec<DiagnosticColumnRun>,
    pub requires_source_fragments: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticColumnRun {
    pub first: usize,
    pub last: usize,
}

fn contiguous_column_runs(columns: &BTreeSet<usize>) -> Vec<DiagnosticColumnRun> {
    let mut columns = columns.iter().copied();
    let Some(first) = columns.next() else {
        return Vec::new();
    };
    let mut runs = Vec::new();
    let mut run_first = first;
    let mut run_last = first;
    for column in columns {
        if column == run_last + 1 {
            run_last = column;
        } else {
            runs.push(DiagnosticColumnRun {
                first: run_first,
                last: run_last,
            });
            run_first = column;
            run_last = column;
        }
    }
    runs.push(DiagnosticColumnRun {
        first: run_first,
        last: run_last,
    });
    runs
}

pub fn observe_partial_coverage_expressiveness(
) -> Result<PartialCoverageExpressivenessObservation, DoomGeometryError> {
    let fixture = partial_paired_sky_far_control_fixture()
        .expect("the built-in partial-coverage fixture must remain valid");
    let paired_sky_source_seg = fixture.map.segs[0].source.record_index;
    let far_wall_source_seg = fixture.map.segs[1].source.record_index;
    let vertical = fixture.observe_classic_vertical_clips(
        41,
        &[DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 128,
        }],
    )?;
    let paired_sky_columns = vertical
        .column_traces
        .iter()
        .filter(|trace| {
            trace
                .paired_sky_boundary_source_segs
                .contains(&paired_sky_source_seg)
        })
        .map(|trace| trace.column)
        .collect::<BTreeSet<_>>();
    let far_wall_columns = vertical
        .column_traces
        .iter()
        .filter(|trace| trace.middle_source_segs.contains(&far_wall_source_seg))
        .map(|trace| trace.column)
        .collect::<BTreeSet<_>>();
    let overlapping = paired_sky_columns
        .intersection(&far_wall_columns)
        .copied()
        .collect::<BTreeSet<_>>();
    let surviving = far_wall_columns
        .difference(&paired_sky_columns)
        .copied()
        .collect::<BTreeSet<_>>();
    let overlapping_runs = contiguous_column_runs(&overlapping);
    let surviving_runs = contiguous_column_runs(&surviving);

    Ok(PartialCoverageExpressivenessObservation {
        paired_sky_source_seg,
        far_wall_source_seg,
        paired_sky_columns: paired_sky_columns.len(),
        far_wall_columns: far_wall_columns.len(),
        overlapping_columns: overlapping.len(),
        far_only_columns: surviving.len(),
        overlapping_runs,
        surviving_runs,
        requires_source_fragments: !overlapping.is_empty() && !surviving.is_empty(),
    })
}

/// Negative control for [`paired_sky_far_control_fixture`]. Only the near
/// front ceiling remains sky, so the same source boundary must retain its
/// ordinary upper-wall presentation and must not gain paired-sky depth-only
/// authority.
pub fn one_sky_far_control_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut fixture = paired_sky_far_control_fixture()?;
    fixture.name = "one-sky-far-control".to_owned();
    // Keep the viewer-side ceiling as the higher sky plane and lower the
    // opposite non-sky ceiling. That source relationship produces an
    // ordinary authored upper wall on the viewer-facing sidedef. Merely
    // changing the texture name on the original low-to-high paired-sky
    // boundary would not create such a wall and would be a false negative
    // control rather than evidence about one-sky behavior.
    fixture.map.sectors[0].ceiling_height = 128;
    fixture.map.sectors[1].ceiling_height = 96;
    fixture.map.sectors[1].ceiling_texture = "CEILING".to_owned();
    fixture.map.sidedefs[0].upper_texture = "WALL".to_owned();
    Ok(fixture)
}

/// Source-only control for a nearer *single* sky ceiling plane.
///
/// This is intentionally distinct from [`paired_sky_far_control_fixture`]: the
/// source sky surface belongs to one enclosed subsector, not to a height
/// discontinuity between two `F_SKY1` ceilings. A presentation experiment may
/// use its explicitly named ceiling plane to constrain a far candidate only in
/// that plane's projected interval. It must not infer that every one-sky wall
/// boundary has depth authority; [`one_sky_far_control_fixture`] remains the
/// counterexample for that claim.
pub fn single_sky_plane_far_control_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError>
{
    let mut builder = DoomFixtureBuilder::new(
        "single-sky-plane-far-control",
        DoomFixtureViewer {
            position: [0, -96],
            heading_radians: std::f64::consts::FRAC_PI_2,
        },
    );
    let sky_sector = builder.sector(0, 96);
    builder.sectors[usize::from(sky_sector)].ceiling_texture = "F_SKY1".to_owned();
    let sky_side = builder.sidedef(sky_sector, "-");
    // A closed decoded-style subsector loop is required here because the
    // control's authority is a source ceiling plane, not a linedef band.
    for [x, y] in [[-48, 0], [48, 0], [48, 48], [-48, 48]] {
        builder.vertex(x, y);
    }
    for [start, end] in [[0, 1], [1, 2], [2, 3], [3, 0]] {
        let linedef = builder.linedef(start, end, Some(sky_side), None);
        builder.seg(start, end, linedef, 0);
    }
    builder.subsector(0, 4);
    builder.watch_subsector(0);
    builder.build()
}

/// Two-sided vertical-aperture control shared by structural and presentation
/// evidence. The viewer-side sector spans `0..128`; the opposite sector spans
/// `24..96`, producing authored upper and lower tiers around a real opening.
/// Runtime presentation may color those roles independently, but this fixture
/// owns only the Doom source facts.
pub fn vertical_aperture_control_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut builder = DoomFixtureBuilder::new(
        "vertical-aperture",
        DoomFixtureViewer {
            position: [0, -64],
            heading_radians: std::f64::consts::FRAC_PI_2,
        },
    );
    let front = builder.sector(0, 128);
    let back = builder.sector(24, 96);
    let front_side = builder.sidedef(front, "-");
    let back_side = builder.sidedef(back, "-");
    builder.sidedefs[usize::from(front_side)].upper_texture = "WALL".to_owned();
    builder.sidedefs[usize::from(front_side)].lower_texture = "WALL".to_owned();
    let start = builder.vertex(-48, 0);
    let end = builder.vertex(48, 0);
    let boundary = builder.linedef(start, end, Some(front_side), Some(back_side));
    builder.seg(start, end, boundary, 0);
    // Retain the opposite BSP leaf as a real back-facing control.
    builder.seg(end, start, boundary, 1);
    builder.subsector(0, 1);
    builder.subsector(1, 1);
    builder.node(DoomFixtureNode {
        point: [0, 0],
        delta: [0, 64],
        right_bbox: [0, 64, 48, -48],
        left_bbox: [0, 64, 48, -48],
        right_child: DoomBspChild::Subsector(1),
        left_child: DoomBspChild::Subsector(0),
    });
    builder.build()
}

/// Two disjoint decoded floor regions that share every ordinary plane-key
/// field (height, flat, and light) while retaining different source sectors.
///
/// The fixture controls a presentation bug class rather than authorizing a
/// renderer cache key: a viewer-relative path may group compatible source
/// facts, but it must not let one projected region overwrite the other merely
/// because their floor metadata matches.
pub fn shared_key_disjoint_plane_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut builder = DoomFixtureBuilder::new(
        "shared-key-disjoint-plane",
        DoomFixtureViewer {
            position: [0, -96],
            heading_radians: std::f64::consts::FRAC_PI_2,
        },
    );
    let near_sector = builder.sector(0, 128);
    let far_sector = builder.sector(0, 128);
    for [x, y] in [
        [-16, 0],
        [16, 0],
        [16, 32],
        [-16, 32],
        [-96, 64],
        [96, 64],
        [96, 128],
        [-96, 128],
    ] {
        builder.vertex(x, y);
    }
    let near_side = builder.sidedef(near_sector, "WALL");
    let far_side = builder.sidedef(far_sector, "WALL");
    for [start, end, side] in [
        [0, 1, near_side],
        [1, 2, near_side],
        [2, 3, near_side],
        [3, 0, near_side],
        [4, 5, far_side],
        [5, 6, far_side],
        [6, 7, far_side],
        [7, 4, far_side],
    ] {
        builder.linedef(start, end, Some(side), None);
    }
    for linedef in 0..8 {
        let line = &builder.linedefs[usize::from(linedef)];
        builder.seg(line.start_vertex, line.end_vertex, linedef, 0);
    }
    builder.subsector(0, 4);
    builder.subsector(4, 4);
    builder.node(DoomFixtureNode {
        point: [0, 0],
        delta: [0, 128],
        right_bbox: [64, 128, 96, -96],
        left_bbox: [0, 32, 16, -16],
        right_child: DoomBspChild::Subsector(1),
        left_child: DoomBspChild::Subsector(0),
    });
    builder.build()
}

/// Two source sectors separated by a doorway boundary. Consumers supply a
/// temporary ceiling snapshot for sector 1 to observe closed/open geometry;
/// this fixture owns neither activation nor timing policy.
pub fn dynamic_door_snapshot_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut builder = DoomFixtureBuilder::new(
        "dynamic-door-snapshot",
        DoomFixtureViewer {
            position: [-32, 32],
            heading_radians: 0.0,
        },
    );
    let west_sector = builder.sector(0, 128);
    let east_sector = builder.sector(0, 128);
    let west_side = builder.sidedef(west_sector, "WALL");
    let east_side = builder.sidedef(east_sector, "WALL");
    for [x, y] in [[-64, 0], [0, 0], [0, 64], [-64, 64], [64, 0], [64, 64]] {
        builder.vertex(x, y);
    }
    let west_bottom = builder.linedef(0, 1, Some(west_side), None);
    let shared = builder.linedef(1, 2, Some(east_side), Some(west_side));
    let west_top = builder.linedef(2, 3, Some(west_side), None);
    let west_outer = builder.linedef(3, 0, Some(west_side), None);
    let east_bottom = builder.linedef(1, 4, Some(east_side), None);
    let east_outer = builder.linedef(4, 5, Some(east_side), None);
    let east_top = builder.linedef(5, 2, Some(east_side), None);
    for [start, end, linedef, direction] in [
        [0, 1, west_bottom, 0],
        [1, 2, shared, 0],
        [2, 3, west_top, 0],
        [3, 0, west_outer, 0],
        [1, 4, east_bottom, 0],
        [4, 5, east_outer, 0],
        [5, 2, east_top, 0],
        [2, 1, shared, 1],
    ] {
        builder.seg(start, end, linedef, direction);
    }
    builder.subsector(0, 4);
    builder.subsector(4, 4);
    builder.node(DoomFixtureNode {
        point: [0, 0],
        delta: [0, 64],
        right_bbox: [0, 64, 64, 0],
        left_bbox: [0, 64, 0, -64],
        right_child: DoomBspChild::Subsector(1),
        left_child: DoomBspChild::Subsector(0),
    });
    builder.build()
}

/// A bounded single-sector platform control. Callers supply temporary floor
/// snapshots for its one source sector; the fixture deliberately owns neither
/// platform activation nor time progression.
pub fn moving_platform_snapshot_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut builder = DoomFixtureBuilder::new(
        "moving-platform-snapshot",
        DoomFixtureViewer {
            position: [0, -96],
            heading_radians: std::f64::consts::FRAC_PI_2,
        },
    );
    let sector = builder.sector(0, 128);
    let side = builder.sidedef(sector, "WALL");
    for [x, y] in [[-32, 0], [32, 0], [32, 64], [-32, 64]] {
        builder.vertex(x, y);
    }
    for [start, end] in [[0, 1], [1, 2], [2, 3], [3, 0]] {
        let linedef = builder.linedef(start, end, Some(side), None);
        builder.seg(start, end, linedef, 0);
    }
    builder.subsector(0, 4);
    builder.node(DoomFixtureNode {
        point: [0, 0],
        delta: [0, 64],
        right_bbox: [0, 64, 32, -32],
        left_bbox: [0, 64, 32, -32],
        right_child: DoomBspChild::Subsector(0),
        left_child: DoomBspChild::Subsector(0),
    });
    builder.build()
}

/// A source SEG with one endpoint behind the viewer. The shared Doom
/// projection observation must fail open: this source segment receives no
/// solid-range authority. It is public solely as a corpus presentation control,
/// not as a general renderer projection fixture.
pub fn projection_near_plane_crossing_fixture(
) -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    projection_forward_seg_fixture("projection-near-plane-crossing", [-32, -1], [32, 3])
}

/// A one-unit-wide source SEG entirely in front of the viewer. It is retained
/// as a valid source wall even when its conservative screen interval is small.
pub fn projection_thin_forward_seg_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError>
{
    projection_forward_seg_fixture("projection-thin-forward", [-1, 64], [0, 64])
}

/// A source SEG extremely close to the viewer but still wholly in front. It
/// remains ordinary source geometry rather than being silently classified as a
/// near-plane failure.
pub fn projection_close_forward_seg_fixture() -> Result<DoomVisibilityFixture, DoomFixtureBuildError>
{
    projection_forward_seg_fixture("projection-close-forward", [-32, 1], [32, 1])
}

fn projection_forward_seg_fixture(
    name: &str,
    first_point: [i16; 2],
    second_point: [i16; 2],
) -> Result<DoomVisibilityFixture, DoomFixtureBuildError> {
    let mut builder = DoomFixtureBuilder::new(
        name,
        DoomFixtureViewer {
            position: [0, 0],
            heading_radians: std::f64::consts::FRAC_PI_2,
        },
    );
    let sector = builder.sector(0, 128);
    let side = builder.sidedef(sector, "WALL");
    let first = builder.vertex(first_point[0], first_point[1]);
    let second = builder.vertex(second_point[0], second_point[1]);
    let linedef = builder.linedef(first, second, Some(side), None);
    builder.seg(first, second, linedef, 0);
    builder.subsector(0, 1);
    builder.node(DoomFixtureNode {
        point: [0, 0],
        delta: [0, 64],
        right_bbox: [
            second_point[1],
            first_point[1],
            second_point[0],
            first_point[0],
        ],
        left_bbox: [
            second_point[1],
            first_point[1],
            second_point[0],
            first_point[0],
        ],
        right_child: DoomBspChild::Subsector(0),
        left_child: DoomBspChild::Subsector(0),
    });
    builder.build()
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomFixtureBuildError {
    #[error("fixture has no vertices")]
    MissingVertices,
    #[error("fixture has no sectors")]
    MissingSectors,
    #[error("linedef {linedef} references vertex {vertex}, but only {available} exist")]
    LinedefVertexOutOfBounds {
        linedef: u16,
        vertex: u16,
        available: usize,
    },
    #[error("linedef {linedef} has no sidedef")]
    MissingLinedefSide { linedef: u16 },
    #[error("linedef {linedef} references sidedef {sidedef}, but only {available} exist")]
    LinedefSideOutOfBounds {
        linedef: u16,
        sidedef: u16,
        available: usize,
    },
    #[error("sidedef {sidedef} references sector {sector}, but only {available} exist")]
    SidedefSectorOutOfBounds {
        sidedef: u16,
        sector: u16,
        available: usize,
    },
    #[error("seg {seg} references vertex {vertex}, but only {available} exist")]
    SegVertexOutOfBounds {
        seg: u16,
        vertex: u16,
        available: usize,
    },
    #[error("seg {seg} references linedef {linedef}, but only {available} exist")]
    SegLinedefOutOfBounds {
        seg: u16,
        linedef: u16,
        available: usize,
    },
    #[error("seg {seg} has unsupported direction {direction}")]
    InvalidSegDirection { seg: u16, direction: u16 },
    #[error("subsector {subsector} has no segs")]
    EmptySubsector { subsector: u16 },
    #[error("subsector {subsector} range [{first_seg}, {end}) exceeds {available} segs")]
    SubsectorRangeOutOfBounds {
        subsector: u16,
        first_seg: u16,
        end: usize,
        available: usize,
    },
    #[error("node {node} references node {child}, but only {available} exist")]
    NodeChildOutOfBounds {
        node: u16,
        child: u16,
        available: usize,
    },
    #[error("node {node} references subsector {child}, but only {available} exist")]
    NodeSubsectorOutOfBounds {
        node: u16,
        child: u16,
        available: usize,
    },
}

fn validate_fixture(builder: &DoomFixtureBuilder) -> Result<(), DoomFixtureBuildError> {
    if builder.vertices.is_empty() {
        return Err(DoomFixtureBuildError::MissingVertices);
    }
    if builder.sectors.is_empty() {
        return Err(DoomFixtureBuildError::MissingSectors);
    }
    for (index, sidedef) in builder.sidedefs.iter().enumerate() {
        if usize::from(sidedef.sector) >= builder.sectors.len() {
            return Err(DoomFixtureBuildError::SidedefSectorOutOfBounds {
                sidedef: index as u16,
                sector: sidedef.sector,
                available: builder.sectors.len(),
            });
        }
    }
    for (index, linedef) in builder.linedefs.iter().enumerate() {
        for vertex in [linedef.start_vertex, linedef.end_vertex] {
            if usize::from(vertex) >= builder.vertices.len() {
                return Err(DoomFixtureBuildError::LinedefVertexOutOfBounds {
                    linedef: index as u16,
                    vertex,
                    available: builder.vertices.len(),
                });
            }
        }
        let sides = [linedef.right_sidedef, linedef.left_sidedef];
        if sides.iter().all(Option::is_none) {
            return Err(DoomFixtureBuildError::MissingLinedefSide {
                linedef: index as u16,
            });
        }
        for sidedef in sides.into_iter().flatten() {
            if usize::from(sidedef) >= builder.sidedefs.len() {
                return Err(DoomFixtureBuildError::LinedefSideOutOfBounds {
                    linedef: index as u16,
                    sidedef,
                    available: builder.sidedefs.len(),
                });
            }
        }
    }
    for (index, seg) in builder.segs.iter().enumerate() {
        for vertex in [seg.start_vertex, seg.end_vertex] {
            if usize::from(vertex) >= builder.vertices.len() {
                return Err(DoomFixtureBuildError::SegVertexOutOfBounds {
                    seg: index as u16,
                    vertex,
                    available: builder.vertices.len(),
                });
            }
        }
        if usize::from(seg.linedef) >= builder.linedefs.len() {
            return Err(DoomFixtureBuildError::SegLinedefOutOfBounds {
                seg: index as u16,
                linedef: seg.linedef,
                available: builder.linedefs.len(),
            });
        }
        if seg.direction > 1 {
            return Err(DoomFixtureBuildError::InvalidSegDirection {
                seg: index as u16,
                direction: seg.direction,
            });
        }
    }
    for (index, subsector) in builder.subsectors.iter().enumerate() {
        if subsector.seg_count == 0 {
            return Err(DoomFixtureBuildError::EmptySubsector {
                subsector: index as u16,
            });
        }
        let end = usize::from(subsector.first_seg) + usize::from(subsector.seg_count);
        if end > builder.segs.len() {
            return Err(DoomFixtureBuildError::SubsectorRangeOutOfBounds {
                subsector: index as u16,
                first_seg: subsector.first_seg,
                end,
                available: builder.segs.len(),
            });
        }
    }
    for (index, node) in builder.nodes.iter().enumerate() {
        for child in [node.right_child, node.left_child] {
            match child {
                DoomBspChild::Node(child) if usize::from(child) >= builder.nodes.len() => {
                    return Err(DoomFixtureBuildError::NodeChildOutOfBounds {
                        node: index as u16,
                        child,
                        available: builder.nodes.len(),
                    })
                }
                DoomBspChild::Subsector(child)
                    if usize::from(child) >= builder.subsectors.len() =>
                {
                    return Err(DoomFixtureBuildError::NodeSubsectorOutOfBounds {
                        node: index as u16,
                        child,
                        available: builder.subsectors.len(),
                    })
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn source(record_index: u16) -> DoomSourceRecord {
    DoomSourceRecord {
        lump_index: FIXTURE_LUMP_INDEX,
        record_index: u32::from(record_index),
    }
}

fn empty_blockmap() -> DoomBlockmapObservation {
    DoomBlockmapObservation {
        lump_index: FIXTURE_LUMP_INDEX,
        origin_x: 0,
        origin_y: 0,
        columns: 0,
        rows: 0,
        cells: 0,
        unique_linedef_lists: 0,
        linedef_references: 0,
        cell_linedefs: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doom_geometry_provider::{
        clip_doom_seg_textured_wall_triangle_to_linedef_interval,
        lower_doom_paired_sky_boundary_triangles, lower_doom_textured_wall_triangles,
        lower_doom_two_sided_wall_bands, observe_doom_seg_occluders,
        observe_doom_two_sided_middle_textures, DoomSegClassicPlaneKind, DoomSegOccluderKind,
        DoomWallBand,
    };

    #[test]
    fn valid_fixture_has_stable_manifest_and_calls_shared_provider() {
        let fixture = two_leaf_fixture();
        let first = fixture.structural_manifest();
        let second = fixture.structural_manifest();
        assert_eq!(first, second);
        assert_eq!(
            (
                first.vertices,
                first.linedefs,
                first.segs,
                first.subsectors,
                first.nodes
            ),
            (8, 8, 8, 2, 1)
        );
        let observation = fixture.observe_classic_bsp().unwrap();
        assert_eq!(observation.leaves_visited, 2);
        assert!(!observation.admitted_seg_order.is_empty());
        assert_eq!(fixture.resolve_subsector_loops().unwrap().len(), 2);
    }

    #[test]
    fn compact_evidence_fingerprint_normalizes_bucket_order_without_merging_authority() {
        let first = compact_evidence_fingerprint(
            ["seg:2".to_owned(), "seg:1".to_owned()],
            ["plane:0".to_owned()],
            ["column:4".to_owned(), "column:3".to_owned()],
            ["edge-on:0".to_owned(), "outside-fov:1".to_owned()],
            ["runtime:static-source".to_owned()],
        );
        let second = compact_evidence_fingerprint(
            ["seg:1".to_owned(), "seg:2".to_owned()],
            ["plane:0".to_owned()],
            ["column:3".to_owned(), "column:4".to_owned()],
            ["outside-fov:1".to_owned(), "edge-on:0".to_owned()],
            ["runtime:static-source".to_owned()],
        );

        assert_eq!(first, second);
        assert!(first.1.contains("source=[\"seg:1\", \"seg:2\"]"));
        assert!(first.1.contains("presentation=[\"plane:0\"]"));
        assert!(first.1.contains("runtime=[\"runtime:static-source\"]"));
    }

    #[test]
    fn whole_seg_and_reconstructed_wall_spans_preserve_identity_and_uv_progression() {
        let mut builder = DoomFixtureBuilder::new("split-wall-representation", viewer());
        let sector = builder.sector(0, 128);
        let side = builder.sidedef(sector, "WALL");
        let left = builder.vertex(-64, 0);
        let middle = builder.vertex(0, 0);
        let right = builder.vertex(64, 0);
        let linedef = builder.linedef(left, right, Some(side), None);
        builder.seg(left, middle, linedef, 0);
        builder.seg(middle, right, linedef, 0);
        let fixture = builder.build().unwrap();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 128,
            height: 128,
        }];

        let whole = lower_doom_textured_wall_triangles(&fixture.map, &extents).unwrap();
        let split = lower_doom_seg_textured_wall_triangles(&fixture.map, &extents).unwrap();
        assert_eq!(whole.len(), 2);
        // Clipping each of the two source triangles at the midpoint can
        // triangulate one clipped polygon into two pieces, so representation
        // count is deliberately not treated as semantic identity.
        assert_eq!(split.len(), 6);
        assert!(split.iter().all(|triangle| {
            triangle.source_linedef == whole[0].source_linedef
                && triangle.source_sidedef == whole[0].source_sidedef
                && triangle.source_sector == whole[0].source_sector
                && triangle.side == whole[0].side
                && triangle.role == whole[0].role
        }));

        let u_range = |triangles: &[doom_geometry_provider::DoomSegTexturedWallTriangle]| {
            triangles
                .iter()
                .flat_map(|triangle| triangle.texture_coordinates)
                .map(|uv| uv[0])
                .fold([f64::INFINITY, f64::NEG_INFINITY], |[min, max], u| {
                    [min.min(u), max.max(u)]
                })
        };
        assert_eq!(u_range(&split), [0.0, 128.0]);

        let reconstructed = split
            .iter()
            .flat_map(|triangle| {
                clip_doom_seg_textured_wall_triangle_to_linedef_interval(
                    &fixture.map,
                    triangle,
                    [0.25, 0.75],
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        assert!(!reconstructed.is_empty());
        assert_eq!(u_range(&reconstructed), [32.0, 96.0]);
        assert!(reconstructed.iter().all(|triangle| {
            triangle.source_linedef == whole[0].source_linedef
                && triangle.source_sidedef == whole[0].source_sidedef
                && triangle.source_sector == whole[0].source_sector
                && triangle.side == whole[0].side
                && triangle.role == whole[0].role
        }));
    }

    #[test]
    fn zero_height_and_tiny_openings_remain_distinct_source_classifications() {
        let closed = observe_doom_seg_occluders(&adjacent_plane_fixture(64, 64).map).unwrap();
        let tiny = observe_doom_seg_occluders(&adjacent_plane_fixture(63, 64).map).unwrap();

        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].kind, DoomSegOccluderKind::BackSectorClosed);
        assert_eq!(tiny.len(), 1);
        assert_eq!(tiny[0].kind, DoomSegOccluderKind::Open);
    }

    #[test]
    fn collinear_consecutive_segs_and_repeated_linedef_membership_keep_a_closed_loop() {
        let mut builder = DoomFixtureBuilder::new("collinear-repeated-membership", viewer());
        let sector = builder.sector(0, 128);
        let side = builder.sidedef(sector, "WALL");
        for [x, y] in [[-64, 0], [0, 0], [64, 0], [64, 64], [-64, 64]] {
            builder.vertex(x, y);
        }
        let split_bottom = builder.linedef(0, 2, Some(side), None);
        let right = builder.linedef(2, 3, Some(side), None);
        let top = builder.linedef(3, 4, Some(side), None);
        let left = builder.linedef(4, 0, Some(side), None);
        builder.seg(0, 1, split_bottom, 0);
        builder.seg(1, 2, split_bottom, 0);
        builder.seg(2, 3, right, 0);
        builder.seg(3, 4, top, 0);
        builder.seg(4, 0, left, 0);
        builder.subsector(0, 5);
        let fixture = builder.build().unwrap();

        let loops = fixture.resolve_subsector_loops().unwrap();
        assert_eq!(loops.len(), 1);
        assert_eq!(
            loops[0].vertices,
            vec![[-64, 0], [0, 0], [64, 0], [64, 64], [-64, 64]]
        );
        assert_eq!(loops[0].source_segs.len(), 5);

        let split = lower_doom_seg_textured_wall_triangles(
            &fixture.map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 128,
                height: 128,
            }],
        )
        .unwrap();
        assert_eq!(
            split
                .iter()
                .filter(|triangle| triangle.source_linedef == source(split_bottom))
                .map(|triangle| triangle.source_seg.record_index)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([0, 1])
        );
    }

    #[test]
    fn temporary_height_snapshots_change_preparation_without_mutating_source() {
        let fixture = moving_platform_snapshot_fixture().unwrap();
        let source_sector = fixture.map.sectors[0].source;
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let source_before = fixture.map.clone();
        let ceiling_states = [128_i16, 96, 64, 96, 128];
        let projected = ceiling_states
            .into_iter()
            .map(|ceiling_height| {
                fixture
                    .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                        source_sector,
                        floor_height: None,
                        ceiling_height: Some(ceiling_height),
                    }])
                    .unwrap()
            })
            .collect::<Vec<_>>();

        assert_eq!(fixture.map, source_before);
        assert_eq!(
            projected
                .iter()
                .map(|snapshot| snapshot.map.sectors[0].ceiling_height)
                .collect::<Vec<_>>(),
            ceiling_states
        );
        assert_eq!(projected[0].map, source_before);
        assert_ne!(projected[1].map, source_before);
        assert_ne!(projected[2].map, source_before);
        assert_ne!(projected[3].map, source_before);
        assert_eq!(projected[4].map, source_before);

        let ceiling_maxima = projected
            .iter()
            .map(|snapshot| {
                lower_doom_seg_textured_wall_triangles(&snapshot.map, &extents)
                    .unwrap()
                    .iter()
                    .flat_map(|triangle| triangle.positions)
                    .map(|position| position[1] as i16)
                    .max()
                    .expect("fixture has lowered wall positions")
            })
            .collect::<Vec<_>>();
        assert_eq!(ceiling_maxima, ceiling_states);
        assert_eq!(
            fixture.observe_classic_bsp().unwrap(),
            projected[2].observe_classic_bsp().unwrap(),
            "height overlays do not rewrite the source-only horizontal traversal"
        );
    }

    #[test]
    fn declared_platform_floor_sequence_changes_preparation_without_mutating_source() {
        let fixture = viewer_plane_fixture();
        let source_sector = fixture.map.sectors[0].source;
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let source_before = fixture.map.clone();
        let floor_states = [0_i16, 16, 32, 16, 0];
        let projected = floor_states
            .into_iter()
            .map(|floor_height| {
                fixture
                    .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                        source_sector,
                        floor_height: Some(floor_height),
                        ceiling_height: None,
                    }])
                    .unwrap()
            })
            .collect::<Vec<_>>();

        assert_eq!(fixture.map, source_before);
        assert_eq!(projected[0].map, source_before);
        assert_eq!(projected[4].map, source_before);
        assert_eq!(
            projected
                .iter()
                .map(|snapshot| snapshot.map.sectors[0].floor_height)
                .collect::<Vec<_>>(),
            floor_states
        );

        let floor_minima = projected
            .iter()
            .map(|snapshot| {
                lower_doom_seg_textured_wall_triangles(&snapshot.map, &extents)
                    .unwrap()
                    .iter()
                    .flat_map(|triangle| triangle.positions)
                    .map(|position| position[1] as i16)
                    .min()
                    .expect("fixture has lowered wall positions")
            })
            .collect::<Vec<_>>();
        assert_eq!(floor_minima, floor_states);
    }

    #[test]
    fn stationary_viewer_observes_declared_height_changes_without_crossing_a_boundary() {
        let fixture = two_leaf_fixture();
        let source_sector = fixture.map.sectors[0].source;
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let states = [0_i16, 24, 0];
        let observations = states
            .into_iter()
            .map(|floor_height| {
                fixture
                    .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                        source_sector,
                        floor_height: Some(floor_height),
                        ceiling_height: None,
                    }])
                    .unwrap()
                    .observe_classic_vertical_clips(41, &extents)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let floor_keys = observations
            .iter()
            .map(|observation| {
                observation
                    .plane_spans
                    .keys
                    .keys()
                    .find(|key| key.kind == DoomSegClassicPlaneKind::Floor)
                    .map(|key| key.height)
                    .expect("stationary viewer retains the fixture floor key")
            })
            .collect::<Vec<_>>();
        assert_eq!(floor_keys, states);
        assert_eq!(observations[0], observations[2]);
        assert_ne!(observations[0], observations[1]);
    }

    #[test]
    fn unavailable_runtime_height_snapshot_is_rejected_explicitly() {
        let fixture = viewer_plane_fixture();
        let unavailable = DoomSourceRecord {
            lump_index: FIXTURE_LUMP_INDEX,
            record_index: 99,
        };
        assert_eq!(
            fixture
                .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                    source_sector: unavailable,
                    floor_height: Some(1),
                    ceiling_height: None,
                }])
                .unwrap_err(),
            DoomGeometryError::RuntimeSnapshotSectorUnavailable {
                source_sector: unavailable
            }
        );
    }

    #[test]
    fn dynamic_boundary_preparation_depends_on_declared_state_not_observer_side() {
        let fixture = dynamic_door_snapshot_fixture().unwrap();
        let source_sector = fixture.map.sectors[1].source;
        let closed = fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: None,
                ceiling_height: Some(0),
            }])
            .unwrap();
        let opened = fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: None,
                ceiling_height: Some(128),
            }])
            .unwrap();

        let closed_bands = lower_doom_two_sided_wall_bands(&closed.map).unwrap();
        let opened_bands = lower_doom_two_sided_wall_bands(&opened.map).unwrap();
        assert!(
            !closed_bands.is_empty(),
            "closed doorway retains its height band"
        );
        assert!(opened_bands.is_empty(), "open doorway has no height band");

        let mut west = opened.clone();
        west.viewer.position = [-32, 32];
        west.viewer.heading_radians = 0.0;
        let mut east = opened;
        east.viewer.position = [32, 32];
        east.viewer.heading_radians = std::f64::consts::PI;

        // Viewer side changes the source traversal, but not the caller's
        // declared doorway state or its prepared opening. This prevents a
        // side-local observer accident from redefining runtime geometry.
        assert_ne!(
            west.observe_classic_bsp().unwrap().admitted_seg_order,
            east.observe_classic_bsp().unwrap().admitted_seg_order
        );
        assert!(lower_doom_two_sided_wall_bands(&west.map)
            .unwrap()
            .is_empty());
        assert!(lower_doom_two_sided_wall_bands(&east.map)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn paired_sky_retains_depth_boundary_without_ordinary_upper_wall_authority() {
        let mut paired = adjacent_plane_fixture(0, 96);
        paired.map.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        paired.map.sectors[1].ceiling_texture = "F_SKY1".to_owned();

        let ordinary_bands = lower_doom_two_sided_wall_bands(&paired.map).unwrap();
        let sky_boundary = lower_doom_paired_sky_boundary_triangles(&paired.map).unwrap();
        let occluder = observe_doom_seg_occluders(&paired.map).unwrap();

        assert!(ordinary_bands.is_empty());
        assert_eq!(sky_boundary.len(), 2);
        assert_eq!(occluder.len(), 1);
        assert_eq!(occluder[0].kind, DoomSegOccluderKind::Open);
    }

    #[test]
    fn paired_sky_reaches_only_its_admitted_vertical_columns_without_upper_wall_authority() {
        let mut fixture = vertical_aperture_fixture();
        fixture.map.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        fixture.map.sectors[1].ceiling_texture = "F_SKY1".to_owned();

        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let vertical = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();

        // The BSP-bearing fixture proves that a paired-sky boundary reaches
        // exactly the columns subtended by its admitted source SEG. It is a
        // Doom-local depth-boundary candidate, not an upper wall and not a
        // horizontal solid range.
        assert_eq!(vertical.paired_sky_adjustments, 1);
        assert!(!vertical.column_traces.is_empty());
        assert!(vertical.column_traces.len() < 320);
        assert!(vertical.column_traces.iter().all(|trace| {
            trace.upper_source_segs.is_empty() && trace.paired_sky_boundary_source_segs.len() == 1
        }));
    }

    #[test]
    fn partial_sky_coverage_falsifies_boolean_whole_seg_selection() {
        let observation = observe_partial_coverage_expressiveness().unwrap();

        assert!(observation.paired_sky_columns > 0);
        assert!(observation.far_wall_columns > observation.paired_sky_columns);
        assert!(observation.overlapping_columns > 0);
        assert!(observation.far_only_columns > 0);
        assert!(observation.requires_source_fragments);

        // Keeping the whole far SEG admits its overlap with the authoritative
        // paired-sky interval. Rejecting it loses the required far-only
        // columns. The source contribution therefore cannot be represented by
        // one Boolean candidate decision at this granularity.
        assert_eq!(
            observation.overlapping_columns + observation.far_only_columns,
            observation.far_wall_columns
        );
        assert_eq!(
            observation
                .overlapping_runs
                .iter()
                .map(|run| run.last - run.first + 1)
                .sum::<usize>(),
            observation.overlapping_columns
        );
        assert_eq!(
            observation
                .surviving_runs
                .iter()
                .map(|run| run.last - run.first + 1)
                .sum::<usize>(),
            observation.far_only_columns
        );
        assert!(observation
            .overlapping_runs
            .iter()
            .chain(&observation.surviving_runs)
            .all(|run| run.first <= run.last));
    }

    #[test]
    fn one_sky_boundary_retains_ordinary_upper_wall_and_no_sky_depth_boundary() {
        let mut one_sky = adjacent_plane_fixture(0, 96);
        one_sky.map.sectors[0].ceiling_texture = "F_SKY1".to_owned();

        let ordinary_bands = lower_doom_two_sided_wall_bands(&one_sky.map).unwrap();
        let sky_boundary = lower_doom_paired_sky_boundary_triangles(&one_sky.map).unwrap();

        assert_eq!(ordinary_bands.len(), 2);
        assert!(ordinary_bands
            .iter()
            .all(|triangle| triangle.band == DoomWallBand::Upper));
        assert!(sky_boundary.is_empty());
    }

    #[test]
    fn one_sky_never_inherits_paired_sky_column_authority() {
        let mut fixture = vertical_aperture_fixture();
        fixture.map.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];

        let vertical = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();

        assert_eq!(vertical.paired_sky_adjustments, 0);
        assert!(vertical.column_traces.iter().all(|trace| {
            trace.paired_sky_boundary_source_segs.is_empty() && !trace.upper_source_segs.is_empty()
        }));
    }

    #[test]
    fn one_sky_far_control_retains_visible_upper_wall_instead_of_depth_boundary() {
        let fixture = one_sky_far_control_fixture().unwrap();
        let boundaries = lower_doom_paired_sky_boundary_triangles(&fixture.map).unwrap();
        let walls = lower_doom_seg_textured_wall_triangles(
            &fixture.map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .unwrap();
        let near = fixture.map.segs[0].source;
        let far = fixture.map.segs[1].source;

        assert!(boundaries.is_empty());
        assert!(walls.iter().any(|triangle| triangle.source_seg == near));
        assert!(walls.iter().any(|triangle| triangle.source_seg == far));
    }

    #[test]
    fn single_sky_plane_control_retains_a_closed_source_plane_without_paired_authority() {
        let fixture = single_sky_plane_far_control_fixture().unwrap();
        assert_eq!(fixture.map.sectors.len(), 1);
        assert_eq!(fixture.map.sectors[0].ceiling_texture, "F_SKY1");
        assert_eq!(fixture.resolve_subsector_loops().unwrap().len(), 1);
        assert!(lower_doom_paired_sky_boundary_triangles(&fixture.map)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn paired_sky_boundary_marks_only_near_columns_while_far_geometry_remains_a_separate_candidate()
    {
        let fixture = paired_sky_far_control_fixture().unwrap();
        let traversal = fixture.observe_classic_bsp().unwrap();
        let vertical = fixture
            .observe_classic_vertical_clips(
                41,
                &[DoomTextureExtent {
                    name: "WALL".to_owned(),
                    width: 64,
                    height: 128,
                }],
            )
            .unwrap();
        let sky_boundary = lower_doom_paired_sky_boundary_triangles(&fixture.map).unwrap();

        let near_seg = fixture.map.segs[0].source.record_index;
        let far_seg = fixture.map.segs[1].source.record_index;
        assert_eq!(sky_boundary.len(), 2);
        assert!(traversal.admitted_seg_order.contains(&near_seg));
        assert!(traversal.admitted_seg_order.contains(&far_seg));
        assert_eq!(vertical.paired_sky_adjustments, 1, "vertical={vertical:?}");

        let paired_columns = vertical
            .column_traces
            .iter()
            .filter(|trace| !trace.paired_sky_boundary_source_segs.is_empty())
            .collect::<Vec<_>>();
        assert!(!paired_columns.is_empty());
        assert!(paired_columns.len() < 320);
        assert!(paired_columns.iter().all(|trace| {
            trace.paired_sky_boundary_source_segs.contains(&near_seg)
                && !trace.paired_sky_boundary_source_segs.contains(&far_seg)
        }));

        // This Level-1 result intentionally does not claim that the far SEG
        // was already hidden. The far one-sided control wall contributes the
        // single ordinary solid range; paired sky contributes none. Deciding
        // their colour/depth interaction belongs to the later Doom
        // presentation fixture, not to generic horizontal occlusion.
        assert_eq!(traversal.solid_range_contributors, 1);
        assert_eq!(traversal.pass_admitted, 1);
    }

    #[test]
    fn authored_masked_middle_does_not_gain_solid_occluder_authority() {
        let mut fixture = adjacent_plane_fixture(0, 128);
        fixture.map.sidedefs[0].middle_texture = "MASKED".to_owned();

        let middle = observe_doom_two_sided_middle_textures(&fixture.map).unwrap();
        let occluder = observe_doom_seg_occluders(&fixture.map).unwrap();

        assert_eq!(middle.len(), 1);
        assert_eq!(middle[0].texture_name, "MASKED");
        assert_eq!(occluder.len(), 1);
        assert_eq!(occluder[0].kind, DoomSegOccluderKind::Open);
    }

    #[test]
    fn two_sided_aperture_retains_independent_upper_lower_opening_and_plane_intervals() {
        let fixture = vertical_aperture_fixture();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];

        let wall_bands = lower_doom_two_sided_wall_bands(&fixture.map).unwrap();
        let vertical = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();

        // The fixture has front 0..128 and back 24..96.  It must therefore
        // retain source-authoritative upper/lower bands around a real opening;
        // a horizontal coverage shortcut may not merge those bands into a
        // solid wall or erase the front floor/ceiling *source facts*.
        assert_eq!(wall_bands.len(), 4);
        assert!(wall_bands
            .iter()
            .any(|triangle| triangle.band == DoomWallBand::Upper));
        assert!(wall_bands
            .iter()
            .any(|triangle| triangle.band == DoomWallBand::Lower));
        assert_eq!(vertical.upper_tier_spans, 1);
        assert_eq!(vertical.lower_tier_spans, 1);
        assert_eq!(vertical.middle_tier_spans, 0);
        assert_eq!(vertical.floor_plane_marks, 1);
        assert_eq!(vertical.ceiling_plane_marks, 1);
        assert!(vertical.ceiling_clip_updates > 0);
        assert!(vertical.floor_clip_updates > 0);
        assert!(vertical.column_traces.len() < 320);
        assert!(vertical.column_traces.iter().all(|trace| {
            // The interval between the clipped upper and lower tiers remains
            // an opening. The fixture uses the same source SEG for both
            // tiers, while the separate sets preserve tier identity.
            trace.ceiling_clip < trace.floor_clip
                && !trace.upper_source_segs.is_empty()
                && !trace.lower_source_segs.is_empty()
                && trace.middle_source_segs.is_empty()
        }));
        assert!(vertical
            .plane_spans
            .keys
            .keys()
            .any(|key| key.kind == DoomSegClassicPlaneKind::Floor));
        // The upper tier can legitimately consume the visible ceiling range
        // at this one aperture.  Retaining its source mark separately from a
        // surviving plane span is precisely the distinction under test.
        assert!(!vertical
            .plane_spans
            .keys
            .keys()
            .any(|key| key.kind == DoomSegClassicPlaneKind::Ceiling));
    }

    #[test]
    fn aperture_edge_jitter_keeps_vertical_contribution_trace_deterministic() {
        let fixture = vertical_aperture_fixture();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let headings = [
            std::f64::consts::FRAC_PI_2 - 1.0e-7,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::FRAC_PI_2 + 1.0e-7,
        ];

        for heading_radians in headings {
            let mut pose = fixture.clone();
            pose.viewer.heading_radians = heading_radians;
            let first = pose.observe_classic_vertical_clips(41, &extents).unwrap();
            let second = pose.observe_classic_vertical_clips(41, &extents).unwrap();
            assert_eq!(first, second, "heading={heading_radians}");
            assert_eq!(first.upper_tier_spans, 1);
            assert_eq!(first.lower_tier_spans, 1);
            assert!(first.ceiling_clip_updates > 0);
            assert!(first.floor_clip_updates > 0);
        }
    }

    #[test]
    fn near_source_solid_prunes_only_the_explicitly_covered_far_subsector() {
        let fixture = near_solid_far_control_fixture(false);
        let observation = fixture.observe_classic_bsp().unwrap();

        assert_eq!(observation.admitted_seg_order, vec![0]);
        assert_eq!(observation.far_children_pruned, 1);
        assert_eq!(observation.watched_subsector_elisions.len(), 1);
        assert!(observation.watched_subsector_elisions[0].contains("reason=solid-range"));
        assert!(observation.watched_subsector_elisions[0].contains("subsectors=[1]"));

        // Source SEG 0 is the near solid contributor.  The far source SEG 1
        // disappears only because its own BSP child projection is wholly
        // covered by that retained source range.
        assert!(observation.admitted_seg_records.contains(&0));
        assert!(!observation.admitted_seg_records.contains(&1));
    }

    #[test]
    fn near_open_aperture_never_borrows_solid_authority_to_prune_far_geometry() {
        let fixture = near_solid_far_control_fixture(true);
        let observation = fixture.observe_classic_bsp().unwrap();

        // A two-sided equal-height boundary is an aperture. It may be a
        // useful presentation boundary later, but it is not allowed to close
        // a generic horizontal solid range merely because it is nearer.
        assert_eq!(
            observation.solid_admitted, 1,
            "only the far one-sided wall is solid"
        );
        assert_eq!(observation.far_children_pruned, 0);
        assert!(observation.admitted_seg_records.contains(&0));
        assert!(observation.admitted_seg_records.contains(&1));
        assert!(observation.watched_subsector_elisions.is_empty());
    }

    #[test]
    fn classic_vertical_clip_observation_reuses_the_provider_preparation_path() {
        let fixture = two_leaf_fixture();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let first = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();
        let second = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();

        assert_eq!(first, second);
        assert!(first.admitted_segs > 0);
        assert!(first.upper_tier_spans + first.lower_tier_spans + first.middle_tier_spans > 0);
        assert!(first.plane_spans.plane_instances > 0);
        assert!(first.plane_spans.populated_columns > 0);
        assert!(first.plane_spans.populated_cells > 0);
    }

    #[test]
    fn continuous_plane_control_keeps_one_source_plane_instance_across_bsp_leaves() {
        let fixture = two_leaf_fixture();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let observation = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();
        let floor = observation
            .plane_spans
            .keys
            .iter()
            .find(|(key, _)| key.kind == DoomSegClassicPlaneKind::Floor)
            .expect("one-source-sector fixture must retain a floor plane");

        // The root divides source SEGs into two subsectors, but neither leaf
        // is entitled to create a second presentation identity for the same
        // decoded floor. This controls the E1M1 failure where whole-subsector
        // selection tore a continuous plane at a BSP boundary.
        assert_eq!(floor.0.height, 0);
        assert_eq!(floor.1.len(), 1);
        assert_eq!(floor.1[0].source_sectors.len(), 1);
        assert_eq!(floor.1[0].source_sectors.iter().next().copied(), Some(0));
        assert!(floor.1[0].source_segs.len() > 1);
        assert!(floor.1[0].columns.iter().any(Option::is_some));
    }

    #[test]
    fn pillar_control_does_not_split_the_shared_floor_identity() {
        let fixture = pillar_fixture();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let observation = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();
        let floor = observation
            .plane_spans
            .keys
            .iter()
            .find(|(key, _)| key.kind == DoomSegClassicPlaneKind::Floor)
            .expect("pillar control must retain the shared floor source key");

        // The nearer square is presentation pressure, not a second source
        // floor. A visibility path may clip ranges later, but it must not turn
        // a structural obstacle into a new floor identity.
        assert_eq!(floor.1.len(), 1);
        assert_eq!(
            floor.1[0]
                .source_sectors
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            [0]
        );
        assert!(!floor.1[0].source_segs.is_empty());
        assert_eq!(observation.admitted_segs, 1);
        assert!(observation.middle_tier_spans > 0);
        assert!(observation.plane_spans.populated_cells > 0);
    }

    #[test]
    fn heading_changes_clip_coverage_without_rewriting_source_plane_identity() {
        let fixture = two_leaf_fixture();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let forward = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();
        let mut reversed_fixture = fixture.clone();
        reversed_fixture.viewer.heading_radians = -std::f64::consts::FRAC_PI_2;
        let reversed = reversed_fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();

        // Viewer heading controls what is admitted and clipped. If no source
        // SEG survives, no projected plane instance exists; the immutable
        // decoded plane-mark facts must still remain unchanged.
        assert_ne!(forward.admitted_segs, reversed.admitted_segs);
        assert_ne!(forward.plane_spans, reversed.plane_spans);
        assert!(reversed.plane_spans.keys.is_empty());
        assert_eq!(
            fixture.observe_plane_marks(41).unwrap(),
            reversed_fixture.observe_plane_marks(41).unwrap()
        );
    }

    #[test]
    fn viewer_plane_control_produces_no_vertical_coverage_claim() {
        let fixture = viewer_plane_fixture();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let observation = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();

        // A viewer exactly on a source wall is classified edge-on by the
        // shared traversal. The vertical stage must inherit that fail-open
        // result rather than manufacture wall, floor, or ceiling coverage.
        assert_eq!(observation.admitted_segs, 0);
        assert_eq!(observation.upper_tier_spans, 0);
        assert_eq!(observation.lower_tier_spans, 0);
        assert_eq!(observation.middle_tier_spans, 0);
        assert_eq!(observation.plane_spans.plane_instances, 0);
        assert!(observation.plane_spans.keys.is_empty());
    }

    #[test]
    fn shared_plane_key_uses_distinct_instances_for_conflicting_coverage() {
        let fixture = shared_key_disjoint_plane_fixture().unwrap();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let observation = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();
        let floor = observation
            .plane_spans
            .keys
            .iter()
            .find(|(key, _)| key.kind == DoomSegClassicPlaneKind::Floor)
            .expect("matching sector planes must retain a shared floor key");

        // Equal height/flat/light semantics are not permission to overwrite
        // overlapping projected cells sourced by a different sector. The
        // provider preserves the common key while splitting incompatible
        // presentation coverage into separate instances.
        assert!(observation.admitted_segs >= 2);
        assert_eq!(floor.0.height, 0);
        assert_eq!(floor.1.len(), 2);
        assert_eq!(observation.plane_spans.collision_splits, 1);
        assert_eq!(
            floor
                .1
                .iter()
                .map(|instance| instance.source_sectors.iter().copied().collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![vec![0], vec![1]]
        );
    }

    #[test]
    fn directional_matrix_retains_deterministic_admission_and_empty_span_behavior() {
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        // These are the two current fixtures that actually contain a BSP
        // subsector boundary. Keep the matrix fixture-local: it proves the
        // shared provider remains deterministic for distinct source topology,
        // not that the two shapes have the same visibility result.
        for fixture in [two_leaf_fixture(), pillar_fixture()] {
            for heading in (0..8).map(|step| f64::from(step) * std::f64::consts::FRAC_PI_4) {
                let mut pose = fixture.clone();
                pose.viewer.heading_radians = heading;
                let first = pose.observe_classic_vertical_clips(41, &extents).unwrap();
                let second = pose.observe_classic_vertical_clips(41, &extents).unwrap();
                assert_eq!(first, second, "fixture={} heading={heading}", fixture.name);
                if first.admitted_segs == 0 {
                    assert!(
                        first.plane_spans.keys.is_empty(),
                        "fixture={} heading={heading}",
                        fixture.name
                    );
                }
            }
        }
    }

    #[test]
    fn micro_jitter_away_from_a_boundary_preserves_semantic_admission() {
        let fixture = two_leaf_fixture();
        let offsets = [-1.0e-7, 0.0, 1.0e-7];
        let manifests = offsets
            .into_iter()
            .map(|offset| {
                let mut pose = fixture.clone();
                pose.viewer.heading_radians = std::f64::consts::FRAC_PI_2 + offset;
                pose.classic_bsp_manifest().unwrap()
            })
            .collect::<Vec<_>>();

        // This pose is intentionally away from the source FOV/partition
        // boundaries. The control does not claim that all boundary crossings
        // must be invariant; it prevents ordinary sub-epsilon camera noise
        // from creating an unexplained admission transition.
        let semantic_observation = |manifest: &DoomClassicBspManifest| {
            (
                manifest.admitted_seg_records.clone(),
                manifest.leaves_visited,
                manifest.backface_rejected,
                manifest.edge_on,
                manifest.outside_fov_rejected,
                manifest.near_plane_fail_open,
                manifest.solid_range_covered_columns,
            )
        };
        assert_eq!(
            semantic_observation(&manifests[0]),
            semantic_observation(&manifests[1])
        );
        assert_eq!(
            semantic_observation(&manifests[1]),
            semantic_observation(&manifests[2])
        );
        assert_ne!(manifests[0].fingerprint, manifests[1].fingerprint);
        assert_ne!(manifests[1].fingerprint, manifests[2].fingerprint);
    }

    #[test]
    fn viewer_crossing_a_fixture_partition_changes_the_source_observation() {
        let fixture = two_leaf_fixture();
        let mut west = fixture.clone();
        west.viewer.position = [-64, -96];
        let mut east = fixture;
        east.viewer.position = [64, -96];

        let west = west.classic_bsp_manifest().unwrap();
        let east = east.classic_bsp_manifest().unwrap();
        assert_ne!(west.admitted_seg_records, east.admitted_seg_records);
        assert_ne!(west.fingerprint, east.fingerprint);
    }

    #[test]
    fn source_fov_controls_keep_crossing_geometry_and_reject_same_side_exterior() {
        let fixture = two_leaf_fixture();
        let mut crossing = fixture.clone();
        crossing.viewer.heading_radians = std::f64::consts::FRAC_PI_4;
        let crossing = crossing.observe_classic_bsp().unwrap();

        let mut same_side = fixture;
        same_side.viewer.heading_radians = 0.0;
        let same_side = same_side.observe_classic_bsp().unwrap();

        assert!(!crossing.admitted_seg_order.is_empty());
        assert!(same_side.outside_fov_rejected > 0);
    }

    #[test]
    fn source_required_contributions_fail_tests_while_extra_candidates_remain_measured() {
        let observation = two_leaf_fixture().observe_classic_bsp().unwrap();

        // The southern boundary of the left authored leaf is deliberately the
        // required contribution for this control pose. Missing it is a
        // correctness failure (a synthetic false negative), not an
        // optimization result. Other admitted SEGs are retained below as
        // unresolved source-only candidates: without a rendered/source
        // presentation oracle, calling them false positives would overclaim.
        let required = [0_u32];
        for source_seg in required {
            assert!(
                observation.admitted_seg_order.contains(&source_seg),
                "required source SEG {source_seg} was discarded; observation={observation:?}"
            );
        }
        let unresolved_extra_candidates = observation
            .admitted_seg_order
            .iter()
            .copied()
            .filter(|source_seg| !required.contains(source_seg))
            .collect::<Vec<_>>();

        assert_eq!(required, [0]);
        assert!(
            !unresolved_extra_candidates.is_empty(),
            "the control should retain extra candidates separately from the required contribution; observation={observation:?}"
        );
    }

    #[test]
    fn fov_edge_micro_jitter_is_deterministic_and_retains_its_transition_trace() {
        let fixture = two_leaf_fixture();
        let offsets = [-1.0e-7, 0.0, 1.0e-7];
        let manifests = offsets
            .into_iter()
            .map(|offset| {
                let mut pose = fixture.clone();
                pose.viewer.heading_radians = std::f64::consts::FRAC_PI_4 + offset;
                let first = pose.classic_bsp_manifest().unwrap();
                let second = pose.classic_bsp_manifest().unwrap();
                assert_eq!(first, second);
                first
            })
            .collect::<Vec<_>>();

        // Exact pose is retained in every trace, so a source FOV transition is
        // explainable rather than hidden behind a count-only comparison.
        assert!(manifests
            .iter()
            .all(|manifest| manifest.trace.contains("outside-fov:")));
        assert_ne!(manifests[0].fingerprint, manifests[1].fingerprint);
        assert_ne!(manifests[1].fingerprint, manifests[2].fingerprint);
    }

    #[test]
    fn viewer_plane_crossing_switches_from_fail_open_to_a_source_classification() {
        let fixture = viewer_plane_fixture();
        let exact = fixture.observe_classic_bsp().unwrap();

        let mut forward = fixture.clone();
        forward.viewer.position = [0, -1];
        let forward = forward.observe_classic_bsp().unwrap();

        let mut reverse = fixture;
        reverse.viewer.position = [0, 1];
        let reverse = reverse.observe_classic_bsp().unwrap();

        assert!(exact.edge_on > 0);
        assert!(forward.solid_admitted > 0 || forward.near_plane_fail_open > 0);
        assert!(reverse.backface_rejected > 0 || reverse.outside_fov_rejected > 0);
    }

    #[test]
    fn viewer_plane_boundary_jitter_retains_every_pose_and_transition_reason() {
        let fixture = viewer_plane_fixture();
        let positions = [[0, -1], [0, 0], [0, 1]];
        let manifests = positions
            .into_iter()
            .map(|position| {
                let mut pose = fixture.clone();
                pose.viewer.position = position;
                let first = pose.classic_bsp_manifest().unwrap();
                let second = pose.classic_bsp_manifest().unwrap();
                assert_eq!(first, second, "position={position:?}");
                assert!(first.trace.contains("edge-on:"));
                assert!(first.trace.contains("backface:"));
                assert!(first.trace.contains("outside-fov:"));
                assert!(first.trace.contains("near-fail-open:"));
                first
            })
            .collect::<Vec<_>>();

        // Crossing this exact source boundary is allowed to change the
        // source result. What must remain deterministic is the pose-specific
        // transition and its named classifier evidence, rather than a
        // count-only fingerprint with no explanation.
        assert_ne!(manifests[0].fingerprint, manifests[1].fingerprint);
        assert_ne!(manifests[1].fingerprint, manifests[2].fingerprint);
        assert!(manifests[1].edge_on > 0);
        assert!(
            manifests[0].admitted_seg_records != manifests[2].admitted_seg_records
                || manifests[0].backface_rejected != manifests[2].backface_rejected
                || manifests[0].outside_fov_rejected != manifests[2].outside_fov_rejected
        );
    }

    #[test]
    fn one_endpoint_behind_the_viewer_fails_open_without_solid_range_closure() {
        let fixture = projection_near_plane_crossing_fixture().unwrap();
        let observation = fixture.observe_classic_bsp().unwrap();

        assert_eq!(observation.solid_admitted, 0);
        assert_eq!(observation.solid_range_covered_columns, 0);
        assert!(observation.near_plane_fail_open > 0);
    }

    #[test]
    fn nearly_zero_width_valid_seg_is_admitted_without_an_unsafe_range_claim() {
        let observation = projection_thin_forward_seg_fixture()
            .unwrap()
            .observe_classic_bsp()
            .unwrap();

        assert_eq!(observation.solid_admitted, 1);
        // The source SEG is one unit wide, but the conservative diagnostic
        // projection may cover its neighbouring columns. This is bounded
        // interval rounding, not a claim of historic pixel-width parity.
        assert!(
            observation.solid_range_covered_columns <= 3,
            "observation={observation:?}"
        );
        assert_eq!(observation.near_plane_fail_open, 0);
    }

    #[test]
    fn extremely_close_valid_seg_is_admitted_as_ordinary_source_geometry() {
        let observation = projection_close_forward_seg_fixture()
            .unwrap()
            .observe_classic_bsp()
            .unwrap();

        assert_eq!(observation.solid_admitted, 1);
        assert_eq!(observation.near_plane_fail_open, 0);
        assert!(observation.solid_range_covered_columns > 0);
    }

    #[test]
    fn reached_bsp_leaves_do_not_invent_plane_coverage_without_admitted_segs() {
        let mut fixture = two_leaf_fixture();
        fixture.viewer.heading_radians = -std::f64::consts::FRAC_PI_2;
        let traversal = fixture.observe_classic_bsp().unwrap();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let vertical = fixture
            .observe_classic_vertical_clips(41, &extents)
            .unwrap();

        assert_eq!(traversal.leaves_visited, 2);
        assert!(traversal.admitted_seg_order.is_empty());
        assert!(vertical.plane_spans.keys.is_empty());
    }

    #[test]
    fn equal_adjacent_sectors_do_not_mark_a_plane_boundary() {
        let fixture = adjacent_plane_fixture(0, 128);
        let marks = fixture.observe_plane_marks(41).unwrap();
        assert_eq!(marks.len(), 1);
        assert!(!marks[0].floor_marked);
        assert!(!marks[0].ceiling_marked);
    }

    #[test]
    fn changed_adjacent_floor_marks_only_the_source_floor_boundary() {
        let fixture = adjacent_plane_fixture(24, 128);
        let marks = fixture.observe_plane_marks(41).unwrap();
        assert_eq!(marks.len(), 1);
        assert!(marks[0].floor_marked);
        assert!(!marks[0].ceiling_marked);
    }

    #[test]
    fn plane_marks_remain_source_facts_across_viewer_heading_controls() {
        let fixture = two_leaf_fixture();
        let baseline = fixture.observe_plane_marks(41).unwrap();
        let mut reversed = fixture.clone();
        reversed.viewer.heading_radians = -std::f64::consts::FRAC_PI_2;

        // Plane marking is a source sector-boundary observation. The later
        // viewer-relative traversal may admit different SEGs, but changing a
        // heading must not erase the decoded plane-mark facts themselves.
        assert_eq!(baseline, reversed.observe_plane_marks(41).unwrap());
        assert!(baseline.iter().all(|mark| {
            mark.floor_marked
                && mark.ceiling_marked
                && mark.front_sector.record_index == 0
                && mark.back_sector.is_none()
        }));
    }

    #[test]
    fn source_view_height_only_suppresses_the_impossible_plane_side() {
        let fixture = adjacent_plane_fixture(24, 128);
        let below_floor = fixture.observe_plane_marks(0).unwrap();
        let within_opening = fixture.observe_plane_marks(41).unwrap();
        let above_ceiling = fixture.observe_plane_marks(160).unwrap();

        assert_eq!(below_floor.len(), 1);
        assert!(!below_floor[0].floor_marked);
        assert!(!below_floor[0].ceiling_marked);
        assert!(within_opening[0].floor_marked);
        assert!(!within_opening[0].ceiling_marked);
        assert!(above_ceiling[0].floor_marked);
        assert!(!above_ceiling[0].ceiling_marked);
    }

    #[test]
    fn rear_facing_control_does_not_admit_the_forward_wall_set() {
        let mut fixture = two_leaf_fixture();
        fixture.viewer.heading_radians = -std::f64::consts::FRAC_PI_2;
        let observation = fixture.observe_classic_bsp().unwrap();
        assert!(observation.admitted_seg_order.is_empty());
        assert!(observation.backface_rejected > 0 || observation.outside_fov_rejected > 0);
    }

    #[test]
    fn classic_bsp_manifest_retains_pose_specific_admission_evidence() {
        let fixture = two_leaf_fixture();
        let first = fixture.classic_bsp_manifest().unwrap();
        let second = fixture.classic_bsp_manifest().unwrap();
        let mut rear_fixture = fixture.clone();
        rear_fixture.viewer.heading_radians = -std::f64::consts::FRAC_PI_2;
        let rear = rear_fixture.classic_bsp_manifest().unwrap();

        assert_eq!(first, second);
        assert!(!first.admitted_seg_records.is_empty());
        assert_ne!(first.fingerprint, rear.fingerprint);
        assert!(rear.admitted_seg_records.is_empty());
        assert!(rear.backface_rejected > 0 || rear.outside_fov_rejected > 0);
    }

    #[test]
    fn viewer_plane_control_fails_open_instead_of_claiming_solid_coverage() {
        let fixture = viewer_plane_fixture();
        let observation = fixture.observe_classic_bsp().unwrap();

        // The observer lies directly on the southern source wall. This is
        // deliberately not a claim of historic Doom column parity: it only
        // guards the current conservative rule that this ambiguous segment
        // must be identified as non-closing rather than silently classified
        // as an ordinary solid-range contributor. At the exact boundary the
        // shared provider classifies the source SEG as edge-on.
        assert!(observation.edge_on > 0);
    }

    #[test]
    fn malformed_bsp_node_reports_a_bounded_provider_diagnostic() {
        let mut fixture = two_leaf_fixture();
        fixture.map.nodes[0].left_child = DoomBspChild::Node(7);
        assert_eq!(
            fixture.observe_classic_bsp().unwrap_err(),
            DoomGeometryError::BspNodeOutOfBounds {
                node_index: 7,
                available: 1,
            }
        );
    }

    #[test]
    fn invalid_vertex_index_is_rejected_before_provider_observation() {
        let mut builder = DoomFixtureBuilder::new("invalid-vertex", viewer());
        let sector = builder.sector(0, 128);
        let side = builder.sidedef(sector, "WALL");
        builder.vertex(0, 0);
        builder.linedef(0, 7, Some(side), None);
        assert_eq!(
            builder.build().unwrap_err(),
            DoomFixtureBuildError::LinedefVertexOutOfBounds {
                linedef: 0,
                vertex: 7,
                available: 1
            }
        );
    }

    #[test]
    fn missing_linedef_side_is_rejected() {
        let mut builder = DoomFixtureBuilder::new("missing-side", viewer());
        let sector = builder.sector(0, 128);
        builder.vertex(0, 0);
        builder.vertex(64, 0);
        let _unused_side = builder.sidedef(sector, "WALL");
        builder.linedef(0, 1, None, None);
        assert_eq!(
            builder.build().unwrap_err(),
            DoomFixtureBuildError::MissingLinedefSide { linedef: 0 }
        );
    }

    #[test]
    fn empty_subsector_is_rejected() {
        let mut builder = base_builder("empty-subsector");
        builder.subsector(0, 0);
        assert_eq!(
            builder.build().unwrap_err(),
            DoomFixtureBuildError::EmptySubsector { subsector: 0 }
        );
    }

    #[test]
    fn contradictory_sector_ownership_is_rejected() {
        let mut builder = DoomFixtureBuilder::new("bad-sector", viewer());
        builder.vertex(0, 0);
        builder.vertex(64, 0);
        builder.sector(0, 128);
        builder.sidedef(7, "WALL");
        assert_eq!(
            builder.build().unwrap_err(),
            DoomFixtureBuildError::SidedefSectorOutOfBounds {
                sidedef: 0,
                sector: 7,
                available: 1
            }
        );
    }

    fn viewer() -> DoomFixtureViewer {
        DoomFixtureViewer {
            position: [0, -96],
            heading_radians: std::f64::consts::FRAC_PI_2,
        }
    }

    fn base_builder(name: &str) -> DoomFixtureBuilder {
        let mut builder = DoomFixtureBuilder::new(name, viewer());
        builder.sector(0, 128);
        builder.vertex(0, 0);
        builder.vertex(64, 0);
        builder
    }

    fn two_leaf_fixture() -> DoomVisibilityFixture {
        let mut builder = DoomFixtureBuilder::new("two-leaf-control", viewer());
        let sector = builder.sector(0, 128);
        let side = builder.sidedef(sector, "WALL");
        let vertices = [
            [-96, 0],
            [-32, 0],
            [-32, 64],
            [-96, 64],
            [32, 0],
            [96, 0],
            [96, 64],
            [32, 64],
        ];
        for [x, y] in vertices {
            builder.vertex(x, y);
        }
        for [start, end] in [
            [0, 1],
            [1, 2],
            [2, 3],
            [3, 0],
            [4, 5],
            [5, 6],
            [6, 7],
            [7, 4],
        ] {
            builder.linedef(start, end, Some(side), None);
        }
        for linedef in 0..8 {
            let line = &builder.linedefs[usize::from(linedef)];
            builder.seg(line.start_vertex, line.end_vertex, linedef, 0);
        }
        builder.subsector(0, 4);
        builder.subsector(4, 4);
        builder.node(DoomFixtureNode {
            point: [0, 0],
            delta: [0, 64],
            right_bbox: [0, 96, 64, -96],
            left_bbox: [0, 96, 64, -96],
            right_child: DoomBspChild::Subsector(1),
            left_child: DoomBspChild::Subsector(0),
        });
        builder.watch_subsector(1);
        builder.build().unwrap()
    }

    fn pillar_fixture() -> DoomVisibilityFixture {
        let mut builder = DoomFixtureBuilder::new("pillar-continuity", viewer());
        let sector = builder.sector(0, 128);
        let side = builder.sidedef(sector, "WALL");
        for [x, y] in [
            [-96, 0],
            [96, 0],
            [96, 96],
            [-96, 96],
            [-16, -16],
            [16, -16],
            [16, 16],
            [-16, 16],
        ] {
            builder.vertex(x, y);
        }
        for [start, end] in [
            [0, 1],
            [1, 2],
            [2, 3],
            [3, 0],
            [4, 5],
            [5, 6],
            [6, 7],
            [7, 4],
        ] {
            builder.linedef(start, end, Some(side), None);
        }
        for linedef in 0..8 {
            let line = &builder.linedefs[usize::from(linedef)];
            builder.seg(line.start_vertex, line.end_vertex, linedef, 0);
        }
        builder.subsector(0, 4);
        builder.subsector(4, 4);
        builder.node(DoomFixtureNode {
            point: [0, 0],
            delta: [0, 96],
            right_bbox: [-16, 16, 16, -16],
            left_bbox: [0, 96, 96, -96],
            right_child: DoomBspChild::Subsector(1),
            left_child: DoomBspChild::Subsector(0),
        });
        builder.build().unwrap()
    }

    fn adjacent_plane_fixture(back_floor: i16, back_ceiling: i16) -> DoomVisibilityFixture {
        let mut builder = DoomFixtureBuilder::new("adjacent-plane", viewer());
        let front = builder.sector(0, 128);
        let back = builder.sector(back_floor, back_ceiling);
        let right = builder.sidedef(front, "-");
        let left = builder.sidedef(back, "-");
        let first = builder.vertex(-32, 0);
        let second = builder.vertex(32, 0);
        let linedef = builder.linedef(first, second, Some(right), Some(left));
        builder.seg(first, second, linedef, 0);
        builder.build().unwrap()
    }

    fn vertical_aperture_fixture() -> DoomVisibilityFixture {
        vertical_aperture_control_fixture().unwrap()
    }

    fn near_solid_far_control_fixture(near_is_open: bool) -> DoomVisibilityFixture {
        let mut builder = DoomFixtureBuilder::new(
            if near_is_open {
                "near-open-far-control"
            } else {
                "near-solid-far-control"
            },
            DoomFixtureViewer {
                position: [0, -96],
                heading_radians: std::f64::consts::FRAC_PI_2,
            },
        );
        let sector = builder.sector(0, 128);
        let right = builder.sidedef(sector, "WALL");
        let left = near_is_open.then(|| builder.sidedef(sector, "-"));
        let near_start = builder.vertex(-128, 0);
        let near_end = builder.vertex(128, 0);
        let far_start = builder.vertex(-24, 64);
        let far_end = builder.vertex(24, 64);
        let near = builder.linedef(near_start, near_end, Some(right), left);
        let far = builder.linedef(far_start, far_end, Some(right), None);
        builder.seg(near_start, near_end, near, 0);
        builder.seg(far_start, far_end, far, 0);
        builder.subsector(0, 1);
        builder.subsector(1, 1);
        builder.node(DoomFixtureNode {
            point: [0, 0],
            delta: [0, 64],
            right_bbox: [64, 64, 24, -24],
            left_bbox: [0, 0, 128, -128],
            right_child: DoomBspChild::Subsector(1),
            left_child: DoomBspChild::Subsector(0),
        });
        builder.watch_subsector(1);
        builder.build().unwrap()
    }

    fn viewer_plane_fixture() -> DoomVisibilityFixture {
        let mut builder = DoomFixtureBuilder::new(
            "viewer-plane",
            DoomFixtureViewer {
                position: [0, 0],
                heading_radians: std::f64::consts::FRAC_PI_2,
            },
        );
        let sector = builder.sector(0, 128);
        let side = builder.sidedef(sector, "WALL");
        for [x, y] in [[-32, 0], [32, 0], [32, 64], [-32, 64]] {
            builder.vertex(x, y);
        }
        for [start, end] in [[0, 1], [1, 2], [2, 3], [3, 0]] {
            let linedef = builder.linedef(start, end, Some(side), None);
            builder.seg(start, end, linedef, 0);
        }
        builder.subsector(0, 4);
        builder.node(DoomFixtureNode {
            point: [0, 0],
            delta: [0, 64],
            right_bbox: [0, 64, 32, -32],
            left_bbox: [0, 64, 32, -32],
            right_child: DoomBspChild::Subsector(0),
            left_child: DoomBspChild::Subsector(0),
        });
        builder.build().unwrap()
    }
}
