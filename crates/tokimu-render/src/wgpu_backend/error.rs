use thiserror::Error;

use crate::{PipelineValidationError, TextureValidationError};

#[derive(Debug, Error)]
pub enum WgpuBackendError {
    #[error("failed to request a compatible GPU adapter")]
    AdapterRequest,
    #[error("failed to request a GPU device: {0}")]
    DeviceRequest(String),
    #[error("failed to create a render surface: {0}")]
    SurfaceCreation(String),
    #[error("surface did not report any supported texture formats")]
    SurfaceFormatUnavailable,
    #[error("failed to acquire the current surface texture: {0}")]
    SurfaceAcquire(String),
    #[error("mesh handle {0} has not been uploaded")]
    MissingMesh(u64),
    #[error("material handle {0} has not been uploaded")]
    MissingMaterial(u64),
    #[error("material color must contain only finite values")]
    InvalidMaterialColor,
    #[error("pipeline handle {0} has not been uploaded")]
    MissingPipeline(u64),
    #[error("renderable handle {0} has not been uploaded")]
    MissingRenderable(u64),
    #[error("texture handle {0} has not been uploaded")]
    MissingTexture(u64),
    #[error("texture handle {0} already exists")]
    TextureAlreadyExists(u64),
    #[error("texture handle {0} is a renderer-owned render target and cannot receive source-pixel updates")]
    TextureIsRenderTarget(u64),
    #[error("texture handle {0} is not a renderer-owned render target")]
    TextureIsNotRenderTarget(u64),
    #[error(
        "render target {target} cannot be released while {material_count} material binding(s) still sample it"
    )]
    RenderTargetStillReferenced { target: u64, material_count: u32 },
    #[error("render target {target} has format {target_format}, which does not match the active surface format {surface_format}")]
    RenderTargetFormatMismatch {
        target: u64,
        target_format: String,
        surface_format: String,
    },
    #[error(
        "render target {target} cannot be sampled by material {material} while it is being written"
    )]
    RenderTargetSelfSampling { target: u64, material: u64 },
    #[error(
        "texture handle {handle} has dimensions {expected_width}x{expected_height}, not {actual_width}x{actual_height}"
    )]
    TextureDimensionsMismatch {
        handle: u64,
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },
    #[error("invalid RGBA8 texture declaration: {0}")]
    InvalidTexture(#[from] TextureValidationError),
    #[error("invalid pipeline declaration: {0}")]
    InvalidPipeline(#[from] PipelineValidationError),
}
