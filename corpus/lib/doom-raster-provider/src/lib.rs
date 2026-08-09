//! Bounded, source-indexed observations for Doom's global indexed-raster data.
//!
//! Palette and colour-map bytes describe source data. They do not prescribe a
//! renderer's lighting equation, colour space, or upload representation.

use doom_wad_provider::{WadLumpObservation, WadManifest};
use raster_image_corpus::{AlphaMode, ColorSpace, DecodedImage, ImageOrientation, PixelFormat};
use thiserror::Error;

const PALETTE_COLORS: usize = 256;
const RGB_BYTES_PER_COLOR: usize = 3;
const PALETTE_BYTES: usize = PALETTE_COLORS * RGB_BYTES_PER_COLOR;
const COLORMAP_BYTES: usize = 256;
const WAD_NAME_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomRasterDecodeLimits {
    pub max_playpal_bytes: usize,
    pub max_palettes: usize,
    pub max_colormap_bytes: usize,
    pub max_colormaps: usize,
    pub max_total_decoded_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomPatchDecodeLimits {
    pub max_patch_bytes: usize,
    pub max_width: usize,
    pub max_height: usize,
    pub max_pixels: usize,
    pub max_posts: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomTextureDecodeLimits {
    pub max_pnames_bytes: usize,
    pub max_texture_bytes: usize,
    pub max_patch_names: usize,
    pub max_textures: usize,
    pub max_patches_per_texture: usize,
    pub max_total_patch_references: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomTextureComposeLimits {
    pub max_width: usize,
    pub max_height: usize,
    pub max_pixels: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomFlatDecodeLimits {
    pub max_flat_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomRgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomPalette {
    pub source_lump_index: u32,
    pub palette_index: u32,
    pub colors: [DoomRgb; PALETTE_COLORS],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomColormap {
    pub source_lump_index: u32,
    pub map_index: u32,
    /// Maps one source palette index to another; it is not renderer lighting.
    pub index_remap: [u8; COLORMAP_BYTES],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomRasterGlobals {
    pub palettes: Vec<DoomPalette>,
    pub colormaps: Vec<DoomColormap>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomPatchPost {
    pub top_delta: u8,
    pub color_indices: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomPatchColumn {
    pub column_index: u32,
    pub posts: Vec<DoomPatchPost>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomPatch {
    pub source_lump_index: u32,
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub left_offset: i16,
    pub top_offset: i16,
    pub opaque_pixels: usize,
    pub columns: Vec<DoomPatchColumn>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomTexturePatchReference {
    pub patch_name_index: u16,
    pub patch_name: String,
    pub origin_x: i16,
    pub origin_y: i16,
    pub step_direction: u16,
    pub colormap: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomTextureDefinition {
    pub source_lump_index: u32,
    pub name: String,
    pub masked: i32,
    pub width: u16,
    pub height: u16,
    pub column_directory: i32,
    pub patches: Vec<DoomTexturePatchReference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomTextureCatalog {
    pub patch_name_source_lump_index: u32,
    pub patch_names: Vec<String>,
    pub textures: Vec<DoomTextureDefinition>,
}

/// Indexed pixels and coverage from Doom patch composition, before palette or
/// renderer policy is selected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomIndexedImage {
    pub source_texture_lump_index: u32,
    pub texture_name: String,
    pub width: u16,
    pub height: u16,
    pub color_indices: Vec<u8>,
    pub coverage: Vec<bool>,
    pub opaque_pixels: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomFlat {
    pub source_lump_index: u32,
    pub name: String,
    pub color_indices: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSpriteFrameRotation {
    pub source_lump_index: u32,
    pub sprite: String,
    pub frame: char,
    pub rotation: u8,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomRasterDecodeError {
    #[error("global lump `{name}` is missing")]
    MissingGlobalLump { name: &'static str },
    #[error(
        "global lump `{name}` has duplicate source entries at {first_index} and {second_index}"
    )]
    DuplicateGlobalLump {
        name: &'static str,
        first_index: u32,
        second_index: u32,
    },
    #[error("{name} range offset={offset}, bytes={size} is outside a {wad_bytes}-byte WAD input")]
    LumpOutOfBounds {
        name: &'static str,
        offset: u32,
        size: u32,
        wad_bytes: usize,
    },
    #[error("{name} has {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    LumpByteLimitExceeded {
        name: &'static str,
        actual_bytes: usize,
        limit_bytes: usize,
    },
    #[error("PLAYPAL has {actual_bytes} bytes, not a positive multiple of {palette_bytes}")]
    InvalidPaletteLength {
        actual_bytes: usize,
        palette_bytes: usize,
    },
    #[error(
        "PLAYPAL declares {actual_palettes} palettes, exceeding the {limit_palettes}-palette limit"
    )]
    PaletteCountLimitExceeded {
        actual_palettes: usize,
        limit_palettes: usize,
    },
    #[error("COLORMAP has {actual_bytes} bytes, not a positive multiple of {colormap_bytes}")]
    InvalidColormapLength {
        actual_bytes: usize,
        colormap_bytes: usize,
    },
    #[error(
        "COLORMAP declares {actual_colormaps} maps, exceeding the {limit_colormaps}-map limit"
    )]
    ColormapCountLimitExceeded {
        actual_colormaps: usize,
        limit_colormaps: usize,
    },
    #[error("decoded raster globals require {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    TotalDecodedBytesLimitExceeded {
        actual_bytes: usize,
        limit_bytes: usize,
    },
    #[error("patch `{name}` was not found in a patch namespace")]
    MissingPatch { name: String },
    #[error("patch `{name}` is {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    PatchByteLimitExceeded {
        name: String,
        actual_bytes: usize,
        limit_bytes: usize,
    },
    #[error("patch `{name}` is {actual_bytes} bytes, smaller than its header")]
    PatchHeaderTooShort { name: String, actual_bytes: usize },
    #[error("patch `{name}` dimensions {width} by {height} exceed configured limits")]
    PatchDimensionLimitExceeded {
        name: String,
        width: u16,
        height: u16,
    },
    #[error(
        "patch `{name}` is {actual_bytes} bytes but needs {required_bytes} for its column offsets"
    )]
    PatchColumnOffsetsTruncated {
        name: String,
        actual_bytes: usize,
        required_bytes: usize,
    },
    #[error("patch `{name}` column {column_index} offset {offset} is invalid")]
    PatchColumnOffsetInvalid {
        name: String,
        column_index: u32,
        offset: u32,
    },
    #[error("patch `{name}` column {column_index} post stream is truncated")]
    PatchPostTruncated { name: String, column_index: u32 },
    #[error("patch `{name}` column {column_index} post exceeds its height")]
    PatchPostOutOfBounds { name: String, column_index: u32 },
    #[error("patch `{name}` exceeds its {limit_posts}-post limit")]
    PatchPostLimitExceeded { name: String, limit_posts: usize },
    #[error("{name} has {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    TextureLumpByteLimitExceeded {
        name: &'static str,
        actual_bytes: usize,
        limit_bytes: usize,
    },
    #[error(
        "PNAMES has {actual_bytes} bytes but requires {required_bytes} for {patch_names} names"
    )]
    PatchNamesTruncated {
        actual_bytes: usize,
        required_bytes: usize,
        patch_names: usize,
    },
    #[error(
        "PNAMES declares {actual_patch_names} names, exceeding the {limit_patch_names}-name limit"
    )]
    PatchNameCountLimitExceeded {
        actual_patch_names: usize,
        limit_patch_names: usize,
    },
    #[error(
        "{name} has {actual_bytes} bytes but requires {required_bytes} for its texture offsets"
    )]
    TextureOffsetsTruncated {
        name: &'static str,
        actual_bytes: usize,
        required_bytes: usize,
    },
    #[error(
        "{name} declares {actual_textures} textures, exceeding the {limit_textures}-texture limit"
    )]
    TextureCountLimitExceeded {
        name: &'static str,
        actual_textures: usize,
        limit_textures: usize,
    },
    #[error("{name} texture {texture_index} offset {offset} is invalid")]
    TextureOffsetInvalid {
        name: &'static str,
        texture_index: u32,
        offset: u32,
    },
    #[error("{name} texture {texture_index} is truncated")]
    TextureRecordTruncated {
        name: &'static str,
        texture_index: u32,
    },
    #[error("{name} texture {texture_index} declares {actual_patches} patches, exceeding the {limit_patches}-patch limit")]
    TexturePatchCountLimitExceeded {
        name: &'static str,
        texture_index: u32,
        actual_patches: usize,
        limit_patches: usize,
    },
    #[error("texture records contain more than the {limit_references} allowed patch references")]
    TotalTexturePatchReferencesLimitExceeded { limit_references: usize },
    #[error("{name} texture {texture_index} refers to missing PNAMES entry {patch_name_index}")]
    TexturePatchNameMissing {
        name: &'static str,
        texture_index: u32,
        patch_name_index: u16,
    },
    #[error("{name} contains duplicate texture name `{texture_name}`")]
    DuplicateTextureName {
        name: &'static str,
        texture_name: String,
    },
    #[error("texture `{name}` was not found in the decoded catalog")]
    MissingTexture { name: String },
    #[error(
        "texture `{name}` dimensions {width} by {height} exceed configured composition limits"
    )]
    TextureCompositionDimensionLimitExceeded {
        name: String,
        width: u16,
        height: u16,
    },
    #[error("flat `{name}` was not found in a flat namespace")]
    MissingFlat { name: String },
    #[error("flat `{name}` is {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    FlatByteLimitExceeded {
        name: String,
        actual_bytes: usize,
        limit_bytes: usize,
    },
    #[error("flat `{name}` has {actual_bytes} bytes; Doom flats require 4096")]
    InvalidFlatLength { name: String, actual_bytes: usize },
    #[error("sprite lump `{name}` has an invalid classic Doom frame/rotation name")]
    InvalidSpriteName { name: String },
    #[error("provider-neutral image lowering failed: {reason}")]
    RasterLoweringFailed { reason: String },
}

