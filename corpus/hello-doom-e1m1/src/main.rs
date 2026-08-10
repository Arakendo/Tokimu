//! Canonical-package preflight for the static E1M1 presentation consumer.

use std::{env, fs, process::ExitCode};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use doom_raster_provider::{
    DoomFlatDecodeLimits, DoomPatchDecodeLimits, DoomRasterDecodeLimits, DoomTextureComposeLimits,
    DoomTextureDecodeLimits,
};
use doom_wad_package::{read_wad_package_member, InspectWadPackageRequest};
use doom_wad_provider::WadReadLimits;
use hello_doom_e1m1::{
    build_static_draw_plan, build_static_texture_uploads, fully_omitted_wall_details,
    prepare_e1m1_flat_textures, prepare_e1m1_flats, prepare_e1m1_masked_middle_cutouts,
    prepare_e1m1_wall_textures, prepare_e1m1_walls, prepared_e1m1_masked_middle_texture_names,
    prepared_e1m1_scene_report, StaticTextureEligibility,
};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;

const WAD_LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);
const MAP_LIMITS: doom_map_provider::DoomMapDecodeLimits = doom_map_provider::DoomMapDecodeLimits {
    max_things: 100_000,
    max_vertices: 100_000,
    max_linedefs: 100_000,
    max_sidedefs: 100_000,
    max_sectors: 100_000,
    max_segs: 100_000,
    max_subsectors: 100_000,
    max_nodes: 100_000,
    max_reject_bytes: 64 * 1024 * 1024,
    max_blockmap_bytes: 64 * 1024 * 1024,
    max_blockmap_cells: 1_000_000,
    max_blockmap_linedef_refs: 10_000_000,
    max_total_record_bytes: 64 * 1024 * 1024,
};
const RASTER_LIMITS: DoomRasterDecodeLimits = DoomRasterDecodeLimits {
    max_playpal_bytes: 64 * 1024 * 1024,
    max_palettes: 4096,
    max_colormap_bytes: 64 * 1024 * 1024,
    max_colormaps: 4096,
    max_total_decoded_bytes: 128 * 1024 * 1024,
};
const FLAT_LIMITS: DoomFlatDecodeLimits = DoomFlatDecodeLimits {
    max_flat_bytes: 4096,
};
const TEXTURE_LIMITS: DoomTextureDecodeLimits = DoomTextureDecodeLimits {
    max_pnames_bytes: 64 * 1024 * 1024,
    max_texture_bytes: 64 * 1024 * 1024,
    max_patch_names: 1_000_000,
    max_textures: 1_000_000,
    max_patches_per_texture: 16_384,
    max_total_patch_references: 10_000_000,
};
const PATCH_LIMITS: DoomPatchDecodeLimits = DoomPatchDecodeLimits {
    max_patch_bytes: 64 * 1024 * 1024,
    max_width: 4096,
    max_height: 4096,
    max_pixels: 16 * 1024 * 1024,
    max_posts: 16 * 1024 * 1024,
};
const COMPOSE_LIMITS: DoomTextureComposeLimits = DoomTextureComposeLimits {
    max_width: 4096,
    max_height: 4096,
    max_pixels: 16 * 1024 * 1024,
};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [package, member] = args.as_slice() else {
        eprintln!("usage: hello-doom-e1m1 <canonical-doom-zip> <WAD-member-name>");
        return ExitCode::from(2);
    };
    match preflight(package, member) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("E1M1 preflight failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn preflight(package: &str, member: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(package)?;
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(5_001), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(5_002);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(5_003), "E1M1 package"),
        folder,
        ResourceMetadata::default(),
    )?;
    let resource_name =
        ResourceName::parse("canonical-doom-package.zip", AddressCasePolicy::Sensitive)?;
    space.insert_resource(
        folder,
        resource_name.clone(),
        bytes,
        ResourceMetadata::default(),
    )?;
    let read = read_wad_package_member(
        &space,
        InspectWadPackageRequest {
            archive: InspectArchiveResourceRequest {
                source_folder: folder,
                source_name: resource_name,
                format: ArchiveFormat::Zip,
                limits: ArchiveReadLimits::new(
                    64 * 1024 * 1024,
                    2048,
                    16 * 1024 * 1024,
                    64 * 1024 * 1024,
                    4096,
                ),
            },
            member_name: member.to_owned(),
            wad_source_label: format!("{package}:{member}"),
            wad_limits: WAD_LIMITS,
        },
        &ZipArchiveProvider,
    )?;
    let flats = prepare_e1m1_flats(&read.bytes, &read.observation.wad, MAP_LIMITS)?;
    let walls = prepare_e1m1_walls(
        &read.bytes,
        &read.observation.wad,
        MAP_LIMITS,
        TEXTURE_LIMITS,
    )?;
    let masked_middle_cutouts = prepare_e1m1_masked_middle_cutouts(
        &read.bytes,
        &read.observation.wad,
        MAP_LIMITS,
        TEXTURE_LIMITS,
    )?;
    let flat_textures = prepare_e1m1_flat_textures(
        &read.bytes,
        &read.observation.wad,
        &flats,
        RASTER_LIMITS,
        FLAT_LIMITS,
    )?;
    let names = hello_doom_e1m1::prepared_e1m1_wall_texture_names(&walls);
    let wall_textures = prepare_e1m1_wall_textures(
        &read.bytes,
        &read.observation.wad,
        &names,
        RASTER_LIMITS,
        TEXTURE_LIMITS,
        PATCH_LIMITS,
        COMPOSE_LIMITS,
    )?;
    let masked_middle_names = prepared_e1m1_masked_middle_texture_names(&walls);
    let masked_middle_textures = prepare_e1m1_wall_textures(
        &read.bytes,
        &read.observation.wad,
        &masked_middle_names,
        RASTER_LIMITS,
        TEXTURE_LIMITS,
        PATCH_LIMITS,
        COMPOSE_LIMITS,
    )?;
    let report = prepared_e1m1_scene_report(&flats, &walls, &flat_textures, &wall_textures);
    let details = fully_omitted_wall_details(&walls);
    let uploads = build_static_texture_uploads(&flat_textures, &wall_textures);
    let draws = build_static_draw_plan(&flats, &walls, &uploads)?;
    let inventory = uploads
        .iter()
        .map(|upload| {
            format!(
                "{:?}:{}=texture:{}:material:{}",
                upload.source_kind, upload.source_name, upload.texture.0, upload.material.0
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let masked_middle_coverage = masked_middle_textures
        .iter()
        .map(|texture| match &texture.eligibility {
            StaticTextureEligibility::Opaque(opaque) => {
                format!("{}:fully-covered", opaque.texture_name)
            }
            StaticTextureEligibility::DeferredAlpha {
                texture_name,
                uncovered_pixels,
                ..
            } => format!("{texture_name}:uncovered={uncovered_pixels}"),
        })
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        "{report} masked_middle_texture_names={masked_middle_names:?} masked_middle_coverage=[{masked_middle_coverage}] experimental_cutout_candidates={} experimental_cutout_omitted_degenerate={} experimental_cutout_intent={:?} static_uploads={} static_draws={} static_upload_inventory=[{inventory}] fully_omitted_wall_details={details:?}",
        masked_middle_cutouts.assembly.candidates.len(),
        masked_middle_cutouts.assembly.omitted_degenerate.len(),
        masked_middle_cutouts
            .assembly
            .candidates
            .first()
            .map(|candidate| candidate.intent),
        uploads.len(),
        draws.len(),
    ))
}
