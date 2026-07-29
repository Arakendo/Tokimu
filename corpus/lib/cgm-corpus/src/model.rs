use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CgmEncoding {
    Binary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DelimiterElement {
    BeginMetafile,
    EndMetafile,
    BeginPicture,
    BeginPictureBody,
    EndPicture,
}

impl DelimiterElement {
    pub(crate) fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::BeginMetafile),
            2 => Some(Self::EndMetafile),
            3 => Some(Self::BeginPicture),
            4 => Some(Self::BeginPictureBody),
            5 => Some(Self::EndPicture),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ElementSupport {
    Lifecycle,
    Descriptor,
    Control,
    Attribute,
    Primitive,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmPartition {
    pub parameter_offset: usize,
    pub parameter_length: usize,
    pub continues: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmElement {
    pub index: usize,
    pub source_offset: usize,
    pub encoded_length: usize,
    pub header_length: usize,
    pub class: u8,
    pub id: u8,
    pub parameter_length: usize,
    pub partitions: Vec<CgmPartition>,
    pub support: ElementSupport,
    pub delimiter: Option<DelimiterElement>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CgmDiagnosticCode {
    UnsupportedElement,
}

/// Coordinate representation declared by the CGM metafile descriptor.
///
/// CGM applies this choice before any picture-local geometry is interpreted.
/// The initial corpus profile only admits integer VDC coordinates, but keeps
/// the distinction visible instead of silently treating real coordinates as
/// integers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CgmVdcType {
    Integer,
    Real,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CgmScalingMode {
    Abstract,
    Metric,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CgmColorSelectionMode {
    Indexed,
    Direct,
}

/// The two source corners of a CGM VDC extent.
///
/// `first` and `second` deliberately retain source ordering. In particular,
/// CGM pictures commonly use a descending Y axis, so replacing them with
/// sorted minimum/maximum corners here would erase orientation information.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmVdcExtent {
    pub first: [i32; 2],
    pub second: [i32; 2],
}

impl CgmVdcExtent {
    /// Maps an integer VDC point into the provider-neutral unit square.
    ///
    /// The mapping uses the source corner order directly. A descending source
    /// axis therefore remains descending after normalization rather than being
    /// hidden by sorting the extent first. Degenerate extents have no valid
    /// normalization and return `None` instead of producing non-finite output.
    pub fn normalize(self, point: [i32; 2]) -> Option<[f32; 2]> {
        // Convert before subtracting: source VDC values may span the full
        // signed range even though the first binary profile currently admits
        // only 16-bit coordinates. Semantic normalization must not inherit a
        // debug-overflow trap from a future wider precision profile.
        let delta_x = f64::from(self.second[0]) - f64::from(self.first[0]);
        let delta_y = f64::from(self.second[1]) - f64::from(self.first[1]);
        if delta_x == 0.0 || delta_y == 0.0 {
            return None;
        }

        let normalized = [
            ((f64::from(point[0]) - f64::from(self.first[0])) / delta_x) as f32,
            ((f64::from(point[1]) - f64::from(self.first[1])) / delta_y) as f32,
        ];
        normalized
            .iter()
            .all(|value| value.is_finite())
            .then_some(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::CgmVdcExtent;

    #[test]
    fn normalization_preserves_source_axis_order_and_remains_finite() {
        let descending_y = CgmVdcExtent {
            first: [0, 1000],
            second: [1000, 0],
        };
        assert_eq!(descending_y.normalize([0, 1000]), Some([0.0, 0.0]));
        assert_eq!(descending_y.normalize([1000, 0]), Some([1.0, 1.0]));

        let full_range = CgmVdcExtent {
            first: [i32::MIN, i32::MIN],
            second: [i32::MAX, i32::MAX],
        };
        let midpoint = full_range
            .normalize([0, 0])
            .expect("non-degenerate full-range extent should normalize");
        assert!(midpoint.iter().all(|value| value.is_finite()));
        assert!((midpoint[0] - 0.5).abs() < 0.000_001);
        assert!((midpoint[1] - 0.5).abs() < 0.000_001);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmMetafileDescriptor {
    pub vdc_type: CgmVdcType,
    pub integer_precision: u16,
    pub color_precision: u16,
    pub color_index_precision: u16,
}

impl Default for CgmMetafileDescriptor {
    fn default() -> Self {
        Self {
            vdc_type: CgmVdcType::Integer,
            integer_precision: 16,
            color_precision: 8,
            color_index_precision: 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmPictureDescriptor {
    pub scaling_mode: CgmScalingMode,
    /// The encoding-specific metric scale factor is preserved until the
    /// normalization policy is admitted. Abstract scaling has no factor.
    pub metric_scale_bytes: Option<Vec<u8>>,
    pub color_selection_mode: CgmColorSelectionMode,
    pub vdc_extent: Option<CgmVdcExtent>,
}

/// Whether the active CGM clipping rectangle applies to subsequent primitives.
///
/// This preserves the source control separately from vector clipping. CGM's
/// paint and edge rules still determine what clipping means for a primitive,
/// so the adapter records rather than executes this state for now.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CgmClipIndicator {
    Off,
    On,
}

/// Picture-body controls active when a primitive was encountered.
///
/// Controls are distinct from CGM attributes, but both are source snapshots
/// that later lowering stages may need to interpret in source order.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmPictureControlState {
    pub clip_rectangle: Option<CgmVdcExtent>,
    pub clip_indicator: Option<CgmClipIndicator>,
}

/// A CGM colour value before a palette or renderer resolves it.
///
/// Direct colours retain their encoded component bytes because their numeric
/// range is defined by additional CGM descriptor state. Indexed colours retain
/// the same information for the colour table stage. Neither form is silently
/// converted to an RGB renderer colour here.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "bytes")]
pub enum CgmColor {
    Indexed(Vec<u8>),
    Direct(Vec<u8>),
}

/// An explicit presentation-attribute mutation observed in a picture body.
///
/// CGM attributes are source-format state. They remain in the CGM importer
/// until primitive lowering has evidence for a provider-neutral style contract.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmAttribute {
    pub source_element: usize,
    pub source_offset: usize,
    pub value: CgmAttributeValue,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum CgmAttributeValue {
    LineWidth { bytes: Vec<u8> },
    LineColor { color: CgmColor },
    InteriorStyle { value: u16 },
    FillColor { color: CgmColor },
    EdgeWidth { bytes: Vec<u8> },
    EdgeColor { color: CgmColor },
    EdgeVisibility { visible: bool },
    LineCap { bytes: Vec<u8> },
    LineJoin { value: u16 },
}

/// The explicit CGM drawing state active when a primitive appears.
///
/// This is a deterministic source-format snapshot, not a renderer style.
/// Absent values deliberately remain absent until the corpus establishes the
/// relevant CGM defaults, bundles, palettes, and rendering policy.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmPresentationState {
    pub line_width: Option<Vec<u8>>,
    pub line_color: Option<CgmColor>,
    pub interior_style: Option<u16>,
    pub fill_color: Option<CgmColor>,
    pub edge_width: Option<Vec<u8>>,
    pub edge_color: Option<CgmColor>,
    pub edge_visible: Option<bool>,
    pub line_cap: Option<Vec<u8>>,
    pub line_join: Option<u16>,
}

impl CgmPresentationState {
    pub const fn is_default(&self) -> bool {
        self.line_width.is_none()
            && self.line_color.is_none()
            && self.interior_style.is_none()
            && self.fill_color.is_none()
            && self.edge_width.is_none()
            && self.edge_color.is_none()
            && self.edge_visible.is_none()
            && self.line_cap.is_none()
            && self.line_join.is_none()
    }

    pub(crate) fn apply(&mut self, attribute: &CgmAttributeValue) {
        match attribute {
            CgmAttributeValue::LineWidth { bytes } => self.line_width = Some(bytes.clone()),
            CgmAttributeValue::LineColor { color } => self.line_color = Some(color.clone()),
            CgmAttributeValue::InteriorStyle { value } => self.interior_style = Some(*value),
            CgmAttributeValue::FillColor { color } => self.fill_color = Some(color.clone()),
            CgmAttributeValue::EdgeWidth { bytes } => self.edge_width = Some(bytes.clone()),
            CgmAttributeValue::EdgeColor { color } => self.edge_color = Some(color.clone()),
            CgmAttributeValue::EdgeVisibility { visible } => self.edge_visible = Some(*visible),
            CgmAttributeValue::LineCap { bytes } => self.line_cap = Some(bytes.clone()),
            CgmAttributeValue::LineJoin { value } => self.line_join = Some(*value),
        }
    }
}

impl CgmPictureControlState {
    pub const fn is_default(&self) -> bool {
        self.clip_rectangle.is_none() && self.clip_indicator.is_none()
    }
}

/// A bounded CGM primitive record before provider-neutral vector lowering.
///
/// `attribute_count` identifies the prefix of the owning picture's ordered
/// attribute mutations that was active when this primitive occurred. `state`
/// is the corresponding explicit source-format snapshot. Keeping both makes
/// primitive interpretation direct while retaining ordered provenance for
/// diagnostics and later corpus work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmPrimitive {
    pub source_element: usize,
    pub source_offset: usize,
    pub attribute_count: usize,
    pub state: CgmPresentationState,
    pub controls: CgmPictureControlState,
    pub kind: CgmPrimitiveKind,
}

/// Source edge behavior for one CGM POLYGON SET record.
///
/// An edge runs from the record point to the next record point. `Close*`
/// instead closes the current boundary back to its first point. This preserves
/// CGM source semantics without implying that the records are already a
/// provider-neutral fill topology.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CgmPolygonSetEdgeFlag {
    Invisible,
    Visible,
    CloseInvisible,
    CloseVisible,
}

/// One source-order point/edge record from a CGM POLYGON SET element.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmPolygonSetRecord {
    pub point: [i32; 2],
    pub edge: CgmPolygonSetEdgeFlag,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum CgmPrimitiveKind {
    Polyline {
        points: Vec<[i32; 2]>,
    },
    Polygon {
        points: Vec<[i32; 2]>,
    },
    PolygonSet {
        records: Vec<CgmPolygonSetRecord>,
    },
    Rectangle {
        first: [i32; 2],
        second: [i32; 2],
    },
    Circle {
        center: [i32; 2],
        radius: i32,
    },
    Ellipse {
        center: [i32; 2],
        first_axis: [i32; 2],
        second_axis: [i32; 2],
    },
    /// An open circular arc. CGM defines the sweep counter-clockwise from the
    /// start vector to the end vector around `center`.
    CircularArc {
        center: [i32; 2],
        start_vector: [i32; 2],
        end_vector: [i32; 2],
        radius: i32,
    },
    /// An open elliptical arc defined by two conjugate-diameter endpoints and
    /// start/end vectors relative to the center.
    EllipticalArc {
        center: [i32; 2],
        first_axis: [i32; 2],
        second_axis: [i32; 2],
        start_vector: [i32; 2],
        end_vector: [i32; 2],
    },
}

impl Default for CgmPictureDescriptor {
    fn default() -> Self {
        Self {
            scaling_mode: CgmScalingMode::Abstract,
            metric_scale_bytes: None,
            color_selection_mode: CgmColorSelectionMode::Indexed,
            vdc_extent: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmDiagnostic {
    pub code: CgmDiagnosticCode,
    pub source_offset: usize,
    pub class: u8,
    pub id: u8,
    pub picture: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmPicture {
    pub name: String,
    pub begin_element: usize,
    pub body_element: Option<usize>,
    pub end_element: usize,
    pub descriptor: CgmPictureDescriptor,
    pub controls: CgmPictureControlState,
    pub attributes: Vec<CgmAttribute>,
    pub primitives: Vec<CgmPrimitive>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmInspection {
    pub encoding: CgmEncoding,
    pub source_bytes: usize,
    pub trailing_padding_bytes: usize,
    pub metafile_name: String,
    pub metafile: CgmMetafileDescriptor,
    pub elements: Vec<CgmElement>,
    pub pictures: Vec<CgmPicture>,
    pub diagnostics: Vec<CgmDiagnostic>,
}
