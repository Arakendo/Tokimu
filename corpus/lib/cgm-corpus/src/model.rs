use std::collections::BTreeMap;

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
    Text,
    Raster,
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
    use super::{CgmColor, CgmColorValueExtent, CgmMetafileDescriptor, CgmVdcExtent};

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

    #[test]
    fn direct_color_normalization_uses_the_declared_component_range() {
        let extent = CgmColorValueExtent {
            minimum: [0, 0, 0],
            maximum: [100, 100, 100],
        };
        assert_eq!(
            extent.normalize_direct_rgb(&[0, 50, 100]),
            Some([0.0, 0.5, 1.0])
        );

        let descriptor = CgmMetafileDescriptor {
            color_value_extent: Some(extent),
            ..CgmMetafileDescriptor::default()
        };
        assert_eq!(
            descriptor.normalize_direct_color(&CgmColor::Direct(vec![25, 50, 75])),
            Some([0.25, 0.5, 0.75])
        );
        assert_eq!(
            descriptor.normalize_direct_color(&CgmColor::Indexed(vec![1])),
            None
        );
    }

    #[test]
    fn direct_color_normalization_rejects_ambiguous_component_data() {
        let extent = CgmColorValueExtent {
            minimum: [0, 0, 0],
            maximum: [100, 100, 100],
        };
        assert_eq!(extent.normalize_direct_rgb(&[101, 0, 0]), None);
        assert_eq!(extent.normalize_direct_rgb(&[0, 50]), None);
        assert_eq!(
            CgmColorValueExtent {
                minimum: [0, 0, 0],
                maximum: [0, 100, 100],
            }
            .normalize_direct_rgb(&[0, 50, 50]),
            None
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmMetafileDescriptor {
    pub vdc_type: CgmVdcType,
    pub integer_precision: u16,
    pub color_precision: u16,
    pub color_index_precision: u16,
    /// The source-defined direct-color component range, when the metafile
    /// declares one. Direct color bytes remain unresolved source data until a
    /// later CGM paint adapter applies this range.
    pub color_value_extent: Option<CgmColorValueExtent>,
}

/// The minimum and maximum direct RGB values declared by a CGM metafile.
///
/// The initial binary profile admits 8-bit components only, so each endpoint
/// contains exactly three encoded component values. This is source metadata,
/// not an engine color or a renderer-ready conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmColorValueExtent {
    pub minimum: [u8; 3],
    pub maximum: [u8; 3],
}

impl CgmColorValueExtent {
    /// Resolves one three-component direct source colour into the unit RGB
    /// range declared by this metafile. Values outside that range, missing
    /// components, and degenerate component ranges remain unresolved rather
    /// than being silently clamped or assigned a renderer default.
    pub fn normalize_direct_rgb(self, components: &[u8]) -> Option<[f32; 3]> {
        let components: [u8; 3] = components.try_into().ok()?;
        let mut normalized = [0.0; 3];
        for index in 0..3 {
            let minimum = self.minimum[index];
            let maximum = self.maximum[index];
            let component = components[index];
            if minimum >= maximum || component < minimum || component > maximum {
                return None;
            }
            normalized[index] = f32::from(component - minimum) / f32::from(maximum - minimum);
        }
        normalized
            .iter()
            .all(|value| value.is_finite())
            .then_some(normalized)
    }
}

impl CgmMetafileDescriptor {
    /// Resolves an explicit direct CGM colour only when this descriptor carries
    /// a usable source range. Indexed colours still require a colour-table
    /// policy and remain source data.
    pub fn normalize_direct_color(&self, color: &CgmColor) -> Option<[f32; 3]> {
        match (self.color_value_extent, color) {
            (Some(extent), CgmColor::Direct(components)) => extent.normalize_direct_rgb(components),
            _ => None,
        }
    }
}

impl Default for CgmMetafileDescriptor {
    fn default() -> Self {
        Self {
            vdc_type: CgmVdcType::Integer,
            integer_precision: 16,
            color_precision: 8,
            color_index_precision: 8,
            color_value_extent: None,
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

/// A bounded interpretation of a CGM `INTERIOR STYLE` value.
///
/// The selected corpus currently demonstrates only the standard solid value.
/// Other values retain their source representation rather than being
/// prematurely interpreted as provider-neutral fills.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum CgmInteriorStyle {
    Solid,
    Other(u16),
}

impl From<u16> for CgmInteriorStyle {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::Solid,
            _ => Self::Other(value),
        }
    }
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
    LineWidth {
        bytes: Vec<u8>,
    },
    LineColor {
        color: CgmColor,
    },
    InteriorStyle {
        style: CgmInteriorStyle,
    },
    FillColor {
        color: CgmColor,
    },
    EdgeWidth {
        bytes: Vec<u8>,
    },
    EdgeColor {
        color: CgmColor,
    },
    EdgeVisibility {
        visible: bool,
    },
    LineCap {
        bytes: Vec<u8>,
    },
    LineJoin {
        value: u16,
    },
    /// Integer-VDC character height retained as CGM source state. This is not
    /// a resolved font size or a renderer-facing text metric.
    CharacterHeight {
        value: i32,
    },
    /// Encoded CGM character-spacing value. Its real-number representation
    /// remains provider-owned source data until a text-layout profile exists.
    CharacterSpacing {
        bytes: Vec<u8>,
    },
    /// The CGM up and base vectors that orient source text. Text shaping and
    /// layout remain outside this importer.
    CharacterOrientation {
        up: [i32; 2],
        base: [i32; 2],
    },
    /// CGM text-path enum retained without converting it into glyph layout.
    TextPath {
        value: u16,
    },
    /// Source text alignment enums and their encoded continuous offsets.
    /// Continuous values remain raw because their real-number representation
    /// belongs to the CGM descriptor profile.
    TextAlignment {
        horizontal: u16,
        vertical: u16,
        continuous_horizontal: [u8; 4],
        continuous_vertical: [u8; 4],
    },
    /// A bounded 8-bit direct-RGB colour-table update. CGM color-table
    /// interpretation remains source state until a later paint adapter uses it.
    ColorTable {
        start_index: u8,
        colors: Vec<[u8; 3]>,
    },
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
    pub interior_style: Option<CgmInteriorStyle>,
    pub fill_color: Option<CgmColor>,
    pub edge_width: Option<Vec<u8>>,
    pub edge_color: Option<CgmColor>,
    pub edge_visible: Option<bool>,
    pub line_cap: Option<Vec<u8>>,
    pub line_join: Option<u16>,
    pub character_height: Option<i32>,
    pub character_spacing: Option<Vec<u8>>,
    pub character_orientation: Option<CgmTextOrientation>,
    pub text_path: Option<u16>,
    pub text_alignment: Option<CgmTextAlignment>,
    /// Explicit picture-local palette entries observed before this primitive.
    /// Missing entries intentionally remain unresolved: CGM defaults and
    /// bundle policy are outside the admitted profile.
    pub color_table: BTreeMap<u8, [u8; 3]>,
}

impl CgmPresentationState {
    pub fn is_default(&self) -> bool {
        self.line_width.is_none()
            && self.line_color.is_none()
            && self.interior_style.is_none()
            && self.fill_color.is_none()
            && self.edge_width.is_none()
            && self.edge_color.is_none()
            && self.edge_visible.is_none()
            && self.line_cap.is_none()
            && self.line_join.is_none()
            && self.character_height.is_none()
            && self.character_spacing.is_none()
            && self.character_orientation.is_none()
            && self.text_path.is_none()
            && self.text_alignment.is_none()
            && self.color_table.is_empty()
    }

    pub(crate) fn apply(&mut self, attribute: &CgmAttributeValue) {
        match attribute {
            CgmAttributeValue::LineWidth { bytes } => self.line_width = Some(bytes.clone()),
            CgmAttributeValue::LineColor { color } => self.line_color = Some(color.clone()),
            CgmAttributeValue::InteriorStyle { style } => self.interior_style = Some(*style),
            CgmAttributeValue::FillColor { color } => self.fill_color = Some(color.clone()),
            CgmAttributeValue::EdgeWidth { bytes } => self.edge_width = Some(bytes.clone()),
            CgmAttributeValue::EdgeColor { color } => self.edge_color = Some(color.clone()),
            CgmAttributeValue::EdgeVisibility { visible } => self.edge_visible = Some(*visible),
            CgmAttributeValue::LineCap { bytes } => self.line_cap = Some(bytes.clone()),
            CgmAttributeValue::LineJoin { value } => self.line_join = Some(*value),
            CgmAttributeValue::CharacterHeight { value } => self.character_height = Some(*value),
            CgmAttributeValue::CharacterSpacing { bytes } => {
                self.character_spacing = Some(bytes.clone())
            }
            CgmAttributeValue::CharacterOrientation { up, base } => {
                self.character_orientation = Some(CgmTextOrientation {
                    up: *up,
                    base: *base,
                });
            }
            CgmAttributeValue::TextPath { value } => self.text_path = Some(*value),
            CgmAttributeValue::TextAlignment {
                horizontal,
                vertical,
                continuous_horizontal,
                continuous_vertical,
            } => {
                self.text_alignment = Some(CgmTextAlignment {
                    horizontal: *horizontal,
                    vertical: *vertical,
                    continuous_horizontal: *continuous_horizontal,
                    continuous_vertical: *continuous_vertical,
                });
            }
            CgmAttributeValue::ColorTable {
                start_index,
                colors,
            } => {
                for (offset, color) in colors.iter().enumerate() {
                    let index = usize::from(*start_index) + offset;
                    if let Ok(index) = u8::try_from(index) {
                        self.color_table.insert(index, *color);
                    }
                }
            }
        }
    }

    /// Resolves an explicitly declared direct or indexed source color through
    /// this primitive's state snapshot. Missing palette entries and all
    /// defaults deliberately remain unresolved.
    pub fn normalize_explicit_color(
        &self,
        metafile: &CgmMetafileDescriptor,
        color: &CgmColor,
    ) -> Option<[f32; 3]> {
        match color {
            CgmColor::Direct(_) => metafile.normalize_direct_color(color),
            CgmColor::Indexed(index) => {
                let [index] = index.as_slice() else {
                    return None;
                };
                let components = self.color_table.get(index)?;
                metafile
                    .color_value_extent?
                    .normalize_direct_rgb(components)
            }
        }
    }
}

/// Text-orientation source state active when a CGM text record appears.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmTextOrientation {
    pub up: [i32; 2],
    pub base: [i32; 2],
}

