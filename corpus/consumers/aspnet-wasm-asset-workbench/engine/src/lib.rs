use cgm_corpus::{
    inspect_binary_cgm, lower_picture_primitives, summarize_diagnostics, CgmDeferredFeature,
    CgmInspection, DecodeLimits as CgmDecodeLimits,
};
use fbx_corpus::{
    decode_ascii_fbx, decode_binary_fbx, lower_static_geometry, resolve_source_scene,
    FbxGeometryEvidence, FbxLimits,
};
use gltf_corpus::{decode_glb, inspect_glb, inspect_gltf, GltfSummary, TransformMatrix};
use presentation_control::{
    PresentationColor, PresentationControl, PresentationControlError, PresentationLayer,
    PresentationOverride, PresentationTargetDescriptor, PresentationTargetId,
    PresentationTargetKind, ResolvedPresentation, SourcePresentation,
};
use raster_image_corpus::{
    decode_bmp, decode_jpeg, decode_png, AlphaMode, ColorSpace, DecodeLimits as RasterDecodeLimits,
    DecodedImage, ImageOrientation, PixelFormat,
};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceEntry, ResourceMetadata,
    ResourceRootDescriptor, ResourceRootId, ResourceSpaceLimits, StoreId,
};
use resource_space_assets::{
    decode_gltf_from_resource_space, resolve_gltf_external_images_from_resource_space,
};
use serde::{Deserialize, Serialize};
use tokimu::World;
use ui_tools::{
    parse_svg_document_vector_records_with_viewport, SvgColor, SvgViewportSource, VectorPath,
};
use wasm_bindgen::prelude::*;

const SCHEMA: u32 = 1;
const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_RESOURCE_SESSION_ENTRIES: usize = 32;
const MAX_RESOURCE_SESSION_BYTES: usize = 128 * 1024 * 1024;
// CGM paths are normalized into the unit square before entering this browser
// diagnostic preview. This is a unit-space hairline, not CGM LINE WIDTH.
const CGM_DIAGNOSTIC_STROKE_WIDTH: f32 = 0.0025;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetObservation {
    schema: u32,
    file_name: String,
    format: String,
    status: String,
    byte_length: usize,
    summary: String,
    properties: Vec<Property>,
    diagnostics: Vec<String>,
    preview: Option<VectorPreview>,
    presentation_targets: Vec<PresentationTargetObservation>,
}

#[derive(Deserialize, Serialize)]
struct Property {
    label: String,
    value: String,
}

#[derive(Deserialize, Serialize)]
struct VectorPreview {
    kind: String,
    paths: Vec<PreviewPath>,
    triangles: Vec<PreviewTriangle>,
}

#[derive(Deserialize, Serialize)]
struct PreviewTriangle {
    points: [[f32; 3]; 3],
    target: Option<PreviewTarget>,
}

#[derive(Deserialize, Serialize)]
struct PreviewPath {
    contours: Vec<PreviewContour>,
    fill: bool,
    stroke: bool,
    color: [f32; 4],
    stroke_width: f32,
    target: Option<PreviewTarget>,
}

#[derive(Deserialize, Serialize)]
struct PreviewContour {
    points: Vec<[f32; 2]>,
    closed: bool,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewTarget {
    kind: PresentationTargetKind,
    key: String,
}

/// A serializable, provider-neutral target advertised by an asset observation.
///
/// Browser callers may display this information or send its stable `kind` and
/// `key` back to a bounded presentation session. They do not infer targets by
/// parsing the original source bytes.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PresentationTargetObservation {
    kind: PresentationTargetKind,
    key: String,
    source_name: Option<String>,
    source: SourcePresentation,
}

impl PresentationTargetObservation {
    fn from_descriptor(
        descriptor: PresentationTargetDescriptor,
        source: SourcePresentation,
    ) -> Self {
        Self {
            kind: descriptor.id().kind(),
            key: descriptor.id().key().to_owned(),
            source_name: descriptor.source_name().map(str::to_owned),
            source,
        }
    }

