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
