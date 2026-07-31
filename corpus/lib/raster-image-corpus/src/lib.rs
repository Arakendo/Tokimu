//! Bounded raster-image decoding evidence for Tokimu's corpus.
//!
//! This crate is intentionally corpus-owned. It does not define a production
//! image provider or renderer texture contract.

mod asset;
mod bmp;
mod jpeg;
mod model;
mod png;
mod runner;

pub use asset::{
    prepare_renderer_texture, register_decoded_image_asset, resolve_bmp_image_asset,
    resolve_jpeg_image_asset, resolve_optional_png_image_asset, resolve_png_image_asset,
    RasterAssetResolutionError, RasterTexturePreparationError, ResolvedImageAsset,
    TextureUploadArtifact, TextureUploadEvidence, TextureUse, TextureUseArtifact,
};
pub use bmp::decode_bmp;
pub use jpeg::{
    decode_jpeg, inspect_jpeg, JfifMetadata, JpegColorModel, JpegIccMetadata, JpegInspection,
};
pub use model::{
    AlphaMode, ColorSpace, DecodeLimits, DecodedImage, DecodedImageArtifact, ImageOrientation,
    PixelFormat, RasterImageError,
};
pub use png::{decode_png, inspect_png, PngIccMetadata, PngInspection};
pub use runner::{
    run_selected_cases, run_selected_cases_with_filter, write_selected_artifacts,
    write_selected_artifacts_with_filter, RasterCaseArtifact, RasterCaseFilter,
};
