use hello_runtime_observation::{
    compare_observation_snapshots, verified_hole_punch_catalog_fixture, CommandRequest,
    ObservationComparisonConfig, ObservationDiffReport, ObservationEnvelope, ObservationLimits,
    PlaybackCommand, PlaybackDisposition, RuntimeInspectionAdapter,
};
use observation_shell::{
    ApplicationCommandDescription, ApplicationCommandInvocation, ApplicationCommandResult,
    ApplicationMutationReceipt, ApplicationQueryField, ApplicationQueryResult,
    DiagnosticObservation, DiagnosticsObservation, EntityObservation, ObservationShell,
    ObservationSource, RelationshipEdgeObservation, RelationshipObservation, TypeObservation,
    WorldObservation,
};
use wasm_bindgen::prelude::*;

mod ratatui_shell;
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

/// Browser-facing Observation Shell facade.
///
/// TypeScript submits plain shell text and a transport sequence. Rust owns the
/// command catalog, application argument interpretation, runtime transition,
/// and the resulting bounded projection.
#[wasm_bindgen]
pub struct WasmObservationShellSession {
    runtime: RuntimeInspectionAdapter,
    previous_observation: Option<ObservationEnvelope>,
    latest_observation_diff: Option<ObservationDiffReport>,
    shell: ObservationShell,
    prompt: String,
    command_sequence: u32,
    history_offset: usize,
    transcript_scroll: usize,
    frame_width: u32,
    frame_height: u32,
}

