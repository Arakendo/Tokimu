mod bitmap;

use bitmap::{bitmap_cell, bitmap_text_fit, physical_alignment, text_lines};
pub use bitmap::{bitmap_glyph_height, layout_bitmap_text, measure_bitmap_text_width};

use crate::{UiLabelAnchor, UiRect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextRole {
    Title,
    Heading,
    Body,
    Caption,
    Button,
    Chip,
    Status,
}

impl UiTextRole {
    pub const ALL: [Self; 7] = [
        Self::Title,
        Self::Heading,
        Self::Body,
        Self::Caption,
        Self::Button,
        Self::Chip,
        Self::Status,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextAlignmentBasis {
    /// Align using the logical advance box, including trailing spacing.
    Advance,
    /// Align using the visible bitmap ink, excluding trailing spacing.
    VisibleInk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextOverflow {
    Clip,
    Ellipsis,
    Wrap,
    /// Emit no presentation when the complete request does not fit. The
    /// pre-policy fit result remains available to diagnostics and callers.
    Defer,
    /// Preserve the complete request by reducing its presentation scale until
    /// it fits the declared rectangle.
    ScaleDown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiTextDiagnosticKind {
    MissingGlyph { character: char },
    ProviderUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiTextDiagnostic {
    pub kind: UiTextDiagnosticKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMissingGlyphPolicy {
    Replace(char),
    Skip,
    Report,
}

/// Provider-neutral measurements for one laid-out text request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiTextMeasure {
    pub advance: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub visible_bounds: Option<UiRect>,
    pub diagnostics: Vec<UiTextDiagnostic>,
}

impl UiTextMeasure {
    /// Reports whether this provider-neutral measurement fits `bounds`.
    ///
    /// Providers measure the complete request they receive. Wrapping and
    /// ellipsis remain semantic policies owned by `UiTextSpec`; this helper
    /// only exposes the pre-policy overflow that a resolved UI node must make
    /// visible to its consumer.
    pub fn fit_in(&self, bounds: UiRect) -> UiTextFit {
        let line_height = self.ascent.max(0.0) + self.descent.abs() + self.line_gap.max(0.0);
        UiTextFit {
            horizontal_overflow: bounds.size[0] > 0.0 && self.advance > bounds.size[0],
            vertical_overflow: bounds.size[1] > 0.0 && line_height > bounds.size[1],
        }
    }
}

/// A provider can expose metrics without loading a rasterizer or renderer.
pub trait UiTextMetricsProvider {
    fn measure(&self, text: &str) -> Result<UiTextMeasure, UiTextDiagnostic>;
}

/// Fixed-size metrics adapter for Tokimu's built-in bitmap text provider.
///
/// The adapter exposes the same provider-neutral contract as external font
/// providers without making semantic UI select a concrete font technology.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiBitmapTextMetricsProvider {
    height: f32,
}

impl UiBitmapTextMetricsProvider {
    pub fn new(height: f32) -> Self {
        Self { height }
    }

    pub fn height(self) -> f32 {
        self.height
    }
}

impl UiTextMetricsProvider for UiBitmapTextMetricsProvider {
    fn measure(&self, text: &str) -> Result<UiTextMeasure, UiTextDiagnostic> {
        if !self.height.is_finite() || self.height <= 0.0 {
            return Err(UiTextDiagnostic {
                kind: UiTextDiagnosticKind::ProviderUnavailable,
            });
        }

        let lines = text.lines().collect::<Vec<_>>();
        let line_count = lines.len().max(1);
        let advance = lines
            .iter()
            .map(|line| measure_bitmap_text_width(line, self.height))
            .fold(0.0, f32::max);
        let glyph_height = bitmap_glyph_height(self.height);
        let line_height = bitmap_cell(self.height) * 9.0;
        let block_height = glyph_height + (line_count.saturating_sub(1) as f32 * line_height);
        let visible_bounds = (advance > 0.0 && block_height > 0.0)
            .then(|| UiRect::new([advance * 0.5, block_height * 0.5], [advance, block_height]));

        Ok(UiTextMeasure {
            advance,
            ascent: block_height,
            descent: 0.0,
            line_gap: 0.0,
            visible_bounds,
            diagnostics: Vec::new(),
        })
    }
}

/// The placement contract for one logical line.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTextLineLayout {
    pub text: String,
    pub origin: [f32; 2],
    pub advance: f32,
    pub baseline: f32,
}

/// Provider-neutral result consumed by native, headless, or diagnostic clients.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTextLayout {
    pub measure: UiTextMeasure,
    pub lines: Vec<UiTextLineLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTextSpec {
    pub text: String,
    pub rect: UiRect,
    pub role: UiTextRole,
    pub direction: UiTextDirection,
    pub align_x: UiTextAlign,
    pub align_y: UiTextAlign,
    pub alignment_basis: UiTextAlignmentBasis,
    pub overflow: UiTextOverflow,
    pub missing_glyph: UiMissingGlyphPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGlyphQuad {
    pub center: [f32; 2],
    pub size: [f32; 2],
}

/// Whether the requested text fits within its declared layout region.
///
/// This reports the semantic request before clipping or ellipsis hides any
/// excess, so corpus consumers can make unfit text visible in diagnostics.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiTextFit {
    pub horizontal_overflow: bool,
    pub vertical_overflow: bool,
}

impl UiTextFit {
    pub fn fits(self) -> bool {
        !self.horizontal_overflow && !self.vertical_overflow
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTextLayoutReport {
    pub text: String,
    pub line_count: usize,
    pub glyph_count: usize,
    pub visible_bounds: Option<UiRect>,
    pub fit: UiTextFit,
}

impl UiTextLayout {
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

impl UiTextSpec {
    pub fn new(text: impl Into<String>, rect: UiRect, role: UiTextRole) -> Self {
        Self {
            text: text.into(),
            rect,
            role,
            direction: UiTextDirection::Ltr,
            align_x: UiTextAlign::Center,
            align_y: UiTextAlign::Center,
            alignment_basis: UiTextAlignmentBasis::VisibleInk,
            overflow: UiTextOverflow::Clip,
            missing_glyph: UiMissingGlyphPolicy::Replace('?'),
        }
    }

    pub fn with_direction(mut self, direction: UiTextDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_alignment(mut self, align_x: UiTextAlign, align_y: UiTextAlign) -> Self {
        self.align_x = align_x;
        self.align_y = align_y;
        self
    }

    pub fn with_alignment_basis(mut self, basis: UiTextAlignmentBasis) -> Self {
        self.alignment_basis = basis;
        self
    }

    pub fn with_overflow(mut self, overflow: UiTextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn with_missing_glyph_policy(mut self, policy: UiMissingGlyphPolicy) -> Self {
        self.missing_glyph = policy;
        self
    }

    pub fn centered_bounds(&self) -> [f32; 2] {
        self.rect.center
    }

    pub fn headless_report(&self, height: f32) -> UiTextLayoutReport {
        let quads = layout_bitmap_text(self, height);
        let fit = bitmap_text_fit(self, height);
        let visible_bounds = quads.first().map(|_| {
            let (min_x, max_x, min_y, max_y) = quads.iter().fold(
                (
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                ),
                |(min_x, max_x, min_y, max_y), quad| {
                    let half_width = quad.size[0] * 0.5;
                    let half_height = quad.size[1] * 0.5;
                    (
                        min_x.min(quad.center[0] - half_width),
                        max_x.max(quad.center[0] + half_width),
                        min_y.min(quad.center[1] - half_height),
                        max_y.max(quad.center[1] + half_height),
                    )
                },
            );
            UiRect::new(
                [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5],
                [max_x - min_x, max_y - min_y],
            )
        });

        UiTextLayoutReport {
            text: self.text.clone(),
            line_count: self.text.lines().count(),
            glyph_count: quads.len(),
            visible_bounds,
            fit,
        }
    }

    /// Produces the provider-neutral line contract for the bitmap proof path.
    pub fn bitmap_layout(&self, height: f32) -> UiTextLayout {
        let requested_fit = bitmap_text_fit(self, height);
        let height = bitmap::resolved_bitmap_height(self, height);
        let lines = if self.overflow == UiTextOverflow::Defer && !requested_fit.fits() {
            Vec::new()
        } else {
            text_lines(self, height)
        };
        let line_height = bitmap_cell(height) * 9.0;
        let measure = UiTextMeasure {
            advance: lines
                .iter()
                .map(|line| measure_bitmap_text_width(line, height))
                .fold(0.0, f32::max),
            ascent: bitmap_glyph_height(height),
            descent: 0.0,
            line_gap: line_height - bitmap_glyph_height(height),
            visible_bounds: None,
            diagnostics: Vec::new(),
        };
        let block_height = measure.ascent + line_height * lines.len().saturating_sub(1) as f32;
        let first_baseline = match self.align_y {
            UiTextAlign::Start => self.rect.center[1] + self.rect.size[1] * 0.5 - measure.ascent,
            UiTextAlign::Center => self.rect.center[1] + block_height * 0.5 - measure.ascent,
            UiTextAlign::End => {
                self.rect.center[1] - self.rect.size[1] * 0.5 + block_height - measure.ascent
            }
        };
        let align = physical_alignment(self.align_x, self.direction);
        let layouts = lines
            .into_iter()
            .enumerate()
            .map(|(index, text)| {
                let advance = measure_bitmap_text_width(&text, height);
                let origin_x = match align {
                    UiTextAlign::Start => self.rect.center[0] - self.rect.size[0] * 0.5,
                    UiTextAlign::Center => self.rect.center[0] - advance * 0.5,
                    UiTextAlign::End => self.rect.center[0] + self.rect.size[0] * 0.5 - advance,
                };
                UiTextLineLayout {
                    text,
                    origin: [origin_x, first_baseline - index as f32 * line_height],
                    advance,
                    baseline: first_baseline - index as f32 * line_height,
                }
            })
            .collect();

        UiTextLayout {
            measure,
            lines: layouts,
        }
    }
}

impl From<UiLabelAnchor> for UiTextAlign {
    fn from(anchor: UiLabelAnchor) -> Self {
        match anchor {
            UiLabelAnchor::Start => Self::Start,
            UiLabelAnchor::Center => Self::Center,
            UiLabelAnchor::End => Self::End,
        }
    }
}

#[cfg(test)]
mod tests;
