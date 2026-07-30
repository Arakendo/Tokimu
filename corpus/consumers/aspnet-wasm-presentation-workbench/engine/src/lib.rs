use presentation_control::{
    PresentationColor, PresentationControl, PresentationControlError, PresentationLayer,
    PresentationOverride, PresentationTargetDescriptor, PresentationTargetId,
    PresentationTargetKind, ResolvedPresentation, SourcePresentation,
};
use serde::{Deserialize, Serialize};
use tokimu::World;
use wasm_bindgen::prelude::*;

const SCHEMA: u32 = 1;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneObservation {
    schema: u32,
    summary: String,
    targets: Vec<PresentationTargetObservation>,
    shapes: Vec<SceneShape>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneShape {
    kind: PresentationTargetKind,
    key: String,
    label: String,
    geometry: ShapeGeometry,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ShapeGeometry {
    Polygon { points: Vec<[f32; 2]> },
    Circle { center: [f32; 2], radius: f32 },
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PresentationTargetObservation {
    kind: PresentationTargetKind,
    key: String,
    source_name: String,
    source: SourcePresentation,
}

impl PresentationTargetObservation {
    fn descriptor(&self) -> Result<PresentationTargetDescriptor, String> {
        PresentationTargetDescriptor::new(
            PresentationTargetId::new(self.kind, self.key.clone())
                .map_err(|error| error.to_string())?,
        )
        .with_source_name(self.source_name.clone())
        .map_err(|error| error.to_string())
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

/// Stateful command boundary for the browser consumer. It owns target
/// semantics, but no DOM, canvas, imported source, or backend-native state.
#[wasm_bindgen]
pub struct PresentationSession {
    control: PresentationControl,
}

#[wasm_bindgen]
pub fn engine_status() -> String {
    let mut world = World::default();
    let entity = world.spawn();
    format!("Tokimu WASM presentation bridge ready; public facade spawned {entity:?}")
}

#[wasm_bindgen]
pub fn presentation_scene() -> String {
    serde_json::to_string(&scene()).expect("fixed presentation scene should serialize")
}

#[wasm_bindgen]
impl PresentationSession {
    #[wasm_bindgen(constructor)]
    pub fn new(scene_json: &str) -> Result<Self, JsValue> {
        let scene = serde_json::from_str::<SceneInput>(scene_json)
            .map_err(|error| JsValue::from_str(&format!("invalid presentation scene: {error}")))?;
        let mut control = PresentationControl::default();
        for target in scene.targets {
            let descriptor = target
                .descriptor()
                .map_err(|error| JsValue::from_str(&error))?;
            control
                .register_target_with_descriptor(descriptor, target.source)
                .map_err(|error| JsValue::from_str(&error.to_string()))?;
        }
        Ok(Self { control })
    }

    pub fn set_override(&mut self, request_json: &str) -> Result<String, JsValue> {
        let response = match serde_json::from_str::<PresentationOverrideRequest>(request_json) {
            Ok(request) => match PresentationTargetId::new(request.kind, request.key) {
                Ok(target) => self
                    .control
                    .set_override(&target, request.layer, request.override_value)
                    .and_then(|()| self.control.resolve(&target))
                    .map(|resolved| PresentationCommandResponse::Resolved { resolved })
                    .unwrap_or_else(rejected_response),
                Err(error) => rejected_response(error),
            },
            Err(error) => rejected_invalid_request(error.to_string()),
        };
        serialize_response(response)
    }

    pub fn clear_override(&mut self, request_json: &str) -> Result<String, JsValue> {
        let response = match serde_json::from_str::<PresentationClearRequest>(request_json) {
            Ok(request) => match PresentationTargetId::new(request.kind, request.key) {
                Ok(target) => self
                    .control
                    .clear_override(&target, request.layer)
                    .and_then(|_| self.control.resolve(&target))
                    .map(|resolved| PresentationCommandResponse::Resolved { resolved })
                    .unwrap_or_else(rejected_response),
                Err(error) => rejected_response(error),
            },
            Err(error) => rejected_invalid_request(error.to_string()),
        };
        serialize_response(response)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SceneInput {
    targets: Vec<PresentationTargetObservation>,
}

fn scene() -> SceneObservation {
    let targets = vec![
        target(
            PresentationTargetKind::VectorRecord,
            "diagram/outline",
            "Vector outline",
            [0.20, 0.78, 0.72],
        ),
        target(
            PresentationTargetKind::MeshPrimitive,
            "machine/housing",
            "Machine housing",
            [0.42, 0.60, 0.88],
        ),
        target(
            PresentationTargetKind::MeshPrimitive,
            "machine/fastener",
            "Machine fastener",
            [0.95, 0.70, 0.24],
        ),
        target(
            PresentationTargetKind::Renderable,
            "hotspot/inspection",
            "Inspection hotspot",
            [0.84, 0.34, 0.28],
        ),
    ];
    let shapes = vec![
        polygon(
            PresentationTargetKind::VectorRecord,
            "diagram/outline",
            "Vector outline",
            vec![
                [0.10, 0.18],
                [0.36, 0.18],
                [0.42, 0.32],
                [0.36, 0.46],
                [0.10, 0.46],
                [0.16, 0.32],
            ],
        ),
        polygon(
            PresentationTargetKind::MeshPrimitive,
            "machine/housing",
            "Machine housing",
            vec![[0.50, 0.18], [0.78, 0.25], [0.78, 0.54], [0.50, 0.47]],
        ),
        polygon(
            PresentationTargetKind::MeshPrimitive,
            "machine/fastener",
            "Machine fastener",
            vec![[0.61, 0.34], [0.69, 0.36], [0.69, 0.52], [0.61, 0.50]],
        ),
        SceneShape {
            kind: PresentationTargetKind::Renderable,
            key: "hotspot/inspection".into(),
            label: "Inspection hotspot".into(),
            geometry: ShapeGeometry::Circle {
                center: [0.34, 0.70],
                radius: 0.10,
            },
        },
    ];
    SceneObservation { schema: SCHEMA, summary: "Fixed Tokimu-owned presentation scene: click a target, author bounded material intent, and inspect the Rust/WASM-resolved result.".into(), targets, shapes }
}

fn target(
    kind: PresentationTargetKind,
    key: &str,
    source_name: &str,
    color: [f32; 3],
) -> PresentationTargetObservation {
    PresentationTargetObservation {
        kind,
        key: key.into(),
        source_name: source_name.into(),
        source: SourcePresentation::new(
            PresentationColor::new(color[0], color[1], color[2]).expect("fixed color"),
            1.0,
            true,
        )
        .expect("fixed source presentation"),
    }
}

fn polygon(
    kind: PresentationTargetKind,
    key: &str,
    label: &str,
    points: Vec<[f32; 2]>,
) -> SceneShape {
    SceneShape {
        kind,
        key: key.into(),
        label: label.into(),
        geometry: ShapeGeometry::Polygon { points },
    }
}

fn rejected_response(error: PresentationControlError) -> PresentationCommandResponse {
    let code = match error {
        PresentationControlError::UnknownTarget { .. } => "unknown-target",
        PresentationControlError::InvalidUnitValue { .. } => "invalid-value",
        PresentationControlError::EmptyTargetKey
        | PresentationControlError::TargetKeyWhitespace
        | PresentationControlError::TargetKeyTooLong { .. }
        | PresentationControlError::TargetKeyControlCharacter => "invalid-target",
        PresentationControlError::DuplicateTarget { .. } => "duplicate-target",
        PresentationControlError::UnknownSourceName { .. }
        | PresentationControlError::AmbiguousSourceName { .. } => "source-name-resolution",
    };
    PresentationCommandResponse::Rejected {
        diagnostic: PresentationDiagnostic {
            code,
            message: error.to_string(),
        },
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

fn serialize_response(response: PresentationCommandResponse) -> Result<String, JsValue> {
    serde_json::to_string(&response).map_err(|error| {
        JsValue::from_str(&format!("presentation response did not serialize: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_scene_registers_unrelated_provider_neutral_targets() {
        let observation = scene();
        assert_eq!(observation.targets.len(), 4);
        assert!(observation
            .targets
            .iter()
            .any(|target| target.kind == PresentationTargetKind::VectorRecord));
        assert_eq!(
            observation
                .targets
                .iter()
                .filter(|target| target.kind == PresentationTargetKind::MeshPrimitive)
                .count(),
            2
        );
    }

    #[test]
    fn application_override_changes_only_the_selected_target() {
        let observation = scene();
        let mut control = PresentationControl::default();
        for target in &observation.targets {
            control
                .register_target_with_descriptor(target.descriptor().unwrap(), target.source)
                .unwrap();
        }
        let selected =
            PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, "machine/housing")
                .unwrap();
        let other =
            PresentationTargetId::new(PresentationTargetKind::VectorRecord, "diagram/outline")
                .unwrap();
        let override_value = PresentationOverride::default()
            .with_tint(presentation_control::PresentationTint::replace(
                PresentationColor::new(1.0, 0.2, 0.1).unwrap(),
            ))
            .with_opacity_multiplier(0.4)
            .unwrap();
        control
            .set_override(&selected, PresentationLayer::Application, override_value)
            .unwrap();

        assert_eq!(control.resolve(&selected).unwrap().opacity, 0.4);
        assert_eq!(control.resolve(&other).unwrap().opacity, 1.0);
    }
}
