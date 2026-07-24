//! Shared corpus-review artifact helpers.
//!
//! This crate writes deterministic CPU image buffers and review metadata. It
//! intentionally does not capture GPU surfaces or depend on a renderer.

use std::{fs, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba8Image<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}

impl<'a> Rgba8Image<'a> {
    pub fn validate(self) -> Result<(), String> {
        let expected = self
            .width
            .checked_mul(self.height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| "image dimensions overflow".to_owned())?;
        if self.pixels.len() != expected as usize {
            return Err(format!(
                "expected {expected} RGBA bytes, got {}",
                self.pixels.len()
            ));
        }
        Ok(())
    }
}

/// Writes a deterministic 32-bit BGRA BMP artifact from an RGBA8 source.
pub fn write_bmp(path: impl AsRef<Path>, image: Rgba8Image<'_>) -> Result<(), String> {
    image.validate()?;
    let row_bytes = image.width * 4;
    let pixel_bytes = row_bytes * image.height;
    let file_size = 54 + pixel_bytes;
    let mut bytes = Vec::with_capacity(file_size as usize);
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&file_size.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&(54_u32).to_le_bytes());
    bytes.extend_from_slice(&(40_u32).to_le_bytes());
    bytes.extend_from_slice(&(image.width as i32).to_le_bytes());
    bytes.extend_from_slice(&(-(image.height as i32)).to_le_bytes());
    bytes.extend_from_slice(&(1_u16).to_le_bytes());
    bytes.extend_from_slice(&(32_u16).to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&pixel_bytes.to_le_bytes());
    bytes.extend_from_slice(&[0; 16]);
    for pixel in image.pixels.chunks_exact(4) {
        bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    fs::write(path, bytes).map_err(|error| error.to_string())
}

/// Writes plain-text metadata beside a captured corpus artifact.
pub fn write_manifest(path: impl AsRef<Path>, entries: &[(&str, &str)]) -> Result<(), String> {
    let mut text = String::new();
    for (key, value) in entries {
        text.push_str(key);
        text.push('=');
        text.push_str(value);
        text.push('\n');
    }
    fs::write(path, text).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn validates_rgba_dimensions() {
        let image = Rgba8Image {
            width: 2,
            height: 1,
            pixels: &[0; 8],
        };
        assert_eq!(image.validate(), Ok(()));
    }

    #[test]
    fn rejects_incomplete_rgba_buffers() {
        let image = Rgba8Image {
            width: 2,
            height: 1,
            pixels: &[0; 4],
        };
        assert!(image.validate().is_err());
    }

    #[test]
    fn writes_top_down_bgra_pixels() {
        let path = std::env::temp_dir().join("tokimu-screenshot-test.bmp");
        let pixels = [10, 20, 30, 40];
        write_bmp(
            &path,
            Rgba8Image {
                width: 1,
                height: 1,
                pixels: &pixels,
            },
        )
        .expect("BMP should be written");
        let bytes = fs::read(&path).expect("BMP should be readable");
        assert_eq!(&bytes[0..2], b"BM");
        assert_eq!(&bytes[18..22], &1_i32.to_le_bytes());
        assert_eq!(&bytes[22..26], &(-1_i32).to_le_bytes());
        assert_eq!(&bytes[54..58], &[30, 20, 10, 40]);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn writes_ordered_manifest_entries() {
        let path = std::env::temp_dir().join("tokimu-screenshot-test.txt");
        write_manifest(&path, &[("example", "corpus"), ("gpu_readback", "false")])
            .expect("manifest should be written");
        let text = fs::read_to_string(&path).expect("manifest should be readable");
        assert_eq!(text, "example=corpus\ngpu_readback=false\n");
        let _ = fs::remove_file(path);
    }
}