    fn descriptor(&self) -> Result<PresentationTargetDescriptor, String> {
        let id = PresentationTargetId::new(self.kind, self.key.clone())
            .map_err(|error| error.to_string())?;
        match &self.source_name {
            Some(source_name) => PresentationTargetDescriptor::new(id)
                .with_source_name(source_name.clone())
                .map_err(|error| error.to_string()),
            None => Ok(PresentationTargetDescriptor::new(id)),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PresentationOverrideRequest {
    kind: PresentationTargetKind,
    key: String,
    layer: PresentationLayer,
    override_value: PresentationOverride,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PresentationClearRequest {
    kind: PresentationTargetKind,
    key: String,
    layer: PresentationLayer,
}

/// A bounded diagnostic returned for a presentation command that cannot be
/// applied. This preserves the semantic error category without exposing Rust
/// parser internals or requiring TypeScript to interpret exception strings.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PresentationDiagnostic {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum PresentationCommandResponse {
    Resolved { resolved: ResolvedPresentation },
    Rejected { diagnostic: PresentationDiagnostic },
}

/// Stateful, provider-neutral presentation-command boundary for WASM hosts.
///
/// This owns no importer data or browser rendering state. It only resolves
/// commands against the target descriptors Tokimu emitted in an observation.
#[wasm_bindgen]
pub struct PresentationSession {
    control: PresentationControl,
}

/// A bounded imported resource root owned by the WASM consumer.
///
/// Browser code supplies explicitly selected byte arrays and logical names.
/// This session owns the provider-neutral hierarchy and dependency lookup; it
/// neither reads browser paths nor asks TypeScript to resolve glTF references.
#[wasm_bindgen]
pub struct ResourceSession {
    space: InMemoryResourceSpace,
    folder: FolderId,
}

#[wasm_bindgen]
pub fn engine_status() -> String {
    let mut world = World::default();
    let entity = world.spawn();
    format!(
        "Tokimu WASM consumer bridge ready; public facade spawned entity {:?}",
        entity
    )
}

#[wasm_bindgen]
pub fn inspect_asset(file_name: &str, bytes: &[u8]) -> String {
    let observation = inspect(file_name, bytes).unwrap_or_else(|message| AssetObservation {
        schema: SCHEMA,
        file_name: file_name.to_owned(),
        format: classify(file_name).to_owned(),
        status: "error".into(),
        byte_length: bytes.len(),
        summary: "Tokimu could not inspect this asset.".into(),
        properties: Vec::new(),
        diagnostics: vec![message],
        preview: None,
        presentation_targets: Vec::new(),
    });

    serde_json::to_string(&observation).expect("asset observation should serialize")
}

#[wasm_bindgen]
impl ResourceSession {
    /// Creates an empty, transient imported root for one browser selection.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        const STORE: StoreId = StoreId::from_u128(0xA55E_7001);
        const ROOT: ResourceRootId = ResourceRootId::from_u128(0xA55E_7002);
        const FOLDER: FolderId = FolderId::from_u128(0xA55E_7003);

        let mut space = InMemoryResourceSpace::with_limits(
            STORE,
            AddressCasePolicy::Sensitive,
            ResourceSpaceLimits {
                max_entries: Some(MAX_RESOURCE_SESSION_ENTRIES),
                max_total_bytes: Some(MAX_RESOURCE_SESSION_BYTES),
                max_bytes_per_entry: Some(MAX_INPUT_BYTES),
            },
        );
        let descriptor = ResourceRootDescriptor::new(ROOT, "Selected browser resources");
        space
            .create_root(descriptor, FOLDER, ResourceMetadata::default())
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self {
            space,
            folder: FOLDER,
        })
    }

    /// Retains one explicitly selected resource beneath the session root.
    ///
    /// Names must be logical relative addresses. This first browser proof
    /// admits same-folder glTF dependencies only, so nested paths are rejected
    /// rather than being silently flattened.
    pub fn add_resource(&mut self, name: &str, bytes: &[u8]) -> Result<(), JsValue> {
        self.add_resource_inner(name, bytes)
            .map_err(|message| JsValue::from_str(&message))
    }

    /// Inspects one selected document, resolving same-folder glTF references
    /// through the resource session instead of a frontend-side importer.
    pub fn inspect_resource(&self, name: &str) -> Result<String, JsValue> {
        self.inspect_resource_inner(name)
            .map_err(|message| JsValue::from_str(&message))
    }

    /// Returns one selected logical resource for a browser-owned download.
    ///
    /// The session never opens a browser save dialog or exposes host paths.
    /// TypeScript owns the user gesture and turns these bytes into a download.
    pub fn resource_bytes(&self, name: &str) -> Result<Vec<u8>, JsValue> {
        self.resource_bytes_inner(name)
            .map_err(|message| JsValue::from_str(&message))
    }

    /// Returns bounded logical-store counts for host diagnostics.
    pub fn summary(&self) -> String {
        let summary = self.space.summary();
        format!(
            "roots={} folders={} resources={} retained_bytes={}",
            summary.roots(),
            summary.folders(),
            summary.resources(),
            summary.retained_bytes()
        )
    }
}

impl ResourceSession {
    fn add_resource_inner(&mut self, name: &str, bytes: &[u8]) -> Result<(), String> {
        if bytes.is_empty() {
            return Err("selected resource is empty".into());
        }
        if bytes.len() > MAX_INPUT_BYTES {
            return Err(format!(
                "selected resource has {} bytes, exceeding the workbench limit of {}",
                bytes.len(),
                MAX_INPUT_BYTES
            ));
        }
        let parsed = self.selected_resource_name(name)?;
        self.space
            .insert_resource(
                self.folder,
                parsed,
                bytes.to_vec(),
                ResourceMetadata::default(),
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn inspect_resource_inner(&self, name: &str) -> Result<String, String> {
        let document = self.entry(name)?;
        let observation = match classify(name) {
            "gltf" => inspect_gltf_resource_asset(&self.space, self.folder, &document),
            _ => inspect(name, document.bytes()),
        }
        .map_err(|message| message.to_string())?;
        serde_json::to_string(&observation)
            .map_err(|error| format!("asset observation did not serialize: {error}"))
    }

    fn resource_bytes_inner(&self, name: &str) -> Result<Vec<u8>, String> {
        Ok(self.entry(name)?.bytes().to_vec())
    }

    fn entry(&self, name: &str) -> Result<ResourceEntry, String> {
        let parsed = self.selected_resource_name(name)?;
        self.space
            .resource(self.folder, &parsed)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("selected resource `{name}` is not present in this session"))
    }

    fn selected_resource_name(&self, name: &str) -> Result<resource_space::ResourceName, String> {
        if name.contains(['/', '\\', ':']) || matches!(name, "." | "..") {
            return Err(format!(
                "selected resource `{name}` must be a same-folder logical file name"
            ));
        }
        self.space
            .resource_name(name)
            .map_err(|error| error.to_string())
    }
}

#[wasm_bindgen]
impl PresentationSession {
    /// Creates a command session from the observation JSON returned by
    /// `inspect_asset`.
    #[wasm_bindgen(constructor)]
    pub fn new(observation_json: &str) -> Result<Self, JsValue> {
        let observation = serde_json::from_str::<AssetObservation>(observation_json)
            .map_err(|error| JsValue::from_str(&format!("invalid asset observation: {error}")))?;
        let mut control = PresentationControl::default();
        for target in observation.presentation_targets {
            let descriptor = target
                .descriptor()
                .map_err(|message| JsValue::from_str(&message))?;
            control
                .register_target_with_descriptor(descriptor, target.source)
                .map_err(|error| JsValue::from_str(&error.to_string()))?;
        }
        Ok(Self { control })
    }

    /// Returns the known targets and source values without exposing provider
    /// parser objects or renderer resources.
    pub fn targets(&self) -> String {
        let targets = self
            .control
            .targets()
            .map(|(_, state)| {
                PresentationTargetObservation::from_descriptor(
                    state.descriptor().clone(),
                    state.source(),
                )
            })
            .collect::<Vec<_>>();
        serde_json::to_string(&targets).expect("presentation targets should serialize")
    }

    /// Applies one bounded transient override and returns either the resolved
    /// provider-neutral presentation or a structured diagnostic as JSON.
    pub fn set_override(&mut self, request_json: &str) -> Result<String, JsValue> {
        let response = match serde_json::from_str::<PresentationOverrideRequest>(request_json) {
            Ok(request) => match PresentationTargetId::new(request.kind, request.key) {
                Ok(target) => match self
                    .control
                    .set_override(&target, request.layer, request.override_value)
                    .and_then(|()| self.control.resolve(&target))
                {
                    Ok(resolved) => PresentationCommandResponse::Resolved { resolved },
                    Err(error) => rejected_response(error),
                },
                Err(error) => rejected_response(error),
            },
            Err(error) => rejected_invalid_request(error.to_string()),
        };
        serialize_command_response(response)
    }