/// Decodes the global indexed palette and colour-map resources from WAD bytes.
pub fn decode_doom_raster_globals(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    limits: DoomRasterDecodeLimits,
) -> Result<DoomRasterGlobals, DoomRasterDecodeError> {
    let playpal = global_lump(manifest, "PLAYPAL")?;
    let colormap = global_lump(manifest, "COLORMAP")?;
    let playpal_bytes = lump_bytes(wad_bytes, playpal, "PLAYPAL")?;
    let colormap_bytes = lump_bytes(wad_bytes, colormap, "COLORMAP")?;
    if playpal_bytes.len() > limits.max_playpal_bytes {
        return Err(DoomRasterDecodeError::LumpByteLimitExceeded {
            name: "PLAYPAL",
            actual_bytes: playpal_bytes.len(),
            limit_bytes: limits.max_playpal_bytes,
        });
    }
    if colormap_bytes.len() > limits.max_colormap_bytes {
        return Err(DoomRasterDecodeError::LumpByteLimitExceeded {
            name: "COLORMAP",
            actual_bytes: colormap_bytes.len(),
            limit_bytes: limits.max_colormap_bytes,
        });
    }
    let total = playpal_bytes.len() + colormap_bytes.len();
    if total > limits.max_total_decoded_bytes {
        return Err(DoomRasterDecodeError::TotalDecodedBytesLimitExceeded {
            actual_bytes: total,
            limit_bytes: limits.max_total_decoded_bytes,
        });
    }
    let palettes = decode_palettes(playpal_bytes, playpal.index, limits)?;
    let colormaps = decode_colormaps(colormap_bytes, colormap.index, limits)?;
    Ok(DoomRasterGlobals {
        palettes,
        colormaps,
    })
}

/// Decodes one named patch from the WAD patch namespaces as indexed columns.
pub fn decode_doom_patch(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    name: &str,
    limits: DoomPatchDecodeLimits,
) -> Result<DoomPatch, DoomRasterDecodeError> {
    decode_named_doom_patch(
        wad_bytes,
        manifest,
        name,
        limits,
        doom_wad_provider::WadNamespaceKind::Patches,
    )
}

/// Decodes one named sprite lump using the classic Doom patch encoding.
pub fn decode_doom_sprite_patch(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    name: &str,
    limits: DoomPatchDecodeLimits,
) -> Result<DoomPatch, DoomRasterDecodeError> {
    decode_named_doom_patch(
        wad_bytes,
        manifest,
        name,
        limits,
        doom_wad_provider::WadNamespaceKind::Sprites,
    )
}

