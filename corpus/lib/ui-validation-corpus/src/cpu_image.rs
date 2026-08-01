use screenshot::{write_bmp, write_manifest, Rgba8Image};
use std::path::Path;
use ui_tools::{layout_bitmap_text, UiDrawCommand, UiDrawList, UiRect, UiSurfaceRole, UiTextRole};

pub const ALGORITHM: &str = "ui-cpu-diagnostic-raster-v1";

pub struct CpuUiImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy)]
enum CpuClip {
    Unbounded,
    Bounds(UiRect),
    Empty,
}

impl CpuUiImage {
    pub fn fingerprint(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in &self.pixels {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    pub fn write_artifacts(&self, root: &Path, draw_list: &UiDrawList) -> Result<(), String> {
        let width = self.width.to_string();
        let height = self.height.to_string();
        let fingerprint = format!("{:016x}", self.fingerprint());
        let draw_fingerprint = format!("{:016x}", draw_list.structural_fingerprint());
        write_bmp(
            root.join("cpu-image.bmp"),
            Rgba8Image {
                width: self.width,
                height: self.height,
                pixels: &self.pixels,
            },
        )?;
        write_manifest(
            root.join("cpu-image-manifest.txt"),
            &[
                ("schema", "tokimu-ui-cpu-image-v1"),
                ("algorithm", ALGORITHM),
                ("format", "rgba8-exported-as-bgra32-bmp"),
                ("width", &width),
                ("height", &height),
                ("pixel_fingerprint_algorithm", "fnv1a64"),
                ("pixel_fingerprint", &fingerprint),
                ("draw_list_fingerprint", &draw_fingerprint),
                ("text_provider", "ui-tools-builtin-bitmap"),
                ("source_stage", "renderer-neutral-draw-list"),
                ("gpu_framebuffer_equivalent", "false"),
            ],
        )
    }
}

pub fn rasterize(draw_list: &UiDrawList, viewport: UiRect, width: u32, height: u32) -> CpuUiImage {
    let pixel_bytes = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .expect("validated corpus image dimensions must fit in u32");
    let mut image = CpuUiImage {
        width,
        height,
        pixels: vec![0; pixel_bytes as usize],
    };
    image.clear([7, 12, 16, 255]);
    let mut clips = Vec::new();

    for entry in draw_list.entries() {
        match &entry.command {
            UiDrawCommand::PushClip(clip) => clips.push(*clip),
            UiDrawCommand::PopClip => {
                clips.pop();
            }
            UiDrawCommand::Surface(command) => {
                let clip = combined_clip(&clips, command.clip);
                let fill = surface_color(command.style.role, command.style.opacity);
                image.fill_rect(viewport, command.rect, clip, fill);
                if let Some(role) = command.style.border_role {
                    let border_width = command
                        .style
                        .border_width
                        .max(viewport.size[0] / width as f32);
                    image.stroke_rect(
                        viewport,
                        command.rect,
                        clip,
                        border_width,
                        surface_color(role, command.style.opacity),
                    );
                }
            }
            UiDrawCommand::Text(command) => {
                let color = text_color(command.style.role, command.style.opacity);
                for quad in layout_bitmap_text(&command.spec, command.style.height) {
                    image.fill_rect(
                        viewport,
                        UiRect::new(quad.center, quad.size),
                        combined_clip(&clips, Some(command.spec.rect)),
                        color,
                    );
                }
            }
        }
    }
    image
}

fn combined_clip(clips: &[UiRect], local: Option<UiRect>) -> CpuClip {
    let mut candidates = clips.iter().copied().chain(local);
    let Some(mut bounds) = candidates.next() else {
        return CpuClip::Unbounded;
    };
    for candidate in candidates {
        let Some(intersection) = bounds.intersection(candidate) else {
            return CpuClip::Empty;
        };
        bounds = intersection;
    }
    CpuClip::Bounds(bounds)
}

impl CpuUiImage {
    fn clear(&mut self, color: [u8; 4]) {
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }

    fn fill_rect(&mut self, viewport: UiRect, rect: UiRect, clip: CpuClip, color: [u8; 4]) {
        let rect = match clip {
            CpuClip::Unbounded => rect,
            CpuClip::Bounds(clip) => {
                let Some(rect) = rect.intersection(clip) else {
                    return;
                };
                rect
            }
            CpuClip::Empty => return,
        };
        let [left, top, right, bottom] = pixel_bounds(viewport, rect, self.width, self.height);
        for y in top..bottom {
            for x in left..right {
                self.blend_pixel(x, y, color);
            }
        }
    }

    fn stroke_rect(
        &mut self,
        viewport: UiRect,
        rect: UiRect,
        clip: CpuClip,
        width: f32,
        color: [u8; 4],
    ) {
        let horizontal = [rect.size[0] + width, width];
        let vertical = [width, (rect.size[1] - width).max(0.0)];
        self.fill_rect(
            viewport,
            UiRect::new(
                [rect.center[0], rect.center[1] + rect.size[1] * 0.5],
                horizontal,
            ),
            clip,
            color,
        );
        self.fill_rect(
            viewport,
            UiRect::new(
                [rect.center[0], rect.center[1] - rect.size[1] * 0.5],
                horizontal,
            ),
            clip,
            color,
        );
        self.fill_rect(
            viewport,
            UiRect::new(
                [rect.center[0] - rect.size[0] * 0.5, rect.center[1]],
                vertical,
            ),
            clip,
            color,
        );
        self.fill_rect(
            viewport,
            UiRect::new(
                [rect.center[0] + rect.size[0] * 0.5, rect.center[1]],
                vertical,
            ),
            clip,
            color,
        );
    }

    fn blend_pixel(&mut self, x: u32, y: u32, source: [u8; 4]) {
        let index = (y as usize * self.width as usize + x as usize) * 4;
        let alpha = u32::from(source[3]);
        let inverse = 255 - alpha;
        for (channel, source_value) in source.iter().copied().take(3).enumerate() {
            self.pixels[index + channel] = ((u32::from(source_value) * alpha
                + u32::from(self.pixels[index + channel]) * inverse
                + 127)
                / 255) as u8;
        }
        self.pixels[index + 3] = 255;
    }
}

fn pixel_bounds(viewport: UiRect, rect: UiRect, width: u32, height: u32) -> [u32; 4] {
    let viewport_left = viewport.center[0] - viewport.size[0] * 0.5;
    let viewport_top = viewport.center[1] + viewport.size[1] * 0.5;
    let rect_left = rect.center[0] - rect.size[0] * 0.5;
    let rect_right = rect.center[0] + rect.size[0] * 0.5;
    let rect_top = rect.center[1] + rect.size[1] * 0.5;
    let rect_bottom = rect.center[1] - rect.size[1] * 0.5;
    let x = |value: f32| (value - viewport_left) / viewport.size[0] * width as f32;
    let y = |value: f32| (viewport_top - value) / viewport.size[1] * height as f32;
    [
        x(rect_left).floor().clamp(0.0, width as f32) as u32,
        y(rect_top).floor().clamp(0.0, height as f32) as u32,
        x(rect_right).ceil().clamp(0.0, width as f32) as u32,
        y(rect_bottom).ceil().clamp(0.0, height as f32) as u32,
    ]
}

fn surface_color(role: UiSurfaceRole, opacity: f32) -> [u8; 4] {
    let rgb = match role {
        UiSurfaceRole::Background => [9, 15, 19],
        UiSurfaceRole::Region => [24, 34, 42],
        UiSurfaceRole::Panel => [55, 66, 80],
        UiSurfaceRole::Card => [75, 87, 104],
        UiSurfaceRole::Toolbar => [63, 76, 92],
        UiSurfaceRole::Raised => [88, 101, 119],
        UiSurfaceRole::Selected => [145, 190, 230],
        UiSurfaceRole::Accent => [106, 226, 211],
        UiSurfaceRole::Overlay => [35, 44, 54],
    };
    [rgb[0], rgb[1], rgb[2], alpha(opacity)]
}

fn text_color(role: UiTextRole, opacity: f32) -> [u8; 4] {
    let rgb = match role {
        UiTextRole::Title => [244, 248, 249],
        UiTextRole::Heading => [222, 232, 236],
        UiTextRole::Body => [204, 218, 224],
        UiTextRole::Caption => [157, 181, 192],
        UiTextRole::Button => [239, 246, 248],
        UiTextRole::Chip => [113, 229, 215],
        UiTextRole::Status => [152, 199, 230],
    };
    [rgb[0], rgb[1], rgb[2], alpha(opacity)]
}

fn alpha(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_tools::{
        lowering::lower_resolved_tree_to_draw_list, UiNodeId, UiNodeLayout, UiNodeSpec, UiTheme,
        UiTree,
    };

    #[test]
    fn raster_is_repeatable_for_identical_draw_lists() {
        let viewport = UiRect::new([0.0, 0.0], [3.2, 2.0]);
        let resolved = UiTree::new(
            UiNodeSpec::text(
                UiNodeId(1),
                &ui_tools::UiTextSpec::new("TEST", viewport, UiTextRole::Heading),
            )
            .with_layout(UiNodeLayout::Fill),
        )
        .resolve(viewport)
        .unwrap();
        let draw_list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 1);

        let first = rasterize(&draw_list, viewport, 320, 200);
        let second = rasterize(&draw_list, viewport, 320, 200);

        assert_eq!(first.fingerprint(), second.fingerprint());
        assert_eq!(first.pixels, second.pixels);
        assert!(first
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel != [7, 12, 16, 255]));
    }
}
