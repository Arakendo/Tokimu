//! Application-local lowering for the selected static E1M1 flat policy.
//!
//! Doom records stay at this corpus edge. The renderer receives only the
//! resulting ordinary mesh with supplied texture coordinates.

use doom_geometry_provider::{
    DoomMiddleTextureObservation, DoomSkySurfaceObservation, DoomSurfacePlane, DoomSurfaceTriangle,
    DoomTextureExtent, DoomTexturedWallTriangle, DoomWallTextureRole,
};
use doom_map_provider::DoomSourceRecord;
use doom_raster_provider::{
    compose_doom_texture, decode_doom_flat, decode_doom_raster_globals,
    decode_doom_texture_catalog, indexed_image_from_doom_flat, lower_doom_indexed_image,
    DoomFlatDecodeLimits, DoomIndexedImage, DoomPatchDecodeLimits, DoomRasterDecodeLimits,
    DoomTextureComposeLimits, DoomTextureDecodeLimits,
};
use doom_wad_package::select_doom_episode_map;
use doom_wad_provider::WadManifest;
use thiserror::Error;
use tokimu::{
    Color, Material, MaterialHandle, Mesh, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    TextureAddressMode, TextureFilter, TextureHandle, TextureSampler,
};

/// Source identity retained beside a submitted mesh rather than embedded in a
/// renderer type or material label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticFlatSource {
    pub subsector: DoomSourceRecord,
    pub sector: DoomSourceRecord,
    pub plane: DoomSurfacePlane,
    pub flat_name: String,
}

/// One opaque-candidate flat triangle under the Slice 5B static map-axis
/// policy. It does not mean that the flat has been selected for drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticFlatMesh {
    pub source: StaticFlatSource,
    pub mesh: Mesh,
}

/// The first static-scene flat assembly. Sky observations are retained rather
/// than inferred from a flat name, so a later sky policy can consume the
/// original source classification without changing the submitted mesh list.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticFlatAssembly {
    pub opaque_flats: Vec<StaticFlatMesh>,
    pub omitted_sky: Vec<DoomSkySurfaceObservation>,
    /// Individually retained zero-area candidates; no normal is fabricated.
    pub omitted_degenerate: Vec<StaticFlatSource>,
}

/// One opaque-candidate wall mesh under the source-coordinate normalization
/// policy. Its texture name is evidence for a later application material
/// lookup, not a renderer material identity.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticWallMesh {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub role: DoomWallTextureRole,
    pub texture_name: String,
    pub mesh: Mesh,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticWallAssembly {
    pub opaque_walls: Vec<StaticWallMesh>,
    pub omitted_masked_middles: Vec<DoomMiddleTextureObservation>,
    pub omitted_degenerate: Vec<DoomTexturedWallTriangle>,
}