fn decode_named_doom_patch(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    name: &str,
    limits: DoomPatchDecodeLimits,
    namespace_kind: doom_wad_provider::WadNamespaceKind,
) -> Result<DoomPatch, DoomRasterDecodeError> {
    let lump = manifest
        .namespaces
        .iter()
        .filter(|namespace| namespace.kind == namespace_kind)
        .flat_map(|namespace| namespace.lump_indices.iter())
        .filter_map(|index| manifest.lumps.get(*index as usize))
        // Classic Doom lump identity is ASCII case-insensitive. Preserve the
        // source spelling on the selected lump for diagnostics, but compare
        // the PNAMES/TEXTURE reference at the Doom semantic lookup boundary.
        .find(|lump| lump.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| DoomRasterDecodeError::MissingPatch {
            name: name.to_owned(),
        })?;
    let bytes = lump_bytes(wad_bytes, lump, "patch")?;
    if bytes.len() > limits.max_patch_bytes {
        return Err(DoomRasterDecodeError::PatchByteLimitExceeded {
            name: name.to_owned(),
            actual_bytes: bytes.len(),
            limit_bytes: limits.max_patch_bytes,
        });
    }
    if bytes.len() < 8 {
        return Err(DoomRasterDecodeError::PatchHeaderTooShort {
            name: name.to_owned(),
            actual_bytes: bytes.len(),
        });
    }
    let width = read_u16(bytes, 0);
    let height = read_u16(bytes, 2);
    if width == 0
        || height == 0
        || usize::from(width) > limits.max_width
        || usize::from(height) > limits.max_height
        || usize::from(width) * usize::from(height) > limits.max_pixels
    {
        return Err(DoomRasterDecodeError::PatchDimensionLimitExceeded {
            name: name.to_owned(),
            width,
            height,
        });
    }
    let header_bytes = 8 + usize::from(width) * 4;
    if bytes.len() < header_bytes {
        return Err(DoomRasterDecodeError::PatchColumnOffsetsTruncated {
            name: name.to_owned(),
            actual_bytes: bytes.len(),
            required_bytes: header_bytes,
        });
    }
    let mut opaque_pixels = 0;
    let mut post_count = 0;
    let mut columns = Vec::with_capacity(usize::from(width));
    for column_index in 0..width {
        let offset = read_u32(bytes, 8 + usize::from(column_index) * 4);
        let mut cursor = offset as usize;
        if cursor < header_bytes || cursor >= bytes.len() {
            return Err(DoomRasterDecodeError::PatchColumnOffsetInvalid {
                name: name.to_owned(),
                column_index: u32::from(column_index),
                offset,
            });
        }
        let mut posts = Vec::new();
        loop {
            let Some(&top_delta) = bytes.get(cursor) else {
                return Err(DoomRasterDecodeError::PatchPostTruncated {
                    name: name.to_owned(),
                    column_index: u32::from(column_index),
                });
            };
            cursor += 1;
            if top_delta == u8::MAX {
                break;
            }
            let Some(&length) = bytes.get(cursor) else {
                return Err(DoomRasterDecodeError::PatchPostTruncated {
                    name: name.to_owned(),
                    column_index: u32::from(column_index),
                });
            };
            cursor += 2; // length plus unused byte
            let end = cursor + usize::from(length);
            let indices = bytes.get(cursor..end).ok_or_else(|| {
                DoomRasterDecodeError::PatchPostTruncated {
                    name: name.to_owned(),
                    column_index: u32::from(column_index),
                }
            })?;
            cursor = end + 1; // trailing unused byte
            if cursor > bytes.len() {
                return Err(DoomRasterDecodeError::PatchPostTruncated {
                    name: name.to_owned(),
                    column_index: u32::from(column_index),
                });
            }
            if usize::from(top_delta) + usize::from(length) > usize::from(height) {
                return Err(DoomRasterDecodeError::PatchPostOutOfBounds {
                    name: name.to_owned(),
                    column_index: u32::from(column_index),
                });
            }
            post_count += 1;
            if post_count > limits.max_posts {
                return Err(DoomRasterDecodeError::PatchPostLimitExceeded {
                    name: name.to_owned(),
                    limit_posts: limits.max_posts,
                });
            }
            opaque_pixels += indices.len();
            posts.push(DoomPatchPost {
                top_delta,
                color_indices: indices.to_vec(),
            });
        }
        columns.push(DoomPatchColumn {
            column_index: u32::from(column_index),
            posts,
        });
    }
    Ok(DoomPatch {
        source_lump_index: lump.index,
        name: name.to_owned(),
        width,
        height,
        left_offset: read_i16(bytes, 4),
        top_offset: read_i16(bytes, 6),
        opaque_pixels,
        columns,
    })
}

/// Decodes PNAMES and TEXTURE1/TEXTURE2 records without composing pixels.
pub fn decode_doom_texture_catalog(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    limits: DoomTextureDecodeLimits,
) -> Result<DoomTextureCatalog, DoomRasterDecodeError> {
    let patch_names_lump = global_lump(manifest, "PNAMES")?;
    let patch_names_bytes = lump_bytes(wad_bytes, patch_names_lump, "PNAMES")?;
    if patch_names_bytes.len() > limits.max_pnames_bytes {
        return Err(DoomRasterDecodeError::TextureLumpByteLimitExceeded {
            name: "PNAMES",
            actual_bytes: patch_names_bytes.len(),
            limit_bytes: limits.max_pnames_bytes,
        });
    }
    let patch_names = decode_patch_names(patch_names_bytes, limits)?;

    let mut textures = decode_texture_lump(
        wad_bytes,
        global_lump(manifest, "TEXTURE1")?,
        "TEXTURE1",
        &patch_names,
        limits,
    )?;
    if let Some(texture2) = optional_global_lump(manifest, "TEXTURE2")? {
        textures.extend(decode_texture_lump(
            wad_bytes,
            texture2,
            "TEXTURE2",
            &patch_names,
            limits,
        )?);
    }
    let mut names = std::collections::BTreeSet::new();
    for texture in &textures {
        if !names.insert(texture.name.clone()) {
            return Err(DoomRasterDecodeError::DuplicateTextureName {
                name: "TEXTURE1/TEXTURE2",
                texture_name: texture.name.clone(),
            });
        }
    }
    Ok(DoomTextureCatalog {
        patch_name_source_lump_index: patch_names_lump.index,
        patch_names,
        textures,
    })
}