#[wasm_bindgen]
impl WasmObservationShellSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        let mut shell = ObservationShell::default();
        register_runtime_shell_commands(&mut shell).map_err(js_error)?;
        build_runtime()
            .map(|runtime| Self {
                runtime,
                previous_observation: None,
                latest_observation_diff: None,
                shell,
                prompt: String::new(),
                command_sequence: 0,
                history_offset: 0,
                transcript_scroll: 0,
                frame_width: 0,
                frame_height: 0,
            })
            .map_err(js_error)
    }

    /// Executes raw owner-qualified shell input at a browser-supplied logical
    /// sequence. The returned record is the sole browser-visible command
    /// outcome; no runtime request or playback type crosses this boundary.
    pub fn execute_json(&mut self, input: &str, sequence: u32) -> Result<String, JsValue> {
        let source = source_from_runtime(self.runtime.observe(
            u64::from(sequence),
            None,
            ObservationLimits::default(),
        ));
        let record = self.shell.execute_at_sequence_with_application_handler(
            &source,
            input,
            u64::from(sequence),
            |invocation| handle_runtime_shell_command(&mut self.runtime, &source, invocation),
        );
        json(record).map_err(js_error)
    }

    /// Returns the same provider-neutral runtime observation used by the
    /// graphical controls. This keeps the browser's two views on one Rust
    /// session rather than creating parallel scenario state.
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
        record_observation(
            &mut self.previous_observation,
            &mut self.latest_observation_diff,
            observation.clone(),
        )?;
        self.record_toolbar_action(
            "observe",
            "Captured the current provider-neutral runtime observation.",
        );
        json(observation).map_err(js_error)
    }

    pub fn latest_observation_diff_json(&mut self) -> Result<String, JsValue> {
        self.record_toolbar_action(
            "latest-observation-diff",
            "Requested the comparison between the latest two browser-visible observations.",
        );
        json(&self.latest_observation_diff).map_err(js_error)
    }

    pub fn ui_snapshot_json(
        &mut self,
        width: u32,
        height: u32,
        sequence: u32,
        selected_entity: Option<u32>,
    ) -> Result<String, JsValue> {
        let snapshot = ui::build_runtime_ui_snapshot(
            &self.runtime,
            [width, height],
            u64::from(sequence),
            selected_entity.map(u64::from),
        )
        .map_err(js_error)?;
        self.record_toolbar_action(
            "observe-ui-contract",
            format!("Built a semantic UI snapshot for a {width} by {height} viewport."),
        );
        json(snapshot).map_err(js_error)
    }

    pub fn enqueue_json(&mut self, request_json: &str) -> Result<String, JsValue> {
        let request = serde_json::from_str::<CommandRequest>(request_json)
            .map_err(|error| JsValue::from_str(&format!("invalid runtime command: {error}")))?;
        let result = self.runtime.enqueue(request);
        self.record_toolbar_action(
            "queue-command",
            "Queued a browser-requested runtime mutation for the next fixed-step application.",
        );
        json(result).map_err(js_error)
    }

    pub fn apply_json(&mut self, tick: u32) -> Result<String, JsValue> {
        let result = self.runtime.apply_pending_at_tick(u64::from(tick));
        self.record_toolbar_action(
            "apply-queue",
            format!("Applied the pending runtime command queue at fixed tick {tick}."),
        );
        json(result).map_err(js_error)
    }

    pub fn presentation_json(&self) -> Result<String, JsValue> {
        json(self.runtime.presentation()).map_err(js_error)
    }

    pub fn select_arm_presentation_json(&mut self) -> Result<String, JsValue> {
        let result = self.runtime.select_arm_presentation();
        self.record_toolbar_action(
            "select-presentation",
            "Selected the scenario-owned arm presentation target.",
        );
        json(result).map_err(js_error)
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
        self.record_toolbar_action(
            "playback-command",
            "Applied a browser-requested playback transition through the shared runtime session.",
        );
        json(result).map_err(js_error)
    }

    pub fn advance_animation_fixed_step(&mut self) -> Result<String, JsValue> {
        self.runtime.advance_animation_fixed_step();
        let playback = self.runtime.playback().clone();
        self.record_toolbar_action(
            "advance-animation",
            "Advanced scenario playback by one fixed animation step.",
        );
        json(playback).map_err(js_error)
    }

    /// Exposes discovery as a bounded catalog, not as a browser-owned command
    /// grammar or a borrowed shell instance.
    pub fn command_catalog_json(&self) -> Result<String, JsValue> {
        json(self.shell.application_commands()).map_err(js_error)
    }

    /// Appends raw host text to the Rust-owned prompt. The browser does not
    /// interpret commands or construct terminal cells.
    pub fn ratatui_append_text(&mut self, text: &str) {
        self.prompt.push_str(text);
        self.history_offset = 0;
        self.transcript_scroll = 0;
    }

    pub fn ratatui_backspace(&mut self) {
        self.prompt.pop();
    }

    pub fn ratatui_clear_prompt(&mut self) {
        self.prompt.clear();
        self.history_offset = 0;
    }

    /// Submits the currently visible prompt through the same semantic shell
    /// handler as `execute_json`.
    pub fn ratatui_submit(&mut self) -> Result<String, JsValue> {
        let input = std::mem::take(&mut self.prompt);
        let sequence = self.command_sequence;
        self.command_sequence = self.command_sequence.saturating_add(1);
        self.history_offset = 0;
        self.transcript_scroll = 0;
        self.execute_json(&input, sequence)
    }

    pub fn ratatui_history_up(&mut self) {
        let inputs: Vec<&str> = self
            .shell
            .history()
            .iter()
            .filter(|record| !record.input.starts_with("[ui] "))
            .map(|record| record.input.as_str())
            .collect();
        if inputs.is_empty() {
            return;
        }
        self.history_offset = (self.history_offset + 1).min(inputs.len());
        if self.history_offset > 0 {
            self.prompt = inputs[inputs.len() - self.history_offset].to_owned();
        }
    }

    pub fn ratatui_history_down(&mut self) {
        if self.history_offset <= 1 {
            self.history_offset = 0;
            self.prompt.clear();
            return;
        }
        self.history_offset -= 1;
        let inputs: Vec<&str> = self
            .shell
            .history()
            .iter()
            .filter(|record| !record.input.starts_with("[ui] "))
            .map(|record| record.input.as_str())
            .collect();
        if inputs.is_empty() {
            self.history_offset = 0;
            self.prompt.clear();
            return;
        }
        self.prompt = inputs[inputs.len() - self.history_offset].to_owned();
    }

    pub fn ratatui_scroll_by(&mut self, lines: i32) {
        self.transcript_scroll = if lines.is_positive() {
            self.transcript_scroll.saturating_add(lines as usize)
        } else {
            self.transcript_scroll
                .saturating_sub(lines.unsigned_abs() as usize)
        };
    }

    /// Renders the live semantic shell through Ratatui and Tokimu's retained
    /// backend. The resulting bytes are an RGBA frame for the browser canvas.
    pub fn ratatui_frame_rgba(&mut self, width: u32, height: u32) -> Result<Vec<u8>, JsValue> {
        let presentation = self.runtime.presentation();
        let emphasis = presentation
            .targets
            .first()
            .and_then(|target| target.resolved.emphasis)
            .map(|value| format!("{value:?}"))
            .unwrap_or_else(|| "none".to_owned());
        let runtime_status = format!(
            " runtime revision={} tick={} | targets={} | emphasis={emphasis}",
            self.runtime.revision(),
            self.runtime.tick(),
            presentation.targets.len(),
        );
        let frame = ratatui_shell::render_shell(
            self.shell.history(),
            &self.prompt,
            self.transcript_scroll,
            &runtime_status,
            width,
            height,
        )
        .map_err(js_error)?;
        self.frame_width = frame.width;
        self.frame_height = frame.height;
        Ok(frame.rgba)
    }

    pub fn ratatui_frame_width(&self) -> u32 {
        self.frame_width
    }

    pub fn ratatui_frame_height(&self) -> u32 {
        self.frame_height
    }
}