    /// Clears one transient override layer and returns either the restored
    /// presentation or a structured diagnostic as JSON.
    pub fn clear_override(&mut self, request_json: &str) -> Result<String, JsValue> {
        let response = match serde_json::from_str::<PresentationClearRequest>(request_json) {
            Ok(request) => match PresentationTargetId::new(request.kind, request.key) {
                Ok(target) => match self
                    .control
                    .clear_override(&target, request.layer)
                    .and_then(|_| self.control.resolve(&target))
                {
                    Ok(resolved) => PresentationCommandResponse::Resolved { resolved },
                    Err(error) => rejected_response(error),
                },
                Err(error) => rejected_response(error),
            },
            Err(error) => rejected_invalid_request(error.to_string()),
        };
        serialize_command_response(response)
    }
}

fn rejected_response(error: PresentationControlError) -> PresentationCommandResponse {
    PresentationCommandResponse::Rejected {
        diagnostic: PresentationDiagnostic {
            code: presentation_error_code(&error),
            message: error.to_string(),
        },
    }
}

fn presentation_error_code(error: &PresentationControlError) -> &'static str {
    match error {
        PresentationControlError::UnknownTarget { .. } => "unknown-target",
        PresentationControlError::InvalidUnitValue { .. } => "invalid-value",
        PresentationControlError::EmptyTargetKey
        | PresentationControlError::TargetKeyWhitespace
        | PresentationControlError::TargetKeyTooLong { .. }
        | PresentationControlError::TargetKeyControlCharacter => "invalid-target",
        PresentationControlError::DuplicateTarget { .. } => "duplicate-target",
        PresentationControlError::UnknownSourceName { .. }
        | PresentationControlError::AmbiguousSourceName { .. } => "source-name-resolution",
    }
}

fn rejected_invalid_request(message: String) -> PresentationCommandResponse {
    PresentationCommandResponse::Rejected {
        diagnostic: PresentationDiagnostic {
            code: "invalid-request",
            message,
        },
    }
}

fn serialize_command_response(response: PresentationCommandResponse) -> Result<String, JsValue> {
    serde_json::to_string(&response).map_err(|error| {
        JsValue::from_str(&format!(
            "presentation command result did not serialize: {error}"
        ))
    })
}

fn inspect(file_name: &str, bytes: &[u8]) -> Result<AssetObservation, String> {
    if bytes.is_empty() {
        return Err("the selected file is empty".into());
    }
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(format!(
            "input has {} bytes, exceeding the workbench limit of {}",
            bytes.len(),
            MAX_INPUT_BYTES
        ));
    }

    match classify(file_name) {
        "svg" => inspect_svg(file_name, bytes),
        "cgm" => inspect_cgm(file_name, bytes),
        "glb" => inspect_glb_asset(file_name, bytes),
        "gltf" => inspect_gltf_asset(file_name, bytes),
        "fbx" => inspect_fbx(file_name, bytes),
        "png" | "jpg" | "jpeg" | "bmp" => inspect_raster_image(file_name, bytes),
        _ => Err(
            "supported extensions are .svg, .cgm, .gltf, .glb, .fbx, .png, .jpg, .jpeg, and .bmp"
                .into(),
        ),
    }
}

fn classify(file_name: &str) -> &'static str {
    match file_name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("svg") => "svg",
        Some("cgm") => "cgm",
        Some("gltf") => "gltf",
        Some("glb") => "glb",
        Some("fbx") => "fbx",
        Some("png") => "png",
        Some("jpg") | Some("jpeg") => "jpeg",
        Some("bmp") => "bmp",
        _ => "unknown",
    }
}

/// Decodes a bounded raster asset in Rust/WASM and returns metadata only.
///
/// The browser receives a provider-neutral observation rather than raw decoded
/// pixels. Browser canvas preview and renderer texture upload remain distinct
/// future consumer claims, so the workbench cannot silently substitute a
/// browser-native image decoder for Tokimu's raster evidence.
fn inspect_raster_image(file_name: &str, bytes: &[u8]) -> Result<AssetObservation, String> {
    let format = classify(file_name);
    let image = match format {
        "png" => decode_png(bytes, RasterDecodeLimits::default()),
        "jpeg" => decode_jpeg(bytes, RasterDecodeLimits::default()),
        "bmp" => decode_bmp(bytes, RasterDecodeLimits::default()),
        _ => unreachable!("raster inspection only receives supported raster formats"),
    }
    .map_err(|error| error.to_string())?;

    Ok(raster_observation(file_name, format, bytes.len(), image))
}

fn raster_observation(
    file_name: &str,
    format: &str,
    byte_length: usize,
    image: DecodedImage,
) -> AssetObservation {
    let decoded_bytes = image.pixels.len();
    AssetObservation {
        schema: SCHEMA,
        file_name: file_name.to_owned(),
        format: format.to_owned(),
        status: "inspected".into(),
        byte_length,
        summary: format!(
            "Tokimu decoded a {}x{} {} raster image.",
            image.width,
            image.height,
            format.to_ascii_uppercase()
        ),
        properties: vec![
            property("Dimensions", format!("{} x {}", image.width, image.height)),
            property("Pixel format", pixel_format_name(image.pixel_format)),
            property("Decoded bytes", decoded_bytes),
            property("Color space", color_space_name(image.color_space)),
            property("Alpha", alpha_mode_name(image.alpha_mode)),
            property("Output orientation", orientation_name(image.output_orientation)),
            property("Pixel fingerprint", image.pixel_fingerprint()),
        ],
        diagnostics: vec![
            "Raster decoding ran inside the Tokimu WASM boundary; browser-native image decoding was not used."
                .into(),
            "Browser pixel preview and renderer texture upload are intentionally deferred as separate consumer claims."
                .into(),
        ],
        preview: None,
        presentation_targets: Vec::new(),
    }
}

fn pixel_format_name(format: PixelFormat) -> &'static str {
    match format {
        PixelFormat::Rgba8 => "RGBA8",
    }
}

fn color_space_name(color_space: ColorSpace) -> &'static str {
    match color_space {
        ColorSpace::Srgb => "sRGB",
        ColorSpace::Unspecified => "unspecified",
    }
}

fn alpha_mode_name(alpha_mode: AlphaMode) -> &'static str {
    match alpha_mode {
        AlphaMode::Opaque => "opaque",
        AlphaMode::Straight => "straight",
        AlphaMode::Unspecified => "unspecified",
    }
}

fn orientation_name(orientation: ImageOrientation) -> &'static str {
    match orientation {
        ImageOrientation::TopDown => "top-down",
        ImageOrientation::BottomUp => "bottom-up",
    }
}

fn source_presentation(color: [f32; 4]) -> SourcePresentation {
    SourcePresentation::new(
        PresentationColor::new(color[0], color[1], color[2])
            .expect("bounded preview colors should be valid"),
        color[3],
        true,
    )
    .expect("bounded preview opacity should be valid")
}

fn inspect_svg(file_name: &str, bytes: &[u8]) -> Result<AssetObservation, String> {
    let source =
        std::str::from_utf8(bytes).map_err(|error| format!("SVG is not UTF-8: {error}"))?;
    let records = parse_svg_document_vector_records_with_viewport(
        source,
        12,
        SvgViewportSource::DocumentViewBox,
        Default::default(),
    )
    .map_err(|error| error.to_string())?;
    let contour_count = records
        .iter()
        .map(|record| record.path.contours.len())
        .sum::<usize>();
    let point_count = records
        .iter()
        .flat_map(|record| &record.path.contours)
        .map(|contour| contour.points.len())
        .sum::<usize>();
    let paths = records
        .into_iter()
        .enumerate()
        .map(|(index, record)| {
            let color = svg_color(record.fill_color.or(record.stroke_color));
            (
                PreviewPath {
                    contours: preview_contours(&record.path),
                    fill: record.fill,
                    stroke: record.stroke,
                    color,
                    stroke_width: record.stroke_width,
                    target: Some(PreviewTarget {
                        kind: PresentationTargetKind::VectorRecord,
                        key: format!("record/{index}"),
                    }),
                },
                PresentationTargetObservation::from_descriptor(
                    PresentationTargetDescriptor::new(
                        PresentationTargetId::new(
                            PresentationTargetKind::VectorRecord,
                            format!("record/{index}"),
                        )
                        .expect("bounded SVG record target should be valid"),
                    ),
                    source_presentation(color),
                ),
            )
        })
        .collect::<Vec<_>>();
    let presentation_targets = paths.iter().map(|(_, target)| target.clone()).collect();
    let paths = paths.into_iter().map(|(path, _)| path).collect::<Vec<_>>();

    Ok(AssetObservation {
        schema: SCHEMA,
        file_name: file_name.into(),
        format: "svg".into(),
        status: "renderable".into(),
        byte_length: bytes.len(),
        summary: format!("Tokimu lowered {} SVG vector records.", paths.len()),
        properties: vec![
            property("Vector records", paths.len()),
            property("Contours", contour_count),
            property("Flattened points", point_count),
        ],
        diagnostics: Vec::new(),
        preview: Some(VectorPreview {
            kind: "vector-contours".into(),
            paths,
            triangles: Vec::new(),
        }),
        presentation_targets,
    })
}