/// Composes a decoded texture's indexed pixels in source patch order.
///
/// Later patch samples replace earlier samples at the same covered pixel.
/// Uncovered pixels retain an unspecified index with `coverage == false`.
pub fn compose_doom_texture(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    catalog: &DoomTextureCatalog,
    name: &str,
    patch_limits: DoomPatchDecodeLimits,
    limits: DoomTextureComposeLimits,
) -> Result<DoomIndexedImage, DoomRasterDecodeError> {
    let texture = catalog
        .textures
        .iter()
        .find(|texture| texture.name == name)
        .ok_or_else(|| DoomRasterDecodeError::MissingTexture {
            name: name.to_owned(),
        })?;
    let width = usize::from(texture.width);
    let height = usize::from(texture.height);
    let pixel_count = width.saturating_mul(height);
    if width == 0
        || height == 0
        || width > limits.max_width
        || height > limits.max_height
        || pixel_count > limits.max_pixels
    {
        return Err(
            DoomRasterDecodeError::TextureCompositionDimensionLimitExceeded {
                name: name.to_owned(),
                width: texture.width,
                height: texture.height,
            },
        );
    }
    let mut color_indices = vec![0; pixel_count];
    let mut coverage = vec![false; pixel_count];
    for reference in &texture.patches {
        let patch = decode_doom_patch(wad_bytes, manifest, &reference.patch_name, patch_limits)?;
        for column in &patch.columns {
            let x = i32::from(reference.origin_x) + column.column_index as i32;
            if !(0..width as i32).contains(&x) {
                continue;
            }
            for post in &column.posts {
                let post_y = i32::from(reference.origin_y) + i32::from(post.top_delta);
                for (offset, &color_index) in post.color_indices.iter().enumerate() {
                    let y = post_y + offset as i32;
                    if !(0..height as i32).contains(&y) {
                        continue;
                    }
                    let target = y as usize * width + x as usize;
                    color_indices[target] = color_index;
                    coverage[target] = true;
                }
            }
        }
    }
    let opaque_pixels = coverage.iter().filter(|&&covered| covered).count();
    Ok(DoomIndexedImage {
        source_texture_lump_index: texture.source_lump_index,
        texture_name: texture.name.clone(),
        width: texture.width,
        height: texture.height,
        color_indices,
        coverage,
        opaque_pixels,
    })
}

/// Decodes one marker-scoped classic Doom flat as 64x64 palette indices.
pub fn decode_doom_flat(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    name: &str,
    limits: DoomFlatDecodeLimits,
) -> Result<DoomFlat, DoomRasterDecodeError> {
    const FLAT_BYTES: usize = 64 * 64;
    let lump = manifest
        .namespaces
        .iter()
        .filter(|namespace| namespace.kind == doom_wad_provider::WadNamespaceKind::Flats)
        .flat_map(|namespace| namespace.lump_indices.iter())
        .filter_map(|index| manifest.lumps.get(*index as usize))
        .find(|lump| lump.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| DoomRasterDecodeError::MissingFlat {
            name: name.to_owned(),
        })?;
    let bytes = lump_bytes(wad_bytes, lump, "flat")?;
    if bytes.len() > limits.max_flat_bytes {
        return Err(DoomRasterDecodeError::FlatByteLimitExceeded {
            name: name.to_owned(),
            actual_bytes: bytes.len(),
            limit_bytes: limits.max_flat_bytes,
        });
    }
    if bytes.len() != FLAT_BYTES {
        return Err(DoomRasterDecodeError::InvalidFlatLength {
            name: name.to_owned(),
            actual_bytes: bytes.len(),
        });
    }
    Ok(DoomFlat {
        source_lump_index: lump.index,
        name: name.to_owned(),
        color_indices: bytes.to_vec(),
    })
}

/// Projects classic sprite lump names into frame/rotation observations.
pub fn decode_doom_sprite_frame_rotations(
    manifest: &WadManifest,
) -> Result<Vec<DoomSpriteFrameRotation>, DoomRasterDecodeError> {
    let mut frames = Vec::new();
    for lump in manifest
        .namespaces
        .iter()
        .filter(|namespace| namespace.kind == doom_wad_provider::WadNamespaceKind::Sprites)
        .flat_map(|namespace| namespace.lump_indices.iter())
        .filter_map(|index| manifest.lumps.get(*index as usize))
    {
        let name = lump.name.as_bytes();
        if !(name.len() == 6 || name.len() == 8) {
            return Err(DoomRasterDecodeError::InvalidSpriteName {
                name: lump.name.clone(),
            });
        }
        let sprite = lump.name[..4].to_owned();
        for pair in [4, 6].into_iter().take((name.len() - 4) / 2) {
            let frame = name[pair] as char;
            let rotation = name[pair + 1];
            if !frame.is_ascii_uppercase() || !matches!(rotation, b'0'..=b'8') {
                return Err(DoomRasterDecodeError::InvalidSpriteName {
                    name: lump.name.clone(),
                });
            }
            frames.push(DoomSpriteFrameRotation {
                source_lump_index: lump.index,
                sprite: sprite.clone(),
                frame,
                rotation: rotation - b'0',
            });
        }
    }
    Ok(frames)
}

