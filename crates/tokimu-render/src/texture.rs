use thiserror::Error;

/// Interprets already-decoded RGBA8 samples when the renderer allocates a texture.
///
/// This describes sampled data interpretation only. It does not select display,
/// browser, HDR, file-decoding, or output-transfer policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Rgba8TextureColorSpace {
    /// Samples are linear UNORM data rather than display-referred color.
    Linear,
    /// Samples are sRGB-encoded color values.
    #[default]
    Srgb,
}

/// Immutable allocation contract for one complete RGBA8 texture.
///
/// The descriptor does not contain encoded-image metadata, sampler policy, or
/// backend objects. A texture's dimensions and color interpretation remain
/// fixed for its lifetime; callers that need a different shape must create a
/// separate texture rather than update an existing identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgba8TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub color_space: Rgba8TextureColorSpace,
}

impl Rgba8TextureDescriptor {
    /// Describes a non-zero-width, non-zero-height RGBA8 texture allocation.
    pub const fn new(width: u32, height: u32, color_space: Rgba8TextureColorSpace) -> Self {
        Self {
            width,
            height,
            color_space,
        }
    }

    /// Returns the exact byte length required for one complete RGBA8 payload.
    pub fn expected_payload_len(self) -> Result<usize, TextureValidationError> {
        if self.width == 0 || self.height == 0 {
            return Err(TextureValidationError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        }

        let width = usize::try_from(self.width).map_err(|_| {
            TextureValidationError::PayloadSizeOverflow {
                width: self.width,
                height: self.height,
            }
        })?;
        let height = usize::try_from(self.height).map_err(|_| {
            TextureValidationError::PayloadSizeOverflow {
                width: self.width,
                height: self.height,
            }
        })?;
        width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(TextureValidationError::PayloadSizeOverflow {
                width: self.width,
                height: self.height,
            })
    }

    /// Rejects incomplete, oversized, or invalid-dimension RGBA8 payloads.
    pub fn validate_payload(self, rgba8: &[u8]) -> Result<(), TextureValidationError> {
        let expected = self.expected_payload_len()?;
        if rgba8.len() != expected {
            return Err(TextureValidationError::PayloadLengthMismatch {
                expected,
                actual: rgba8.len(),
            });
        }
        Ok(())
    }
}

/// Validation failures that occur before a renderer allocates or writes a texture.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum TextureValidationError {
    #[error("RGBA8 texture dimensions must be non-zero, received {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("RGBA8 payload size overflow for texture dimensions {width}x{height}")]
    PayloadSizeOverflow { width: u32, height: u32 },
    #[error("RGBA8 payload length mismatch: expected {expected} bytes, received {actual}")]
    PayloadLengthMismatch { expected: usize, actual: usize },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

impl Texture {
    pub fn rgba8(width: u32, height: u32, rgba8: Vec<u8>) -> Self {
        assert_eq!(rgba8.len(), (width * height * 4) as usize);
        Self {
            width,
            height,
            rgba8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_computes_checked_rgba8_payload_size() {
        let descriptor = Rgba8TextureDescriptor::new(1920, 1080, Rgba8TextureColorSpace::Srgb);

        assert_eq!(descriptor.expected_payload_len(), Ok(8_294_400));
        assert_eq!(descriptor.validate_payload(&vec![0; 8_294_400]), Ok(()));
    }

    #[test]
    fn descriptor_rejects_invalid_dimensions_and_payloads() {
        let empty = Rgba8TextureDescriptor::new(0, 1, Rgba8TextureColorSpace::Linear);
        assert_eq!(
            empty.expected_payload_len(),
            Err(TextureValidationError::InvalidDimensions {
                width: 0,
                height: 1,
            })
        );

        let descriptor = Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Linear);
        assert_eq!(
            descriptor.validate_payload(&[0; 15]),
            Err(TextureValidationError::PayloadLengthMismatch {
                expected: 16,
                actual: 15,
            })
        );
    }
}
