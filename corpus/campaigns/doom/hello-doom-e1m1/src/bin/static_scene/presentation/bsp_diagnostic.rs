//! Full-submission visualization of a Doom-private shadow BSP observation.
//!
//! This module never removes a draw. It associates each original declaration
//! with a semantic-family texture and a conservative disposition tint so a
//! human can compare the shadow observation with the actually rendered scene.

use std::collections::{BTreeMap, BTreeSet};

use doom_geometry_provider::{DoomSurfacePlane, DoomWallTextureRole};
use doom_map_provider::DoomMapCore;
use hello_doom_e1m1::{
    classify_static_draw_frustum_rejection, DoomComparativeEmbedding, StaticDrawAabb,
    StaticDrawPlanEntry, StaticDrawSource,
};
use raster_image_corpus::{decode_png, prepare_renderer_texture, DecodeLimits, TextureUse};
use tokimu::{
    Color, DrawMeshCommand, DrawMeshMaterialOverrideCommand, Material, MaterialHandle,
    MaterialOverride, PlatformResult, RenderCommand, TextureAddressMode, TextureFilter,
    TextureHandle, TextureSampler, WgpuBackend,
};
use tokimu_core::math::Mat4;

use crate::{observe_doom_seg_classic_bsp, observer_doom_source_pose, ObserverLook, SpawnObserver};

const BSP_DIAGNOSTIC_TEXTURE_BASE: u64 = 9_100_000;
const BSP_DIAGNOSTIC_MATERIAL_BASE: u64 = 9_100_100;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BspDiagnosticFamily {
    Floor,
    Ceiling,
    Wall,
    Door,
    TwoSidedBoundary,
    HeightTransitionBoundary,
    MaskedMiddle,
    Skybox,
}

impl BspDiagnosticFamily {
    const ALL: [Self; 8] = [
        Self::Floor,
        Self::Ceiling,
        Self::Wall,
        Self::Door,
        Self::TwoSidedBoundary,
        Self::HeightTransitionBoundary,
        Self::MaskedMiddle,
        Self::Skybox,
    ];

    const fn index(self) -> u64 {
        match self {
            Self::Floor => 0,
            Self::Ceiling => 1,
            Self::Wall => 2,
            Self::Door => 3,
            Self::TwoSidedBoundary => 4,
            Self::HeightTransitionBoundary => 5,
            Self::MaskedMiddle => 6,
            Self::Skybox => 7,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Ceiling => "ceiling",
            Self::Wall => "wall",
            Self::Door => "door",
            Self::TwoSidedBoundary => "two-sided-boundary",
            Self::HeightTransitionBoundary => "height-transition-boundary",
            Self::MaskedMiddle => "masked-middle",
            Self::Skybox => "skybox",
        }
    }

    pub(crate) const fn material(self) -> MaterialHandle {
        MaterialHandle(BSP_DIAGNOSTIC_MATERIAL_BASE + self.index())
    }

    const fn texture(self) -> TextureHandle {
        TextureHandle(BSP_DIAGNOSTIC_TEXTURE_BASE + self.index())
    }