impl WasmObservationShellSession {
    /// Retains a browser-toolbar action in the same Rust-owned transcript as
    /// terminal input without representing it as text the user typed.
    fn record_toolbar_action(&mut self, action: &str, summary: impl Into<String>) {
        let sequence = self.command_sequence;
        self.command_sequence = self.command_sequence.saturating_add(1);
        self.history_offset = 0;
        self.transcript_scroll = 0;

        self.shell.record_application_query_at_sequence(
            &format!("[ui] {action}"),
            u64::from(sequence),
            ApplicationCommandInvocation {
                owner: "runtime".to_owned(),
                command: format!("toolbar-{action}"),
                arguments: Vec::new(),
            },
            ApplicationQueryResult {
                summary: summary.into(),
                fields: vec![ApplicationQueryField::visible("source", "browser toolbar")],
            },
        );
    }
}

impl WasmRuntimeObservationSession {
    fn record_observation(&mut self, observation: ObservationEnvelope) -> Result<(), JsValue> {
        record_observation(
            &mut self.previous_observation,
            &mut self.latest_observation_diff,
            observation,
        )
    }
}

fn record_observation(
    previous_observation: &mut Option<ObservationEnvelope>,
    latest_observation_diff: &mut Option<ObservationDiffReport>,
    observation: ObservationEnvelope,
) -> Result<(), JsValue> {
    *latest_observation_diff = previous_observation
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
    *previous_observation = Some(observation);
    Ok(())
}

fn build_runtime() -> Result<RuntimeInspectionAdapter, String> {
    RuntimeInspectionAdapter::from_animation_catalog(
        MAX_PENDING_COMMANDS,
        verified_hole_punch_catalog_fixture(),
    )
}