fn inspect_cgm(file_name: &str, bytes: &[u8]) -> Result<AssetObservation, String> {
    let inspection =
        inspect_binary_cgm(bytes, CgmDecodeLimits::default()).map_err(|error| error.to_string())?;
    let preview_paths = inspection
        .pictures
        .first()
        .map(lower_picture_primitives)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, primitive)| {
            let color = [0.53, 0.84, 0.78, 1.0];
            (
                PreviewPath {
                    contours: preview_contours(&primitive.path),
                    // CGM contour closure records topology, not an admitted fill intent.
                    // Until CGM attributes drive a provider-neutral paint contract, this
                    // consumer intentionally presents all lowered CGM primitives as
                    // diagnostic outlines.
                    fill: false,
                    stroke: true,
                    color,
                    stroke_width: CGM_DIAGNOSTIC_STROKE_WIDTH,
                    target: Some(PreviewTarget {
                        kind: PresentationTargetKind::VectorRecord,
                        key: format!("picture/0/primitive/{index}"),
                    }),
                },
                PresentationTargetObservation::from_descriptor(
                    PresentationTargetDescriptor::new(
                        PresentationTargetId::new(
                            PresentationTargetKind::VectorRecord,
                            format!("picture/0/primitive/{index}"),
                        )
                        .expect("bounded CGM primitive target should be valid"),
                    ),
                    source_presentation(color),
                ),
            )
        })
        .collect::<Vec<_>>();
    let presentation_targets = preview_paths
        .iter()
        .map(|(_, target)| target.clone())
        .collect();
    let preview_paths = preview_paths
        .into_iter()
        .map(|(path, _)| path)
        .collect::<Vec<_>>();
    let deferred_features = summarize_diagnostics(&inspection.diagnostics);
    let text_record_count = inspection
        .pictures
        .iter()
        .map(|picture| picture.text_records.len())
        .sum::<usize>();
    let cell_array_count = inspection
        .pictures
        .iter()
        .map(|picture| picture.cell_arrays.len())
        .sum::<usize>();
    let diagnostics = cgm_observation_diagnostics(
        &inspection,
        &deferred_features,
        text_record_count,
        cell_array_count,
        !preview_paths.is_empty(),
    );

    Ok(AssetObservation {
        schema: SCHEMA,
        file_name: file_name.into(),
        format: "cgm".into(),
        status: if preview_paths.is_empty() {
            "inspected".into()
        } else {
            "previewable".into()
        },
        byte_length: bytes.len(),
        summary: format!(
            "Tokimu inspected {} elements across {} CGM pictures.",
            inspection.elements.len(),
            inspection.pictures.len()
        ),
        properties: vec![
            property("Metafile", inspection.metafile_name),
            property("Elements", inspection.elements.len()),
            property("Pictures", inspection.pictures.len()),
            property("Preview primitives", preview_paths.len()),
            property("Text source records", text_record_count),
            property("Cell-array source records", cell_array_count),
            property(
                "Deferred elements",
                format!(
                    "{} across {} feature kinds",
                    inspection.diagnostics.len(),
                    deferred_features.len()
                ),
            ),
        ],
        diagnostics,
        preview: (!preview_paths.is_empty()).then_some(VectorPreview {
            kind: "vector-contours".into(),
            paths: preview_paths,
            triangles: Vec::new(),
        }),
        presentation_targets,
    })
}

fn cgm_observation_diagnostics(
    inspection: &CgmInspection,
    deferred_features: &[CgmDeferredFeature],
    text_record_count: usize,
    cell_array_count: usize,
    has_preview: bool,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if has_preview {
        diagnostics.push(
            "CGM preview is diagnostic outline geometry; CGM fill, edge, and color semantics are deferred."
                .into(),
        );
    }

    if text_record_count > 0 {
        diagnostics.push(format!(
            "CGM retained {text_record_count} text source record{}; text layout and rendering are deferred.",
            if text_record_count == 1 { "" } else { "s" }
        ));
    }

    if cell_array_count > 0 {
        diagnostics.push(format!(
            "CGM retained {cell_array_count} cell-array source record{}; raster decode and texture presentation are deferred.",
            if cell_array_count == 1 { "" } else { "s" }
        ));
    }

    if deferred_features.is_empty() {
        return diagnostics;
    }

    diagnostics.push(format!(
        "{} source elements remain deferred across {} CGM feature kinds; the corpus artifacts retain each occurrence.",
        inspection.diagnostics.len(),
        deferred_features.len()
    ));
    let mut summaries = deferred_features.to_vec();
    summaries.sort_by_key(|feature| (std::cmp::Reverse(feature.count), feature.class, feature.id));

    const MAX_FEATURE_SUMMARIES: usize = 8;
    for feature in summaries.iter().take(MAX_FEATURE_SUMMARIES) {
        diagnostics.push(format!(
            "{}: {} occurrence{} deferred.",
            feature.feature,
            feature.count,
            if feature.count == 1 { "" } else { "s" }
        ));
    }
    let remaining = summaries.len().saturating_sub(MAX_FEATURE_SUMMARIES);
    if remaining > 0 {
        diagnostics.push(format!(
            "{remaining} additional CGM feature kind{} remain deferred.",
            if remaining == 1 { "" } else { "s" }
        ));
    }
    diagnostics
}

fn inspect_glb_asset(file_name: &str, bytes: &[u8]) -> Result<AssetObservation, String> {
    let inspection = inspect_glb(bytes).map_err(|error| error.to_string())?;
    let decoded = decode_glb(bytes).map_err(|error| error.to_string())?;
    let triangles = preview_triangles(&decoded.primitives, &decoded.nodes, &decoded.scenes);
    let presentation_targets = mesh_presentation_targets(&decoded.primitives);
    let mut observation = model_observation(
        file_name,
        "glb",
        bytes.len(),
        &inspection.summary,
        vec![property("Container chunks", inspection.chunks.len())],
    );
    if triangles.is_empty() {
        observation
            .diagnostics
            .push("No triangle primitives were admitted for browser preview.".into());
    } else {
        observation.status = "renderable".into();
        observation.summary = format!(
            "Tokimu decoded {} GLB triangles into a provider-neutral scene preview.",
            triangles.len()
        );
        observation
            .properties
            .push(property("Preview triangles", triangles.len()));
        observation.preview = Some(VectorPreview {
            kind: "mesh-triangles".into(),
            paths: Vec::new(),
            triangles,
        });
        observation.presentation_targets = presentation_targets;
        observation.diagnostics = vec![
            "Browser preview is an interactive diagnostic perspective view of Tokimu-decoded scene geometry. It applies browser-side projection, depth ordering, and back-face culling; materials, lighting, textures, and animation are pending.".into(),
        ];
    }
    Ok(observation)
}

