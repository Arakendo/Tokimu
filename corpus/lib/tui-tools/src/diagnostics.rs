use crate::TuiRect;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TuiDiagnostic {
    EmptyRegion {
        region: TuiRect,
    },
    Undersized {
        axis: &'static str,
        available: u16,
        required: u16,
    },
    TextClipped {
        region: TuiRect,
        omitted_characters: usize,
    },
    ContentTruncated {
        region: TuiRect,
        omitted_lines: usize,
    },
    ViewportClamped {
        requested_offset: u16,
        actual_offset: u16,
    },
    EmptyViewport {
        viewport_rows: u16,
    },
}