/// Returns a stable FNV-1a fingerprint of ordered sprite frame/rotation observations.
pub fn doom_sprite_frame_rotation_fingerprint(frames: &[DoomSpriteFrameRotation]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for frame in frames {
        for byte in frame
            .source_lump_index
            .to_le_bytes()
            .into_iter()
            .chain(frame.sprite.bytes())
            .chain([frame.frame as u8, frame.rotation])
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("{hash:016x}")
}

/// Lowers indexed pixels using an explicitly selected Doom palette.
///
/// The palette has no embedded colour profile, so the result records an
/// unspecified colour space. Coverage becomes straight alpha; no renderer or
/// COLORMAP lighting behavior is selected here.
pub fn lower_doom_indexed_image(
    image: &DoomIndexedImage,
    palette: &DoomPalette,
) -> Result<DecodedImage, DoomRasterDecodeError> {
    let mut pixels = Vec::with_capacity(image.color_indices.len() * 4);
    for (&index, &covered) in image.color_indices.iter().zip(&image.coverage) {
        let color = palette.colors[usize::from(index)];
        pixels.extend_from_slice(&[
            color.red,
            color.green,
            color.blue,
            if covered { u8::MAX } else { 0 },
        ]);
    }
    let alpha_mode = if image.coverage.iter().all(|covered| *covered) {
        AlphaMode::Opaque
    } else {
        AlphaMode::Straight
    };
    let lowered = DecodedImage {
        width: u32::from(image.width),
        height: u32::from(image.height),
        row_stride: usize::from(image.width) * 4,
        pixel_format: PixelFormat::Rgba8,
        color_space: ColorSpace::Unspecified,
        alpha_mode,
        source_orientation: ImageOrientation::TopDown,
        output_orientation: ImageOrientation::TopDown,
        source_bit_depth: 8,
        source_row_stride: usize::from(image.width),
        pixels,
    };
    lowered
        .validate()
        .map_err(|error| DoomRasterDecodeError::RasterLoweringFailed {
            reason: error.to_string(),
        })?;
    Ok(lowered)
}

/// Makes a standalone indexed image from one decoded patch.
pub fn indexed_image_from_doom_patch(patch: &DoomPatch) -> DoomIndexedImage {
    let width = usize::from(patch.width);
    let height = usize::from(patch.height);
    let mut color_indices = vec![0; width * height];
    let mut coverage = vec![false; width * height];
    for column in &patch.columns {
        for post in &column.posts {
            for (offset, &color_index) in post.color_indices.iter().enumerate() {
                let target =
                    (usize::from(post.top_delta) + offset) * width + column.column_index as usize;
                color_indices[target] = color_index;
                coverage[target] = true;
            }
        }
    }
    DoomIndexedImage {
        source_texture_lump_index: patch.source_lump_index,
        texture_name: patch.name.clone(),
        width: patch.width,
        height: patch.height,
        color_indices,
        coverage,
        opaque_pixels: patch.opaque_pixels,
    }
}

/// Makes a fully covered indexed image from one fixed-size Doom flat.
pub fn indexed_image_from_doom_flat(flat: &DoomFlat) -> DoomIndexedImage {
    DoomIndexedImage {
        source_texture_lump_index: flat.source_lump_index,
        texture_name: flat.name.clone(),
        width: 64,
        height: 64,
        color_indices: flat.color_indices.clone(),
        coverage: vec![true; flat.color_indices.len()],
        opaque_pixels: flat.color_indices.len(),
    }
}

/// Forms a canonical 256x1 index ramp for a selected Doom palette artifact.
pub fn indexed_image_from_doom_palette(palette: &DoomPalette) -> DoomIndexedImage {
    DoomIndexedImage {
        source_texture_lump_index: palette.source_lump_index,
        texture_name: format!("PLAYPAL-{}", palette.palette_index),
        width: 256,
        height: 1,
        color_indices: (0..=u8::MAX).collect(),
        coverage: vec![true; 256],
        opaque_pixels: 256,
    }
}

fn decode_patch_names(
    bytes: &[u8],
    limits: DoomTextureDecodeLimits,
) -> Result<Vec<String>, DoomRasterDecodeError> {
    if bytes.len() < 4 {
        return Err(DoomRasterDecodeError::PatchNamesTruncated {
            actual_bytes: bytes.len(),
            required_bytes: 4,
            patch_names: 0,
        });
    }
    let count = read_u32(bytes, 0) as usize;
    if count > limits.max_patch_names {
        return Err(DoomRasterDecodeError::PatchNameCountLimitExceeded {
            actual_patch_names: count,
            limit_patch_names: limits.max_patch_names,
        });
    }
    let required = 4usize.saturating_add(count.saturating_mul(WAD_NAME_BYTES));
    if bytes.len() < required {
        return Err(DoomRasterDecodeError::PatchNamesTruncated {
            actual_bytes: bytes.len(),
            required_bytes: required,
            patch_names: count,
        });
    }
    Ok((0..count)
        .map(|index| decode_wad_name(&bytes[4 + index * WAD_NAME_BYTES..][..WAD_NAME_BYTES]))
        .collect())
}

fn decode_texture_lump(
    wad_bytes: &[u8],
    lump: &WadLumpObservation,
    name: &'static str,
    patch_names: &[String],
    limits: DoomTextureDecodeLimits,
) -> Result<Vec<DoomTextureDefinition>, DoomRasterDecodeError> {
    let bytes = lump_bytes(wad_bytes, lump, name)?;
    if bytes.len() > limits.max_texture_bytes {
        return Err(DoomRasterDecodeError::TextureLumpByteLimitExceeded {
            name,
            actual_bytes: bytes.len(),
            limit_bytes: limits.max_texture_bytes,
        });
    }
    if bytes.len() < 4 {
        return Err(DoomRasterDecodeError::TextureOffsetsTruncated {
            name,
            actual_bytes: bytes.len(),
            required_bytes: 4,
        });
    }
    let count = read_u32(bytes, 0) as usize;
    if count > limits.max_textures {
        return Err(DoomRasterDecodeError::TextureCountLimitExceeded {
            name,
            actual_textures: count,
            limit_textures: limits.max_textures,
        });
    }
    let offset_table_bytes = 4usize.saturating_add(count.saturating_mul(4));
    if bytes.len() < offset_table_bytes {
        return Err(DoomRasterDecodeError::TextureOffsetsTruncated {
            name,
            actual_bytes: bytes.len(),
            required_bytes: offset_table_bytes,
        });
    }

    let mut total_patch_references = 0;
    let mut textures = Vec::with_capacity(count);
    for texture_index in 0..count {
        let offset = read_u32(bytes, 4 + texture_index * 4);
        let start = offset as usize;
        if start < offset_table_bytes || start > bytes.len() {
            return Err(DoomRasterDecodeError::TextureOffsetInvalid {
                name,
                texture_index: texture_index as u32,
                offset,
            });
        }
        const HEADER_BYTES: usize = 22;
        let header = bytes.get(start..start + HEADER_BYTES).ok_or(
            DoomRasterDecodeError::TextureRecordTruncated {
                name,
                texture_index: texture_index as u32,
            },
        )?;
        let patch_count = read_u16(header, 20) as usize;
        if patch_count > limits.max_patches_per_texture {
            return Err(DoomRasterDecodeError::TexturePatchCountLimitExceeded {
                name,
                texture_index: texture_index as u32,
                actual_patches: patch_count,
                limit_patches: limits.max_patches_per_texture,
            });
        }
        total_patch_references += patch_count;
        if total_patch_references > limits.max_total_patch_references {
            return Err(
                DoomRasterDecodeError::TotalTexturePatchReferencesLimitExceeded {
                    limit_references: limits.max_total_patch_references,
                },
            );
        }
        let record_bytes = HEADER_BYTES + patch_count * 10;
        let record = bytes.get(start..start + record_bytes).ok_or(
            DoomRasterDecodeError::TextureRecordTruncated {
                name,
                texture_index: texture_index as u32,
            },
        )?;
        let mut patches = Vec::with_capacity(patch_count);
        for patch_offset in (HEADER_BYTES..record_bytes).step_by(10) {
            let patch_name_index = read_u16(record, patch_offset + 4);
            let patch_name = patch_names.get(usize::from(patch_name_index)).ok_or(
                DoomRasterDecodeError::TexturePatchNameMissing {
                    name,
                    texture_index: texture_index as u32,
                    patch_name_index,
                },
            )?;
            patches.push(DoomTexturePatchReference {
                patch_name_index,
                patch_name: patch_name.clone(),
                origin_x: read_i16(record, patch_offset),
                origin_y: read_i16(record, patch_offset + 2),
                step_direction: read_u16(record, patch_offset + 6),
                colormap: read_u16(record, patch_offset + 8),
            });
        }
        textures.push(DoomTextureDefinition {
            source_lump_index: lump.index,
            name: decode_wad_name(&header[..WAD_NAME_BYTES]),
            masked: read_i32(header, 8),
            width: read_u16(header, 12),
            height: read_u16(header, 14),
            column_directory: read_i32(header, 16),
            patches,
        });
    }
    Ok(textures)
}

fn global_lump<'a>(
    manifest: &'a WadManifest,
    name: &'static str,
) -> Result<&'a WadLumpObservation, DoomRasterDecodeError> {
    let mut matches = manifest.lumps.iter().filter(|lump| lump.name == name);
    let first = matches
        .next()
        .ok_or(DoomRasterDecodeError::MissingGlobalLump { name })?;
    if let Some(second) = matches.next() {
        return Err(DoomRasterDecodeError::DuplicateGlobalLump {
            name,
            first_index: first.index,
            second_index: second.index,
        });
    }
    Ok(first)
}