fn register_runtime_shell_commands(shell: &mut ObservationShell) -> Result<(), String> {
    for command in [
        ApplicationCommandDescription::query(
            "runtime",
            "world-summary",
            "Inspect the runtime-owned structural world summary.",
            Vec::new(),
        ),
        ApplicationCommandDescription::query(
            "runtime",
            "relationships",
            "Inspect runtime-owned relationship types and edge counts.",
            Vec::new(),
        ),
        ApplicationCommandDescription::query(
            "diagnostics",
            "records",
            "Inspect bounded copied diagnostic records from their producer.",
            Vec::new(),
        ),
        ApplicationCommandDescription::query(
            "runtime",
            "list-animations",
            "List the scenario-owned hole-punch playback catalog.",
            Vec::new(),
        ),
        ApplicationCommandDescription::query(
            "runtime",
            "playback",
            "Inspect scenario-owned hole-punch playback state.",
            Vec::new(),
        ),
        ApplicationCommandDescription::query(
            "runtime",
            "presentation",
            "Inspect scenario-owned resolved presentation targets.",
            Vec::new(),
        ),
        ApplicationCommandDescription::mutation(
            "runtime",
            "play",
            "Start one scenario-owned animation clip by catalog index.",
            vec!["clip".to_owned()],
        ),
        ApplicationCommandDescription::mutation(
            "runtime",
            "pause",
            "Pause scenario-owned playback.",
            Vec::new(),
        ),
        ApplicationCommandDescription::mutation(
            "runtime",
            "resume",
            "Resume scenario-owned playback.",
            Vec::new(),
        ),
        ApplicationCommandDescription::mutation(
            "runtime",
            "stop",
            "Stop scenario-owned playback.",
            Vec::new(),
        ),
        ApplicationCommandDescription::mutation(
            "runtime",
            "reset",
            "Reset scenario-owned playback.",
            Vec::new(),
        ),
        ApplicationCommandDescription::mutation(
            "runtime",
            "advance",
            "Advance scenario-owned playback by one fixed step.",
            Vec::new(),
        ),
        ApplicationCommandDescription::mutation(
            "runtime",
            "select-arm",
            "Select the scenario-owned arm presentation target.",
            Vec::new(),
        ),
    ] {
        shell
            .register_application_command(command)
            .map_err(|error| format!("runtime shell command registration failed: {error:?}"))?;
    }
    Ok(())
}