/// Text-alignment source state active when a CGM text record appears.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmTextAlignment {
    pub horizontal: u16,
    pub vertical: u16,
    pub continuous_horizontal: [u8; 4],
    pub continuous_vertical: [u8; 4],
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

/// One text-bearing CGM source record before text layout or rendering.
///
/// CGM owns the source coordinates, restriction values, final flag, and
/// string. It does not select a font, shape text, create glyph outlines, or
/// imply a renderer-facing text command.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmTextRecord {
    pub source_element: usize,
    pub source_offset: usize,
    pub attribute_count: usize,
    pub state: CgmPresentationState,
    pub controls: CgmPictureControlState,
    pub kind: CgmTextRecordKind,
}

/// Source-level cell-array metadata. The encoded payload remains CGM-owned
/// raster data; this record intentionally does not select a texture format,
/// decode pixels, or create renderer resources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmCellArrayRecord {
    pub source_element: usize,
    pub source_offset: usize,
    pub attribute_count: usize,
    pub state: CgmPresentationState,
    pub controls: CgmPictureControlState,
    pub first: [i32; 2],
    pub second: [i32; 2],
    pub third: [i32; 2],
    pub dimensions: [u16; 2],
    pub local_color_precision: u16,
    pub representation: u16,
    pub payload_bytes: usize,
}

