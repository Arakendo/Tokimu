use hello_runtime_observation::{
    compare_observation_snapshots, verified_hole_punch_catalog_fixture, CommandRequest,
    ObservationComparisonConfig, ObservationDiffReport, ObservationEnvelope, ObservationLimits,
    PlaybackCommand, RuntimeInspectionAdapter,
};
use wasm_bindgen::prelude::*;

mod ui;

const MAX_PENDING_COMMANDS: usize = 16;

/// Browser-facing observation facade for the runtime corpus.
///
/// The browser receives owned JSON records and can submit semantic requests.
/// It neither receives a `World` nor parses source GLB data.
#[wasm_bindgen]
pub struct WasmRuntimeObservationSession {
    runtime: RuntimeInspectionAdapter,
    previous_observation: Option<ObservationEnvelope>,
    latest_observation_diff: Option<ObservationDiffReport>,
}

#[wasm_bindgen]
impl WasmRuntimeObservationSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        build_runtime()
            .map(|runtime| Self {
                runtime,
                previous_observation: None,
                latest_observation_diff: None,
            })
            .map_err(js_error)
    }

    /// Returns a bounded summary or selected-entity observation.
    pub fn observation_json(
        &mut self,
        sequence: u32,
        selected_entity: Option<u32>,
    ) -> Result<String, JsValue> {
        let observation = self.runtime.observe_entity_id(
            u64::from(sequence),
            selected_entity.map(u64::from),
            ObservationLimits::default(),
        );
        self.record_observation(observation.clone())?;
        json(observation).map_err(js_error)
    }

    /// Returns the comparison between the two most recent browser-visible
    /// observations. The first observation intentionally has no predecessor.
    pub fn latest_observation_diff_json(&self) -> Result<String, JsValue> {
        json(&self.latest_observation_diff).map_err(js_error)
    }

    /// Resolves the current observation into a provider-neutral semantic UI
    /// artifact. The browser receives evidence, not renderer resources or a
    /// second authoritative layout model.
    pub fn ui_snapshot_json(
        &self,
        width: u32,
        height: u32,
        sequence: u32,
        selected_entity: Option<u32>,
    ) -> Result<String, JsValue> {
        ui::build_runtime_ui_snapshot(
            &self.runtime,
            [width, height],
            u64::from(sequence),
            selected_entity.map(u64::from),
        )
        .and_then(json)
        .map_err(js_error)
    }

    /// Admits one application-owned command into the bounded queue. Command
    /// JSON is parsed by Rust and remains only a request until `apply_json`.
    pub fn enqueue_json(&mut self, request_json: &str) -> Result<String, JsValue> {
        let request = serde_json::from_str::<CommandRequest>(request_json)
            .map_err(|error| JsValue::from_str(&format!("invalid runtime command: {error}")))?;
        json(self.runtime.enqueue(request)).map_err(js_error)
    }

    /// Applies the FIFO command queue at the caller-selected lifecycle tick.
    pub fn apply_json(&mut self, tick: u32) -> Result<String, JsValue> {
        json(self.runtime.apply_pending_at_tick(u64::from(tick))).map_err(js_error)
    }

    pub fn presentation_json(&self) -> Result<String, JsValue> {
        json(self.runtime.presentation()).map_err(js_error)
    }

    /// Selects the scenario's explicitly mapped arm target. The target is not
    /// guessed from an ECS entity ID by the browser.
    pub fn select_arm_presentation_json(&mut self) -> Result<String, JsValue> {
        json(self.runtime.select_arm_presentation()).map_err(js_error)
    }

    pub fn animation_catalog_json(&self) -> Result<String, JsValue> {
        json(self.runtime.animation_catalog()).map_err(js_error)
    }

    pub fn playback_json(&self) -> Result<String, JsValue> {
        json(self.runtime.playback()).map_err(js_error)
    }

    pub fn playback_command_json(&mut self, command_json: &str) -> Result<String, JsValue> {
        let command = serde_json::from_str::<PlaybackCommand>(command_json)
            .map_err(|error| JsValue::from_str(&format!("invalid playback command: {error}")))?;
        let result = self.runtime.apply_playback_command(command);
        json(result).map_err(js_error)
    }

    /// Advances only the fixed-step playback policy; it does not mutate the
    /// scenario world or create a browser-owned animation model.
    pub fn advance_animation_fixed_step(&mut self) -> Result<String, JsValue> {
        self.runtime.advance_animation_fixed_step();
        self.playback_json()
    }
}

impl WasmRuntimeObservationSession {
    fn record_observation(&mut self, observation: ObservationEnvelope) -> Result<(), JsValue> {
        self.latest_observation_diff = self
            .previous_observation
            .as_ref()
            .map(|previous| {
                compare_observation_snapshots(
                    previous,
                    &observation,
                    &ObservationComparisonConfig::default(),
                )
            })
            .transpose()
            .map_err(js_error)?;
        self.previous_observation = Some(observation);
        Ok(())
    }
}

fn build_runtime() -> Result<RuntimeInspectionAdapter, String> {
    RuntimeInspectionAdapter::from_animation_catalog(
        MAX_PENDING_COMMANDS,
        verified_hole_punch_catalog_fixture(),
    )
}