fn mesh_presentation_targets(
    primitives: &[gltf_corpus::DecodedPrimitive],
) -> Vec<PresentationTargetObservation> {
    primitives
        .iter()
        .map(|primitive| {
            let location = primitive.location;
            let key = format!("mesh/{}/primitive/{}", location.mesh, location.primitive);
            let source_name = format!("Mesh {} / Primitive {}", location.mesh, location.primitive);
            let descriptor = PresentationTargetDescriptor::new(
                PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, key)
                    .expect("decoded GLB primitive target should be valid"),
            )
            .with_source_name(source_name)
            .expect("decoded GLB primitive source name should be valid");
            PresentationTargetObservation::from_descriptor(
                descriptor,
                source_presentation([0.53, 0.84, 0.78, 1.0]),
            )
        })
        .collect()
}

fn inspect_gltf_asset(file_name: &str, bytes: &[u8]) -> Result<AssetObservation, String> {
    let inspection = inspect_gltf(bytes).map_err(|error| error.to_string())?;
    Ok(model_observation(
        file_name,
        "gltf",
        bytes.len(),
        &inspection.summary,
        vec![
            property("Materials", inspection.materials.len()),
            property("Images", inspection.images.len()),
        ],
    ))
}

/// Produces the same bounded mesh observation as GLB, but resolves a JSON glTF
/// document's same-folder buffers and image sources through resource-space.
fn inspect_gltf_resource_asset(
    space: &InMemoryResourceSpace,
    folder: FolderId,
    document: &ResourceEntry,
) -> Result<AssetObservation, String> {
    let inspection = inspect_gltf(document.bytes()).map_err(|error| error.to_string())?;
    let decoded = decode_gltf_from_resource_space(space, folder, document)
        .map_err(|error| error.to_string())?;
    let images = resolve_gltf_external_images_from_resource_space(space, folder, document)
        .map_err(|error| error.to_string())?;
    let triangles = preview_triangles(&decoded.primitives, &decoded.nodes, &decoded.scenes);
    let presentation_targets = mesh_presentation_targets(&decoded.primitives);
    let mut observation = model_observation(
        document.name().as_str(),
        "gltf",
        document.bytes().len(),
        &decoded.summary,
        vec![
            property("Resolved external images", images.len()),
            property("Resolved external buffers", inspection.buffers.len()),
        ],
    );
    if triangles.is_empty() {
        observation.diagnostics.push(
            "No triangle primitives were admitted for browser preview after logical dependency resolution."
                .into(),
        );
    } else {
        observation.status = "renderable".into();
        observation.summary = format!(
            "Tokimu resolved {} same-folder dependencies and decoded {} glTF triangles into a provider-neutral scene preview.",
            images.len() + inspection.buffers.len(),
            triangles.len()
        );
        observation
            .properties
            .push(property("Preview triangles", triangles.len()));
        observation.preview = Some(VectorPreview {
            kind: "mesh-triangles".into(),
            paths: Vec::new(),
            triangles,
        });
        observation.presentation_targets = presentation_targets;
        observation.diagnostics = vec![
            "Browser preview is an interactive diagnostic perspective view of Tokimu-decoded scene geometry. The WASM resource session resolves same-folder external buffers and image references; image decoding, textures, materials, lighting, and animation remain separate capabilities."
                .into(),
        ];
    }
    Ok(observation)
}

fn model_observation(
    file_name: &str,
    format: &'static str,
    byte_length: usize,
    summary: &GltfSummary,
    mut properties: Vec<Property>,
) -> AssetObservation {
    properties.extend([
        property("Scenes", summary.scenes),
        property("Nodes", summary.nodes),
        property("Meshes", summary.meshes),
        property("Primitives", summary.primitives),
        property("Animations", summary.animations),
    ]);
    AssetObservation {
        schema: SCHEMA,
        file_name: file_name.into(),
        format: format.into(),
        status: "inspected".into(),
        byte_length,
        summary: "Tokimu recognized the model structure; browser mesh rendering is pending.".into(),
        properties,
        diagnostics: vec![
            "Rendering is intentionally deferred until a provider-neutral scene/mesh consumer boundary is selected.".into(),
        ],
        preview: None,
        presentation_targets: Vec::new(),
    }
}

fn preview_triangles(
    primitives: &[gltf_corpus::DecodedPrimitive],
    nodes: &[gltf_corpus::DecodedNode],
    scenes: &[gltf_corpus::DecodedScene],
) -> Vec<PreviewTriangle> {
    const MAX_PREVIEW_TRIANGLES: usize = 20_000;
    let instances = scenes
        .first()
        .into_iter()
        .flat_map(|scene| scene.traversal.iter())
        .filter_map(|scene_node| {
            nodes
                .get(scene_node.node)
                .and_then(|node| node.mesh.map(|mesh| (mesh, scene_node.world_transform)))
        })
        .collect::<Vec<_>>();

    primitives
        .iter()
        .flat_map(|primitive| {
            let target = PreviewTarget {
                kind: PresentationTargetKind::MeshPrimitive,
                key: format!(
                    "mesh/{}/primitive/{}",
                    primitive.location.mesh, primitive.location.primitive
                ),
            };
            let transform = instances
                .iter()
                // A mesh can be instanced more than once. This bounded preview shows
                // the first scene instance until a real scene-instance contract exists.
                .find_map(|(mesh, transform)| {
                    (*mesh == primitive.location.mesh).then_some(*transform)
                })
                .unwrap_or_else(identity_transform);
            primitive
                .indices
                .chunks_exact(3)
                .filter_map(move |indices| {
                    let points = [
                        primitive.positions[indices[0] as usize],
                        primitive.positions[indices[1] as usize],
                        primitive.positions[indices[2] as usize],
                    ];
                    let points = points.map(|point| transform_point(transform, point));
                    points
                        .iter()
                        .all(|point| point.iter().all(|value| value.is_finite()))
                        .then_some(PreviewTriangle {
                            points,
                            target: Some(target.clone()),
                        })
                })
        })
        .take(MAX_PREVIEW_TRIANGLES)
        .collect()
}

