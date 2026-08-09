//! Headless WAD-container observation consumer.
//!
//! Filesystem selection belongs here at the corpus edge. `doom-wad-provider`
//! receives only caller-labelled bytes and does not acquire or retain files.

use std::{collections::BTreeMap, env, ffi::OsString, fs, process::ExitCode};

use archive_provider::{ArchiveFormat, ArchiveReadLimits};
use doom_geometry_provider::{
    audit_doom_pegging_flags, audit_doom_subsector_bsp_paths, audit_doom_subsector_loop_closure,
    audit_doom_subsector_region_endpoints, audit_doom_vertical_topology, audit_doom_wall_topology,
    locate_doom_point_subsector, lower_doom_one_sided_walls, lower_doom_subsector_surfaces,
    lower_doom_textured_wall_triangles, lower_doom_two_sided_middle_walls,
    lower_doom_two_sided_wall_bands, observe_doom_sky_surfaces,
    observe_doom_two_sided_middle_textures, observe_doom_wall_texture_axes,
    resolve_doom_subsector_bsp_paths, resolve_doom_subsector_loops, resolve_doom_subsector_regions,
    resolve_doom_subsector_sector_ownership, resolve_doom_wall_candidates,
    resolve_doom_wall_texture_bindings, resolve_doom_wall_texture_placements, DoomTextureExtent,
};
use doom_map_provider::{
    decode_doom_map_core, locate_doom_blockmap_cell, resolve_doom_player_one_start, DoomMapCore,
    DoomMapDecodeLimits,
};
use doom_raster_provider::{
    compose_doom_texture, decode_doom_flat, decode_doom_patch, decode_doom_raster_globals,
    decode_doom_sprite_frame_rotations, decode_doom_sprite_patch, decode_doom_texture_catalog,
    doom_sprite_frame_rotation_fingerprint, indexed_image_from_doom_flat,
    indexed_image_from_doom_palette, indexed_image_from_doom_patch, lower_doom_indexed_image,
    DoomFlatDecodeLimits, DoomPatchDecodeLimits, DoomRasterDecodeLimits, DoomTextureComposeLimits,
    DoomTextureDecodeLimits,
};
use doom_wad_package::{
    read_wad_package_member, select_doom_episode_map, InspectWadPackageRequest,
};
use doom_wad_provider::{inspect_wad, WadManifest, WadReadLimits};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;

const LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);
const MAP_LIMITS: DoomMapDecodeLimits = DoomMapDecodeLimits {
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
const PATCH_LIMITS: DoomPatchDecodeLimits = DoomPatchDecodeLimits {
    max_patch_bytes: 64 * 1024 * 1024,
    max_width: 4096,
    max_height: 4096,
    max_pixels: 16 * 1024 * 1024,
    max_posts: 16 * 1024 * 1024,
};
const TEXTURE_LIMITS: DoomTextureDecodeLimits = DoomTextureDecodeLimits {
    max_pnames_bytes: 64 * 1024 * 1024,
    max_texture_bytes: 64 * 1024 * 1024,
    max_patch_names: 1_000_000,
    max_textures: 1_000_000,
    max_patches_per_texture: 16_384,
    max_total_patch_references: 10_000_000,
};
const TEXTURE_COMPOSE_LIMITS: DoomTextureComposeLimits = DoomTextureComposeLimits {
    max_width: 4096,
    max_height: 4096,
    max_pixels: 16 * 1024 * 1024,
};
const FLAT_LIMITS: DoomFlatDecodeLimits = DoomFlatDecodeLimits {
    max_flat_bytes: 4096,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MapSvgMode {
    SourceTopology,
    SectorColor,
    WallNormals,
}

#[derive(Clone, Debug)]
struct MapSvgRequest {
    output: OsString,
    mode: MapSvgMode,
}

fn main() -> ExitCode {
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [path] => inspect_loose(path),
        [flag, package, member] if flag == "--zip" => {
            inspect_zip_member(package, member, None, None, None, None, None, None, None)
        }
        [flag, package, member, map] if flag == "--zip" => inspect_zip_member(
            package,
            member,
            Some(map),
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        [flag, package, member, texture_flag, texture]
            if flag == "--zip" && texture_flag == "--texture" =>
        {
            inspect_zip_member(
                package,
                member,
                None,
                None,
                Some(texture),
                None,
                None,
                None,
                None,
            )
        }
        [flag, package, member, artifact_flag, texture, output]
            if flag == "--zip" && artifact_flag == "--texture-ppm" =>
        {
            inspect_zip_member(
                package,
                member,
                None,
                None,
                Some(texture),
                None,
                Some(output),
                None,
                None,
            )
        }
        [flag, package, member, artifact_flag, patch, output]
            if flag == "--zip" && artifact_flag == "--patch-ppm" =>
        {
            inspect_zip_member(
                package,
                member,
                None,
                Some(patch),
                None,
                None,
                Some(output),
                None,
                None,
            )
        }
        [flag, package, member, artifact_flag, flat, output]
            if flag == "--zip" && artifact_flag == "--flat-ppm" =>
        {
            inspect_zip_member(
                package,
                member,
                None,
                None,
                None,
                Some(flat),
                Some(output),
                None,
                None,
            )
        }
        [flag, package, member, artifact_flag, sprite, output]
            if flag == "--zip" && artifact_flag == "--sprite-ppm" =>
        {
            inspect_zip_member(
                package,
                member,
                None,
                None,
                None,
                None,
                Some(output),
                Some(sprite),
                None,
            )
        }
        [flag, package, member, artifact_flag, output]
            if flag == "--zip" && artifact_flag == "--palette-ppm" =>
        {
            inspect_zip_member(
                package,
                member,
                None,
                None,
                None,
                None,
                Some(output),
                None,
                None,
            )
        }
        [flag, package, member, flat_flag, flat] if flag == "--zip" && flat_flag == "--flat" => {
            inspect_zip_member(
                package,
                member,
                None,
                None,
                None,
                Some(flat),
                None,
                None,
                None,
            )
        }
        [flag, package, member, map, patch] if flag == "--zip" => inspect_zip_member(
            package,
            member,
            Some(map),
            Some(patch),
            None,
            None,
            None,
            None,
            None,
        ),
        [flag, package, member, artifact_flag, map, output]
            if flag == "--zip" && artifact_flag == "--map-svg" =>
        {
            inspect_zip_member(
                package,
                member,
                Some(map),
                None,
                None,
                None,
                None,
                None,
                Some(MapSvgRequest {
                    output: output.clone(),
                    mode: MapSvgMode::SourceTopology,
                }),
            )
        }
        [flag, package, member, artifact_flag, map, output]
            if flag == "--zip" && artifact_flag == "--map-sector-svg" =>
        {
            inspect_zip_member(
                package,
                member,
                Some(map),
                None,
                None,
                None,
                None,
                None,
                Some(MapSvgRequest {
                    output: output.clone(),
                    mode: MapSvgMode::SectorColor,
                }),
            )
        }
        [flag, package, member, artifact_flag, map, output]
            if flag == "--zip" && artifact_flag == "--map-normal-svg" =>
        {
            inspect_zip_member(
                package,
                member,
                Some(map),
                None,
                None,
                None,
                None,
                None,
                Some(MapSvgRequest {
                    output: output.clone(),
                    mode: MapSvgMode::WallNormals,
                }),
            )
        }
        _ => {
            eprintln!("usage: hello-wad-inspect <path-to-iwad-or-pwad>");
            eprintln!(
                "       hello-wad-inspect --zip <package.zip> <member-name> [E#M#] [patch-name]"
            );
            eprintln!(
                "       hello-wad-inspect --zip <package.zip> <member-name> --texture <name>"
            );
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --texture-ppm <name> <output.ppm>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --patch-ppm <name> <output.ppm>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --flat-ppm <name> <output.ppm>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --sprite-ppm <name> <output.ppm>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --palette-ppm <output.ppm>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --flat <name>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --map-svg <E#M#> <output.svg>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --map-sector-svg <E#M#> <output.svg>");
            eprintln!("       hello-wad-inspect --zip <package.zip> <member-name> --map-normal-svg <E#M#> <output.svg>");
            ExitCode::from(2)
        }
    }
}

fn inspect_loose(path: &OsString) -> ExitCode {
    let label = path.to_string_lossy().into_owned();
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("unable to read `{label}`: {error}");
            return ExitCode::FAILURE;
        }
    };
    let manifest = match inspect_wad(&label, &bytes, LIMITS) {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("unable to inspect `{label}`: {error}");
            return ExitCode::FAILURE;
        }
    };

    print_manifest(&manifest);
    ExitCode::SUCCESS
}

