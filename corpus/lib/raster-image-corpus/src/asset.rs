use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokimu_assets::{AssetHandle, AssetLifecycleObservation, AssetStore, AssetStoreError};
use tokimu_render::Texture;

use crate::{
    decode_bmp, decode_jpeg, decode_png, AlphaMode, ColorSpace, DecodeLimits, DecodedImage,
    ImageOrientation, RasterImageError,
};

/// Corpus-owned evidence that encoded image bytes were resolved through a
/// stable Tokimu asset identity.
///
/// The source label exists only in `AssetStore` lifecycle diagnostics. Renderer
/// and presentation consumers receive this opaque handle and the normalized
/// decoded image; no PNG-specific object or source path crosses the boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedImageAsset {
    pub handle: AssetHandle<DecodedImage>,
    pub image: DecodedImage,
    pub allocation: AssetLifecycleObservation,
    pub prepared: AssetLifecycleObservation,
}

#[derive(Debug, Error)]
pub enum RasterAssetResolutionError {
    #[error("image asset bytes are unavailable for `{source_label}`")]
    MissingSourceBytes { source_label: String },
    #[error("raster image decoding failed before asset resolution: {0}")]
    Decode(#[from] RasterImageError),
    #[error("asset lifecycle transition failed after image decoding: {0}")]
    Asset(#[from] AssetStoreError),
}

/// The application-level meaning supplied when normalized pixels cross into
/// the current renderer texture contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextureUse {
    /// Color pixels will use the renderer's present `Rgba8UnormSrgb` path.
    ColorSrgb,
    /// The current renderer contract has no linear/data texture upload path.
    LinearData,
}

/// Corpus evidence for the conversion into the renderer's narrow RGBA8+sRGB
/// texture input. This records assumptions; it does not perform color-space or
/// alpha conversion and does not allocate a GPU texture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextureUploadEvidence {
    pub texture: Texture,
    pub texture_use: TextureUse,
    pub source_color_space: ColorSpace,
    pub source_alpha_mode: AlphaMode,
    pub source_orientation: ImageOrientation,
    pub target_gpu_format: &'static str,
}

/// Deterministic evidence for the boundary immediately before a renderer-owned
/// texture upload. This is not a GPU upload record or framebuffer capture.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextureUploadArtifact {
    pub schema: u32,
    pub artifact_kind: &'static str,
    pub source_stage: &'static str,
    pub gpu_upload_performed: bool,
    pub width: u32,
    pub height: u32,
    pub decoded_bytes: usize,
    pub texture_use: TextureUseArtifact,
    pub source_color_space: ColorSpace,
    pub source_alpha_mode: AlphaMode,
    pub source_orientation: ImageOrientation,
    pub target_gpu_format: &'static str,
    pub fingerprint_algorithm: &'static str,
    pub pixel_fingerprint: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextureUseArtifact {
    ColorSrgb,
}

#[derive(Debug, Error)]
pub enum RasterTexturePreparationError {
    #[error("decoded image is not valid for texture preparation: {0}")]
    InvalidDecodedImage(#[from] RasterImageError),
    #[error("renderer texture upload currently accepts top-down RGBA8 only")]
    UnsupportedOrientation,
    #[error("renderer texture upload does not yet support linear/data texture intent")]
    UnsupportedTextureUse,
}

impl TextureUploadEvidence {
    /// Produces pre-GPU evidence that can be compared independently of backend
    /// texture allocation, sampler policy, or framebuffer presentation.
    pub fn artifact(&self) -> TextureUploadArtifact {
        TextureUploadArtifact {
            schema: 1,
            artifact_kind: "texture-upload-preparation",
            source_stage: "decoded-image",
            gpu_upload_performed: false,
            width: self.texture.width,
            height: self.texture.height,
            decoded_bytes: self.texture.rgba8.len(),
            texture_use: TextureUseArtifact::ColorSrgb,
            source_color_space: self.source_color_space,
            source_alpha_mode: self.source_alpha_mode,
            source_orientation: self.source_orientation,
            target_gpu_format: self.target_gpu_format,
            fingerprint_algorithm: "fnv1a64",
            pixel_fingerprint: rgba8_fingerprint(&self.texture.rgba8),
        }
    }