fn optional_global_lump<'a>(
    manifest: &'a WadManifest,
    name: &'static str,
) -> Result<Option<&'a WadLumpObservation>, DoomRasterDecodeError> {
    let mut matches = manifest.lumps.iter().filter(|lump| lump.name == name);
    let Some(first) = matches.next() else {
        return Ok(None);
    };
    if let Some(second) = matches.next() {
        return Err(DoomRasterDecodeError::DuplicateGlobalLump {
            name,
            first_index: first.index,
            second_index: second.index,
        });
    }
    Ok(Some(first))
}

fn lump_bytes<'a>(
    wad_bytes: &'a [u8],
    lump: &WadLumpObservation,
    name: &'static str,
) -> Result<&'a [u8], DoomRasterDecodeError> {
    let start = lump.offset as usize;
    let end =
        start
            .checked_add(lump.size as usize)
            .ok_or(DoomRasterDecodeError::LumpOutOfBounds {
                name,
                offset: lump.offset,
                size: lump.size,
                wad_bytes: wad_bytes.len(),
            })?;
    wad_bytes
        .get(start..end)
        .ok_or(DoomRasterDecodeError::LumpOutOfBounds {
            name,
            offset: lump.offset,
            size: lump.size,
            wad_bytes: wad_bytes.len(),
        })
}

fn decode_palettes(
    bytes: &[u8],
    lump_index: u32,
    limits: DoomRasterDecodeLimits,
) -> Result<Vec<DoomPalette>, DoomRasterDecodeError> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(PALETTE_BYTES) {
        return Err(DoomRasterDecodeError::InvalidPaletteLength {
            actual_bytes: bytes.len(),
            palette_bytes: PALETTE_BYTES,
        });
    }
    let count = bytes.len() / PALETTE_BYTES;
    if count > limits.max_palettes {
        return Err(DoomRasterDecodeError::PaletteCountLimitExceeded {
            actual_palettes: count,
            limit_palettes: limits.max_palettes,
        });
    }
    Ok((0..count)
        .map(|palette_index| DoomPalette {
            source_lump_index: lump_index,
            palette_index: palette_index as u32,
            colors: std::array::from_fn(|color_index| {
                let offset = palette_index * PALETTE_BYTES + color_index * RGB_BYTES_PER_COLOR;
                DoomRgb {
                    red: bytes[offset],
                    green: bytes[offset + 1],
                    blue: bytes[offset + 2],
                }
            }),
        })
        .collect())
}

fn decode_colormaps(
    bytes: &[u8],
    lump_index: u32,
    limits: DoomRasterDecodeLimits,
) -> Result<Vec<DoomColormap>, DoomRasterDecodeError> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(COLORMAP_BYTES) {
        return Err(DoomRasterDecodeError::InvalidColormapLength {
            actual_bytes: bytes.len(),
            colormap_bytes: COLORMAP_BYTES,
        });
    }
    let count = bytes.len() / COLORMAP_BYTES;
    if count > limits.max_colormaps {
        return Err(DoomRasterDecodeError::ColormapCountLimitExceeded {
            actual_colormaps: count,
            limit_colormaps: limits.max_colormaps,
        });
    }
    Ok((0..count)
        .map(|map_index| DoomColormap {
            source_lump_index: lump_index,
            map_index: map_index as u32,
            index_remap: bytes[map_index * COLORMAP_BYTES..][..COLORMAP_BYTES]
                .try_into()
                .expect("validated colour-map range"),
        })
        .collect())
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed field"))
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed field"))
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed field"))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed field"))
}