/// Consumer-owned eligibility for the first opaque static scene. It neither
/// uploads pixels nor treats source coverage as a renderer alpha policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticTextureEligibility {
    Opaque(StaticOpaqueTexture),
    DeferredAlpha {
        texture_name: String,
        uncovered_pixels: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticOpaqueTexture {
    pub texture_name: String,
    pub descriptor: Rgba8TextureDescriptor,
    pub sampler: TextureSampler,
    pub selected_palette: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlatExtent {
    pub width: u16,
    pub height: u16,
}

impl FlatExtent {
    pub const E1M1: Self = Self {
        width: 64,
        height: 64,
    };
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum StaticFlatLoweringError {
    #[error("flat extent must be non-zero; got {width} by {height}")]
    ZeroExtent { width: u16, height: u16 },
    #[error("wall texture `{name}` has no supplied source extent")]
    MissingWallTextureExtent { name: String },
    #[error("surface position {vertex} has a non-finite {component} coordinate")]
    NonFinitePosition {
        vertex: usize,
        component: &'static str,
    },
    #[error("surface triangle is degenerate and has no stable facing normal")]
    DegenerateTriangle,
    #[error("renderer mesh validation failed: {0}")]
    Mesh(#[from] tokimu::MeshValidationError),
}

/// Source-traceable E1M1 flat preparation before texture upload or rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedE1m1Flats {
    pub map_name: String,
    pub flat_assembly: StaticFlatAssembly,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedE1m1Walls {
    pub map_name: String,
    pub wall_assembly: StaticWallAssembly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedStaticTexture {
    pub eligibility: StaticTextureEligibility,
    pub rgba8: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticTextureSourceKind {
    Flat,
    Wall,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticTextureUpload {
    pub source_kind: StaticTextureSourceKind,
    pub source_name: String,
    pub texture: TextureHandle,
    pub material: MaterialHandle,
    pub descriptor: Rgba8TextureDescriptor,
    pub rgba8: Vec<u8>,
    pub material_value: Material,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticDrawPlanEntry {
    pub mesh: Mesh,
    pub material: MaterialHandle,
    pub source_label: String,
}

/// Aggregate impact of the explicitly omitted zero-area candidates. This is
/// evidence for the Slice 5B escalation rule, not renderer input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticDegenerateOmissionImpact {
    pub flat_candidates: usize,
    pub flat_subsectors: usize,
    pub flat_sectors: usize,
    pub fully_omitted_flat_subsectors: usize,
    pub wall_candidates: usize,
    pub wall_linedefs: usize,
    pub wall_sidedefs: usize,
    pub wall_sectors: usize,
    pub fully_omitted_wall_linedefs: usize,
}

/// One source-level wall omission summary. It preserves the vertical span that
/// the lowerer attempted, making a collapsed authored span distinguishable
/// from a later conversion failure.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticDegenerateWallOmission {
    pub linedef_index: u32,
    pub sidedef_index: u32,
    pub sector_index: u32,
    pub texture_name: String,
    pub role: DoomWallTextureRole,
    pub minimum_height: f64,
    pub maximum_height: f64,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum StaticDrawPlanError {
    #[error("no opaque material was prepared for {source_kind:?} texture `{source_name}`")]
    MissingMaterial {
        source_kind: StaticTextureSourceKind,
        source_name: String,
    },
}

/// Creates deterministic opaque-only renderer inputs. Deferred-alpha sources
/// intentionally receive no handle or material.
pub fn build_static_texture_uploads(
    flat_textures: &[PreparedStaticTexture],
    wall_textures: &[PreparedStaticTexture],
) -> Vec<StaticTextureUpload> {
    let mut uploads = Vec::new();
    for (kind, textures) in [
        (StaticTextureSourceKind::Flat, flat_textures),
        (StaticTextureSourceKind::Wall, wall_textures),
    ] {
        let mut textures = textures.iter().collect::<Vec<_>>();
        textures.sort_by_key(|texture| match &texture.eligibility {
            StaticTextureEligibility::Opaque(texture) => texture.texture_name.as_str(),
            StaticTextureEligibility::DeferredAlpha { texture_name, .. } => texture_name.as_str(),
        });
        for texture in textures {
            let StaticTextureEligibility::Opaque(opaque) = &texture.eligibility else {
                continue;
            };
            let index = uploads.len() as u64 + 1;
            uploads.push(StaticTextureUpload {
                source_kind: kind,
                source_name: opaque.texture_name.clone(),
                texture: TextureHandle(index),
                material: MaterialHandle(index),
                descriptor: opaque.descriptor,
                rgba8: texture.rgba8.clone(),
                material_value: Material::new(
                    format!("doom-{kind:?}-{}", opaque.texture_name),
                    Color::rgb(1.0, 1.0, 1.0),
                )
                .with_texture(TextureHandle(index))
                .with_texture_sampler(opaque.sampler),
            });
        }
    }
    uploads
}

/// Lowers prepared application evidence to ordinary mesh/material draw inputs.
/// The renderer receives neither Doom identifiers nor source classifications.
pub fn build_static_draw_plan(
    flats: &PreparedE1m1Flats,
    walls: &PreparedE1m1Walls,
    uploads: &[StaticTextureUpload],
) -> Result<Vec<StaticDrawPlanEntry>, StaticDrawPlanError> {
    let material_for = |kind, name: &str| {
        uploads
            .iter()
            .find(|upload| upload.source_kind == kind && upload.source_name == name)
            .map(|upload| upload.material)
            .ok_or_else(|| StaticDrawPlanError::MissingMaterial {
                source_kind: kind,
                source_name: name.to_owned(),
            })
    };
    let mut draws = Vec::with_capacity(
        flats.flat_assembly.opaque_flats.len() + walls.wall_assembly.opaque_walls.len(),
    );
    for flat in &flats.flat_assembly.opaque_flats {
        draws.push(StaticDrawPlanEntry {
            mesh: flat.mesh.clone(),
            material: material_for(StaticTextureSourceKind::Flat, &flat.source.flat_name)?,
            source_label: format!(
                "flat:{}:{}",
                flat.source.sector.record_index, flat.source.flat_name
            ),
        });
    }
    for wall in &walls.wall_assembly.opaque_walls {
        draws.push(StaticDrawPlanEntry {
            mesh: wall.mesh.clone(),
            material: material_for(StaticTextureSourceKind::Wall, &wall.texture_name)?,
            source_label: format!(
                "wall:{}:{}",
                wall.source_linedef.record_index, wall.texture_name
            ),
        });
    }
    Ok(draws)
}

/// Counts the independently identifiable source records affected by the
/// intentionally omitted degenerate candidates. A caller can use this compact
/// report to decide whether a local omission remains acceptable evidence or
/// has become systemic topology loss.
pub fn static_degenerate_omission_impact(
    flats: &PreparedE1m1Flats,
    walls: &PreparedE1m1Walls,
) -> StaticDegenerateOmissionImpact {
    let source_key = |source: DoomSourceRecord| (source.lump_index, source.record_index);
    let flat_subsectors = flats
        .flat_assembly
        .omitted_degenerate
        .iter()
        .map(|source| source_key(source.subsector))
        .collect::<std::collections::BTreeSet<_>>();
    let mut flat_subsector_totals = std::collections::BTreeMap::<(u32, u32), usize>::new();
    let mut flat_subsector_omissions = std::collections::BTreeMap::<(u32, u32), usize>::new();
    for flat in &flats.flat_assembly.opaque_flats {
        *flat_subsector_totals
            .entry(source_key(flat.source.subsector))
            .or_default() += 1;
    }
    for flat in &flats.flat_assembly.omitted_degenerate {
        let key = source_key(flat.subsector);
        *flat_subsector_totals.entry(key).or_default() += 1;
        *flat_subsector_omissions.entry(key).or_default() += 1;
    }
    let flat_sectors = flats
        .flat_assembly
        .omitted_degenerate
        .iter()
        .map(|source| source_key(source.sector))
        .collect::<std::collections::BTreeSet<_>>();
    let wall_linedefs = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .map(|wall| source_key(wall.source_linedef))
        .collect::<std::collections::BTreeSet<_>>();
    let mut wall_linedef_totals = std::collections::BTreeMap::<(u32, u32), usize>::new();
    let mut wall_linedef_omissions = std::collections::BTreeMap::<(u32, u32), usize>::new();
    for wall in &walls.wall_assembly.opaque_walls {
        *wall_linedef_totals
            .entry(source_key(wall.source_linedef))
            .or_default() += 1;
    }
    for wall in &walls.wall_assembly.omitted_degenerate {
        let key = source_key(wall.source_linedef);
        *wall_linedef_totals.entry(key).or_default() += 1;
        *wall_linedef_omissions.entry(key).or_default() += 1;
    }
    let wall_sidedefs = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .map(|wall| source_key(wall.source_sidedef))
        .collect::<std::collections::BTreeSet<_>>();
    let wall_sectors = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .map(|wall| source_key(wall.source_sector))
        .collect::<std::collections::BTreeSet<_>>();
    StaticDegenerateOmissionImpact {
        flat_candidates: flats.flat_assembly.omitted_degenerate.len(),
        flat_subsectors: flat_subsectors.len(),
        flat_sectors: flat_sectors.len(),
        fully_omitted_flat_subsectors: flat_subsector_omissions
            .iter()
            .filter(|(key, omitted)| flat_subsector_totals.get(*key) == Some(omitted))
            .count(),
        wall_candidates: walls.wall_assembly.omitted_degenerate.len(),
        wall_linedefs: wall_linedefs.len(),
        wall_sidedefs: wall_sidedefs.len(),
        wall_sectors: wall_sectors.len(),
        fully_omitted_wall_linedefs: wall_linedef_omissions
            .iter()
            .filter(|(key, omitted)| wall_linedef_totals.get(*key) == Some(omitted))
            .count(),
    }
}

/// Stable source-linedef identities whose complete emitted wall candidate set
/// was omitted as degenerate. This is intentionally diagnostic-only so a
/// caller can inspect concrete topology without broadening render input.
pub fn fully_omitted_wall_linedef_indices(walls: &PreparedE1m1Walls) -> Vec<u32> {
    let source_key = |source: DoomSourceRecord| (source.lump_index, source.record_index);
    let mut totals = std::collections::BTreeMap::<(u32, u32), usize>::new();
    let mut omitted = std::collections::BTreeMap::<(u32, u32), usize>::new();
    for wall in &walls.wall_assembly.opaque_walls {
        *totals.entry(source_key(wall.source_linedef)).or_default() += 1;
    }
    for wall in &walls.wall_assembly.omitted_degenerate {
        let key = source_key(wall.source_linedef);
        *totals.entry(key).or_default() += 1;
        *omitted.entry(key).or_default() += 1;
    }
    omitted
        .into_iter()
        .filter_map(|(key @ (_, index), count)| (totals.get(&key) == Some(&count)).then_some(index))
        .collect()
}

/// Stable, one-per-linedef detail for the full wall-omission cases.
pub fn fully_omitted_wall_details(walls: &PreparedE1m1Walls) -> Vec<StaticDegenerateWallOmission> {
    let fully_omitted = fully_omitted_wall_linedef_indices(walls)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut details = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .filter(|wall| fully_omitted.contains(&wall.source_linedef.record_index))
        .map(|wall| {
            let heights = wall.positions.map(|position| position[1]);
            StaticDegenerateWallOmission {
                linedef_index: wall.source_linedef.record_index,
                sidedef_index: wall.source_sidedef.record_index,
                sector_index: wall.source_sector.record_index,
                texture_name: wall.texture_name.clone(),
                role: wall.role,
                minimum_height: heights.into_iter().fold(f64::INFINITY, f64::min),
                maximum_height: heights.into_iter().fold(f64::NEG_INFINITY, f64::max),
            }
        })
        .collect::<Vec<_>>();
    details.sort_by_key(|detail| detail.linedef_index);
    details.dedup_by_key(|detail| detail.linedef_index);
    details
}

/// Stable, sorted source names for the texture-upload stage. This is source
/// vocabulary at the application boundary, not a renderer handle table.
pub fn opaque_texture_names(textures: &[PreparedStaticTexture]) -> Vec<String> {
    let mut names = textures
        .iter()
        .filter_map(|texture| match &texture.eligibility {
            StaticTextureEligibility::Opaque(texture) => Some(texture.texture_name.clone()),
            StaticTextureEligibility::DeferredAlpha { .. } => None,
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

pub fn prepared_e1m1_wall_texture_names(prepared: &PreparedE1m1Walls) -> Vec<String> {
    let mut names = prepared
        .wall_assembly
        .opaque_walls
        .iter()
        .map(|wall| wall.texture_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

/// Compact deterministic preparation evidence emitted before renderer upload.
pub fn prepared_e1m1_report(
    prepared: &PreparedE1m1Flats,
    textures: &[PreparedStaticTexture],
) -> String {
    let opaque_textures = textures
        .iter()
        .filter(|texture| matches!(texture.eligibility, StaticTextureEligibility::Opaque(_)))
        .count();
    let deferred_alpha = textures.len() - opaque_textures;
    format!(
        "map={} opaque_flat_triangles={} omitted_sky_surfaces={} omitted_degenerate_flats={} prepared_textures={} opaque_textures={} deferred_alpha_textures={}",
        prepared.map_name,
        prepared.flat_assembly.opaque_flats.len(),
        prepared.flat_assembly.omitted_sky.len(),
        prepared.flat_assembly.omitted_degenerate.len(),
        textures.len(),
        opaque_textures,
        deferred_alpha,
    )
}

/// Deterministic cross-assembly report before any renderer resource allocation.
pub fn prepared_e1m1_scene_report(
    flats: &PreparedE1m1Flats,
    walls: &PreparedE1m1Walls,
    flat_textures: &[PreparedStaticTexture],
    wall_textures: &[PreparedStaticTexture],
) -> String {
    let wall_opaque = opaque_texture_names(wall_textures).len();
    let floors = flats
        .flat_assembly
        .opaque_flats
        .iter()
        .filter(|flat| flat.source.plane == DoomSurfacePlane::Floor)
        .count();
    let ceilings = flats.flat_assembly.opaque_flats.len() - floors;
    let wall_middle = walls
        .wall_assembly
        .opaque_walls
        .iter()
        .filter(|wall| wall.role == DoomWallTextureRole::Middle)
        .count();
    let wall_upper = walls
        .wall_assembly
        .opaque_walls
        .iter()
        .filter(|wall| wall.role == DoomWallTextureRole::Upper)
        .count();
    let wall_lower = walls
        .wall_assembly
        .opaque_walls
        .iter()
        .filter(|wall| wall.role == DoomWallTextureRole::Lower)
        .count();
    let degenerate = static_degenerate_omission_impact(flats, walls);
    let fully_omitted_linedefs = fully_omitted_wall_linedef_indices(walls);
    format!(
        "{} submitted_floor_triangles={floors} submitted_ceiling_triangles={ceilings} opaque_wall_triangles={} submitted_wall_middle_triangles={wall_middle} submitted_wall_upper_triangles={wall_upper} submitted_wall_lower_triangles={wall_lower} omitted_masked_middles={} omitted_degenerate_walls={} wall_texture_names={} wall_opaque_textures={} wall_deferred_alpha_textures={} degenerate_flat_subsectors={} degenerate_flat_sectors={} fully_omitted_flat_subsectors={} degenerate_wall_linedefs={} degenerate_wall_sidedefs={} degenerate_wall_sectors={} fully_omitted_wall_linedefs={} fully_omitted_wall_linedef_indices={fully_omitted_linedefs:?}",
        prepared_e1m1_report(flats, flat_textures),
        walls.wall_assembly.opaque_walls.len(),
        walls.wall_assembly.omitted_masked_middles.len(),
        walls.wall_assembly.omitted_degenerate.len(),
        prepared_e1m1_wall_texture_names(walls).len(),
        wall_opaque,
        wall_textures.len() - wall_opaque,
        degenerate.flat_subsectors,
        degenerate.flat_sectors,
        degenerate.fully_omitted_flat_subsectors,
        degenerate.wall_linedefs,
        degenerate.wall_sidedefs,
        degenerate.wall_sectors,
        degenerate.fully_omitted_wall_linedefs,
    )
}

#[derive(Debug, Error)]
pub enum E1m1PreparationError {
    #[error("E1M1 selection failed: {0}")]
    MapSelection(#[from] doom_wad_package::DoomMapSelectionError),
    #[error("E1M1 map decode failed: {0}")]
    MapDecode(#[from] doom_map_provider::DoomMapDecodeError),
    #[error("E1M1 geometry preparation failed: {0}")]
    Geometry(#[from] doom_geometry_provider::DoomGeometryError),
    #[error("E1M1 static flat lowering failed: {0}")]
    Flat(#[from] StaticFlatLoweringError),
    #[error("E1M1 raster preparation failed: {0}")]
    Raster(#[from] doom_raster_provider::DoomRasterDecodeError),
}

/// Decodes caller-selected wall texture names through the retained composition
/// provider, retaining alpha deferrals rather than choosing a fallback draw.
pub fn prepare_e1m1_wall_textures(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    names: &[String],
    raster_limits: DoomRasterDecodeLimits,
    texture_limits: DoomTextureDecodeLimits,
    patch_limits: DoomPatchDecodeLimits,
    compose_limits: DoomTextureComposeLimits,
) -> Result<Vec<PreparedStaticTexture>, E1m1PreparationError> {
    let globals = decode_doom_raster_globals(wad_bytes, manifest, raster_limits)?;
    let catalog = decode_doom_texture_catalog(wad_bytes, manifest, texture_limits)?;
    let mut names = names.to_vec();
    names.sort();
    names.dedup();
    names
        .iter()
        .map(|name| {
            let indexed = compose_doom_texture(
                wad_bytes,
                manifest,
                &catalog,
                name,
                patch_limits,
                compose_limits,
            )?;
            let lowered = lower_doom_indexed_image(&indexed, &globals.palettes[0])?;
            Ok(PreparedStaticTexture {
                eligibility: classify_static_texture(&indexed),
                rgba8: lowered.pixels,
            })
        })
        .collect()
}

/// Decodes exactly the non-sky flat names selected by a prepared scene using
/// palette zero. The result is ready for a later caller-owned texture upload.
pub fn prepare_e1m1_flat_textures(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    prepared: &PreparedE1m1Flats,
    raster_limits: DoomRasterDecodeLimits,
    flat_limits: DoomFlatDecodeLimits,
) -> Result<Vec<PreparedStaticTexture>, E1m1PreparationError> {
    let globals = decode_doom_raster_globals(wad_bytes, manifest, raster_limits)?;
    let mut names = prepared
        .flat_assembly
        .opaque_flats
        .iter()
        .map(|flat| flat.source.flat_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
        .into_iter()
        .map(|name| {
            let flat = decode_doom_flat(wad_bytes, manifest, name, flat_limits)?;
            let indexed = indexed_image_from_doom_flat(&flat);
            let lowered = lower_doom_indexed_image(&indexed, &globals.palettes[0])?;
            Ok(PreparedStaticTexture {
                eligibility: classify_static_texture(&indexed),
                rgba8: lowered.pixels,
            })
        })
        .collect()
}

/// Builds the first static-flat scene input from a caller-owned, already
/// inspected WAD. This does not acquire files, decode textures, allocate GPU
/// resources, or submit a frame.
pub fn prepare_e1m1_flats(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    limits: doom_map_provider::DoomMapDecodeLimits,
) -> Result<PreparedE1m1Flats, E1m1PreparationError> {
    let selection = select_doom_episode_map(manifest, "E1M1")?;
    let map = doom_map_provider::decode_doom_map_core(wad_bytes, &selection, limits)?;
    let paths = doom_geometry_provider::resolve_doom_subsector_bsp_paths(&map)?;
    let surfaces = doom_geometry_provider::lower_doom_subsector_surfaces(&map, &paths)?;
    let sky = doom_geometry_provider::observe_doom_sky_surfaces(&map, &paths)?;
    Ok(PreparedE1m1Flats {
        map_name: selection.map_name,
        flat_assembly: assemble_static_opaque_flats(&surfaces, &sky, FlatExtent::E1M1)?,
    })
}

/// Builds source-textured wall candidates and source-classified masked-middle
/// omissions from a caller-owned, already-inspected E1M1 WAD.
pub fn prepare_e1m1_walls(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    map_limits: doom_map_provider::DoomMapDecodeLimits,
    texture_limits: DoomTextureDecodeLimits,
) -> Result<PreparedE1m1Walls, E1m1PreparationError> {
    let selection = select_doom_episode_map(manifest, "E1M1")?;
    let map = doom_map_provider::decode_doom_map_core(wad_bytes, &selection, map_limits)?;
    let catalog = decode_doom_texture_catalog(wad_bytes, manifest, texture_limits)?;
    let extents = catalog
        .textures
        .iter()
        .map(|texture| DoomTextureExtent {
            name: texture.name.clone(),
            width: texture.width,
            height: texture.height,
        })
        .collect::<Vec<_>>();
    let walls = doom_geometry_provider::lower_doom_textured_wall_triangles(&map, &extents)?;
    let masked = doom_geometry_provider::observe_doom_two_sided_middle_textures(&map)?;
    Ok(PreparedE1m1Walls {
        map_name: selection.map_name,
        wall_assembly: assemble_static_opaque_walls(&walls, &masked, &extents)?,
    })
}

/// Lowers a Doom floor or ceiling candidate into an ordinary supplied-UV mesh.
///
/// This is intentionally a declared modern static projection rather than a
/// claim to reproduce Doom's original view-dependent plane-span renderer.
pub fn lower_static_flat_triangle(
    triangle: &DoomSurfaceTriangle,
    extent: FlatExtent,
) -> Result<StaticFlatMesh, StaticFlatLoweringError> {
    if extent.width == 0 || extent.height == 0 {
        return Err(StaticFlatLoweringError::ZeroExtent {
            width: extent.width,
            height: extent.height,
        });
    }

    let positions = [
        lower_position(triangle.positions[0], 0)?,
        lower_position(triangle.positions[1], 1)?,
        lower_position(triangle.positions[2], 2)?,
    ];
    let normal = triangle_normal(positions)?;
    let coordinates = positions
        .iter()
        .map(|[x, _, z]| [*x / f32::from(extent.width), -*z / f32::from(extent.height)])
        .collect();
    let mesh =
        Mesh::uniform_normal(positions.to_vec(), normal).with_texture_coordinates(coordinates)?;

    Ok(StaticFlatMesh {
        source: StaticFlatSource {
            subsector: triangle.source_subsector,
            sector: triangle.source_sector,
            plane: triangle.plane,
            flat_name: triangle.texture_name.clone(),
        },
        mesh,
    })
}

/// Applies the selected first-scene sky omission before lowering the remaining
/// floor and ceiling candidates. This function deliberately has no material,
/// texture upload, pipeline, or renderer dependency.
pub fn assemble_static_opaque_flats(
    surfaces: &[DoomSurfaceTriangle],
    sky: &[DoomSkySurfaceObservation],
    extent: FlatExtent,
) -> Result<StaticFlatAssembly, StaticFlatLoweringError> {
    let mut opaque_flats = Vec::with_capacity(surfaces.len());
    let mut omitted_degenerate = Vec::new();
    for surface in surfaces {
        if sky.iter().any(|observation| {
            observation.source_subsector == surface.source_subsector
                && observation.source_sector == surface.source_sector
                && observation.plane == surface.plane
        }) {
            continue;
        }
        match lower_static_flat_triangle(surface, extent) {
            Ok(flat) => opaque_flats.push(flat),
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                omitted_degenerate.push(StaticFlatSource {
                    subsector: surface.source_subsector,
                    sector: surface.source_sector,
                    plane: surface.plane,
                    flat_name: surface.texture_name.clone(),
                });
            }
            Err(error) => return Err(error),
        }
    }
    Ok(StaticFlatAssembly {
        opaque_flats,
        omitted_sky: sky.to_vec(),
        omitted_degenerate,
    })
}

/// Lowers one source-textured wall triangle without repeating the Doom pegging
/// calculation. Coordinates are normalized from the retained source texels.
pub fn lower_static_wall_triangle(
    triangle: &DoomTexturedWallTriangle,
    extent: DoomTextureExtent,
) -> Result<StaticWallMesh, StaticFlatLoweringError> {
    if extent.width == 0 || extent.height == 0 {
        return Err(StaticFlatLoweringError::ZeroExtent {
            width: extent.width,
            height: extent.height,
        });
    }
    let positions = [
        lower_position(triangle.positions[0], 0)?,
        lower_position(triangle.positions[1], 1)?,
        lower_position(triangle.positions[2], 2)?,
    ];
    let normal = triangle_normal(positions)?;
    let coordinates = triangle.texture_coordinates.map(|[u, v]| {
        [
            u as f32 / f32::from(extent.width),
            v as f32 / f32::from(extent.height),
        ]
    });
    let mesh = Mesh::uniform_normal(positions.to_vec(), normal)
        .with_texture_coordinates(coordinates.to_vec())?;
    Ok(StaticWallMesh {
        source_linedef: triangle.source_linedef,
        source_sidedef: triangle.source_sidedef,
        source_sector: triangle.source_sector,
        role: triangle.role,
        texture_name: triangle.texture_name.clone(),
        mesh,
    })
}

/// Excludes only explicitly observed two-sided masked middles before lowering
/// the remaining source-textured wall candidates.
pub fn assemble_static_opaque_walls(
    walls: &[DoomTexturedWallTriangle],
    masked_middles: &[DoomMiddleTextureObservation],
    extents: &[DoomTextureExtent],
) -> Result<StaticWallAssembly, StaticFlatLoweringError> {
    let mut opaque_walls = Vec::with_capacity(walls.len());
    let mut omitted_degenerate = Vec::new();
    for wall in walls {
        let masked_middle = wall.role == DoomWallTextureRole::Middle
            && masked_middles.iter().any(|middle| {
                middle.source_linedef == wall.source_linedef
                    && middle.source_sidedef == wall.source_sidedef
                    && middle.side == wall.side
            });
        if masked_middle {
            continue;
        }
        let extent = extents
            .iter()
            .find(|extent| extent.name == wall.texture_name)
            .cloned()
            .ok_or_else(|| StaticFlatLoweringError::MissingWallTextureExtent {
                name: wall.texture_name.clone(),
            })?;
        match lower_static_wall_triangle(wall, extent) {
            Ok(lowered) => opaque_walls.push(lowered),
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                omitted_degenerate.push(wall.clone())
            }
            Err(error) => return Err(error),
        }
    }
    Ok(StaticWallAssembly {
        opaque_walls,
        omitted_masked_middles: masked_middles.to_vec(),
        omitted_degenerate,
    })
}

/// Chooses the bounded palette-zero/sRGB/point-repeat profile only when every
/// source pixel is covered. Any hole remains an explicit AR-0023 deferral.
pub fn classify_static_texture(image: &DoomIndexedImage) -> StaticTextureEligibility {
    let uncovered_pixels = image.coverage.iter().filter(|covered| !**covered).count();
    if uncovered_pixels > 0 {
        return StaticTextureEligibility::DeferredAlpha {
            texture_name: image.texture_name.clone(),
            uncovered_pixels,
        };
    }
    StaticTextureEligibility::Opaque(StaticOpaqueTexture {
        texture_name: image.texture_name.clone(),
        descriptor: Rgba8TextureDescriptor::new(
            u32::from(image.width),
            u32::from(image.height),
            Rgba8TextureColorSpace::Srgb,
        ),
        sampler: TextureSampler {
            filter: TextureFilter::Point,
            address_u: TextureAddressMode::Repeat,
            address_v: TextureAddressMode::Repeat,
        },
        selected_palette: 0,
    })
}

fn lower_position(position: [f64; 3], vertex: usize) -> Result<[f32; 3], StaticFlatLoweringError> {
    Ok([
        finite_f32(position[0], vertex, "x")?,
        finite_f32(position[1], vertex, "height")?,
        finite_f32(position[2], vertex, "z")?,
    ])
}

fn finite_f32(
    value: f64,
    vertex: usize,
    component: &'static str,
) -> Result<f32, StaticFlatLoweringError> {
    if !value.is_finite() || value < f64::from(f32::MIN) || value > f64::from(f32::MAX) {
        return Err(StaticFlatLoweringError::NonFinitePosition { vertex, component });
    }
    Ok(value as f32)
}

fn triangle_normal(positions: [[f32; 3]; 3]) -> Result<[f32; 3], StaticFlatLoweringError> {
    let ab = subtract(positions[1], positions[0]);
    let ac = subtract(positions[2], positions[0]);
    let cross = cross(ab, ac);
    let length_squared = dot(cross, cross);
    if !length_squared.is_finite() || length_squared <= f32::EPSILON {
        return Err(StaticFlatLoweringError::DegenerateTriangle);
    }
    let reciprocal_length = length_squared.sqrt().recip();
    Ok(cross.map(|component| component * reciprocal_length))
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> DoomSurfaceTriangle {
        DoomSurfaceTriangle {
            source_subsector: DoomSourceRecord {
                lump_index: 6,
                record_index: 12,
            },
            source_sector: DoomSourceRecord {
                lump_index: 8,
                record_index: 3,
            },
            plane: DoomSurfacePlane::Floor,
            texture_name: "FLOOR0_1".into(),
            positions: [[64.0, 0.0, 128.0], [128.0, 0.0, 128.0], [64.0, 0.0, 192.0]],
        }
    }

    fn wall_candidate() -> DoomTexturedWallTriangle {
        DoomTexturedWallTriangle {
            source_linedef: DoomSourceRecord {
                lump_index: 2,
                record_index: 7,
            },
            source_sidedef: DoomSourceRecord {
                lump_index: 3,
                record_index: 9,
            },
            source_sector: DoomSourceRecord {
                lump_index: 8,
                record_index: 3,
            },
            side: doom_geometry_provider::DoomWallSideKind::Right,
            role: DoomWallTextureRole::Middle,
            texture_name: "STARTAN3".into(),
            positions: [[0.0, 64.0, 0.0], [64.0, 64.0, 0.0], [0.0, 0.0, 0.0]],
            texture_coordinates: [[32.0, 16.0], [96.0, 16.0], [32.0, 80.0]],
        }
    }

    fn raster(coverage: Vec<bool>) -> DoomIndexedImage {
        DoomIndexedImage {
            source_texture_lump_index: 17,
            texture_name: "STARTAN3".into(),
            width: 2,
            height: 2,
            color_indices: vec![0; 4],
            opaque_pixels: coverage.iter().filter(|covered| **covered).count(),
            coverage,
        }
    }

    fn opaque_prepared(name: &str) -> PreparedStaticTexture {
        PreparedStaticTexture {
            eligibility: StaticTextureEligibility::Opaque(StaticOpaqueTexture {
                texture_name: name.into(),
                descriptor: Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb),
                sampler: TextureSampler {
                    filter: TextureFilter::Point,
                    address_u: TextureAddressMode::Repeat,
                    address_v: TextureAddressMode::Repeat,
                },
                selected_palette: 0,
            }),
            rgba8: vec![255; 16],
        }
    }

    #[test]
    fn lowers_map_axes_to_the_documented_supplied_uv_policy() {
        let lowered = lower_static_flat_triangle(&candidate(), FlatExtent::E1M1).unwrap();
        assert_eq!(
            lowered.mesh.texture_coordinates,
            vec![[1.0, -2.0], [2.0, -2.0], [1.0, -3.0]]
        );
        assert!(lowered.mesh.has_texture_coordinates());
        assert_eq!(lowered.source.flat_name, "FLOOR0_1");
        assert_eq!(lowered.source.subsector.record_index, 12);
    }

    #[test]
    fn rejects_degenerate_source_triangles() {
        let mut triangle = candidate();
        triangle.positions[2] = triangle.positions[1];
        assert_eq!(
            lower_static_flat_triangle(&triangle, FlatExtent::E1M1),
            Err(StaticFlatLoweringError::DegenerateTriangle)
        );
    }

    #[test]
    fn assembly_retains_collinear_candidate_as_an_omission_and_continues() {
        let valid = candidate();
        let mut collinear = candidate();
        collinear.positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let assembly =
            assemble_static_opaque_flats(&[valid, collinear], &[], FlatExtent::E1M1).unwrap();
        assert_eq!(assembly.opaque_flats.len(), 1);
        assert_eq!(assembly.omitted_degenerate.len(), 1);
        assert_eq!(assembly.omitted_degenerate[0].flat_name, "FLOOR0_1");
    }

    #[test]
    fn rejects_zero_extent_before_dividing_coordinates() {
        assert_eq!(
            lower_static_flat_triangle(
                &candidate(),
                FlatExtent {
                    width: 0,
                    height: 64
                }
            ),
            Err(StaticFlatLoweringError::ZeroExtent {
                width: 0,
                height: 64
            })
        );
    }

    #[test]
    fn assembly_uses_retained_sky_classification_not_a_texture_name_heuristic() {
        let floor = candidate();
        let mut ceiling = candidate();
        ceiling.plane = DoomSurfacePlane::Ceiling;
        ceiling.texture_name = "CEIL1_1".into();
        let sky = DoomSkySurfaceObservation {
            source_subsector: ceiling.source_subsector,
            source_sector: ceiling.source_sector,
            plane: DoomSurfacePlane::Ceiling,
            texture_name: "F_SKY1".into(),
        };

        let assembly =
            assemble_static_opaque_flats(&[floor, ceiling], &[sky.clone()], FlatExtent::E1M1)
                .unwrap();

        assert_eq!(assembly.opaque_flats.len(), 1);
        assert_eq!(
            assembly.opaque_flats[0].source.plane,
            DoomSurfacePlane::Floor
        );
        assert_eq!(assembly.omitted_sky, vec![sky]);
    }

    #[test]
    fn wall_lowering_normalizes_retained_source_texels_without_repegging() {
        let lowered = lower_static_wall_triangle(
            &wall_candidate(),
            DoomTextureExtent {
                name: "STARTAN3".into(),
                width: 128,
                height: 128,
            },
        )
        .unwrap();
        assert_eq!(
            lowered.mesh.texture_coordinates,
            vec![[0.25, 0.125], [0.75, 0.125], [0.25, 0.625]]
        );
        assert_eq!(lowered.source_linedef.record_index, 7);
    }

    #[test]
    fn wall_assembly_excludes_only_explicitly_classified_masked_middles() {
        let wall = wall_candidate();
        let masked = DoomMiddleTextureObservation {
            source_linedef: wall.source_linedef,
            source_sidedef: wall.source_sidedef,
            source_sector: wall.source_sector,
            side: wall.side,
            texture_name: wall.texture_name.clone(),
            opening_floor: 0,
            opening_ceiling: 64,
        };
        let assembly = assemble_static_opaque_walls(
            &[wall],
            &[masked.clone()],
            &[DoomTextureExtent {
                name: "STARTAN3".into(),
                width: 128,
                height: 128,
            }],
        )
        .unwrap();
        assert!(assembly.opaque_walls.is_empty());
        assert_eq!(assembly.omitted_masked_middles, vec![masked]);
    }

    #[test]
    fn opaque_raster_selects_the_declared_static_material_profile() {
        assert_eq!(
            classify_static_texture(&raster(vec![true; 4])),
            StaticTextureEligibility::Opaque(StaticOpaqueTexture {
                texture_name: "STARTAN3".into(),
                descriptor: Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb),
                sampler: TextureSampler {
                    filter: TextureFilter::Point,
                    address_u: TextureAddressMode::Repeat,
                    address_v: TextureAddressMode::Repeat,
                },
                selected_palette: 0,
            })
        );
    }

    #[test]
    fn uncovered_raster_is_deferred_instead_of_selecting_alpha_behavior() {
        assert_eq!(
            classify_static_texture(&raster(vec![true, false, true, false])),
            StaticTextureEligibility::DeferredAlpha {
                texture_name: "STARTAN3".into(),
                uncovered_pixels: 2,
            }
        );
    }

    #[test]
    fn opaque_uploads_are_deterministic_and_deferred_alpha_receives_no_handle() {
        let uploads = build_static_texture_uploads(
            &[
                opaque_prepared("ZFLAT"),
                opaque_prepared("AFLAT"),
                PreparedStaticTexture {
                    eligibility: StaticTextureEligibility::DeferredAlpha {
                        texture_name: "MASKED".into(),
                        uncovered_pixels: 1,
                    },
                    rgba8: vec![0; 16],
                },
            ],
            &[opaque_prepared("WALL")],
        );
        assert_eq!(uploads.len(), 3);
        assert_eq!(uploads[0].source_kind, StaticTextureSourceKind::Flat);
        assert_eq!(uploads[0].source_name, "AFLAT");
        assert_eq!(uploads[0].texture, TextureHandle(1));
        assert_eq!(uploads[1].source_name, "ZFLAT");
        assert_eq!(uploads[2].source_kind, StaticTextureSourceKind::Wall);
        assert_eq!(uploads[2].material, MaterialHandle(3));
    }

    #[test]
    fn draw_plan_converts_only_ordinary_mesh_and_material_inputs() {
        let flats = PreparedE1m1Flats {
            map_name: "E1M1".into(),
            flat_assembly: assemble_static_opaque_flats(&[candidate()], &[], FlatExtent::E1M1)
                .unwrap(),
        };
        let wall = wall_candidate();
        let walls = PreparedE1m1Walls {
            map_name: "E1M1".into(),
            wall_assembly: assemble_static_opaque_walls(
                &[wall],
                &[],
                &[DoomTextureExtent {
                    name: "STARTAN3".into(),
                    width: 128,
                    height: 128,
                }],
            )
            .unwrap(),
        };
        let uploads = build_static_texture_uploads(
            &[opaque_prepared("FLOOR0_1")],
            &[opaque_prepared("STARTAN3")],
        );
        let draws = build_static_draw_plan(&flats, &walls, &uploads).unwrap();
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].material, MaterialHandle(1));
        assert_eq!(draws[0].source_label, "flat:3:FLOOR0_1");
        assert_eq!(draws[1].material, MaterialHandle(2));
        assert_eq!(draws[1].source_label, "wall:7:STARTAN3");
    }
}