fn inspect_zip_member(
    package: &OsString,
    member: &OsString,
    map: Option<&OsString>,
    patch: Option<&OsString>,
    texture: Option<&OsString>,
    flat: Option<&OsString>,
    ppm_output: Option<&OsString>,
    sprite: Option<&OsString>,
    map_svg_request: Option<MapSvgRequest>,
) -> ExitCode {
    let package_label = package.to_string_lossy().into_owned();
    let member_name = member.to_string_lossy().into_owned();
    let archive_bytes = match fs::read(package) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("unable to read `{package_label}`: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(900), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(901);
    if let Err(error) = space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(902), "WAD inspector package"),
        folder,
        ResourceMetadata::default(),
    ) {
        eprintln!("unable to create logical package root: {error}");
        return ExitCode::FAILURE;
    }
    let resource_name =
        match ResourceName::parse("selected-package.zip", AddressCasePolicy::Sensitive) {
            Ok(name) => name,
            Err(error) => {
                eprintln!("unable to name logical package resource: {error}");
                return ExitCode::FAILURE;
            }
        };
    if let Err(error) = space.insert_resource(
        folder,
        resource_name.clone(),
        archive_bytes,
        ResourceMetadata::default(),
    ) {
        eprintln!("unable to retain package bytes: {error}");
        return ExitCode::FAILURE;
    }
    let package_read = match read_wad_package_member(
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
            member_name: member_name.clone(),
            wad_source_label: format!("{package_label}:{member_name}"),
            wad_limits: LIMITS,
        },
        &archive_provider::ZipArchiveProvider,
    ) {
        Ok(observation) => observation,
        Err(error) => {
            eprintln!("unable to inspect `{package_label}:{member_name}`: {error}");
            return ExitCode::FAILURE;
        }
    };
    let observation = package_read.observation;
    println!(
        "package member {} (archive fingerprint {:?}):",
        observation.member.normalized_name, observation.archive_fingerprint
    );
    print_manifest(&observation.wad);
    let globals =
        match decode_doom_raster_globals(&package_read.bytes, &observation.wad, RASTER_LIMITS) {
            Ok(globals) => {
                println!(
                    "raster globals: palettes={}, colormaps={}, first_palette_rgb=({}, {}, {})",
                    globals.palettes.len(),
                    globals.colormaps.len(),
                    globals.palettes[0].colors[0].red,
                    globals.palettes[0].colors[0].green,
                    globals.palettes[0].colors[0].blue,
                );
                globals
            }
            Err(error) => {
                eprintln!("unable to decode global raster data: {error}");
                return ExitCode::FAILURE;
            }
        };
    match lower_doom_indexed_image(
        &indexed_image_from_doom_palette(&globals.palettes[0]),
        &globals.palettes[0],
    ) {
        Ok(lowered) => {
            if ppm_output.is_some() && texture.is_none() && patch.is_none() && flat.is_none() {
                if let Err(error) = write_ppm(
                    ppm_output.expect("checked above"),
                    lowered.width,
                    lowered.height,
                    &lowered.pixels,
                ) {
                    eprintln!("unable to write palette PPM: {error}");
                    return ExitCode::FAILURE;
                }
            }
            println!(
                "palette 0: 256x1, rgba8_fingerprint={}",
                lowered.pixel_fingerprint(),
            );
        }
        Err(error) => {
            eprintln!("unable to lower palette 0: {error}");
            return ExitCode::FAILURE;
        }
    }
    let texture_catalog =
        match decode_doom_texture_catalog(&package_read.bytes, &observation.wad, TEXTURE_LIMITS) {
            Ok(catalog) => {
                println!(
                    "texture catalog: patch_names={}, textures={}, patch_references={}",
                    catalog.patch_names.len(),
                    catalog.textures.len(),
                    catalog
                        .textures
                        .iter()
                        .map(|texture| texture.patches.len())
                        .sum::<usize>(),
                );
                catalog
            }
            Err(error) => {
                eprintln!("unable to decode texture catalog: {error}");
                return ExitCode::FAILURE;
            }
        };
    match decode_doom_sprite_frame_rotations(&observation.wad) {
        Ok(frames) => println!(
            "sprite frame rotations: {}, fingerprint={}",
            frames.len(),
            doom_sprite_frame_rotation_fingerprint(&frames),
        ),
        Err(error) => {
            eprintln!("unable to decode sprite frame rotations: {error}");
            return ExitCode::FAILURE;
        }
    }
    if let Some(map) = map {
        let map_name = map.to_string_lossy();
        let selection = match select_doom_episode_map(&observation.wad, &map_name) {
            Ok(selection) => selection,
            Err(error) => {
                eprintln!("unable to select map `{map_name}`: {error}");
                return ExitCode::FAILURE;
            }
        };
        println!(
            "map {}: marker #{}, local lump indices {:?}, {} required lumps",
            selection.map_name,
            selection.marker.index,
            selection.local_range,
            selection.required_lumps.len(),
        );
        match decode_doom_map_core(&package_read.bytes, &selection, MAP_LIMITS) {
            Ok(core) => {
                if let Some(request) = map_svg_request {
                    if let Err(error) = write_map_svg(&request.output, &core, request.mode) {
                        eprintln!(
                            "unable to write map SVG `{}`: {error}",
                            request.output.to_string_lossy()
                        );
                        return ExitCode::FAILURE;
                    }
                    println!(
                        "map SVG: {} diagnostic written to `{}`",
                        map_svg_mode_label(request.mode),
                        request.output.to_string_lossy()
                    );
                }
                println!(
                    "map core: things={}, vertices={}, linedefs={}, sidedefs={}, sectors={}, segs={}, subsectors={}, nodes={}",
                    core.things.len(),
                    core.vertices.len(),
                    core.linedefs.len(),
                    core.sidedefs.len(),
                    core.sectors.len(),
                    core.segs.len(),
                    core.subsectors.len(),
                    core.nodes.len(),
                );
                let mut thing_kinds = BTreeMap::new();
                let mut thing_flags = BTreeMap::new();
                for thing in &core.things {
                    *thing_kinds.entry(thing.kind).or_insert(0_usize) += 1;
                    *thing_flags.entry(thing.flags).or_insert(0_usize) += 1;
                }
                println!(
                    "thing inventory: distinct_kinds={}, kind_counts={}, distinct_flag_sets={}",
                    thing_kinds.len(),
                    format_bounded_count_inventory(&thing_kinds),
                    thing_flags.len(),
                );
                let mut linedef_specials = BTreeMap::new();
                for linedef in core.linedefs.iter().filter(|linedef| linedef.special != 0) {
                    *linedef_specials.entry(linedef.special).or_insert(0_usize) += 1;
                }
                let mut sector_specials = BTreeMap::new();
                for sector in core.sectors.iter().filter(|sector| sector.special != 0) {
                    *sector_specials.entry(sector.special).or_insert(0_usize) += 1;
                }
                println!(
                    "map special inventory: linedef_codes={}, linedef_counts={}, sector_codes={}, sector_counts={}",
                    linedef_specials.len(),
                    format_bounded_count_inventory(&linedef_specials),
                    sector_specials.len(),
                    format_bounded_count_inventory(&sector_specials),
                );
                match resolve_doom_player_one_start(&core.things) {
                    Ok(start) => println!(
                        "player-one start: source=THINGS #{} at=({}, {}) angle={} flags={:#06x}",
                        start.source.record_index,
                        start.position[0],
                        start.position[1],
                        start.angle,
                        start.flags,
                    ),
                    Err(error) => {
                        eprintln!("unable to resolve player-one start: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                println!(
                    "map auxiliaries: reject={} bytes (minimum {}), blockmap={}x{} cells={}, unique_lists={}, linedef_refs={}",
                    core.reject.byte_len,
                    core.reject.required_min_bytes,
                    core.blockmap.columns,
                    core.blockmap.rows,
                    core.blockmap.cells,
                    core.blockmap.unique_linedef_lists,
                    core.blockmap.linedef_references,
                );
                let player_start = resolve_doom_player_one_start(&core.things)
                    .expect("the player-one start was resolved before blockmap inspection");
                match locate_doom_blockmap_cell(&core.blockmap, player_start.position) {
                    Some(cell) => println!(
                        "player-one BLOCKMAP cell: index={} column={} row={} candidate_linedefs={}",
                        cell.cell_index,
                        cell.column,
                        cell.row,
                        cell.linedefs.len(),
                    ),
                    None => println!(
                        "player-one BLOCKMAP cell: outside decoded grid at=({}, {})",
                        player_start.position[0], player_start.position[1],
                    ),
                }
                match resolve_doom_wall_candidates(&core) {
                    Ok(candidates) => {
                        let audit = audit_doom_wall_topology(&candidates);
                        println!(
                            "wall candidates: total={}, one_sided={}, two_sided={}, same_sector_two_sided={}",
                            audit.candidates,
                            audit.one_sided,
                            audit.two_sided,
                            audit.same_sector_two_sided,
                        );
                    }
                    Err(error) => {
                        eprintln!("unable to resolve map wall candidates: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match audit_doom_vertical_topology(&core) {
                    Ok(audit) => println!(
                        "vertical topology: sectors={}, sectors_without_positive_clearance={}, two_sided_openings={}, closed_two_sided_openings={}",
                        audit.sectors,
                        audit.sectors_without_positive_clearance,
                        audit.two_sided_openings,
                        audit.two_sided_openings_without_positive_clearance,
                    ),
                    Err(error) => {
                        eprintln!("unable to audit map vertical topology: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match resolve_doom_subsector_sector_ownership(&core) {
                    Ok(ownership) => {
                        let distinct_sectors = ownership
                            .iter()
                            .map(|entry| entry.sector_index)
                            .collect::<std::collections::BTreeSet<_>>();
                        println!(
                            "subsector sector ownership: total={}, distinct_sectors={}",
                            ownership.len(),
                            distinct_sectors.len(),
                        );
                    }
                    Err(error) => {
                        eprintln!("unable to resolve map subsector sector ownership: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match lower_doom_one_sided_walls(&core) {
                    Ok(triangles) => {
                        println!("one-sided wall triangles: total={}", triangles.len())
                    }
                    Err(error) => {
                        eprintln!("unable to lower one-sided map walls: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match lower_doom_two_sided_wall_bands(&core) {
                    Ok(triangles) => {
                        let upper = triangles
                            .iter()
                            .filter(|triangle| {
                                matches!(triangle.band, doom_geometry_provider::DoomWallBand::Upper)
                            })
                            .count();
                        println!(
                            "two-sided wall-band triangles: total={}, upper={}, lower={}",
                            triangles.len(),
                            upper,
                            triangles.len() - upper,
                        );
                    }
                    Err(error) => {
                        eprintln!("unable to lower two-sided map wall bands: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match lower_doom_two_sided_middle_walls(&core) {
                    Ok(triangles) => println!(
                        "two-sided middle-wall triangles: total={} (positive shared-opening clip; material alpha deferred)",
                        triangles.len()
                    ),
                    Err(error) => {
                        eprintln!("unable to lower two-sided middle map walls: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match observe_doom_two_sided_middle_textures(&core) {
                    Ok(observations) => {
                        let textures = observations
                            .iter()
                            .map(|observation| observation.texture_name.as_str())
                            .collect::<std::collections::BTreeSet<_>>();
                        println!(
                            "two-sided middle textures: observations={}, distinct_names={}",
                            observations.len(),
                            textures.len(),
                        );
                    }
                    Err(error) => {
                        eprintln!("unable to observe two-sided middle textures: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match observe_doom_wall_texture_axes(&core) {
                    Ok(axes) => println!("wall texture axes: total={}", axes.len()),
                    Err(error) => {
                        eprintln!("unable to observe wall texture axes: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                let texture_extents = texture_catalog
                    .textures
                    .iter()
                    .map(|texture| DoomTextureExtent {
                        name: texture.name.clone(),
                        width: texture.width,
                        height: texture.height,
                    })
                    .collect::<Vec<_>>();
                match resolve_doom_wall_texture_bindings(&core, &texture_extents) {
                    Ok(bindings) => println!("wall texture bindings: total={}", bindings.len()),
                    Err(error) => {
                        eprintln!("unable to resolve wall texture bindings: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match resolve_doom_wall_texture_placements(&core, &texture_extents) {
                    Ok(placements) => println!(
                        "wall texture placements: total={} (source texturemid anchors)",
                        placements.len()
                    ),
                    Err(error) => {
                        eprintln!("unable to resolve wall texture placements: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match lower_doom_textured_wall_triangles(&core, &texture_extents) {
                    Ok(triangles) => println!(
                        "textured wall triangles: total={} (source texel coordinates; material alpha deferred)",
                        triangles.len()
                    ),
                    Err(error) => {
                        eprintln!("unable to lower textured wall triangles: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match audit_doom_pegging_flags(&core) {
                    Ok(audit) => println!(
                        "pegging flag audit: upper_axes={}, upper_unpegged={}, lower_axes={}, lower_unpegged={}",
                        audit.upper_axes,
                        audit.upper_unpegged,
                        audit.lower_axes,
                        audit.lower_unpegged,
                    ),
                    Err(error) => {
                        eprintln!("unable to audit pegging flags: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                match resolve_doom_subsector_bsp_paths(&core) {
                    Ok(paths) => {
                        let audit = audit_doom_subsector_bsp_paths(&paths);
                        println!(
                            "subsector BSP paths: total={}, depth={}..{}",
                            audit.subsectors, audit.minimum_depth, audit.maximum_depth,
                        );
                        let player_start = resolve_doom_player_one_start(&core.things)
                            .expect("the player-one start was resolved before BSP inspection");
                        match locate_doom_point_subsector(player_start.position, &paths) {
                            Ok(location) => {
                                match resolve_doom_subsector_sector_ownership(&core) {
                                    Ok(ownership) => {
                                        let sector = ownership
                                            .iter()
                                            .find(|entry| {
                                                entry.source_subsector == location.source_subsector
                                            })
                                            .expect(
                                                "every retained BSP subsector has sector ownership",
                                            );
                                        println!(
                                        "player-one start location: subsector #{} sector #{} floor={} ceiling={}",
                                        location.source_subsector.record_index,
                                        sector.source_sector.record_index,
                                        core.sectors[usize::from(sector.sector_index)].floor_height,
                                        core.sectors[usize::from(sector.sector_index)].ceiling_height,
                                    );
                                    }
                                    Err(error) => {
                                        eprintln!("unable to resolve player-start sector ownership: {error}");
                                        return ExitCode::FAILURE;
                                    }
                                }
                            }
                            Err(error) => {
                                eprintln!("unable to locate player-one start in BSP: {error}");
                                return ExitCode::FAILURE;
                            }
                        }
                        let region_audit = audit_doom_subsector_region_endpoints(&core, &paths);
                        println!(
                            "subsector BSP endpoint audit: endpoints={}, outside_paths={}, max_outside_distance={}",
                            region_audit.seg_endpoints,
                            region_audit.endpoints_outside_paths,
                            region_audit.maximum_outside_distance,
                        );
                        match resolve_doom_subsector_regions(&core, &paths) {
                            Ok(regions) => {
                                let boundary_vertices = regions
                                    .iter()
                                    .map(|region| region.vertices.len())
                                    .sum::<usize>();
                                println!(
                                    "subsector BSP regions: total={}, boundary_vertices={}",
                                    regions.len(),
                                    boundary_vertices,
                                );
                            }
                            Err(error) => {
                                eprintln!("unable to resolve map subsector BSP regions: {error}");
                                return ExitCode::FAILURE;
                            }
                        }
                        match lower_doom_subsector_surfaces(&core, &paths) {
                            Ok(triangles) => {
                                let floors = triangles
                                    .iter()
                                    .filter(|triangle| {
                                        matches!(
                                            triangle.plane,
                                            doom_geometry_provider::DoomSurfacePlane::Floor
                                        )
                                    })
                                    .count();
                                println!(
                                    "subsector surface triangles: total={}, floors={}, ceilings={}",
                                    triangles.len(),
                                    floors,
                                    triangles.len() - floors,
                                );
                            }
                            Err(error) => {
                                eprintln!("unable to lower map subsector surfaces: {error}");
                                return ExitCode::FAILURE;
                            }
                        }
                        match observe_doom_sky_surfaces(&core, &paths) {
                            Ok(observations) => {
                                println!("sky surface observations: total={}", observations.len(),)
                            }
                            Err(error) => {
                                eprintln!("unable to observe sky surfaces: {error}");
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("unable to resolve map subsector BSP paths: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                let loop_closure_audit = audit_doom_subsector_loop_closure(&core);
                let mut too_small = 0;
                let mut open = 0;
                let mut ambiguous = 0;
                let mut degenerate = 0;
                for error in &loop_closure_audit.rejected {
                    match error {
                        doom_geometry_provider::DoomGeometryError::SubsectorTooSmall { .. } => {
                            too_small += 1
                        }
                        doom_geometry_provider::DoomGeometryError::SubsectorBoundaryOpen {
                            ..
                        } => open += 1,
                        doom_geometry_provider::DoomGeometryError::SubsectorBoundaryAmbiguous {
                            ..
                        } => ambiguous += 1,
                        doom_geometry_provider::DoomGeometryError::DegenerateSeg { .. } => {
                            degenerate += 1
                        }
                        _ => unreachable!("strict subsector-loop audit has only loop errors"),
                    }
                }
                println!(
                    "subsector loop-closure audit: total={}, closed={}, rejected={} (too_small={}, open={}, ambiguous={}, degenerate={})",
                    loop_closure_audit.subsectors,
                    loop_closure_audit.closed_loops,
                    loop_closure_audit.rejected.len(),
                    too_small,
                    open,
                    ambiguous,
                    degenerate,
                );
                match resolve_doom_subsector_loops(&core) {
                    Ok(loops) => {
                        let boundary_segs = loops
                            .iter()
                            .map(|boundary| boundary.vertices.len())
                            .sum::<usize>();
                        println!(
                            "subsector loops: total={}, boundary_segs={}",
                            loops.len(),
                            boundary_segs,
                        );
                    }
                    Err(error) => {
                        eprintln!("unable to resolve map subsector loops: {error}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            Err(error) => {
                eprintln!("unable to decode map `{map_name}`: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    if let Some(patch) = patch {
        let patch_name = patch.to_string_lossy();
        match decode_doom_patch(
            &package_read.bytes,
            &observation.wad,
            &patch_name,
            PATCH_LIMITS,
        ) {
            Ok(image) => {
                let post_count: usize = image.columns.iter().map(|column| column.posts.len()).sum();
                match lower_doom_indexed_image(
                    &indexed_image_from_doom_patch(&image),
                    &globals.palettes[0],
                ) {
                    Ok(lowered) => {
                        if let Some(output) = ppm_output {
                            if let Err(error) =
                                write_ppm(output, lowered.width, lowered.height, &lowered.pixels)
                            {
                                eprintln!(
                                    "unable to write patch PPM `{}`: {error}",
                                    output.to_string_lossy()
                                );
                                return ExitCode::FAILURE;
                            }
                        }
                        println!(
                            "patch {}: {}x{}, columns={}, posts={}, opaque_pixels={}, origin=({}, {}), palette=0, rgba8_fingerprint={}",
                            image.name, image.width, image.height, image.columns.len(), post_count,
                            image.opaque_pixels, image.left_offset, image.top_offset,
                            lowered.pixel_fingerprint(),
                        );
                    }
                    Err(error) => {
                        eprintln!("unable to lower patch `{patch_name}`: {error}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            Err(error) => {
                eprintln!("unable to decode patch `{patch_name}`: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    if let Some(sprite) = sprite {
        let sprite_name = sprite.to_string_lossy();
        match decode_doom_sprite_patch(
            &package_read.bytes,
            &observation.wad,
            &sprite_name,
            PATCH_LIMITS,
        ) {
            Ok(image) => match lower_doom_indexed_image(
                &indexed_image_from_doom_patch(&image),
                &globals.palettes[0],
            ) {
                Ok(lowered) => {
                    if let Some(output) = ppm_output {
                        if let Err(error) =
                            write_ppm(output, lowered.width, lowered.height, &lowered.pixels)
                        {
                            eprintln!(
                                "unable to write sprite PPM `{}`: {error}",
                                output.to_string_lossy()
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                    println!(
                        "sprite {}: {}x{}, opaque_pixels={}, palette=0, rgba8_fingerprint={}",
                        image.name,
                        image.width,
                        image.height,
                        image.opaque_pixels,
                        lowered.pixel_fingerprint()
                    );
                }
                Err(error) => {
                    eprintln!("unable to lower sprite `{sprite_name}`: {error}");
                    return ExitCode::FAILURE;
                }
            },
            Err(error) => {
                eprintln!("unable to decode sprite `{sprite_name}`: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    if let Some(texture) = texture {
        let texture_name = texture.to_string_lossy();
        match compose_doom_texture(
            &package_read.bytes,
            &observation.wad,
            &texture_catalog,
            &texture_name,
            PATCH_LIMITS,
            TEXTURE_COMPOSE_LIMITS,
        ) {
            Ok(image) => match lower_doom_indexed_image(&image, &globals.palettes[0]) {
                Ok(lowered) => {
                    if let Some(output) = ppm_output {
                        if let Err(error) =
                            write_ppm(output, lowered.width, lowered.height, &lowered.pixels)
                        {
                            eprintln!(
                                "unable to write texture PPM `{}`: {error}",
                                output.to_string_lossy()
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                    println!(
                        "texture {}: {}x{}, opaque_pixels={}, palette=0, rgba8_fingerprint={}",
                        image.texture_name,
                        image.width,
                        image.height,
                        image.opaque_pixels,
                        lowered.pixel_fingerprint(),
                    );
                }
                Err(error) => {
                    eprintln!("unable to lower texture `{texture_name}`: {error}");
                    return ExitCode::FAILURE;
                }
            },
            Err(error) => {
                eprintln!("unable to compose texture `{texture_name}`: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    if let Some(flat) = flat {
        let flat_name = flat.to_string_lossy();
        match decode_doom_flat(
            &package_read.bytes,
            &observation.wad,
            &flat_name,
            FLAT_LIMITS,
        ) {
            Ok(image) => match lower_doom_indexed_image(
                &indexed_image_from_doom_flat(&image),
                &globals.palettes[0],
            ) {
                Ok(lowered) => {
                    if let Some(output) = ppm_output {
                        if let Err(error) =
                            write_ppm(output, lowered.width, lowered.height, &lowered.pixels)
                        {
                            eprintln!(
                                "unable to write flat PPM `{}`: {error}",
                                output.to_string_lossy()
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                    println!(
                        "flat {}: 64x64, indexed_pixels={}, palette=0, rgba8_fingerprint={}",
                        image.name,
                        image.color_indices.len(),
                        lowered.pixel_fingerprint(),
                    );
                }
                Err(error) => {
                    eprintln!("unable to lower flat `{flat_name}`: {error}");
                    return ExitCode::FAILURE;
                }
            },
            Err(error) => {
                eprintln!("unable to decode flat `{flat_name}`: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

fn write_ppm(path: &OsString, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
    for pixel in rgba.chunks_exact(4) {
        if pixel[3] == 0 {
            bytes.extend_from_slice(&[0, 0, 0]);
        } else {
            bytes.extend_from_slice(&pixel[..3]);
        }
    }
    fs::write(path, bytes).map_err(|error| error.to_string())
}

/// Emits a source-map diagnostic. This is deliberately not a renderer input:
/// it has no materials, BSP fills, or visual policy.
fn write_map_svg(path: &OsString, core: &DoomMapCore, mode: MapSvgMode) -> Result<(), String> {
    let minimum_x = core
        .vertices
        .iter()
        .map(|vertex| vertex.x)
        .min()
        .ok_or_else(|| "map has no vertices".to_owned())?;
    let maximum_x = core
        .vertices
        .iter()
        .map(|vertex| vertex.x)
        .max()
        .expect("the non-empty vertex set already has a maximum");
    let minimum_y = core
        .vertices
        .iter()
        .map(|vertex| vertex.y)
        .min()
        .expect("the non-empty vertex set already has a minimum");
    let maximum_y = core
        .vertices
        .iter()
        .map(|vertex| vertex.y)
        .max()
        .expect("the non-empty vertex set already has a maximum");

    const CLASSIC_BLOCKMAP_CELL_SPAN: i32 = 128;
    let blockmap_min_x = i32::from(core.blockmap.origin_x);
    let blockmap_min_y = i32::from(core.blockmap.origin_y);
    let blockmap_max_x =
        blockmap_min_x + i32::from(core.blockmap.columns) * CLASSIC_BLOCKMAP_CELL_SPAN;
    let blockmap_max_y =
        blockmap_min_y + i32::from(core.blockmap.rows) * CLASSIC_BLOCKMAP_CELL_SPAN;
    let minimum_x = i32::from(minimum_x).min(blockmap_min_x);
    let maximum_x = i32::from(maximum_x).max(blockmap_max_x);
    let minimum_y = i32::from(minimum_y).min(blockmap_min_y);
    let maximum_y = i32::from(maximum_y).max(blockmap_max_y);
    let padding = 64_i32;
    let view_x = minimum_x - padding;
    let view_y = -maximum_y - padding;
    let view_width = (maximum_x - minimum_x + (padding * 2)).max(1);
    let view_height = (maximum_y - minimum_y + (padding * 2)).max(1);
    let mut document = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{view_x} {view_y} {view_width} {view_height}\" role=\"img\" aria-label=\"{map_name}\">\n<rect x=\"{view_x}\" y=\"{view_y}\" width=\"{view_width}\" height=\"{view_height}\" fill=\"#071112\"/>\n<rect x=\"{blockmap_min_x}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#a78bfa\" stroke-width=\"8\" stroke-dasharray=\"32 24\"><title>BLOCKMAP: origin ({}, {}), {} by {} cells, classic cell span 128</title></rect>\n<g fill=\"none\" stroke-linecap=\"square\">\n",
        -blockmap_max_y,
        blockmap_max_x - blockmap_min_x,
        blockmap_max_y - blockmap_min_y,
        core.blockmap.origin_x,
        core.blockmap.origin_y,
        core.blockmap.columns,
        core.blockmap.rows,
        map_name = escape_xml(&format!("{} {}", core.map_name, map_svg_mode_label(mode))),
    );
    for linedef in &core.linedefs {
        let start = &core.vertices[usize::from(linedef.start_vertex)];
        let end = &core.vertices[usize::from(linedef.end_vertex)];
        let (color, sector) = match mode {
            MapSvgMode::SourceTopology => (
                if linedef.left_sidedef.is_some() && linedef.right_sidedef.is_some() {
                    "#637b7d"
                } else {
                    "#67e8dc"
                },
                None,
            ),
            MapSvgMode::SectorColor => {
                let sector = linedef
                    .right_sidedef
                    .or(linedef.left_sidedef)
                    .map(|sidedef| core.sidedefs[usize::from(sidedef)].sector);
                (sector_diagnostic_color(sector), sector)
            }
            MapSvgMode::WallNormals => ("#637b7d", None),
        };
        document.push_str(&format!(
            "<path d=\"M {} {} L {} {}\" stroke=\"{color}\" stroke-width=\"8\" data-source-lump=\"{}\" data-source-record=\"{}\" data-source-sector=\"{:?}\"><title>LINEDEF #{}: flags {:#06x}, special {}, tag {}, right {:?}, left {:?}, diagnostic sector {:?}</title></path>\n",
            start.x,
            -i32::from(start.y),
            end.x,
            -i32::from(end.y),
            linedef.source.lump_index,
            linedef.source.record_index,
            sector,
            linedef.source.record_index,
            linedef.flags,
            linedef.special,
            linedef.tag,
            linedef.right_sidedef,
            linedef.left_sidedef,
            sector,
        ));
    }
    if mode == MapSvgMode::WallNormals {
        append_wall_normal_svg(&mut document, core);
    }
    document.push_str("</g>\n<g fill=\"#fbbf24\" stroke=\"#071112\" stroke-width=\"4\">\n");
    for thing in &core.things {
        let (color, radius) = if thing.kind == 1 {
            ("#22d3ee", 28)
        } else {
            ("#fbbf24", 16)
        };
        document.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{radius}\" fill=\"{color}\" data-source-lump=\"{}\" data-source-record=\"{}\"><title>THING #{}: kind {}, angle {}, flags {:#06x}</title></circle>\n",
            thing.x,
            -i32::from(thing.y),
            thing.source.lump_index,
            thing.source.record_index,
            thing.source.record_index,
            thing.kind,
            thing.angle,
            thing.flags,
        ));
    }
    document.push_str("</g>\n</svg>\n");
    fs::write(path, document).map_err(|error| error.to_string())
}

fn map_svg_mode_label(mode: MapSvgMode) -> &'static str {
    match mode {
        MapSvgMode::SourceTopology => "top-down source topology",
        MapSvgMode::SectorColor => "top-down source-sector-color",
        MapSvgMode::WallNormals => "top-down wall-normal",
    }
}

fn append_wall_normal_svg(document: &mut String, core: &DoomMapCore) {
    const NORMAL_LENGTH: f64 = 48.0;
    document.push_str("</g>\n<g fill=\"none\" stroke-linecap=\"round\" stroke-width=\"8\">\n");
    for linedef in &core.linedefs {
        let start = &core.vertices[usize::from(linedef.start_vertex)];
        let end = &core.vertices[usize::from(linedef.end_vertex)];
        let delta_x = f64::from(end.x) - f64::from(start.x);
        let delta_y = f64::from(end.y) - f64::from(start.y);
        let length = delta_x.hypot(delta_y);
        if length == 0.0 {
            continue;
        }
        let midpoint_x = (f64::from(start.x) + f64::from(end.x)) * 0.5;
        let midpoint_y = (f64::from(start.y) + f64::from(end.y)) * 0.5;
        for (side, exists, color, normal_x, normal_y) in [
            (
                "right",
                linedef.right_sidedef.is_some(),
                "#22d3ee",
                delta_y,
                -delta_x,
            ),
            (
                "left",
                linedef.left_sidedef.is_some(),
                "#f472b6",
                -delta_y,
                delta_x,
            ),
        ] {
            if !exists {
                continue;
            }
            let endpoint_x = midpoint_x + normal_x / length * NORMAL_LENGTH;
            let endpoint_y = midpoint_y + normal_y / length * NORMAL_LENGTH;
            document.push_str(&format!(
                "<path d=\"M {midpoint_x} {} L {endpoint_x} {}\" stroke=\"{color}\" data-source-lump=\"{}\" data-source-record=\"{}\"><title>LINEDEF #{} {side} sidedef normal: WAD right/front is cyan and left/back is magenta; this headless arrow matches the lowered triangle winding, not lighting or visibility behavior</title></path>\n",
                -midpoint_y,
                -endpoint_y,
                linedef.source.lump_index,
                linedef.source.record_index,
                linedef.source.record_index,
            ));
        }
    }
    document.push_str("</g>\n<g fill=\"none\" stroke-linecap=\"square\">\n");
}

fn sector_diagnostic_color(sector: Option<u16>) -> &'static str {
    const COLORS: [&str; 12] = [
        "#22d3ee", "#a3e635", "#fbbf24", "#fb7185", "#c084fc", "#fb923c", "#2dd4bf", "#60a5fa",
        "#f472b6", "#bef264", "#facc15", "#818cf8",
    ];
    sector
        .map(|index| COLORS[usize::from(index) % COLORS.len()])
        .unwrap_or("#f8fafc")
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn format_bounded_count_inventory(inventory: &BTreeMap<u16, usize>) -> String {
    const MAXIMUM_ENTRIES: usize = 32;

    let mut entries = inventory
        .iter()
        .take(MAXIMUM_ENTRIES)
        .map(|(kind, count)| format!("{kind}:{count}"))
        .collect::<Vec<_>>();
    if inventory.len() > MAXIMUM_ENTRIES {
        entries.push(format!("…+{}", inventory.len() - MAXIMUM_ENTRIES));
    }
    format!("[{}]", entries.join(","))
}

fn print_manifest(manifest: &WadManifest) {
    println!(
        "{:?}: {} lumps, {} namespaces, {} declared bytes, source blake3 {}",
        manifest.kind,
        manifest.lumps.len(),
        manifest.namespaces.len(),
        manifest.total_lump_bytes,
        manifest.source.blake3,
    );
    for namespace in &manifest.namespaces {
        println!(
            "namespace {:?}: marker #{} through #{}, {} member lumps",
            namespace.kind,
            namespace.start_marker_index,
            namespace.end_marker_index,
            namespace.lump_indices.len(),
        );
    }
    for lump in &manifest.lumps {
        println!(
            "#{:04} {:<8} offset={} bytes={}",
            lump.index, lump.name, lump.offset, lump.size
        );
    }
}
