#[derive(Clone, Debug, PartialEq)]
pub struct UiGlyphOutline {
    pub character: char,
    pub units_per_em: f32,
    pub contours: Vec<UiGlyphContour>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiGlyphContour {
    pub start: [f32; 2],
    pub segments: Vec<UiGlyphOutlineSegment>,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiGlyphOutlineSegment {
    LineTo([f32; 2]),
    QuadTo {
        control: [f32; 2],
        end: [f32; 2],
    },
    CubicTo {
        control1: [f32; 2],
        control2: [f32; 2],
        end: [f32; 2],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGlyphOutlineDiagnosticKind {
    MissingOutline,
    InvalidUnitsPerEm,
    NonFiniteCoordinate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiGlyphOutlineDiagnostic {
    pub kind: UiGlyphOutlineDiagnosticKind,
    pub character: char,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGlyphVectorOptions {
    /// Output-space size of one em.
    pub units_per_em_scale: f32,
    /// Output-space glyph origin, normally the positioned baseline pen.
    pub origin: [f32; 2],
    /// Maximum accepted curve-to-chord deviation in output coordinates.
    pub flatten_tolerance: f32,
    /// Negate native font y coordinates for top-left-origin presentation.
    pub flip_y: bool,
}

impl UiGlyphVectorOptions {
    pub fn new(units_per_em_scale: f32, origin: [f32; 2], flatten_tolerance: f32) -> Self {
        Self {
            units_per_em_scale,
            origin,
            flatten_tolerance,
            flip_y: false,
        }
    }

    pub fn with_flipped_y(mut self, flip_y: bool) -> Self {
        self.flip_y = flip_y;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGlyphVectorDiagnosticKind {
    InvalidScale,
    InvalidTolerance,
    NonFiniteOrigin,
    InvalidOutline,
    MissingOutline,
    UnsupportedTopology,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiGlyphVectorDiagnostic {
    pub kind: UiGlyphVectorDiagnosticKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGlyphFillTopology {
    SingleConvexContour,
    SingleConcaveContour,
    MultipleContours,
    Invalid,
}

impl UiGlyphOutlineDiagnostic {
    pub(super) fn new(
        kind: UiGlyphOutlineDiagnosticKind,
        character: char,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            character,
            message: message.into(),
        }
    }
}

impl UiGlyphOutlineSegment {
    pub fn end(&self) -> [f32; 2] {
        match self {
            Self::LineTo(end) | Self::QuadTo { end, .. } | Self::CubicTo { end, .. } => *end,
        }
    }

    pub fn is_finite(&self) -> bool {
        match self {
            Self::LineTo(end) => point_is_finite(*end),
            Self::QuadTo { control, end } => point_is_finite(*control) && point_is_finite(*end),
            Self::CubicTo {
                control1,
                control2,
                end,
            } => point_is_finite(*control1) && point_is_finite(*control2) && point_is_finite(*end),
        }
    }
}

impl UiGlyphContour {
    pub fn is_finite(&self) -> bool {
        point_is_finite(self.start) && self.segments.iter().all(UiGlyphOutlineSegment::is_finite)
    }
}

impl UiGlyphOutline {
    pub fn is_finite(&self) -> bool {
        self.units_per_em.is_finite()
            && self.units_per_em > 0.0
            && self.contours.iter().all(UiGlyphContour::is_finite)
    }
}

pub(super) fn point_is_finite(point: [f32; 2]) -> bool {
    point[0].is_finite() && point[1].is_finite()
}

pub(super) fn vector_diagnostic(
    kind: UiGlyphVectorDiagnosticKind,
    message: impl Into<String>,
) -> UiGlyphVectorDiagnostic {
    UiGlyphVectorDiagnostic {
        kind,
        message: message.into(),
    }
}