fn handle_runtime_shell_command(
    runtime: &mut RuntimeInspectionAdapter,
    source: &ObservationSource,
    invocation: &ApplicationCommandInvocation,
) -> ApplicationCommandResult {
    match (invocation.owner.as_str(), invocation.command.as_str()) {
        ("runtime", "world-summary") => ApplicationCommandResult::Query {
            result: ApplicationQueryResult {
                summary: format!(
                    "revision {}; {} entities; {} relationship types",
                    source.world.revision,
                    source.world.entities.len(),
                    source.world.relationship_types.len()
                ),
                fields: vec![
                    ApplicationQueryField::visible(
                        "component types",
                        source.world.component_types.len().to_string(),
                    ),
                    ApplicationQueryField::visible(
                        "resource types",
                        source.world.resource_types.len().to_string(),
                    ),
                    ApplicationQueryField::visible(
                        "relationship edges",
                        source
                            .world
                            .relationship_types
                            .iter()
                            .map(|relationship| relationship.edges.len())
                            .sum::<usize>()
                            .to_string(),
                    ),
                ],
            },
        },
        ("runtime", "relationships") => ApplicationCommandResult::Query {
            result: ApplicationQueryResult {
                summary: format!(
                    "{} runtime-owned relationship types",
                    source.world.relationship_types.len()
                ),
                fields: source
                    .world
                    .relationship_types
                    .iter()
                    .map(|relationship| {
                        ApplicationQueryField::visible(
                            relationship.type_name.clone(),
                            format!("{} edge source(s)", relationship.edges.len()),
                        )
                    })
                    .collect(),
            },
        },
        ("diagnostics", "records") => ApplicationCommandResult::Query {
            result: ApplicationQueryResult {
                summary: format!(
                    "{} copied diagnostic record(s); {} dropped",
                    source.diagnostics.records.len(),
                    source.diagnostics.dropped_records
                ),
                fields: source
                    .diagnostics
                    .records
                    .iter()
                    .map(|record| {
                        ApplicationQueryField::visible(
                            format!("{}:{}", record.source, record.kind),
                            record.message.clone(),
                        )
                    })
                    .collect(),
            },
        },
        ("runtime", "list-animations") => ApplicationCommandResult::Query {
            result: ApplicationQueryResult {
                summary: format!(
                    "{} scenario-owned animation clips",
                    runtime.animation_catalog().len()
                ),
                fields: runtime
                    .animation_catalog()
                    .iter()
                    .map(|clip| {
                        ApplicationQueryField::visible(
                            format!("clip {}", clip.id),
                            format!(
                                "{}; {:.3}s; {} translation channel(s); nodes {:?}",
                                clip.name,
                                clip.duration_seconds,
                                clip.translation_channels,
                                clip.animated_nodes
                            ),
                        )
                    })
                    .collect(),
            },
        },
        ("runtime", "playback") => ApplicationCommandResult::Query {
            result: ApplicationQueryResult {
                summary: "scenario-owned playback state".to_owned(),
                fields: playback_fields(runtime),
            },
        },
        ("runtime", "presentation") => {
            let presentation = runtime.presentation();
            ApplicationCommandResult::Query {
                result: ApplicationQueryResult {
                    summary: format!(
                        "{} resolved presentation targets",
                        presentation.targets.len()
                    ),
                    fields: presentation
                        .targets
                        .into_iter()
                        .map(|target| {
                            ApplicationQueryField::visible(
                                target.target.to_string(),
                                format!(
                                    "visible={}; opacity={:.2}",
                                    target.resolved.visible, target.resolved.opacity
                                ),
                            )
                        })
                        .collect(),
                },
            }
        }
        ("runtime", "play") => {
            let Some(clip) = invocation
                .arguments
                .first()
                .and_then(|argument| argument.parse::<usize>().ok())
            else {
                return rejected_receipt("runtime requires: play <clip>");
            };
            playback_receipt(runtime, PlaybackCommand::Play { clip })
        }
        ("runtime", "pause") => playback_receipt(runtime, PlaybackCommand::Pause),
        ("runtime", "resume") => playback_receipt(runtime, PlaybackCommand::Resume),
        ("runtime", "stop") => playback_receipt(runtime, PlaybackCommand::Stop),
        ("runtime", "reset") => playback_receipt(runtime, PlaybackCommand::Reset),
        ("runtime", "advance") => {
            runtime.advance_animation_fixed_step();
            ApplicationCommandResult::Mutation {
                receipt: ApplicationMutationReceipt {
                    accepted: true,
                    applied_tick: Some(runtime.tick()),
                    resulting_revision: Some(runtime.revision()),
                    message: format!("advanced playback; mode={:?}", runtime.playback().mode),
                },
            }
        }
        ("runtime", "select-arm") => {
            let result = runtime.select_arm_presentation();
            ApplicationCommandResult::Mutation {
                receipt: ApplicationMutationReceipt {
                    accepted: matches!(
                        result.disposition,
                        hello_runtime_observation::PresentationCommandDisposition::Accepted
                    ),
                    applied_tick: Some(runtime.tick()),
                    resulting_revision: Some(runtime.revision()),
                    message: result
                        .diagnostic
                        .map(|diagnostic| diagnostic.message)
                        .unwrap_or_else(|| {
                            format!("presentation result: {:?}", result.disposition)
                        }),
                },
            }
        }
        _ => rejected_receipt("runtime command is not available in this shell scenario"),
    }
}

fn playback_fields(runtime: &RuntimeInspectionAdapter) -> Vec<ApplicationQueryField> {
    let state = runtime.playback();
    vec![
        ApplicationQueryField::visible("selected clip", state.selected_clip.to_string()),
        ApplicationQueryField::visible("mode", format!("{:?}", state.mode)),
        ApplicationQueryField::visible("local time", format!("{:.3}s", state.local_time_seconds)),
    ]
}

