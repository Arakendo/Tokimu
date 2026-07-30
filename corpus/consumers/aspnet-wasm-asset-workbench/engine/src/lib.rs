use cgm_corpus::{inspect_binary_cgm, lower_picture_primitives, CgmInspection, DecodeLimits};
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
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};
use tokimu::World;
use ui_tools::{
    parse_svg_document_vector_records_with_viewport, SvgColor, SvgViewportSource, VectorPath,
};
use wasm_bindgen::prelude::*;

const SCHEMA: u32 = 1;
const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
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
        _ => Err("supported extensions are .svg, .cgm, .gltf, .glb, and .fbx".into()),
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
        _ => "unknown",
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
        inspect_binary_cgm(bytes, DecodeLimits::default()).map_err(|error| error.to_string())?;
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
    let unsupported_element_kinds = inspection
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.class, diagnostic.id))
        .collect::<BTreeSet<_>>()
        .len();
    let diagnostics = cgm_observation_diagnostics(&inspection, !preview_paths.is_empty());

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
            property(
                "Deferred elements",
                format!(
                    "{} across {} feature kinds",
                    inspection.diagnostics.len(),
                    unsupported_element_kinds
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

fn cgm_observation_diagnostics(inspection: &CgmInspection, has_preview: bool) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if has_preview {
        diagnostics.push(
            "CGM preview is diagnostic outline geometry; CGM fill, edge, and color semantics are deferred."
                .into(),
        );
    }

    let mut unsupported = BTreeMap::<(u8, u8), usize>::new();
    for diagnostic in &inspection.diagnostics {
        *unsupported
            .entry((diagnostic.class, diagnostic.id))
            .or_default() += 1;
    }
    if unsupported.is_empty() {
        return diagnostics;
    }

    diagnostics.push(format!(
        "{} source elements remain deferred across {} CGM feature kinds; the corpus artifacts retain each occurrence.",
        inspection.diagnostics.len(),
        unsupported.len()
    ));
    let mut summaries = unsupported.into_iter().collect::<Vec<_>>();
    summaries.sort_by_key(|((class, id), count)| (Reverse(*count), *class, *id));

    const MAX_FEATURE_SUMMARIES: usize = 8;
    for ((class, id), count) in summaries.iter().take(MAX_FEATURE_SUMMARIES) {
        diagnostics.push(format!(
            "{}: {count} occurrence{} deferred.",
            cgm_element_name(*class, *id),
            if *count == 1 { "" } else { "s" }
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

fn cgm_element_name(class: u8, id: u8) -> String {
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
        (5, 15) => "character height",
        (5, 16) => "character orientation",
        (5, 18) => "text alignment",
        (5, 34) => "color table",
        _ => return format!("CGM class {class} element {id}"),
    };
    format!("CGM {name}")
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
        assert_eq!(classify("unknown.bin"), "unknown");
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
            .any(|message| message.starts_with("CGM text primitive: ")));
        assert!(!observation
            .diagnostics
            .iter()
            .any(|message| message.contains("not decoded by the lifecycle profile")));
    }
}