    pub fn artifact_json(&self) -> Result<String, RasterImageError> {
        serde_json::to_string_pretty(&self.artifact())
            .map(|json| format!("{json}\n"))
            .map_err(|error| RasterImageError::ArtifactSerialization(error.to_string()))
    }
}

/// Registers normalized pixels behind an opaque Tokimu asset handle.
///
/// This is the provider-neutral asset boundary. Format-specific adapters must
/// decode before calling it, so callers after this point cannot depend on PNG,
/// JPEG, BMP, or provider-native decoder state.
pub fn register_decoded_image_asset(
    assets: &mut AssetStore,
    source_label: impl Into<String>,
    image: DecodedImage,
) -> Result<ResolvedImageAsset, RasterAssetResolutionError> {
    image.validate()?;
    let (handle, allocation) =
        assets.allocate_with_source_observed::<DecodedImage, _>(source_label);
    let prepared = assets.mark_prepared(handle)?;

    Ok(ResolvedImageAsset {
        handle,
        image,
        allocation,
        prepared,
    })
}

/// Decodes PNG bytes and registers the normalized result behind an opaque
/// Tokimu asset handle. This is integration evidence, not a production loader.
pub fn resolve_png_image_asset(
    assets: &mut AssetStore,
    source_label: impl Into<String>,
    source: &[u8],
    limits: DecodeLimits,
) -> Result<ResolvedImageAsset, RasterAssetResolutionError> {
    resolve_optional_png_image_asset(assets, source_label, Some(source), limits)
}

/// Resolves an optional PNG dependency while keeping absence distinct from an
/// invalid encoded PNG. The caller owns obtaining bytes from disk, a package,
/// or a browser boundary; this helper only records the asset-resolution result.
pub fn resolve_optional_png_image_asset(
    assets: &mut AssetStore,
    source_label: impl Into<String>,
    source: Option<&[u8]>,
    limits: DecodeLimits,
) -> Result<ResolvedImageAsset, RasterAssetResolutionError> {
    let source_label = source_label.into();
    let source = source.ok_or_else(|| RasterAssetResolutionError::MissingSourceBytes {
        source_label: source_label.clone(),
    })?;
    register_decoded_image_asset(assets, source_label, decode_png(source, limits)?)
}

/// Decodes baseline JPEG bytes and registers the normalized result behind an
/// opaque Tokimu asset handle.
pub fn resolve_jpeg_image_asset(
    assets: &mut AssetStore,
    source_label: impl Into<String>,
    source: &[u8],
    limits: DecodeLimits,
) -> Result<ResolvedImageAsset, RasterAssetResolutionError> {
    register_decoded_image_asset(assets, source_label, decode_jpeg(source, limits)?)
}

/// Decodes bounded BMP bytes and registers the normalized result behind an
/// opaque Tokimu asset handle.
pub fn resolve_bmp_image_asset(
    assets: &mut AssetStore,
    source_label: impl Into<String>,
    source: &[u8],
    limits: DecodeLimits,
) -> Result<ResolvedImageAsset, RasterAssetResolutionError> {
    register_decoded_image_asset(assets, source_label, decode_bmp(source, limits)?)
}

/// Prepares normalized decoded pixels for the current renderer texture input.
/// The caller, not the encoded image format, declares whether the pixels are
/// intended as sRGB color. The renderer receives only dimensions and RGBA8
/// pixels through `Texture`.
pub fn prepare_renderer_texture(
    image: &DecodedImage,
    texture_use: TextureUse,
) -> Result<TextureUploadEvidence, RasterTexturePreparationError> {
    image.validate()?;
    if image.output_orientation != ImageOrientation::TopDown {
        return Err(RasterTexturePreparationError::UnsupportedOrientation);
    }
    if texture_use != TextureUse::ColorSrgb {
        return Err(RasterTexturePreparationError::UnsupportedTextureUse);
    }

    Ok(TextureUploadEvidence {
        texture: Texture::rgba8(image.width, image.height, image.pixels.clone()),
        texture_use,
        source_color_space: image.color_space,
        source_alpha_mode: image.alpha_mode,
        source_orientation: image.output_orientation,
        target_gpu_format: "Rgba8UnormSrgb",
    })
}

fn rgba8_fingerprint(pixels: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in pixels {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