/// The bounded text forms observed by the selected WebCGM fixture.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum CgmTextRecordKind {
    /// Source text with an explicit position and two CGM restriction values.
    Restricted {
        position: [i32; 2],
        restrictions: [i32; 2],
        final_flag: u16,
        text: String,
    },
    /// Source text appended to the active CGM text path.
    Append { final_flag: u16, text: String },
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
    /// CGM POLYBEZIER source records.
    ///
    /// The continuity indicator belongs to CGM's source grammar. It is
    /// preserved with the VDC control points until corpus evidence defines a
    /// provider-neutral cubic-path lowering policy.
    PolyBezier {
        continuity: u16,
        points: Vec<[i32; 2]>,
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

/// One grouped deferred CGM source feature.
///
/// Individual [`CgmDiagnostic`] records remain the authoritative evidence.
/// This summary exists for consumers that need to present repeated source
/// features without independently recreating CGM grouping policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CgmDeferredFeature {
    pub code: CgmDiagnosticCode,
    pub class: u8,
    pub id: u8,
    pub feature: String,
    pub count: usize,
    pub message: String,
}

/// Returns a conservative, human-readable name for one CGM class/element
/// identity.
///
/// The bounded CGM profile owns these source-format names. Unknown identities
/// deliberately remain addressed by class and element ID instead of gaining
/// inferred semantics in a consumer.
pub fn cgm_element_name(class: u8, id: u8) -> String {
    let name = match (class, id) {
        (1, 1) => "metafile version",
        (1, 2) => "metafile description",
        (1, 5) => "real precision",
        (1, 6) => "index precision",
        (1, 9) => "maximum color index",
        (1, 10) => "color value extent",
        (1, 11) => "metafile element list",
        (1, 13) => "font list",
        (1, 14) => "character set list",
        (1, 15) => "character coding announcer",
        (2, 3) => "line-width specification mode",
        (2, 4) => "marker-size specification mode",
        (2, 5) => "edge-width specification mode",
        (2, 7) => "background color",
        (4, 5) => "text primitive",
        (4, 6) => "append text primitive",
        (4, 9) => "cell array raster primitive",
        (4, 26) => "polybezier primitive",
        (5, 15) => "character height",
        (5, 16) => "character orientation",
        (5, 18) => "text alignment",
        (5, 34) => "color table",
        _ => return format!("CGM class {class} element {id}"),
    };
    format!("CGM {name}")
}

/// Groups repeated CGM diagnostics by their source element identity.
///
/// The provider owns this transformation because both artifact writers and
/// interactive consumers need the same conservative CGM feature identity.
pub fn summarize_diagnostics(diagnostics: &[CgmDiagnostic]) -> Vec<CgmDeferredFeature> {
    let mut groups = BTreeMap::<(u8, u8), CgmDeferredFeature>::new();

    for diagnostic in diagnostics {
        let feature = groups
            .entry((diagnostic.class, diagnostic.id))
            .or_insert_with(|| CgmDeferredFeature {
                code: diagnostic.code,
                class: diagnostic.class,
                id: diagnostic.id,
                feature: cgm_element_name(diagnostic.class, diagnostic.id),
                count: 0,
                message: diagnostic.message.clone(),
            });
        feature.count += 1;
    }

    groups.into_values().collect()
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
    /// Text-bearing source records retained independently from geometry.
    pub text_records: Vec<CgmTextRecord>,
    /// Cell-array headers retained independently from vector geometry.
    pub cell_arrays: Vec<CgmCellArrayRecord>,
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