fn identity_transform() -> TransformMatrix {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn transform_point(transform: TransformMatrix, point: [f32; 3]) -> [f32; 3] {
    [
        transform[0] * point[0] + transform[4] * point[1] + transform[8] * point[2] + transform[12],
        transform[1] * point[0] + transform[5] * point[1] + transform[9] * point[2] + transform[13],
        transform[2] * point[0]
            + transform[6] * point[1]
            + transform[10] * point[2]
            + transform[14],
    ]
}

fn inspect_fbx(file_name: &str, bytes: &[u8]) -> Result<AssetObservation, String> {
    let limits = FbxLimits::default();
    let (encoding, version, records, geometry) = if bytes.starts_with(b"Kaydara FBX Binary") {
        let document = decode_binary_fbx(bytes, limits).map_err(|error| error.to_string())?;
        let scene = resolve_source_scene(&document).map_err(|error| error.to_string())?;
        let geometry =
            lower_static_geometry(&document, &scene).map_err(|error| error.to_string())?;
        ("binary", document.version, document.records.len(), geometry)
    } else {
        let document = decode_ascii_fbx(bytes, limits).map_err(|error| error.to_string())?;
        let scene = resolve_source_scene(&document).map_err(|error| error.to_string())?;
        let geometry =
            lower_static_geometry(&document, &scene).map_err(|error| error.to_string())?;
        ("ascii", document.version, document.records.len(), geometry)
    };
    let triangles = fbx_preview_triangles(&geometry);

    if !triangles.is_empty() {
        return Ok(AssetObservation {
            schema: SCHEMA,
            file_name: file_name.into(),
            format: "fbx".into(),
            status: "renderable".into(),
            byte_length: bytes.len(),
            summary: format!(
                "Tokimu lowered {} static FBX triangles into a provider-neutral scene preview.",
                triangles.len()
            ),
            properties: vec![
                property("Encoding", encoding),
                property("Version", version),
                property("Root records", records),
                property("Static meshes", geometry.meshes.len()),
                property("Preview triangles", triangles.len()),
            ],
            diagnostics: vec![
                "Browser preview is an interactive diagnostic perspective view of Tokimu-lowered static FBX geometry. FBX model transforms, materials, textures, skinning, morphs, and animation remain pending.".into(),
            ],
            preview: Some(VectorPreview {
                kind: "mesh-triangles".into(),
                paths: Vec::new(),
                triangles,
            }),
            presentation_targets: Vec::new(),
        });
    }

    Ok(AssetObservation {
        schema: SCHEMA,
        file_name: file_name.into(),
        format: "fbx".into(),
        status: "inspected".into(),
        byte_length: bytes.len(),
        summary: "Tokimu decoded the bounded FBX record graph, but no static triangle geometry was admitted for browser preview.".into(),
        properties: vec![
            property("Encoding", encoding),
            property("Version", version),
            property("Root records", records),
        ],
        diagnostics: vec![
            "Provider-native FBX records remain below the WASM boundary by design; static triangle lowering requires admissible geometry and source-scene links.".into(),
        ],
        preview: None,
        presentation_targets: Vec::new(),
    })
}

fn fbx_preview_triangles(geometry: &FbxGeometryEvidence) -> Vec<PreviewTriangle> {
    const MAX_PREVIEW_TRIANGLES: usize = 20_000;
    geometry
        .meshes
        .iter()
        .flat_map(|mesh| {
            mesh.triangles.iter().filter_map(|indices| {
                let points = indices.map(|index| mesh.control_points.get(index as usize).copied());
                let [Some(a), Some(b), Some(c)] = points else {
                    return None;
                };
                let points =
                    [a, b, c].map(|point| [point[0] as f32, point[1] as f32, point[2] as f32]);
                points
                    .iter()
                    .all(|point| point.iter().all(|value| value.is_finite()))
                    .then_some(PreviewTriangle {
                        points,
                        target: None,
                    })
            })
        })
        .take(MAX_PREVIEW_TRIANGLES)
        .collect()
}

fn preview_contours(path: &VectorPath) -> Vec<PreviewContour> {
    path.contours
        .iter()
        .map(|contour| PreviewContour {
            points: contour.points.clone(),
            closed: contour.closed,
        })
        .collect()
}

fn svg_color(color: Option<SvgColor>) -> [f32; 4] {
    match color {
        Some(SvgColor::Rgba(rgba)) => rgba,
        None => [0.64, 0.82, 0.98, 1.0],
    }
}

fn property(label: impl Into<String>, value: impl ToString) -> Property {
    Property {
        label: label.into(),
        value: value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_admitted_extensions_without_case_sensitivity() {
        assert_eq!(classify("shape.SVG"), "svg");
        assert_eq!(classify("part.glb"), "glb");
        assert_eq!(classify("texture.PNG"), "png");
        assert_eq!(classify("photo.JPEG"), "jpeg");
        assert_eq!(classify("bitmap.bmp"), "bmp");
        assert_eq!(classify("unknown.bin"), "unknown");
    }

    #[test]
    fn resource_session_resolves_selected_gltf_buffer_inside_wasm() {
        let document = include_bytes!(
            "../../../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF/Box.gltf"
        );
        let buffer = include_bytes!(
            "../../../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF/Box0.bin"
        );
        let mut session = ResourceSession::new().expect("resource session should initialize");
        session
            .add_resource("Box.gltf", document)
            .expect("document should be retained");
        session
            .add_resource("Box0.bin", buffer)
            .expect("selected buffer should be retained");

        let json = session
            .inspect_resource("Box.gltf")
            .expect("same-folder buffer should resolve in Rust/WASM");
        let observation: AssetObservation =
            serde_json::from_str(&json).expect("observation should remain serializable");

        assert_eq!(observation.format, "gltf");
        assert_eq!(observation.status, "renderable");
        assert!(observation.preview.is_some());
        assert!(
            observation
                .properties
                .iter()
                .any(|property| property.label == "Resolved external buffers"
                    && property.value == "1")
        );
        assert!(session.summary().contains("resources=2"));
    }

    #[test]
    fn resource_session_rejects_entries_beyond_its_explicit_selection_budget() {
        let mut session = ResourceSession::new().expect("resource session should initialize");
        for index in 0..MAX_RESOURCE_SESSION_ENTRIES {
            session
                .add_resource(&format!("selected-{index}.bin"), &[index as u8])
                .expect("entry within the selection limit should be retained");
        }

        let error = session
            .add_resource_inner("over-limit.bin", &[0])
            .expect_err("the next selected entry should exceed the session limit");

        assert!(error.contains("entry limit"));
        assert!(session
            .summary()
            .contains(&format!("resources={MAX_RESOURCE_SESSION_ENTRIES}")));
    }

    #[test]
    fn resource_session_returns_selected_bytes_without_exposing_host_paths() {
        let mut session = ResourceSession::new().expect("resource session should initialize");
        session
            .add_resource("nested-name-is-rejected.bin", &[0xCA, 0xFE, 0xBA, 0xBE])
            .expect("logical resource should be retained");

        assert_eq!(
            session
                .resource_bytes_inner("nested-name-is-rejected.bin")
                .expect("selected bytes should be readable"),
            vec![0xCA, 0xFE, 0xBA, 0xBE]
        );
        assert!(session
            .resource_bytes_inner("C:/host-path.bin")
            .expect_err("host path syntax must not resolve")
            .contains("same-folder logical file name"));
    }

    #[test]
    fn repeated_browser_reads_do_not_change_session_retention() {
        let mut session = ResourceSession::new().expect("resource session should initialize");
        session
            .add_resource("selected.bin", &[0xCA, 0xFE, 0xBA, 0xBE])
            .expect("logical resource should be retained");
        let summary_before = session.summary();

        for _ in 0..64 {
            assert_eq!(
                session
                    .resource_bytes_inner("selected.bin")
                    .expect("selected bytes should remain readable"),
                vec![0xCA, 0xFE, 0xBA, 0xBE]
            );
        }

        assert_eq!(session.summary(), summary_before);
        assert!(summary_before.contains("resources=1"));
        assert!(summary_before.contains("retained_bytes=4"));
    }

    #[test]
    fn png_observation_decodes_inside_the_wasm_boundary_without_browser_preview() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn2c08.png"
        );
        let observation = inspect("basn2c08.png", source).expect("PNG fixture should decode");

        assert_eq!(observation.format, "png");
        assert_eq!(observation.status, "inspected");
        assert!(observation.preview.is_none());
        assert!(observation.presentation_targets.is_empty());
        assert!(observation
            .properties
            .iter()
            .any(|property| property.label == "Pixel fingerprint"));
        assert!(observation
            .diagnostics
            .iter()
            .any(|message| message.contains("WASM boundary")));
        assert!(observation
            .diagnostics
            .iter()
            .any(|message| message.contains("Browser pixel preview")));
    }

    #[test]
    fn bmp_observation_preserves_raster_metadata_without_texture_claims() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp"
        );
        let observation = inspect("shira_bird8.bmp", source).expect("BMP fixture should decode");

        assert_eq!(observation.format, "bmp");
        assert_eq!(observation.status, "inspected");
        assert!(observation.preview.is_none());
        assert!(
            observation
                .properties
                .iter()
                .any(|property| property.label == "Output orientation"
                    && property.value == "top-down")
        );
    }

    #[test]
    fn jpeg_observation_uses_the_same_bounded_wasm_contract_as_other_rasters() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testimgint.jpg"
        );
        let observation =
            inspect("testimgint.jpg", source).expect("baseline JPEG fixture should decode");

        assert_eq!(observation.format, "jpeg");
        assert_eq!(observation.status, "inspected");
        assert!(observation.preview.is_none());
        assert!(observation.presentation_targets.is_empty());
        assert!(observation
            .properties
            .iter()
            .any(|property| property.label == "Pixel format" && property.value == "RGBA8"));
        assert!(observation
            .diagnostics
            .iter()
            .any(|message| message.contains("browser-native image decoding was not used")));
    }

    #[test]
    fn malformed_jpeg_is_rejected_by_tokimu_before_any_browser_presentation_can_occur() {
        let error = match inspect("truncated.jpg", &[0xFF, 0xD8, 0xFF]) {
            Ok(_) => panic!("truncated JPEG framing must not produce an observation"),
            Err(error) => error,
        };

        assert!(
            error.contains("truncated") || error.contains("marker"),
            "unexpected JPEG diagnostic: {error}"
        );
    }

    #[test]
    fn rejects_empty_input_before_provider_dispatch() {
        let error = match inspect("empty.svg", &[]) {
            Ok(_) => panic!("empty input should fail"),
            Err(error) => error,
        };
        assert!(error.contains("empty"));
    }

    #[test]
    fn svg_observation_contains_preview_contours() {
        let source = br#"<svg viewBox="0 0 10 10"><rect x="1" y="1" width="8" height="8"/></svg>"#;
        let observation = inspect("box.svg", source).expect("SVG should inspect");
        assert_eq!(observation.status, "renderable");
        assert!(observation.preview.is_some());
    }

    #[test]
    fn box_glb_produces_a_bounded_triangle_preview() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb"
        );
        let observation = inspect("Box.glb", source).expect("Box fixture should decode");
        assert_eq!(observation.status, "renderable");
        let preview = observation.preview.expect("Box should provide a preview");
        assert_eq!(preview.kind, "mesh-triangles");
        assert!(!preview.triangles.is_empty());
        assert_eq!(observation.presentation_targets.len(), 1);
        let target = &observation.presentation_targets[0];
        assert_eq!(target.kind, PresentationTargetKind::MeshPrimitive);
        assert_eq!(target.key, "mesh/0/primitive/0");
        assert_eq!(target.source_name.as_deref(), Some("Mesh 0 / Primitive 0"));
    }

    #[test]
    fn identical_glb_bytes_advertise_stable_presentation_target_ids() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb"
        );
        let first = inspect("Box.glb", source).expect("first Box fixture should decode");
        let second = inspect("Box.glb", source).expect("second Box fixture should decode");
        let first_ids = first
            .presentation_targets
            .iter()
            .map(|target| (target.kind, target.key.as_str()))
            .collect::<Vec<_>>();
        let second_ids = second
            .presentation_targets
            .iter()
            .map(|target| (target.kind, target.key.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(first_ids, second_ids);
        assert_eq!(
            first_ids,
            vec![(PresentationTargetKind::MeshPrimitive, "mesh/0/primitive/0")]
        );
    }

    #[test]
    fn wasm_presentation_session_resolves_glb_target_without_frontend_parsing() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb"
        );
        let observation = inspect("Box.glb", source).expect("Box fixture should decode");
        let observation_json = serde_json::to_string(&observation).unwrap();
        let mut session = PresentationSession::new(&observation_json)
            .expect("observation targets should create a session");
        let request = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/0/primitive/0",
            "layer": "hotspot",
            "overrideValue": {
                "tint": { "color": { "red": 1.0, "green": 0.35, "blue": 0.1 }, "mode": "replace" },
                "opacityMultiplier": 0.45,
                "visible": true,
                "emphasis": "hotspot"
            }
        });

        let resolved = session
            .set_override(&request.to_string())
            .expect("bounded hotspot request should resolve");
        let resolved: serde_json::Value = serde_json::from_str(&resolved).unwrap();
        assert_eq!(resolved["status"], "resolved");
        assert_eq!(resolved["resolved"]["color"]["red"], 1.0);
        assert_eq!(resolved["resolved"]["color"]["green"], 0.35);
        assert_eq!(resolved["resolved"]["color"]["blue"], 0.1);
        assert_eq!(resolved["resolved"]["opacity"], 0.45);
        assert_eq!(resolved["resolved"]["emphasis"], "hotspot");

        let clear = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/0/primitive/0",
            "layer": "hotspot"
        });
        let restored = session
            .clear_override(&clear.to_string())
            .expect("bounded restore request should resolve");
        let restored: serde_json::Value = serde_json::from_str(&restored).unwrap();
        assert_eq!(restored["status"], "resolved");
        assert_eq!(restored["resolved"]["color"]["red"], 0.53);
        assert_eq!(restored["resolved"]["color"]["green"], 0.84);
        assert_eq!(restored["resolved"]["color"]["blue"], 0.78);
        assert_eq!(restored["resolved"]["opacity"], 1.0);
        assert!(restored["resolved"]["emphasis"].is_null());

        let unknown = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/99/primitive/99",
            "layer": "application"
        });
        let rejected = session
            .clear_override(&unknown.to_string())
            .expect("unknown target should produce a structured response");
        let rejected: serde_json::Value = serde_json::from_str(&rejected).unwrap();
        assert_eq!(rejected["status"], "rejected");
        assert_eq!(rejected["diagnostic"]["code"], "unknown-target");

        let invalid_value = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/0/primitive/0",
            "layer": "application",
            "overrideValue": {
                "opacityMultiplier": 1.5
            }
        });
        let rejected = session
            .set_override(&invalid_value.to_string())
            .expect("invalid value should produce a structured response");
        let rejected: serde_json::Value = serde_json::from_str(&rejected).unwrap();
        assert_eq!(rejected["status"], "rejected");
        assert_eq!(rejected["diagnostic"]["code"], "invalid-value");
    }

    #[test]
    fn wasm_reset_of_application_layer_preserves_independent_hotspot_layer() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb"
        );
        let observation = inspect("Box.glb", source).expect("Box fixture should decode");
        let observation_json = serde_json::to_string(&observation).unwrap();
        let mut session = PresentationSession::new(&observation_json)
            .expect("observation targets should create a session");

        let application = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/0/primitive/0",
            "layer": "application",
            "overrideValue": {
                "tint": { "color": { "red": 0.2, "green": 0.4, "blue": 0.9 }, "mode": "replace" },
                "emphasis": "selected"
            }
        });
        session
            .set_override(&application.to_string())
            .expect("application presentation should resolve");

        let hotspot = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/0/primitive/0",
            "layer": "hotspot",
            "overrideValue": {
                "tint": { "color": { "red": 1.0, "green": 0.35, "blue": 0.1 }, "mode": "replace" },
                "emphasis": "hotspot"
            }
        });
        session
            .set_override(&hotspot.to_string())
            .expect("hotspot presentation should resolve");

        let clear_application = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/0/primitive/0",
            "layer": "application"
        });
        let resolved = session
            .clear_override(&clear_application.to_string())
            .expect("application reset should resolve the remaining hotspot layer");
        let resolved: serde_json::Value = serde_json::from_str(&resolved).unwrap();

        assert_eq!(resolved["status"], "resolved");
        assert_eq!(resolved["resolved"]["color"]["red"], 1.0);
        assert_eq!(resolved["resolved"]["color"]["green"], 0.35);
        assert_eq!(resolved["resolved"]["color"]["blue"], 0.1);
        assert_eq!(resolved["resolved"]["emphasis"], "hotspot");
    }

    #[test]
    fn wasm_command_resolution_matches_direct_native_presentation_resolution() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb"
        );
        let observation = inspect("Box.glb", source).expect("Box fixture should decode");
        let target_observation = observation
            .presentation_targets
            .first()
            .expect("Box fixture should expose one presentation target");
        let request = serde_json::json!({
            "kind": "mesh-primitive",
            "key": "mesh/0/primitive/0",
            "layer": "hotspot",
            "overrideValue": {
                "tint": { "color": { "red": 1.0, "green": 0.35, "blue": 0.1 }, "mode": "replace" },
                "opacityMultiplier": 0.45,
                "visible": true,
                "emphasis": "hotspot"
            }
        });
        let command: PresentationOverrideRequest =
            serde_json::from_value(request.clone()).expect("fixture request should deserialize");
        let target = PresentationTargetId::new(command.kind, command.key.clone())
            .expect("fixture target should remain valid");
        let mut native_control = PresentationControl::default();
        native_control
            .register_target_with_descriptor(
                target_observation
                    .descriptor()
                    .expect("observation target should remain valid"),
                target_observation.source,
            )
            .expect("native target registration should succeed");
        native_control
            .set_override(&target, command.layer, command.override_value)
            .expect("native override should resolve");
        let native_resolved = native_control
            .resolve(&target)
            .expect("native target should resolve");

        let observation_json = serde_json::to_string(&observation).unwrap();
        let mut wasm_session =
            PresentationSession::new(&observation_json).expect("session should construct");
        let wasm_response = wasm_session
            .set_override(&request.to_string())
            .expect("WASM command boundary should resolve");
        let wasm_response: serde_json::Value = serde_json::from_str(&wasm_response).unwrap();
        let wasm_resolved: ResolvedPresentation =
            serde_json::from_value(wasm_response["resolved"].clone())
                .expect("resolved WASM response should preserve the provider-neutral contract");

        assert_eq!(wasm_response["status"], "resolved");
        assert_eq!(
            wasm_resolved, native_resolved,
            "the WASM command boundary must preserve native resolved presentation semantics"
        );
    }

    #[test]
    fn maya_cube_fbx_produces_a_bounded_static_triangle_preview() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/fbx-corpus/upstream/data/maya_cube_7500_binary.fbx"
        );
        let observation =
            inspect("maya_cube_7500_binary.fbx", source).expect("Maya cube fixture should decode");
        assert_eq!(observation.status, "renderable");
        let preview = observation
            .preview
            .expect("FBX cube should provide a preview");
        assert_eq!(preview.kind, "mesh-triangles");
        assert!(!preview.triangles.is_empty());
        assert!(observation
            .diagnostics
            .iter()
            .any(|message| message.contains("static FBX geometry")));
    }

    #[test]
    fn polyln01_cgm_preview_stays_outline_only_until_fill_semantics_are_admitted() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/webcgm-test-suite/upstream/static10/POLYLN01.cgm"
        );
        let observation = inspect("POLYLN01.cgm", source).expect("CGM fixture should inspect");
        assert_eq!(observation.status, "previewable");
        let preview = observation
            .preview
            .expect("CGM should provide an outline preview");
        assert!(!preview.paths.is_empty());
        assert!(preview.paths.iter().all(|path| !path.fill && path.stroke));
        assert!(preview
            .paths
            .iter()
            .all(|path| path.stroke_width == CGM_DIAGNOSTIC_STROKE_WIDTH));
        assert!(observation
            .diagnostics
            .iter()
            .any(|message| message.contains("diagnostic outline geometry")));
        assert!(observation
            .diagnostics
            .iter()
            .any(|message| message.starts_with("CGM retained ")));
        assert!(!observation
            .diagnostics
            .iter()
            .any(|message| message.contains("not decoded by the lifecycle profile")));
    }

    #[test]
    fn celary01_cgm_retains_raster_source_metadata_without_claiming_texture_output() {
        let source = include_bytes!(
            "../../../../../third-party/fixtures/webcgm-test-suite/upstream/static10/CELARY01.cgm"
        );
        let observation =
            inspect("CELARY01.cgm", source).expect("cell-array fixture should inspect");
        assert!(observation.properties.iter().any(|property| {
            property.label == "Cell-array source records" && property.value == "1"
        }));
        assert!(
            observation
                .diagnostics
                .iter()
                .any(|message| message
                    .contains("raster decode and texture presentation are deferred"))
        );
        assert!(!observation
            .diagnostics
            .iter()
            .any(|message| message.contains("CGM cell array raster primitive")));
    }
}