fn decode_wad_name(bytes: &[u8]) -> String {
    bytes
        .iter()
        .copied()
        .take_while(|byte| *byte != 0)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use doom_wad_provider::{
        WadKind, WadNamespaceKind, WadNamespaceObservation, WadSourceIdentity,
    };

    fn limits() -> DoomRasterDecodeLimits {
        DoomRasterDecodeLimits {
            max_playpal_bytes: 4096,
            max_palettes: 4,
            max_colormap_bytes: 4096,
            max_colormaps: 16,
            max_total_decoded_bytes: 8192,
        }
    }

    fn manifest(playpal_bytes: usize, colormap_bytes: usize) -> WadManifest {
        WadManifest {
            source: WadSourceIdentity {
                label: "synthetic/raster.wad".to_owned(),
                byte_len: playpal_bytes + colormap_bytes,
                blake3: "synthetic".to_owned(),
            },
            kind: WadKind::Iwad,
            directory_offset: 0,
            directory_bytes: 0,
            total_lump_bytes: (playpal_bytes + colormap_bytes) as u64,
            lumps: vec![
                WadLumpObservation {
                    index: 0,
                    offset: 0,
                    size: playpal_bytes as u32,
                    name: "PLAYPAL".to_owned(),
                },
                WadLumpObservation {
                    index: 1,
                    offset: playpal_bytes as u32,
                    size: colormap_bytes as u32,
                    name: "COLORMAP".to_owned(),
                },
            ],
            namespaces: Vec::new(),
        }
    }

    fn patch_manifest(bytes: usize) -> WadManifest {
        WadManifest {
            source: WadSourceIdentity {
                label: "synthetic/patch.wad".to_owned(),
                byte_len: bytes,
                blake3: "synthetic".to_owned(),
            },
            kind: WadKind::Iwad,
            directory_offset: 0,
            directory_bytes: 0,
            total_lump_bytes: bytes as u64,
            lumps: vec![WadLumpObservation {
                index: 0,
                offset: 0,
                size: bytes as u32,
                name: "PATCH1".to_owned(),
            }],
            namespaces: vec![WadNamespaceObservation {
                kind: WadNamespaceKind::Patches,
                start_marker_index: 0,
                end_marker_index: 0,
                lump_indices: vec![0],
            }],
        }
    }

    fn patch_limits() -> DoomPatchDecodeLimits {
        DoomPatchDecodeLimits {
            max_patch_bytes: 1024,
            max_width: 16,
            max_height: 16,
            max_pixels: 256,
            max_posts: 16,
        }
    }

    fn texture_limits() -> DoomTextureDecodeLimits {
        DoomTextureDecodeLimits {
            max_pnames_bytes: 256,
            max_texture_bytes: 256,
            max_patch_names: 8,
            max_textures: 8,
            max_patches_per_texture: 4,
            max_total_patch_references: 8,
        }
    }

    fn texture_manifest(pnames_bytes: usize, texture_bytes: usize) -> WadManifest {
        WadManifest {
            source: WadSourceIdentity {
                label: "synthetic/textures.wad".to_owned(),
                byte_len: pnames_bytes + texture_bytes,
                blake3: "synthetic".to_owned(),
            },
            kind: WadKind::Iwad,
            directory_offset: 0,
            directory_bytes: 0,
            total_lump_bytes: (pnames_bytes + texture_bytes) as u64,
            lumps: vec![
                WadLumpObservation {
                    index: 0,
                    offset: 0,
                    size: pnames_bytes as u32,
                    name: "PNAMES".to_owned(),
                },
                WadLumpObservation {
                    index: 1,
                    offset: pnames_bytes as u32,
                    size: texture_bytes as u32,
                    name: "TEXTURE1".to_owned(),
                },
            ],
            namespaces: Vec::new(),
        }
    }

    fn synthetic_texture_bytes(patch_index: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(b"PATCH1\0\0");
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        bytes.extend_from_slice(b"TEX1\0\0\0\0");
        bytes.extend_from_slice(&0_i32.to_le_bytes());
        bytes.extend_from_slice(&64_u16.to_le_bytes());
        bytes.extend_from_slice(&32_u16.to_le_bytes());
        bytes.extend_from_slice(&0_i32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&(-2_i16).to_le_bytes());
        bytes.extend_from_slice(&3_i16.to_le_bytes());
        bytes.extend_from_slice(&patch_index.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes
    }

    fn composition_manifest(texture_bytes: usize, patch_bytes: usize) -> WadManifest {
        let mut manifest = texture_manifest(12, texture_bytes);
        manifest.source.byte_len += patch_bytes;
        manifest.total_lump_bytes += patch_bytes as u64;
        manifest.lumps.push(WadLumpObservation {
            index: 2,
            offset: (12 + texture_bytes) as u32,
            size: patch_bytes as u32,
            name: "PATCH1".to_owned(),
        });
        manifest.namespaces.push(WadNamespaceObservation {
            kind: WadNamespaceKind::Patches,
            start_marker_index: 2,
            end_marker_index: 2,
            lump_indices: vec![2],
        });
        manifest
    }

    fn flat_manifest(bytes: usize) -> WadManifest {
        let mut manifest = patch_manifest(bytes);
        manifest.lumps[0].name = "FLAT1".to_owned();
        manifest.namespaces[0].kind = WadNamespaceKind::Flats;
        manifest
    }

    fn sprite_manifest(names: &[&str]) -> WadManifest {
        WadManifest {
            source: WadSourceIdentity {
                label: "synthetic/sprites.wad".to_owned(),
                byte_len: 0,
                blake3: "synthetic".to_owned(),
            },
            kind: WadKind::Iwad,
            directory_offset: 0,
            directory_bytes: 0,
            total_lump_bytes: 0,
            lumps: names
                .iter()
                .enumerate()
                .map(|(index, name)| WadLumpObservation {
                    index: index as u32,
                    offset: 0,
                    size: 0,
                    name: (*name).to_owned(),
                })
                .collect(),
            namespaces: vec![WadNamespaceObservation {
                kind: WadNamespaceKind::Sprites,
                start_marker_index: 0,
                end_marker_index: names.len() as u32,
                lump_indices: (0..names.len() as u32).collect(),
            }],
        }
    }

    fn synthetic_patch_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&23_u32.to_le_bytes());
        bytes.extend_from_slice(&[0, 2, 0, 1, 2, 0, 255]);
        bytes.extend_from_slice(&[0, 2, 0, 3, 4, 0, 255]);
        bytes
    }

    #[test]
    fn decodes_palette_colours_and_index_remaps() {
        let mut bytes = vec![0_u8; PALETTE_BYTES + COLORMAP_BYTES];
        bytes[..3].copy_from_slice(&[1, 2, 3]);
        for (index, value) in bytes[PALETTE_BYTES..].iter_mut().enumerate() {
            *value = 255 - index as u8;
        }
        let globals =
            decode_doom_raster_globals(&bytes, &manifest(PALETTE_BYTES, COLORMAP_BYTES), limits())
                .expect("synthetic globals should decode");
        assert_eq!(globals.palettes.len(), 1);
        assert_eq!(
            globals.palettes[0].colors[0],
            DoomRgb {
                red: 1,
                green: 2,
                blue: 3
            }
        );
        assert_eq!(globals.colormaps[0].index_remap[0], 255);
        assert_eq!(globals.colormaps[0].index_remap[255], 0);
    }

    #[test]
    fn malformed_palette_and_colormap_lengths_are_rejected() {
        let palette_bytes = vec![0; PALETTE_BYTES - 1 + COLORMAP_BYTES];
        assert!(matches!(
            decode_doom_raster_globals(
                &palette_bytes,
                &manifest(PALETTE_BYTES - 1, COLORMAP_BYTES),
                limits()
            ),
            Err(DoomRasterDecodeError::InvalidPaletteLength { .. })
        ));

        let colormap_bytes = vec![0; PALETTE_BYTES + COLORMAP_BYTES - 1];
        assert!(matches!(
            decode_doom_raster_globals(
                &colormap_bytes,
                &manifest(PALETTE_BYTES, COLORMAP_BYTES - 1),
                limits()
            ),
            Err(DoomRasterDecodeError::InvalidColormapLength { .. })
        ));
    }

    #[test]
    fn decodes_column_posts_and_transparent_coverage() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&3_u16.to_le_bytes());
        bytes.extend_from_slice(&(-1_i16).to_le_bytes());
        bytes.extend_from_slice(&2_i16.to_le_bytes());
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&23_u32.to_le_bytes());
        bytes.extend_from_slice(&[0, 2, 0, 7, 8, 0, 255]);
        bytes.extend_from_slice(&[1, 1, 0, 9, 0, 255]);
        let patch = decode_doom_patch(
            &bytes,
            &patch_manifest(bytes.len()),
            "PATCH1",
            patch_limits(),
        )
        .expect("synthetic patch should decode");
        assert_eq!(patch.width, 2);
        assert_eq!(patch.opaque_pixels, 3);
        assert_eq!(patch.columns[0].posts[0].color_indices, [7, 8]);
        assert_eq!(patch.columns[1].posts[0].top_delta, 1);

        let mut truncated = bytes.clone();
        truncated.truncate(20);
        assert!(matches!(
            decode_doom_patch(
                &truncated,
                &patch_manifest(truncated.len()),
                "PATCH1",
                patch_limits()
            ),
            Err(DoomRasterDecodeError::PatchPostTruncated { .. })
        ));

        let mut sprite_manifest = patch_manifest(bytes.len());
        sprite_manifest.lumps[0].name = "SPRTA0".to_owned();
        sprite_manifest.namespaces[0].kind = WadNamespaceKind::Sprites;
        assert_eq!(
            decode_doom_sprite_patch(&bytes, &sprite_manifest, "SPRTA0", patch_limits())
                .expect("sprite patches share the patch encoding")
                .opaque_pixels,
            3,
        );
    }

    #[test]
    fn decodes_texture_patch_references_without_composing_pixels() {
        let bytes = synthetic_texture_bytes(0);
        let catalog = decode_doom_texture_catalog(
            &bytes,
            &texture_manifest(12, bytes.len() - 12),
            texture_limits(),
        )
        .expect("synthetic texture catalog should decode");
        assert_eq!(catalog.patch_names, ["PATCH1"]);
        assert_eq!(catalog.textures.len(), 1);
        let texture = &catalog.textures[0];
        assert_eq!(texture.name, "TEX1");
        assert_eq!((texture.width, texture.height), (64, 32));
        assert_eq!(texture.patches[0].patch_name, "PATCH1");
        assert_eq!(
            (texture.patches[0].origin_x, texture.patches[0].origin_y),
            (-2, 3)
        );

        let missing_patch = synthetic_texture_bytes(1);
        assert!(matches!(
            decode_doom_texture_catalog(
                &missing_patch,
                &texture_manifest(12, missing_patch.len() - 12),
                texture_limits(),
            ),
            Err(DoomRasterDecodeError::TexturePatchNameMissing { .. })
        ));
    }

    #[test]
    fn composes_texture_patches_in_source_order_with_coverage() {
        let mut bytes = synthetic_texture_bytes(0);
        bytes[42..44].copy_from_slice(&0_i16.to_le_bytes());
        bytes[44..46].copy_from_slice(&0_i16.to_le_bytes());
        let texture_bytes = bytes.len() - 12;
        let patch = synthetic_patch_bytes();
        bytes.extend_from_slice(&patch);
        let manifest = composition_manifest(texture_bytes, patch.len());
        let catalog = decode_doom_texture_catalog(&bytes, &manifest, texture_limits())
            .expect("synthetic texture catalog should decode");
        let image = compose_doom_texture(
            &bytes,
            &manifest,
            &catalog,
            "TEX1",
            patch_limits(),
            DoomTextureComposeLimits {
                max_width: 128,
                max_height: 128,
                max_pixels: 16_384,
            },
        )
        .expect("synthetic texture should compose");
        assert_eq!(image.opaque_pixels, 4);
        assert_eq!(&image.color_indices[..2], [1, 3]);
        assert_eq!(&image.color_indices[64..66], [2, 4]);
        assert!(image.coverage[..2].iter().all(|covered| *covered));
        assert!(!image.coverage[2]);
    }

    #[test]
    fn decodes_fixed_size_flat_indices() {
        let mut bytes = vec![0_u8; 64 * 64];
        bytes[0] = 7;
        bytes[4095] = 9;
        let flat = decode_doom_flat(
            &bytes,
            &flat_manifest(bytes.len()),
            "FLAT1",
            DoomFlatDecodeLimits {
                max_flat_bytes: 4096,
            },
        )
        .expect("synthetic flat should decode");
        assert_eq!(flat.color_indices[0], 7);
        assert_eq!(flat.color_indices[4095], 9);
        assert!(matches!(
            decode_doom_flat(
                &bytes[..4095],
                &flat_manifest(4095),
                "FLAT1",
                DoomFlatDecodeLimits {
                    max_flat_bytes: 4096,
                },
            ),
            Err(DoomRasterDecodeError::InvalidFlatLength { .. })
        ));
    }

    #[test]
    fn projects_single_and_paired_sprite_rotations() {
        let frames = decode_doom_sprite_frame_rotations(&sprite_manifest(&["TROOA2A8", "SARGM0"]))
            .expect("synthetic sprite names should decode");
        assert_eq!(frames.len(), 3);
        assert_eq!(
            (
                frames[0].sprite.as_str(),
                frames[0].frame,
                frames[0].rotation
            ),
            ("TROO", 'A', 2)
        );
        assert_eq!((frames[1].frame, frames[1].rotation), ('A', 8));
        assert_eq!(
            (
                frames[2].sprite.as_str(),
                frames[2].frame,
                frames[2].rotation
            ),
            ("SARG", 'M', 0)
        );
        assert!(matches!(
            decode_doom_sprite_frame_rotations(&sprite_manifest(&["TROOA9"])),
            Err(DoomRasterDecodeError::InvalidSpriteName { .. })
        ));
    }

    #[test]
    fn lowers_indexed_coverage_with_an_explicit_palette() {
        let palette = DoomPalette {
            source_lump_index: 0,
            palette_index: 0,
            colors: std::array::from_fn(|index| DoomRgb {
                red: index as u8,
                green: 2,
                blue: 3,
            }),
        };
        let lowered = lower_doom_indexed_image(
            &DoomIndexedImage {
                source_texture_lump_index: 0,
                texture_name: "TEST".to_owned(),
                width: 2,
                height: 1,
                color_indices: vec![7, 9],
                coverage: vec![true, false],
                opaque_pixels: 1,
            },
            &palette,
        )
        .expect("indexed image should lower");
        assert_eq!(lowered.pixels, [7, 2, 3, 255, 9, 2, 3, 0]);
        assert_eq!(lowered.alpha_mode, AlphaMode::Straight);
        assert_eq!(lowered.color_space, ColorSpace::Unspecified);
    }
}
