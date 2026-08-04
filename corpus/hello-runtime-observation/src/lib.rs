mod animation;
mod diff;
mod presentation;

pub use animation::{
    catalog_from_glb_bytes, load_hole_punch_catalog, sample_hole_punch_translations,
    verified_hole_punch_catalog_fixture, AnimationClipObservation, PlaybackCommand,
    PlaybackCommandResult, PlaybackDisposition, PlaybackMode, PlaybackPolicy, PlaybackState,
};
pub use diff::{
    compare_observation_snapshots, ObservationComparisonConfig, ObservationDiffError,
    ObservationDiffReport, ObservationProvenance,
};
pub use presentation::{
    IdentityMappingObservation, ImportedNodeId, PresentationCommand,
    PresentationCommandDisposition, PresentationCommandResult, PresentationObservation,
    ResolvedPresentationObservation, ScenarioPresentation,
};

use serde::{Deserialize, Serialize};
use std::any::type_name;
use tokimu_core::{world::WorldSnapshot, EntityId, World};

pub const OBSERVATION_SCHEMA: &str = "tokimu.corpus.runtime-observation";
pub const OBSERVATION_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Enabled(pub bool);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParentOf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScenarioSettings {
    pub fixed_step_hz: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationContext {
    pub sequence: u64,
    pub tick: u64,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationLimits {
    pub max_entities: usize,
    pub max_relationship_edges: usize,
    pub max_diagnostics: usize,
}

impl Default for ObservationLimits {
    fn default() -> Self {
        Self {
            max_entities: 32,
            max_relationship_edges: 64,
            max_diagnostics: 16,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ObservationEnvelope {
    pub schema: &'static str,
    pub version: u16,
    pub sequence: u64,
    pub tick: u64,
    pub revision: u64,
    pub kind: &'static str,
    pub payload: WorldObservation,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WorldObservation {
    pub owner: &'static str,
    pub partial: bool,
    pub entity_count: usize,
    pub entities: Vec<u64>,
    pub component_types: Vec<TypeObservation>,
    pub resource_types: Vec<TypeObservation>,
    pub relationship_types: Vec<RelationshipTypeObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<EntityDetailObservation>,
    pub diagnostics: Vec<ObservationDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TypeObservation {
    pub type_name: String,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RelationshipTypeObservation {
    pub type_name: String,
    pub edges: Vec<RelationshipEdgeObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RelationshipEdgeObservation {
    pub source: u64,
    pub targets: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EntityDetailObservation {
    pub entity: u64,
    pub components: Vec<ComponentValueObservation>,
    pub relationships: Vec<RelationshipTypeObservation>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "component", content = "value")]
pub enum ComponentValueObservation {
    Position(Position),
    Enabled(bool),
}

impl ComponentValueObservation {
    fn name(&self) -> &'static str {
        match self {
            Self::Position(_) => "Position",
            Self::Enabled(_) => "Enabled",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObservationDiagnostic {
    pub code: &'static str,
    pub owner: &'static str,
    pub message: String,
}

pub struct Scenario {
    pub world: World,
    pub root: EntityId,
    pub arm: EntityId,
}

/// Application-owned command intent for this corpus scenario.
///
/// These variants deliberately name scenario semantics rather than exposing a
/// generic "write component" operation over `World`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum RuntimeCommand {
    MoveBy { delta: Position },
    SetEnabled { enabled: bool },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CommandRequest {
    pub id: u64,
    pub target: u64,
    pub authority: CommandAuthority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<u64>,
    pub command: RuntimeCommand,
}

/// Corpus-local mutation permission. This proves request admission without
/// claiming a general engine authorization capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandAuthority {
    Observer,
    Operator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandDisposition {
    Queued,
    Accepted,
    RejectedInvalid,
    RejectedQueueFull,
    RejectedUnauthorized,
    RejectedUnknownTarget,
    RejectedStaleRevision,
    RejectedUnsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandResult {
    pub id: u64,
    pub disposition: CommandDisposition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_tick: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resulting_revision: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<ObservationDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CommandTrace {
    pub owner: &'static str,
    pub phase: &'static str,
    pub tick: u64,
    pub initial_revision: u64,
    pub final_revision: u64,
    pub results: Vec<CommandResult>,
}

/// A bounded, application-owned mutation boundary around a `World`.
///
/// Queued commands have no effect until `apply_pending_at_tick` runs. The
/// corpus uses that call as its documented "apply commands" lifecycle phase.
pub struct RuntimeObservationSession {
    world: World,
    root: EntityId,
    arm: EntityId,
    tick: u64,
    revision: u64,
    pending: Vec<CommandRequest>,
    max_pending_commands: usize,
}

impl RuntimeObservationSession {
    pub fn from_scenario(scenario: Scenario, max_pending_commands: usize) -> Self {
        Self {
            world: scenario.world,
            root: scenario.root,
            arm: scenario.arm,
            tick: 0,
            revision: 0,
            pending: Vec::new(),
            max_pending_commands,
        }
    }

    /// Stable scenario identity exposed to consumers without exposing `World`.
    pub fn root_id(&self) -> EntityId {
        self.root
    }

    /// Stable scenario identity exposed to consumers without exposing `World`.
    pub fn arm_id(&self) -> EntityId {
        self.arm
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Produce an owned, bounded observation. Consumers cannot inspect or
    /// mutate the scenario world through this operation.
    pub fn observe(
        &self,
        sequence: u64,
        selected: Option<EntityId>,
        limits: ObservationLimits,
    ) -> ObservationEnvelope {
        observe_world(
            &self.world,
            ObservationContext {
                sequence,
                tick: self.tick,
                revision: self.revision,
            },
            selected,
            limits,
        )
    }

    /// Queue intent without mutating simulation state. Queue capacity is a
    /// corpus safety bound, not a kernel scheduling policy.
    pub fn enqueue(&mut self, request: CommandRequest) -> CommandResult {
        if request.authority != CommandAuthority::Operator {
            return CommandResult {
                id: request.id,
                disposition: CommandDisposition::RejectedUnauthorized,
                applied_tick: None,
                resulting_revision: None,
                diagnostic: Some(command_diagnostic(
                    "command_authority_denied",
                    format!(
                        "command {} requires operator authority in this scenario",
                        request.id
                    ),
                )),
            };
        }
        if self.pending.len() >= self.max_pending_commands {
            return CommandResult {
                id: request.id,
                disposition: CommandDisposition::RejectedQueueFull,
                applied_tick: None,
                resulting_revision: None,
                diagnostic: Some(command_diagnostic(
                    "command_queue_full",
                    format!(
                        "command queue limit {} rejected command {}",
                        self.max_pending_commands, request.id
                    ),
                )),
            };
        }

        let id = request.id;
        self.pending.push(request);
        CommandResult {
            id,
            disposition: CommandDisposition::Queued,
            applied_tick: None,
            resulting_revision: None,
            diagnostic: None,
        }
    }

    /// Apply the current FIFO queue at the corpus' explicit mutation phase.
    /// Every accepted command advances the scenario revision exactly once.
    pub fn apply_pending_at_tick(&mut self, tick: u64) -> CommandTrace {
        self.tick = tick;
        let initial_revision = self.revision;
        let pending = std::mem::take(&mut self.pending);
        let results = pending
            .into_iter()
            .map(|request| self.validate_and_apply(request))
            .collect();

        CommandTrace {
            owner: "application",
            phase: "apply_commands",
            tick: self.tick,
            initial_revision,
            final_revision: self.revision,
            results,
        }
    }

    fn validate_and_apply(&mut self, request: CommandRequest) -> CommandResult {
        if request
            .expected_revision
            .is_some_and(|expected| expected != self.revision)
        {
            return self.rejected(
                request.id,
                CommandDisposition::RejectedStaleRevision,
                "stale_expected_revision",
                format!(
                    "command {} expected revision {:?}, but the current revision is {}",
                    request.id, request.expected_revision, self.revision
                ),
            );
        }

        let target = EntityId(request.target);
        if !self
            .world
            .snapshot()
            .entities
            .iter()
            .any(|entity| entity.id == target)
        {
            return self.rejected(
                request.id,
                CommandDisposition::RejectedUnknownTarget,
                "unknown_command_target",
                format!("command {} targets unknown entity {}", request.id, target.0),
            );
        }

        if let RuntimeCommand::MoveBy { delta } = request.command {
            if !delta.x.is_finite() || !delta.y.is_finite() || !delta.z.is_finite() {
                return self.rejected(
                    request.id,
                    CommandDisposition::RejectedInvalid,
                    "invalid_move_delta",
                    format!(
                        "command {} contains a non-finite movement delta for entity {}",
                        request.id, target.0
                    ),
                );
            }
        }

        // `World::component_mut` creates a store on demand. Check support via
        // the immutable query first so an unsupported command has no hidden
        // structural side effect.
        let supported = match request.command {
            RuntimeCommand::MoveBy { delta }
                if self.world.component::<Position>(target).is_some() =>
            {
                let position = self
                    .world
                    .component_mut::<Position>(target)
                    .expect("component support was checked before mutation");
                position.x += delta.x;
                position.y += delta.y;
                position.z += delta.z;
                true
            }
            RuntimeCommand::SetEnabled { enabled }
                if self.world.component::<Enabled>(target).is_some() =>
            {
                self.world
                    .component_mut::<Enabled>(target)
                    .expect("component support was checked before mutation")
                    .0 = enabled;
                true
            }
            _ => false,
        };

        if !supported {
            return self.rejected(
                request.id,
                CommandDisposition::RejectedUnsupported,
                "unsupported_command_target",
                format!(
                    "command {} is not supported by entity {} in this scenario",
                    request.id, target.0
                ),
            );
        }

        self.revision += 1;
        CommandResult {
            id: request.id,
            disposition: CommandDisposition::Accepted,
            applied_tick: Some(self.tick),
            resulting_revision: Some(self.revision),
            diagnostic: None,
        }
    }

    fn rejected(
        &self,
        id: u64,
        disposition: CommandDisposition,
        code: &'static str,
        message: String,
    ) -> CommandResult {
        CommandResult {
            id,
            disposition,
            applied_tick: None,
            resulting_revision: None,
            diagnostic: Some(command_diagnostic(code, message)),
        }
    }
}

fn command_diagnostic(code: &'static str, message: String) -> ObservationDiagnostic {
    ObservationDiagnostic {
        code,
        owner: "application_command_adapter",
        message,
    }
}

pub fn build_scenario() -> Scenario {
    let mut world = World::default();
    let root = world.spawn();
    let arm = world.spawn();

    world.insert_component(
        root,
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    );
    world.insert_component(
        arm,
        Position {
            x: 2.0,
            y: 1.0,
            z: -0.5,
        },
    );
    world.insert_component(arm, Enabled(true));
    world.insert_resource(ScenarioSettings { fixed_step_hz: 60 });

    world.add_relationship::<ParentOf>(root, arm);

    Scenario { world, root, arm }
}

pub fn build_session(max_pending_commands: usize) -> RuntimeObservationSession {
    RuntimeObservationSession::from_scenario(build_scenario(), max_pending_commands)
}

/// Scenario-local façade for inspection consumers.
///
/// This deliberately composes world, animation, and presentation state inside
/// the corpus scenario. Native and WASM consumers can request observations and
/// submit semantic commands without rebuilding importer or presentation state.
pub struct RuntimeInspectionAdapter {
    session: RuntimeObservationSession,
    presentation: ScenarioPresentation,
    animation_catalog: Vec<AnimationClipObservation>,
    playback: PlaybackState,
}

impl RuntimeInspectionAdapter {
    pub fn new(max_pending_commands: usize) -> Result<Self, String> {
        let session = build_session(max_pending_commands);
        let presentation = ScenarioPresentation::for_hole_punch(session.arm_id().0);
        let animation_catalog = load_hole_punch_catalog()?;
        Ok(Self::from_parts(session, presentation, animation_catalog))
    }

    /// Construct from provider-neutral importer evidence.
    ///
    /// This keeps native file acquisition and WASM fixture embedding outside
    /// the runtime facade while preserving one playback contract.
    pub fn from_animation_catalog(
        max_pending_commands: usize,
        animation_catalog: Vec<AnimationClipObservation>,
    ) -> Result<Self, String> {
        if animation_catalog.is_empty() {
            return Err("runtime inspection requires a non-empty animation catalog".to_owned());
        }
        let session = build_session(max_pending_commands);
        let presentation = ScenarioPresentation::for_hole_punch(session.arm_id().0);
        Ok(Self::from_parts(session, presentation, animation_catalog))
    }

    fn from_parts(
        session: RuntimeObservationSession,
        presentation: ScenarioPresentation,
        animation_catalog: Vec<AnimationClipObservation>,
    ) -> Self {
        Self {
            session,
            presentation,
            animation_catalog,
            playback: PlaybackState::initial(PlaybackPolicy {
                hold_completed_steps: true,
            }),
        }
    }

    pub fn root_id(&self) -> EntityId {
        self.session.root_id()
    }

    pub fn arm_id(&self) -> EntityId {
        self.session.arm_id()
    }

    pub fn tick(&self) -> u64 {
        self.session.tick()
    }

    pub fn revision(&self) -> u64 {
        self.session.revision()
    }

    pub fn observe(
        &self,
        sequence: u64,
        selected: Option<EntityId>,
        limits: ObservationLimits,
    ) -> ObservationEnvelope {
        self.session.observe(sequence, selected, limits)
    }

    /// Observe a wire-format entity identity without exposing `EntityId` to a
    /// browser or other foreign-language adapter.
    pub fn observe_entity_id(
        &self,
        sequence: u64,
        selected: Option<u64>,
        limits: ObservationLimits,
    ) -> ObservationEnvelope {
        self.observe(sequence, selected.map(EntityId), limits)
    }

    pub fn enqueue(&mut self, request: CommandRequest) -> CommandResult {
        self.session.enqueue(request)
    }

    pub fn apply_pending_at_tick(&mut self, tick: u64) -> CommandTrace {
        self.session.apply_pending_at_tick(tick)
    }

    pub fn presentation(&self) -> PresentationObservation {
        self.presentation.observe()
    }

    pub fn select_arm_presentation(&mut self) -> PresentationCommandResult {
        let target = self
            .presentation
            .mapping_for_entity(self.arm_id().0)
            .expect("static arm presentation mapping must exist")
            .presentation_target
            .clone();
        self.presentation
            .apply(PresentationCommand::Select { target })
    }

    pub fn animation_catalog(&self) -> &[AnimationClipObservation] {
        &self.animation_catalog
    }

    pub fn playback(&self) -> &PlaybackState {
        &self.playback
    }

    pub fn next_animation_step(&mut self) -> PlaybackCommandResult {
        self.apply_playback_command(PlaybackCommand::NextStep)
    }

    pub fn play_selected_animation(&mut self) -> PlaybackCommandResult {
        self.apply_playback_command(PlaybackCommand::Play {
            clip: self.playback.selected_clip,
        })
    }

    /// Applies one provider-neutral playback request to the catalog-backed
    /// state. The adapter retains all importer data internally.
    pub fn apply_playback_command(&mut self, command: PlaybackCommand) -> PlaybackCommandResult {
        self.playback
            .apply_command(&self.animation_catalog, command)
    }

    pub fn advance_animation_fixed_step(&mut self) {
        self.playback.advance_fixed_step(&self.animation_catalog);
    }
}

pub fn scripted_command_requests(session: &RuntimeObservationSession) -> Vec<CommandRequest> {
    vec![
        CommandRequest {
            id: 1,
            target: session.arm_id().0,
            authority: CommandAuthority::Operator,
            expected_revision: Some(0),
            command: RuntimeCommand::MoveBy {
                delta: Position {
                    x: 1.0,
                    y: -0.5,
                    z: 0.25,
                },
            },
        },
        CommandRequest {
            id: 2,
            target: session.arm_id().0,
            authority: CommandAuthority::Operator,
            expected_revision: Some(1),
            command: RuntimeCommand::SetEnabled { enabled: false },
        },
        CommandRequest {
            id: 3,
            target: session.arm_id().0,
            authority: CommandAuthority::Operator,
            expected_revision: Some(0),
            command: RuntimeCommand::MoveBy {
                delta: Position {
                    x: 99.0,
                    y: 99.0,
                    z: 99.0,
                },
            },
        },
        CommandRequest {
            id: 4,
            target: 99,
            authority: CommandAuthority::Operator,
            expected_revision: Some(2),
            command: RuntimeCommand::SetEnabled { enabled: true },
        },
        CommandRequest {
            id: 5,
            target: session.root_id().0,
            authority: CommandAuthority::Operator,
            expected_revision: Some(2),
            command: RuntimeCommand::SetEnabled { enabled: true },
        },
    ]
}

pub fn observe_world(
    world: &World,
    context: ObservationContext,
    selected: Option<EntityId>,
    limits: ObservationLimits,
) -> ObservationEnvelope {
    let snapshot = world.snapshot();
    let detail_requested = selected.is_some();
    let mut diagnostics = Vec::new();
    let mut partial = false;

    let entity_count = snapshot.entities.len();
    let entities = snapshot
        .entities
        .iter()
        .take(limits.max_entities)
        .map(|entity| entity.id.0)
        .collect::<Vec<_>>();
    if entities.len() < entity_count {
        partial = true;
        diagnostics.push(ObservationDiagnostic {
            code: "entity_limit_reached",
            owner: "observation_adapter",
            message: format!(
                "entity summary truncated from {entity_count} to {} items",
                entities.len()
            ),
        });
    }

    let mut remaining_edges = limits.max_relationship_edges;
    let relationship_types = relationship_observations(&snapshot, &mut remaining_edges);
    let observed_edge_count = relationship_types
        .iter()
        .map(|relationship| relationship.edges.len())
        .sum::<usize>();
    let source_edge_count = snapshot
        .relationship_types
        .iter()
        .map(|relationship| relationship.edges.len())
        .sum::<usize>();
    if observed_edge_count < source_edge_count {
        partial = true;
        diagnostics.push(ObservationDiagnostic {
            code: "relationship_edge_limit_reached",
            owner: "observation_adapter",
            message: format!(
                "relationship summary truncated from {source_edge_count} to {observed_edge_count} edges"
            ),
        });
    }

    let selected = selected.and_then(|entity| {
        if !snapshot.entities.iter().any(|item| item.id == entity) {
            diagnostics.push(ObservationDiagnostic {
                code: "unknown_entity",
                owner: "world",
                message: format!("entity {} does not exist in this snapshot", entity.0),
            });
            return None;
        }

        let mut components = Vec::new();
        if let Some(position) = world.component::<Position>(entity) {
            components.push(ComponentValueObservation::Position(*position));
        }
        if let Some(enabled) = world.component::<Enabled>(entity) {
            components.push(ComponentValueObservation::Enabled(enabled.0));
        }
        components.sort_by_key(ComponentValueObservation::name);

        if components.is_empty() {
            diagnostics.push(ObservationDiagnostic {
                code: "selected_detail_unavailable",
                owner: "application",
                message: format!(
                    "entity {} has no component detail registered by this corpus",
                    entity.0
                ),
            });
        }

        let relationships = relationship_types
            .iter()
            .filter_map(|relationship| {
                let edges = relationship
                    .edges
                    .iter()
                    .filter(|edge| edge.source == entity.0)
                    .cloned()
                    .collect::<Vec<_>>();
                (!edges.is_empty()).then(|| RelationshipTypeObservation {
                    type_name: relationship.type_name.clone(),
                    edges,
                })
            })
            .collect();

        Some(EntityDetailObservation {
            entity: entity.0,
            components,
            relationships,
        })
    });

    if diagnostics.len() > limits.max_diagnostics {
        diagnostics.truncate(limits.max_diagnostics);
        partial = true;
    }

    ObservationEnvelope {
        schema: OBSERVATION_SCHEMA,
        version: OBSERVATION_VERSION,
        sequence: context.sequence,
        tick: context.tick,
        revision: context.revision,
        kind: if detail_requested {
            "world_selected_detail"
        } else {
            "world_summary"
        },
        payload: WorldObservation {
            owner: "simulation",
            partial,
            entity_count,
            entities,
            component_types: type_observations(&snapshot.component_types),
            resource_types: type_observations(&snapshot.resource_types),
            relationship_types,
            selected,
            diagnostics,
        },
    }
}

pub fn serialize_observation(
    observation: &ObservationEnvelope,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec_pretty(observation)
}

fn type_observations(types: &[tokimu_core::world::TypeSnapshot]) -> Vec<TypeObservation> {
    types
        .iter()
        .map(|item| TypeObservation {
            type_name: item.type_name.to_owned(),
            count: item.count,
        })
        .collect()
}

fn relationship_observations(
    snapshot: &WorldSnapshot,
    remaining_edges: &mut usize,
) -> Vec<RelationshipTypeObservation> {
    snapshot
        .relationship_types
        .iter()
        .map(|relationship| {
            let take = (*remaining_edges).min(relationship.edges.len());
            let edges = relationship
                .edges
                .iter()
                .take(take)
                .map(|(source, targets)| RelationshipEdgeObservation {
                    source: source.0,
                    targets: targets.iter().map(|target| target.0).collect(),
                })
                .collect();
            *remaining_edges -= take;
            RelationshipTypeObservation {
                type_name: relationship.type_name.to_owned(),
                edges,
            }
        })
        .collect()
}

pub fn expected_type_names() -> [&'static str; 4] {
    [
        type_name::<Enabled>(),
        type_name::<Position>(),
        type_name::<ScenarioSettings>(),
        type_name::<ParentOf>(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTEXT: ObservationContext = ObservationContext {
        sequence: 7,
        tick: 12,
        revision: 3,
    };

    #[test]
    fn unchanged_observations_serialize_identically_without_mutating_world() {
        let scenario = build_scenario();
        let before = scenario.world.snapshot();

        let first = serialize_observation(&observe_world(
            &scenario.world,
            CONTEXT,
            Some(scenario.arm),
            ObservationLimits::default(),
        ))
        .unwrap();
        let second = serialize_observation(&observe_world(
            &scenario.world,
            CONTEXT,
            Some(scenario.arm),
            ObservationLimits::default(),
        ))
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(scenario.world.snapshot(), before);
    }

    #[test]
    fn relationship_targets_are_in_entity_order() {
        let scenario = build_scenario();
        let observation = observe_world(
            &scenario.world,
            CONTEXT,
            Some(scenario.root),
            ObservationLimits::default(),
        );

        assert_eq!(
            observation.payload.relationship_types[0].edges[0].targets,
            vec![scenario.arm.0]
        );
    }

    #[test]
    fn unknown_and_unregistered_detail_are_explicit() {
        let scenario = build_scenario();
        let unknown = observe_world(
            &scenario.world,
            CONTEXT,
            Some(EntityId(99)),
            ObservationLimits::default(),
        );
        assert!(unknown.payload.selected.is_none());
        assert_eq!(unknown.kind, "world_selected_detail");
        assert_eq!(unknown.payload.diagnostics[0].code, "unknown_entity");

        let unregistered = observe_world(
            &scenario.world,
            CONTEXT,
            Some(scenario.root),
            ObservationLimits::default(),
        );
        assert!(unregistered.payload.selected.is_some());
        assert!(unregistered.payload.diagnostics.is_empty());

        let mut world = World::default();
        let empty = world.spawn();
        let unavailable = observe_world(&world, CONTEXT, Some(empty), ObservationLimits::default());
        assert_eq!(
            unavailable.payload.diagnostics[0].code,
            "selected_detail_unavailable"
        );
    }

    #[test]
    fn summary_limits_report_partial_observations() {
        let scenario = build_scenario();
        let observation = observe_world(
            &scenario.world,
            CONTEXT,
            None,
            ObservationLimits {
                max_entities: 1,
                max_relationship_edges: 0,
                max_diagnostics: 8,
            },
        );

        assert!(observation.payload.partial);
        assert_eq!(observation.payload.entities, vec![0]);
        assert_eq!(observation.payload.diagnostics.len(), 2);
    }

    #[test]
    fn diagnostic_limits_are_deterministic_and_bounded() {
        let scenario = build_scenario();
        let limits = ObservationLimits {
            max_entities: 1,
            max_relationship_edges: 0,
            max_diagnostics: 1,
        };

        let first = observe_world(&scenario.world, CONTEXT, None, limits);
        let second = observe_world(&scenario.world, CONTEXT, None, limits);

        assert!(first.payload.partial);
        assert_eq!(first.payload.diagnostics.len(), 1);
        assert_eq!(first.payload.diagnostics[0].code, "entity_limit_reached");
        assert_eq!(first, second);
    }

    #[test]
    fn inspection_adapter_exposes_observation_and_commands_without_world_access() {
        let mut adapter = RuntimeInspectionAdapter::new(2).unwrap();
        let arm = adapter.arm_id();
        let before = adapter.observe(0, Some(arm), ObservationLimits::default());
        assert_eq!(before.revision, 0);
        assert_eq!(adapter.animation_catalog().len(), 5);
        assert_eq!(adapter.presentation().targets.len(), 1);

        let queued = adapter.enqueue(CommandRequest {
            id: 1,
            target: arm.0,
            authority: CommandAuthority::Operator,
            expected_revision: Some(adapter.revision()),
            command: RuntimeCommand::SetEnabled { enabled: false },
        });
        assert_eq!(queued.disposition, CommandDisposition::Queued);

        let trace = adapter.apply_pending_at_tick(adapter.tick().saturating_add(1));
        assert_eq!(trace.results[0].disposition, CommandDisposition::Accepted);
        assert_eq!(
            adapter
                .observe(1, Some(arm), ObservationLimits::default())
                .revision,
            1
        );
    }

    #[test]
    fn observations_are_owned_and_cannot_mutate_session_state() {
        let session = build_session(4);
        let before = session.world.snapshot();
        let observation = observe_world(
            &session.world,
            ObservationContext {
                sequence: 0,
                tick: session.tick,
                revision: session.revision,
            },
            Some(session.arm),
            ObservationLimits::default(),
        );
        let mut edited = observation.clone();
        edited.payload.entities.clear();
        edited.payload.selected.as_mut().unwrap().components.clear();

        assert_eq!(session.world.snapshot(), before);
        assert_eq!(session.revision, 0);
    }

    #[test]
    fn command_results_are_explicit_and_rejections_do_not_mutate_state() {
        let mut session = build_session(8);
        for request in scripted_command_requests(&session) {
            assert_eq!(
                session.enqueue(request).disposition,
                CommandDisposition::Queued
            );
        }

        let trace = session.apply_pending_at_tick(4);
        assert_eq!(trace.phase, "apply_commands");
        assert_eq!(trace.final_revision, 2);
        assert_eq!(
            trace
                .results
                .iter()
                .map(|result| result.disposition)
                .collect::<Vec<_>>(),
            vec![
                CommandDisposition::Accepted,
                CommandDisposition::Accepted,
                CommandDisposition::RejectedStaleRevision,
                CommandDisposition::RejectedUnknownTarget,
                CommandDisposition::RejectedUnsupported,
            ]
        );
        assert_eq!(trace.results[0].applied_tick, Some(4));
        assert_eq!(trace.results[1].resulting_revision, Some(2));
        assert_eq!(
            session.world.component::<Position>(session.arm),
            Some(&Position {
                x: 3.0,
                y: 0.5,
                z: -0.25
            })
        );
        assert_eq!(
            session.world.component::<Enabled>(session.arm),
            Some(&Enabled(false))
        );
    }

    #[test]
    fn queue_capacity_is_bounded_without_mutating_the_world() {
        let mut session = build_session(1);
        let requests = scripted_command_requests(&session);
        let before = session.world.snapshot();
        assert_eq!(
            session.enqueue(requests[0].clone()).disposition,
            CommandDisposition::Queued
        );
        assert_eq!(
            session.enqueue(requests[1].clone()).disposition,
            CommandDisposition::RejectedQueueFull
        );
        assert_eq!(session.world.snapshot(), before);
        assert_eq!(session.revision, 0);
    }

    #[test]
    fn observer_authority_cannot_admit_a_mutation_command() {
        let mut session = build_session(1);
        let mut request = scripted_command_requests(&session).remove(0);
        request.authority = CommandAuthority::Observer;
        let before = session.world.snapshot();

        let result = session.enqueue(request);

        assert_eq!(result.disposition, CommandDisposition::RejectedUnauthorized);
        assert_eq!(result.diagnostic.unwrap().code, "command_authority_denied");
        assert_eq!(session.world.snapshot(), before);
        assert!(session.apply_pending_at_tick(1).results.is_empty());
    }

    #[test]
    fn non_finite_move_input_is_rejected_without_mutating_state() {
        let mut session = build_session(4);
        let before = session.world.snapshot();
        let request = CommandRequest {
            id: 88,
            target: session.arm.0,
            authority: CommandAuthority::Operator,
            expected_revision: Some(session.revision),
            command: RuntimeCommand::MoveBy {
                delta: Position {
                    x: f32::NAN,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };

        assert_eq!(
            session.enqueue(request).disposition,
            CommandDisposition::Queued
        );
        let trace = session.apply_pending_at_tick(1);

        assert_eq!(trace.results.len(), 1);
        assert_eq!(
            trace.results[0].disposition,
            CommandDisposition::RejectedInvalid
        );
        assert_eq!(
            trace.results[0].diagnostic.as_ref().unwrap().code,
            "invalid_move_delta"
        );
        assert_eq!(session.world.snapshot(), before);
        assert_eq!(session.revision, 0);
    }

    #[test]
    fn command_script_replays_to_identical_evidence() {
        fn run_script() -> (CommandTrace, Vec<u8>) {
            let mut session = build_session(8);
            for request in scripted_command_requests(&session) {
                session.enqueue(request);
            }
            let trace = session.apply_pending_at_tick(4);
            let evidence = serialize_observation(&observe_world(
                &session.world,
                ObservationContext {
                    sequence: 2,
                    tick: session.tick,
                    revision: session.revision,
                },
                Some(session.arm),
                ObservationLimits::default(),
            ))
            .unwrap();
            (trace, evidence)
        }

        assert_eq!(run_script(), run_script());
    }
}