    const fn asset(self) -> &'static str {
        match self {
            Self::Floor => "PNG/Light/texture_04.png",
            Self::Ceiling => "PNG/Light/texture_07.png",
            Self::Wall => "PNG/Light/texture_03.png",
            Self::Door => "PNG/Light/texture_10.png",
            Self::TwoSidedBoundary => "PNG/Light/texture_06.png",
            Self::HeightTransitionBoundary => "PNG/Light/texture_12.png",
            Self::MaskedMiddle => "PNG/Light/texture_01.png",
            Self::Skybox => "PNG/Light/texture_13.png",
        }
    }

    const fn category_tint(self) -> Color {
        match self {
            Self::Floor => Color::rgb(0.05, 1.0, 0.18),
            Self::Ceiling => Color::rgb(1.0, 0.88, 0.08),
            Self::Wall => Color::rgb(1.0, 0.03, 0.08),
            Self::Door => Color::rgb(1.0, 0.45, 0.02),
            Self::TwoSidedBoundary => Color::rgb(0.02, 1.0, 1.0),
            Self::HeightTransitionBoundary => Color::rgb(0.08, 0.28, 1.0),
            Self::MaskedMiddle => Color::rgb(1.0, 0.02, 0.85),
            Self::Skybox => Color::rgb(0.04, 0.12, 0.55),
        }
    }

    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Floor => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_04.png")
            }
            Self::Ceiling => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_07.png")
            }
            Self::Wall => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_03.png")
            }
            Self::Door => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_10.png")
            }
            Self::TwoSidedBoundary => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_06.png")
            }
            Self::HeightTransitionBoundary => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_12.png")
            }
            Self::MaskedMiddle => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_01.png")
            }
            Self::Skybox => {
                include_bytes!("../../../../../../../assets/PNG/Light/texture_13.png")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BspDiagnosticDisposition {
    Accepted,
    RejectedSolidRange,
    RejectedOutsideFrustum,
    UnresolvedFailOpen,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BspDiagnosticReason {
    OrderedSourceSegAdmitted,
    SourcePlaneSubsectorReachedOccurrenceUnproven,
    PositiveTerminalSolidRange,
    PreparedGeometryOutsideFrustum,
    SourcePlaneChildBoundsOutsideFov,
    SourceWallSegBoundsOutsideFov,
    PreparedGeometryFrustumVetoedPlaneRejection,
    ProjectionOrTraversalAmbiguous,
    PresentationGlobal,
}

impl BspDiagnosticReason {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::OrderedSourceSegAdmitted => "ordered-source-seg-admitted",
            Self::SourcePlaneSubsectorReachedOccurrenceUnproven => {
                "source-plane-subsector-reached-occurrence-unproven"
            }
            Self::PositiveTerminalSolidRange => "solid-range-covered",
            Self::PreparedGeometryOutsideFrustum => "prepared-geometry-outside-frustum",
            Self::SourcePlaneChildBoundsOutsideFov => "source-plane-child-seg-bounds-outside-fov",
            Self::SourceWallSegBoundsOutsideFov => "source-wall-seg-bounds-outside-fov",
            Self::PreparedGeometryFrustumVetoedPlaneRejection => {
                "prepared-geometry-frustum-vetoed-plane-rejection"
            }
            Self::ProjectionOrTraversalAmbiguous => "projection-or-traversal-ambiguous",
            Self::PresentationGlobal => "presentation-global-not-bsp-classified",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BspDiagnosticFocus {
    All,
    Accepted,
    Rejected,
    Unresolved,
}

impl BspDiagnosticFocus {
    pub(crate) fn from_args(args: &[String], enabled: bool) -> PlatformResult<Self> {
        let values = args
            .iter()
            .filter_map(|argument| argument.strip_prefix("--bsp-diagnostic-focus="))
            .collect::<Vec<_>>();
        if !enabled && !values.is_empty() {
            return Err("--bsp-diagnostic-focus requires --bsp-diagnostic-full".into());
        }
        match values.as_slice() {
            [] => Ok(Self::All),
            [value] => match *value {
                "all" => Ok(Self::All),
                "accepted" => Ok(Self::Accepted),
                "rejected" => Ok(Self::Rejected),
                "unresolved" => Ok(Self::Unresolved),
                _ => Err(
                    "--bsp-diagnostic-focus expects all, accepted, rejected, or unresolved".into(),
                ),
            },
            _ => Err("choose only one --bsp-diagnostic-focus value".into()),
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Unresolved => "unresolved",
        }
    }

    const fn emphasizes(self, disposition: BspDiagnosticDisposition) -> bool {
        match self {
            Self::All => true,
            Self::Accepted => matches!(disposition, BspDiagnosticDisposition::Accepted),
            Self::Rejected => matches!(
                disposition,
                BspDiagnosticDisposition::RejectedSolidRange
                    | BspDiagnosticDisposition::RejectedOutsideFrustum
            ),
            Self::Unresolved => matches!(disposition, BspDiagnosticDisposition::UnresolvedFailOpen),
        }
    }
}

impl BspDiagnosticDisposition {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::RejectedSolidRange => "rejected-solid-range",
            Self::RejectedOutsideFrustum => "rejected-outside-frustum",
            Self::UnresolvedFailOpen => "unresolved-fail-open",
        }
    }

    fn tint(self, family: BspDiagnosticFamily) -> Color {
        match self {
            Self::Accepted => family.category_tint(),
            Self::RejectedSolidRange | Self::RejectedOutsideFrustum => {
                let category = family.category_tint();
                Color::rgb(
                    0.08 + category.r * 0.14,
                    0.08 + category.g * 0.14,
                    0.08 + category.b * 0.14,
                )
            }
            // Pattern retains the family while purple means classification is
            // unavailable. It must never be mistaken for a terminal reject.
            Self::UnresolvedFailOpen => Color::rgb(0.72, 0.05, 0.92),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BspDiagnosticDraw {
    pub(crate) family: BspDiagnosticFamily,
    pub(crate) disposition: BspDiagnosticDisposition,
    pub(crate) reason: BspDiagnosticReason,
}

#[derive(Clone, Debug)]
pub(crate) struct BspDiagnosticManifest {
    pub(crate) opaque: Vec<BspDiagnosticDraw>,
    pub(crate) cutouts: Vec<BspDiagnosticDraw>,
    counts: BTreeMap<(BspDiagnosticFamily, BspDiagnosticDisposition), usize>,
    reason_counts: BTreeMap<BspDiagnosticReason, usize>,
    leaves_visited: usize,
    far_children_pruned: usize,
    far_children_outside_fov: usize,
    far_children_fail_open: usize,
}

impl BspDiagnosticManifest {
    pub(crate) fn report(&self) -> String {
        let counts = self
            .counts
            .iter()
            .map(|((family, disposition), count)| {
                format!("{}:{}={count}", family.label(), disposition.label())
            })
            .collect::<Vec<_>>()
            .join(",");
        let reasons = self
            .reason_counts
            .iter()
            .map(|(reason, count)| format!("{}={count}", reason.label()))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "opaque={}; cutout={}; conserved={}; bsp=[leaves-visited:{},far-pruned:{},far-outside-fov:{},far-fail-open:{}]; counts=[{}]; reasons=[{}]",
            self.opaque.len(),
            self.cutouts.len(),
            self.opaque.len() + self.cutouts.len(),
            self.leaves_visited,
            self.far_children_pruned,
            self.far_children_outside_fov,
            self.far_children_fail_open,
            counts,
            reasons,
        )
    }
}

pub(crate) fn describe_bsp_diagnostic_hit(
    manifest: &BspDiagnosticManifest,
    draw: &StaticDrawPlanEntry,
    cutout: bool,
    opaque_draws: &[StaticDrawPlanEntry],
    cutout_draws: &[StaticDrawPlanEntry],
) -> String {
    let diagnostic = bsp_diagnostic_hit(manifest, draw, cutout, opaque_draws, cutout_draws);
    diagnostic.map_or_else(
        || "bsp_shadow_classification=unavailable:hit-index".to_owned(),
        |diagnostic| {
            format!(
                "bsp_shadow_classification=family:{},classification:{},reason:{}",
                diagnostic.family.label(),
                diagnostic.disposition.label(),
                diagnostic.reason.label(),
            )
        },
    )
}

pub(crate) fn bsp_diagnostic_hit(
    manifest: &BspDiagnosticManifest,
    draw: &StaticDrawPlanEntry,
    cutout: bool,
    opaque_draws: &[StaticDrawPlanEntry],
    cutout_draws: &[StaticDrawPlanEntry],
) -> Option<BspDiagnosticDraw> {
    if cutout {
        cutout_draws
            .iter()
            .position(|candidate| std::ptr::eq(candidate, draw))
            .and_then(|index| manifest.cutouts.get(index))
    } else {
        opaque_draws
            .iter()
            .position(|candidate| std::ptr::eq(candidate, draw))
            .and_then(|index| manifest.opaque.get(index))
    }
    .copied()
}

pub(crate) fn upload_bsp_diagnostic_materials(renderer: &mut WgpuBackend) -> PlatformResult<()> {
    for family in BspDiagnosticFamily::ALL {
        let decoded = decode_png(family.bytes(), DecodeLimits::default())?;
        let prepared = prepare_renderer_texture(&decoded, TextureUse::ColorSrgb)
            .map_err(std::io::Error::other)?;
        renderer.create_texture_rgba8(
            family.texture(),
            tokimu::Rgba8TextureDescriptor::new(
                prepared.texture.width,
                prepared.texture.height,
                tokimu::Rgba8TextureColorSpace::Srgb,
            ),
            &prepared.texture.rgba8,
        )?;
        renderer.upload_material(
            family.material(),
            &Material::new(
                format!("e1m1-bsp-diagnostic-{}", family.label()),
                Color::rgb(1.0, 1.0, 1.0),
            )
            .with_texture(family.texture())
            .with_texture_sampler(TextureSampler {
                filter: TextureFilter::Point,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Repeat,
            }),
        )?;
    }
    Ok(())
}

pub(crate) fn bsp_diagnostic_legend() -> String {
    let families = BspDiagnosticFamily::ALL
        .iter()
        .map(|family| format!("{}={}", family.label(), family.asset()))
        .collect::<Vec<_>>()
        .join(",");
    "families=[".to_owned()
        + &families
        + "]; category-colors=[floor=green,ceiling=yellow,wall=red,door=orange,two-sided=cyan,height-transition=blue,masked-middle=magenta,skybox=dark-blue]; dispositions=[accepted=bright-category,rejected-solid-range=dark-desaturated-category,rejected-outside-frustum=dark-desaturated-category,unresolved-fail-open=purple]"
}

pub(crate) fn bsp_diagnostic_command(
    draw: DrawMeshCommand,
    diagnostic: BspDiagnosticDraw,
    focus: BspDiagnosticFocus,
    family_colors_only: bool,
) -> PlatformResult<RenderCommand> {
    let draw = DrawMeshCommand {
        material: diagnostic.family.material(),
        ..draw
    };
    Ok(RenderCommand::DrawMeshMaterialOverride(
        DrawMeshMaterialOverrideCommand {
            draw,
            material_override: MaterialOverride::with_replacement_color(
                if focus.emphasizes(diagnostic.disposition) {
                    if family_colors_only {
                        diagnostic.family.category_tint()
                    } else {
                        diagnostic.disposition.tint(diagnostic.family)
                    }
                } else {
                    Color::rgb(0.08, 0.08, 0.08)
                },
            )?,
        },
    ))
}

pub(crate) fn observe_bsp_diagnostic_manifest(
    map: &DoomMapCore,
    opaque: &[StaticDrawPlanEntry],
    cutouts: &[StaticDrawPlanEntry],
    observer: SpawnObserver,
    look: ObserverLook,
    embedding: DoomComparativeEmbedding,
    view_projection: Mat4,
) -> PlatformResult<BspDiagnosticManifest> {
    let (viewer, heading) = observer_doom_source_pose(observer, look, embedding);
    observe_bsp_diagnostic_manifest_at_source(
        map,
        opaque,
        cutouts,
        viewer,
        heading,
        Some(view_projection),
    )
}

pub(crate) fn observe_bsp_diagnostic_manifest_at_source(
    map: &DoomMapCore,
    opaque: &[StaticDrawPlanEntry],
    cutouts: &[StaticDrawPlanEntry],
    viewer: [i16; 2],
    heading: f64,
    view_projection: Option<Mat4>,
) -> PlatformResult<BspDiagnosticManifest> {
    let watched = (0..map.subsectors.len())
        .filter_map(|index| u16::try_from(index).ok())
        .collect::<BTreeSet<_>>();
    let observation = observe_doom_seg_classic_bsp(map, viewer, heading, &watched)?;
    let solid_pruned = elided_subsectors(&observation.watched_subsector_elisions, "solid-range");
    let outside_fov = elided_subsectors(&observation.watched_subsector_elisions, "outside-fov");
    let seg_subsectors = seg_subsector_map(map);
    let door_sectors = manual_door_sectors(map);

    let classify = |draw: &StaticDrawPlanEntry, cutout: bool| {
        classify_draw(
            map,
            draw,
            &observation.admitted_seg_records,
            &observation.visited_subsectors,
            &solid_pruned,
            &outside_fov,
            &seg_subsectors,
            &door_sectors,
            cutout,
            view_projection,
        )
    };
    let opaque = opaque
        .iter()
        .map(|draw| classify(draw, false))
        .collect::<Vec<_>>();
    let cutouts = cutouts
        .iter()
        .map(|draw| classify(draw, true))
        .collect::<Vec<_>>();
    let mut counts = BTreeMap::new();
    let mut reason_counts = BTreeMap::new();
    for diagnostic in opaque.iter().chain(cutouts.iter()) {
        *counts
            .entry((diagnostic.family, diagnostic.disposition))
            .or_default() += 1;
        *reason_counts.entry(diagnostic.reason).or_default() += 1;
    }
    Ok(BspDiagnosticManifest {
        opaque,
        cutouts,
        counts,
        reason_counts,
        leaves_visited: observation.leaves_visited,
        far_children_pruned: observation.far_children_pruned,
        far_children_outside_fov: observation.far_children_outside_fov,
        far_children_fail_open: observation.far_children_fail_open,
    })
}

#[allow(clippy::too_many_arguments)]
fn classify_draw(
    map: &DoomMapCore,
    draw: &StaticDrawPlanEntry,
    admitted_segs: &BTreeSet<u32>,
    visited_subsectors: &BTreeSet<u16>,
    solid_pruned: &BTreeSet<u16>,
    outside_fov: &BTreeSet<u16>,
    seg_subsectors: &BTreeMap<u32, u16>,
    door_sectors: &BTreeSet<u32>,
    cutout: bool,
    view_projection: Option<Mat4>,
) -> BspDiagnosticDraw {
    match draw.source {
        StaticDrawSource::Flat {
            source_subsector,
            plane,
            ..
        } => {
            let subsector = u16::try_from(source_subsector.record_index).ok();
            let prepared_geometry_definitely_outside =
                view_projection.and_then(|view_projection| {
                    StaticDrawAabb::from_positions(&draw.mesh.positions).map(|bounds| {
                        classify_static_draw_frustum_rejection(bounds, view_projection).is_some()
                    })
                });
            let (disposition, reason) = subsector.map_or(
                (
                    BspDiagnosticDisposition::UnresolvedFailOpen,
                    BspDiagnosticReason::ProjectionOrTraversalAmbiguous,
                ),
                |s| {
                    classify_plane_participation(
                        prepared_geometry_definitely_outside,
                        visited_subsectors.contains(&s),
                        solid_pruned.contains(&s),
                        outside_fov.contains(&s),
                    )
                },
            );
            BspDiagnosticDraw {
                family: match plane {
                    DoomSurfacePlane::Floor => BspDiagnosticFamily::Floor,
                    DoomSurfacePlane::Ceiling => BspDiagnosticFamily::Ceiling,
                },
                disposition,
                reason,
            }
        }
        StaticDrawSource::Wall {
            source_linedef,
            source_sidedef,
            source_sector,
            role,
        } => {
            let matching = matching_side_segs(
                map,
                source_linedef.record_index,
                source_sidedef.record_index,
            );
            let accepted = matching.iter().any(|seg| admitted_segs.contains(seg));
            let rejected = !matching.is_empty()
                && matching.iter().all(|seg| {
                    seg_subsectors
                        .get(seg)
                        .is_some_and(|subsector| solid_pruned.contains(subsector))
                });
            let outside_fov = !matching.is_empty()
                && matching.iter().all(|seg| {
                    seg_subsectors
                        .get(seg)
                        .is_some_and(|subsector| outside_fov.contains(subsector))
                });
            let (disposition, reason) = if accepted {
                (
                    BspDiagnosticDisposition::Accepted,
                    BspDiagnosticReason::OrderedSourceSegAdmitted,
                )
            } else if rejected {
                (
                    BspDiagnosticDisposition::RejectedSolidRange,
                    BspDiagnosticReason::PositiveTerminalSolidRange,
                )
            } else if outside_fov {
                (
                    BspDiagnosticDisposition::UnresolvedFailOpen,
                    BspDiagnosticReason::SourceWallSegBoundsOutsideFov,
                )
            } else {
                (
                    BspDiagnosticDisposition::UnresolvedFailOpen,
                    BspDiagnosticReason::ProjectionOrTraversalAmbiguous,
                )
            };
            let linedef = map
                .linedefs
                .iter()
                .find(|line| line.source.record_index == source_linedef.record_index);
            let two_sided = linedef
                .is_some_and(|line| line.right_sidedef.is_some() && line.left_sidedef.is_some());
            let height_transition =
                linedef.is_some_and(|line| linedef_has_height_transition(map, line));
            let family = if door_sectors.contains(&source_sector.record_index)
                || linedef.is_some_and(|line| line.special == 1)
            {
                BspDiagnosticFamily::Door
            } else if cutout {
                BspDiagnosticFamily::MaskedMiddle
            } else if height_transition {
                BspDiagnosticFamily::HeightTransitionBoundary
            } else if two_sided
                || matches!(
                    role,
                    DoomWallTextureRole::Upper | DoomWallTextureRole::Lower
                )
            {
                BspDiagnosticFamily::TwoSidedBoundary
            } else {
                BspDiagnosticFamily::Wall
            };
            BspDiagnosticDraw {
                family,
                disposition,
                reason,
            }
        }
    }
}

fn classify_plane_participation(
    prepared_geometry_definitely_outside: Option<bool>,
    subsector_reached: bool,
    solid_pruned: bool,
    outside_fov: bool,
) -> (BspDiagnosticDisposition, BspDiagnosticReason) {
    if prepared_geometry_definitely_outside == Some(true) {
        return (
            BspDiagnosticDisposition::RejectedOutsideFrustum,
            BspDiagnosticReason::PreparedGeometryOutsideFrustum,
        );
    }
    if subsector_reached {
        return (
            BspDiagnosticDisposition::UnresolvedFailOpen,
            BspDiagnosticReason::SourcePlaneSubsectorReachedOccurrenceUnproven,
        );
    }
    if solid_pruned {
        return if prepared_geometry_definitely_outside == Some(false) {
            (
                BspDiagnosticDisposition::UnresolvedFailOpen,
                BspDiagnosticReason::PreparedGeometryFrustumVetoedPlaneRejection,
            )
        } else {
            (
                BspDiagnosticDisposition::UnresolvedFailOpen,
                BspDiagnosticReason::ProjectionOrTraversalAmbiguous,
            )
        };
    }
    if outside_fov {
        return (
            BspDiagnosticDisposition::UnresolvedFailOpen,
            BspDiagnosticReason::SourcePlaneChildBoundsOutsideFov,
        );
    }
    (
        BspDiagnosticDisposition::UnresolvedFailOpen,
        BspDiagnosticReason::ProjectionOrTraversalAmbiguous,
    )
}

fn linedef_has_height_transition(
    map: &DoomMapCore,
    linedef: &doom_map_provider::DoomLinedef,
) -> bool {
    let sectors = [linedef.right_sidedef, linedef.left_sidedef]
        .into_iter()
        .flatten()
        .filter_map(|side| map.sidedefs.get(usize::from(side)))
        .filter_map(|side| map.sectors.get(usize::from(side.sector)))
        .collect::<Vec<_>>();
    let [front, back] = sectors.as_slice() else {
        return false;
    };
    front.floor_height != back.floor_height || front.ceiling_height != back.ceiling_height
}

fn matching_side_segs(map: &DoomMapCore, linedef_record: u32, sidedef_record: u32) -> Vec<u32> {
    map.segs
        .iter()
        .filter_map(|seg| {
            let linedef = map.linedefs.get(usize::from(seg.linedef))?;
            if linedef.source.record_index != linedef_record {
                return None;
            }
            let side = match seg.direction {
                0 => linedef.right_sidedef,
                1 => linedef.left_sidedef,
                _ => None,
            }?;
            (map.sidedefs[usize::from(side)].source.record_index == sidedef_record)
                .then_some(seg.source.record_index)
        })
        .collect()
}

fn seg_subsector_map(map: &DoomMapCore) -> BTreeMap<u32, u16> {
    let mut result = BTreeMap::new();
    for (subsector_index, subsector) in map.subsectors.iter().enumerate() {
        let Some(subsector_index) = u16::try_from(subsector_index).ok() else {
            continue;
        };
        let first = usize::from(subsector.first_seg);
        let end = first + usize::from(subsector.seg_count);
        for seg in &map.segs[first..end] {
            result.insert(seg.source.record_index, subsector_index);
        }
    }
    result
}

fn manual_door_sectors(map: &DoomMapCore) -> BTreeSet<u32> {
    map.linedefs
        .iter()
        .filter(|line| line.special == 1)
        .filter_map(|line| line.left_sidedef)
        .filter_map(|side| map.sidedefs.get(usize::from(side)))
        .filter_map(|side| map.sectors.get(usize::from(side.sector)))
        .map(|sector| sector.source.record_index)
        .collect()
}

fn elided_subsectors(records: &[String], reason: &str) -> BTreeSet<u16> {
    let mut result = BTreeSet::new();
    for record in records.iter().filter(|record| {
        record
            .split(':')
            .any(|part| part == format!("reason={reason}"))
    }) {
        let Some(values) = record
            .split(':')
            .find_map(|part| part.strip_prefix("subsectors=["))
            .and_then(|part| part.strip_suffix(']'))
        else {
            continue;
        };
        result.extend(
            values
                .split(',')
                .filter_map(|value| value.trim().parse::<u16>().ok()),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        classify_plane_participation, elided_subsectors, BspDiagnosticDisposition,
        BspDiagnosticFocus, BspDiagnosticReason,
    };

    #[test]
    fn plane_participation_separates_geometric_rejection_from_source_evidence() {
        assert_eq!(
            classify_plane_participation(Some(true), true, false, false),
            (
                BspDiagnosticDisposition::RejectedOutsideFrustum,
                BspDiagnosticReason::PreparedGeometryOutsideFrustum,
            )
        );
        assert_eq!(
            classify_plane_participation(Some(false), true, false, false),
            (
                BspDiagnosticDisposition::UnresolvedFailOpen,
                BspDiagnosticReason::SourcePlaneSubsectorReachedOccurrenceUnproven,
            )
        );
        assert_eq!(
            classify_plane_participation(Some(false), false, true, false),
            (
                BspDiagnosticDisposition::UnresolvedFailOpen,
                BspDiagnosticReason::PreparedGeometryFrustumVetoedPlaneRejection,
            )
        );
        assert_eq!(
            classify_plane_participation(None, false, true, false),
            (
                BspDiagnosticDisposition::UnresolvedFailOpen,
                BspDiagnosticReason::ProjectionOrTraversalAmbiguous,
            )
        );
    }

    #[test]
    fn extracts_only_positive_solid_range_elisions() {
        let records = vec![
            "node=4:reason=solid-range:subsectors=[2, 7]:interval=Some([0, 3]):covering-range=Some([0, 5])".to_owned(),
            "node=8:reason=outside-fov:subsectors=[9]:interval=None:covering-range=None".to_owned(),
        ];
        assert_eq!(
            elided_subsectors(&records, "solid-range")
                .into_iter()
                .collect::<Vec<_>>(),
            vec![2, 7]
        );
    }

    #[test]
    fn focus_controls_require_the_full_diagnostic_and_never_imply_membership() {
        let args = vec!["--bsp-diagnostic-focus=rejected".to_owned()];
        assert!(BspDiagnosticFocus::from_args(&args, false).is_err());
        assert_eq!(
            BspDiagnosticFocus::from_args(&args, true).unwrap(),
            BspDiagnosticFocus::Rejected
        );
        assert_eq!(
            BspDiagnosticFocus::from_args(&[], true).unwrap(),
            BspDiagnosticFocus::All
        );
    }
}