fn playback_receipt(
    runtime: &mut RuntimeInspectionAdapter,
    command: PlaybackCommand,
) -> ApplicationCommandResult {
    let result = runtime.apply_playback_command(command);
    let accepted = matches!(result.disposition, PlaybackDisposition::Accepted);
    ApplicationCommandResult::Mutation {
        receipt: ApplicationMutationReceipt {
            accepted,
            applied_tick: accepted.then(|| runtime.tick()),
            resulting_revision: accepted.then(|| runtime.revision()),
            message: result
                .diagnostic
                .map(|diagnostic| diagnostic.message)
                .unwrap_or_else(|| format!("playback result: {:?}", result.disposition)),
        },
    }
}

fn rejected_receipt(message: impl Into<String>) -> ApplicationCommandResult {
    ApplicationCommandResult::Mutation {
        receipt: ApplicationMutationReceipt {
            accepted: false,
            applied_tick: None,
            resulting_revision: None,
            message: message.into(),
        },
    }
}

fn source_from_runtime(envelope: ObservationEnvelope) -> ObservationSource {
    let payload = envelope.payload;
    ObservationSource {
        world: WorldObservation {
            revision: envelope.revision,
            entities: payload
                .entities
                .into_iter()
                .map(|id| EntityObservation { id })
                .collect(),
            component_types: payload
                .component_types
                .into_iter()
                .map(|entry| TypeObservation {
                    type_name: entry.type_name,
                    count: entry.count,
                })
                .collect(),
            resource_types: payload
                .resource_types
                .into_iter()
                .map(|entry| TypeObservation {
                    type_name: entry.type_name,
                    count: entry.count,
                })
                .collect(),
            relationship_types: payload
                .relationship_types
                .into_iter()
                .map(|relation| RelationshipObservation {
                    type_name: relation.type_name,
                    edges: relation
                        .edges
                        .into_iter()
                        .map(|edge| RelationshipEdgeObservation {
                            source: edge.source,
                            targets: edge.targets,
                        })
                        .collect(),
                })
                .collect(),
        },
        diagnostics: DiagnosticsObservation {
            dropped_records: 0,
            records: payload
                .diagnostics
                .into_iter()
                .enumerate()
                .map(|(sequence, diagnostic)| DiagnosticObservation {
                    sequence: sequence as u64,
                    severity: "information".to_owned(),
                    kind: diagnostic.code.to_owned(),
                    source: diagnostic.owner.to_owned(),
                    message: diagnostic.message,
                    performance: None,
                })
                .collect(),
        },
    }
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
    use super::{
        build_runtime, WasmObservationShellSession, WasmRuntimeObservationSession,
        MAX_PENDING_COMMANDS,
    };
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

    #[test]
    fn browser_shell_routes_raw_text_to_the_runtime_owned_handler() {
        let mut session =
            WasmObservationShellSession::new().expect("the browser shell fixture should construct");

        let catalog: serde_json::Value = serde_json::from_str(
            &session
                .command_catalog_json()
                .expect("catalog should serialize"),
        )
        .expect("catalog should be JSON");
        assert!(catalog
            .as_array()
            .expect("catalog should be an array")
            .iter()
            .any(|command| command["owner"] == "runtime" && command["command"] == "play"));
        assert!(catalog
            .as_array()
            .expect("catalog should be an array")
            .iter()
            .any(|command| {
                command["owner"] == "diagnostics" && command["command"] == "records"
            }));

        let world = serde_json::from_str::<serde_json::Value>(
            &session
                .execute_json("application runtime world-summary", 6)
                .expect("world query should serialize"),
        )
        .expect("world query record should be JSON");
        assert_eq!(world["response"]["owner"], "runtime");
        assert_eq!(world["response"]["data"]["kind"], "application_query");
        assert!(world["response"]["data"]["result"]["summary"]
            .as_str()
            .expect("world summary should be text")
            .contains("entities"));

        let diagnostics = serde_json::from_str::<serde_json::Value>(
            &session
                .execute_json("application diagnostics records", 7)
                .expect("diagnostics query should serialize"),
        )
        .expect("diagnostics query record should be JSON");
        assert_eq!(diagnostics["response"]["owner"], "diagnostics");

        let listed: serde_json::Value = serde_json::from_str(
            &session
                .execute_json("application runtime list-animations", 8)
                .expect("query should serialize"),
        )
        .expect("query record should be JSON");
        assert_eq!(listed["input"], "application runtime list-animations");
        assert_eq!(listed["response"]["owner"], "runtime");
        assert_eq!(listed["response"]["data"]["kind"], "application_query");
        assert_eq!(
            listed["response"]["data"]["result"]["summary"],
            "5 scenario-owned animation clips"
        );

        let played: serde_json::Value = serde_json::from_str(
            &session
                .execute_json("application runtime play 0", 9)
                .expect("mutation should serialize"),
        )
        .expect("mutation record should be JSON");
        assert_eq!(played["response"]["owner"], "runtime");
        assert_eq!(played["response"]["data"]["kind"], "application_mutation");
        assert_eq!(played["response"]["data"]["receipt"]["accepted"], true);
    }

    #[test]
    fn browser_shell_keeps_application_argument_validation_in_rust() {
        let mut session =
            WasmObservationShellSession::new().expect("the browser shell fixture should construct");
        let record: serde_json::Value = serde_json::from_str(
            &session
                .execute_json("application runtime play not-a-clip", 9)
                .expect("rejection should serialize"),
        )
        .expect("rejection record should be JSON");

        assert_eq!(record["response"]["data"]["kind"], "application_mutation");
        assert_eq!(record["response"]["data"]["receipt"]["accepted"], false);
        assert_eq!(
            record["response"]["data"]["receipt"]["message"],
            "runtime requires: play <clip>"
        );
    }

    #[test]
    fn ratatui_surface_projects_live_prompt_and_submitted_shell_output() {
        let mut session =
            WasmObservationShellSession::new().expect("the browser shell fixture should construct");
        let idle = session
            .ratatui_frame_rgba(720, 432)
            .expect("idle Ratatui frame should render");

        session.ratatui_append_text("application runtime world-summary");
        let editing = session
            .ratatui_frame_rgba(720, 432)
            .expect("prompt editing should render");
        assert_ne!(
            editing, idle,
            "the Rust-owned prompt must change the terminal projection"
        );

        let submitted: serde_json::Value = serde_json::from_str(
            &session
                .ratatui_submit()
                .expect("the shell command should execute"),
        )
        .expect("submitted shell record should be JSON");
        assert_eq!(submitted["response"]["owner"], "runtime");

        let completed = session
            .ratatui_frame_rgba(720, 432)
            .expect("submitted output should render");
        assert_ne!(
            completed, editing,
            "the semantic command result must change the terminal projection"
        );
        assert_eq!(session.ratatui_frame_width(), 720);
        assert_eq!(session.ratatui_frame_height(), 432);
    }

    #[test]
    fn semantic_controls_and_ratatui_share_one_rust_owned_runtime_session() {
        let mut session = WasmObservationShellSession::new()
            .expect("the shared browser fixture should construct");

        let before = session
            .presentation_json()
            .expect("initial presentation should serialize");
        let terminal_before = session
            .ratatui_frame_rgba(720, 432)
            .expect("initial Ratatui frame should render");

        session
            .select_arm_presentation_json()
            .expect("semantic presentation selection should serialize");

        let after = session
            .presentation_json()
            .expect("changed presentation should serialize");
        let terminal_after = session
            .ratatui_frame_rgba(720, 432)
            .expect("changed Ratatui frame should render");
        assert_ne!(
            before, after,
            "the semantic facade must mutate the shared runtime"
        );
        assert!(
            terminal_before != terminal_after,
            "Ratatui must project the same runtime state changed by the semantic facade"
        );

        let shell_result: serde_json::Value = serde_json::from_str(
            &session
                .execute_json("application runtime presentation", 14)
                .expect("terminal query should serialize"),
        )
        .expect("terminal query should be JSON");
        assert_eq!(shell_result["response"]["owner"], "runtime");
        assert_eq!(
            shell_result["response"]["data"]["result"]["summary"],
            "1 resolved presentation targets"
        );
    }

    #[test]
    fn semantic_controls_append_explicit_ratatui_transcript_records() {
        let mut session = WasmObservationShellSession::new()
            .expect("the shared browser fixture should construct");
        let idle = session
            .ratatui_frame_rgba(720, 432)
            .expect("idle Ratatui frame should render");

        session
            .select_arm_presentation_json()
            .expect("semantic presentation selection should serialize");

        let record = session
            .shell
            .history()
            .last()
            .expect("toolbar selection should be retained in the transcript");
        assert_eq!(record.input, "[ui] select-presentation");
        assert_eq!(record.response.owner, "runtime");
        assert!(record.projection.contains("toolbar-select-presentation"));
        assert!(record.projection.contains("browser toolbar"));

        let updated = session
            .ratatui_frame_rgba(720, 432)
            .expect("toolbar transcript entry should render");
        assert_ne!(
            idle, updated,
            "a semantic control must visibly update the shared Ratatui transcript"
        );
    }

    #[test]
    fn browser_shell_playback_observation_matches_the_direct_runtime_fixture() {
        let mut shell =
            WasmObservationShellSession::new().expect("the browser shell fixture should construct");
        let shell_played: serde_json::Value = serde_json::from_str(
            &shell
                .execute_json("application runtime play 1", 10)
                .expect("shell mutation should serialize"),
        )
        .expect("shell mutation must be JSON");
        assert_eq!(
            shell_played["response"]["data"]["receipt"]["accepted"],
            true
        );

        let shell_playback: serde_json::Value = serde_json::from_str(
            &shell
                .execute_json("application runtime playback", 11)
                .expect("shell query should serialize"),
        )
        .expect("shell query must be JSON");

        let mut direct =
            WasmRuntimeObservationSession::new().expect("the direct fixture should construct");
        direct
            .playback_command_json(r#"{"command":"play","clip":1}"#)
            .expect("direct mutation should serialize");
        let direct_playback: serde_json::Value = serde_json::from_str(
            &direct
                .playback_json()
                .expect("direct playback should serialize"),
        )
        .expect("direct playback must be JSON");

        let selected_clip = shell_playback["response"]["data"]["result"]["fields"]
            .as_array()
            .expect("shell playback fields must be an array")
            .iter()
            .find(|field| field["name"] == "selected clip")
            .expect("shell playback must report the selected clip");
        assert_eq!(
            selected_clip["disclosure"]["value"],
            direct_playback["selected_clip"].to_string()
        );
    }

    #[test]
    fn browser_shell_observation_json_is_not_runtime_state() {
        let mut session =
            WasmObservationShellSession::new().expect("the browser shell fixture should construct");
        let mut projected: serde_json::Value = serde_json::from_str(
            &session
                .execute_json("application runtime playback", 12)
                .expect("playback query should serialize"),
        )
        .expect("playback record must be JSON");

        // The browser receives a projection. Editing it cannot reach the
        // scenario-owned runtime behind the WASM shell boundary.
        projected["response"]["data"]["result"]["summary"] =
            serde_json::Value::String("forged browser playback".to_owned());
        projected["response"]["data"]["result"]["fields"] = serde_json::json!([
            {
                "name": "selected clip",
                "disclosure": { "visibility": "visible", "value": "forged" }
            }
        ]);

        let observed_again: serde_json::Value = serde_json::from_str(
            &session
                .execute_json("application runtime playback", 13)
                .expect("second playback query should serialize"),
        )
        .expect("second playback record must be JSON");
        assert_eq!(
            observed_again["response"]["data"]["result"]["summary"],
            "scenario-owned playback state"
        );
        assert!(observed_again["response"]["data"]["result"]["fields"]
            .as_array()
            .expect("runtime fields must be an array")
            .iter()
            .all(|field| field["disclosure"]["value"] != "forged"));
    }
}