fn json<T: serde::Serialize>(value: T) -> Result<String, String> {
    serde_json::to_string(&value)
        .map_err(|error| format!("runtime observation serialization failed: {error}"))
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_runtime, WasmRuntimeObservationSession, MAX_PENDING_COMMANDS};
    use hello_runtime_observation::{
        CommandAuthority, CommandRequest, ObservationLimits, PlaybackCommand, Position,
        RuntimeCommand, RuntimeInspectionAdapter,
    };

    #[test]
    fn embedded_fixture_produces_a_bounded_runtime_catalog() {
        let runtime = build_runtime().expect("embedded GLB should decode");
        assert_eq!(runtime.animation_catalog().len(), 5);
        assert_eq!(runtime.animation_catalog()[0].name, "step1");
        assert_eq!(
            runtime
                .observe_entity_id(7, Some(runtime.arm_id().0), ObservationLimits::default())
                .sequence,
            7
        );
    }

    #[test]
    fn playback_commands_remain_provider_neutral() {
        let mut runtime = build_runtime().expect("embedded GLB should decode");
        let result = runtime.apply_playback_command(PlaybackCommand::Play { clip: 3 });
        assert_eq!(result.state.selected_clip, 3);
    }

    #[test]
    fn unknown_entity_observation_remains_explicit_at_the_consumer_boundary() {
        let runtime = build_runtime().expect("checked fixture should build a runtime");
        let observation = runtime.observe_entity_id(4, Some(99), ObservationLimits::default());

        assert!(observation.payload.selected.is_none());
        assert_eq!(observation.payload.diagnostics[0].code, "unknown_entity");
    }

    #[test]
    fn wasm_fixture_matches_the_native_glb_observation_contract() {
        let native = RuntimeInspectionAdapter::new(MAX_PENDING_COMMANDS)
            .expect("native GLB catalog should decode");
        let wasm_facing = build_runtime().expect("checked fixture should build a runtime");

        assert_eq!(wasm_facing.animation_catalog(), native.animation_catalog());
        assert_eq!(
            wasm_facing.observe_entity_id(11, None, ObservationLimits::default()),
            native.observe_entity_id(11, None, ObservationLimits::default()),
        );
    }

    #[test]
    fn native_and_wasm_facing_adapters_replay_the_same_command_trace() {
        let mut native = RuntimeInspectionAdapter::new(MAX_PENDING_COMMANDS)
            .expect("native GLB catalog should decode");
        let mut wasm_facing = build_runtime().expect("checked fixture should build a runtime");

        let requests = [
            CommandRequest {
                id: 1,
                target: native.arm_id().0,
                authority: CommandAuthority::Operator,
                expected_revision: Some(0),
                command: RuntimeCommand::MoveBy {
                    delta: Position {
                        x: 0.25,
                        y: 0.0,
                        z: 0.0,
                    },
                },
            },
            CommandRequest {
                id: 2,
                target: native.arm_id().0,
                authority: CommandAuthority::Operator,
                expected_revision: Some(0),
                command: RuntimeCommand::SetEnabled { enabled: false },
            },
        ];

        for request in requests {
            assert_eq!(
                wasm_facing.enqueue(request.clone()),
                native.enqueue(request)
            );
        }

        assert_eq!(
            wasm_facing.apply_pending_at_tick(4),
            native.apply_pending_at_tick(4),
        );
        assert_eq!(
            wasm_facing.observe_entity_id(
                5,
                Some(wasm_facing.arm_id().0),
                ObservationLimits::default()
            ),
            native.observe_entity_id(5, Some(native.arm_id().0), ObservationLimits::default()),
        );
    }

    #[test]
    fn browser_observations_expose_a_provider_neutral_previous_snapshot_diff() {
        let mut session = WasmRuntimeObservationSession {
            runtime: build_runtime().expect("checked fixture should build a runtime"),
            previous_observation: None,
            latest_observation_diff: None,
        };
        let arm = session.runtime.arm_id().0;

        session
            .observation_json(
                0,
                Some(u32::try_from(arm).expect("fixture ID must fit WASM API")),
            )
            .expect("initial observation should serialize");
        assert_eq!(
            session
                .latest_observation_diff_json()
                .expect("initial comparison should serialize"),
            "null"
        );

        session.runtime.enqueue(CommandRequest {
            id: 1,
            target: arm,
            authority: CommandAuthority::Operator,
            expected_revision: Some(0),
            command: RuntimeCommand::MoveBy {
                delta: Position {
                    x: 0.25,
                    y: 0.0,
                    z: 0.0,
                },
            },
        });
        session.runtime.apply_pending_at_tick(1);
        session
            .observation_json(
                1,
                Some(u32::try_from(arm).expect("fixture ID must fit WASM API")),
            )
            .expect("changed observation should serialize");

        let comparison: serde_json::Value = serde_json::from_str(
            &session
                .latest_observation_diff_json()
                .expect("comparison should serialize"),
        )
        .expect("comparison must be JSON");
        assert_eq!(comparison["before"]["revision"], 0);
        assert_eq!(comparison["after"]["revision"], 1);
        assert_eq!(comparison["payload"]["equal"], false);
    }
}
