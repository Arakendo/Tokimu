//! Provider-neutral, read-only shell semantics for corpus consumers.
//!
//! This is intentionally an incubating corpus library. Hosts such as Ratatui,
//! native terminals, and browser islands may project its results, but must not
//! replace its owner-qualified command and observation contracts.

use serde::Serialize;
use tokimu_core::world::WorldSnapshot;
use tokimu_core::{DiagnosticKind, DiagnosticSeverity, Diagnostics, PerformanceUnit, World};

const DEFAULT_HISTORY_LIMIT: usize = 64;
const DEFAULT_NAVIGATION_LIMIT: usize = 16;
const DEFAULT_WATCH_LIMIT: usize = 8;
const DEFAULT_MAX_INPUT_BYTES: usize = 1024;
const DEFAULT_MAX_ARGUMENTS: usize = 16;
const DEFAULT_MAX_PROJECTION_BYTES: usize = 16 * 1024;
const DEFAULT_MAX_COMMANDS_PER_SEQUENCE: usize = 16;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ObservationSource {
    pub world: WorldObservation,
    pub diagnostics: DiagnosticsObservation,
}

impl ObservationSource {
    /// Copies observable state. The shell retains no reference to mutable world truth.
    pub fn from_world_and_diagnostics(world: &World, diagnostics: &Diagnostics) -> Self {
        Self {
            world: WorldObservation::from_snapshot(&world.snapshot()),
            diagnostics: DiagnosticsObservation::from_diagnostics(diagnostics),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorldObservation {
    pub revision: u64,
    pub entities: Vec<EntityObservation>,
    pub component_types: Vec<TypeObservation>,
    pub resource_types: Vec<TypeObservation>,
    pub relationship_types: Vec<RelationshipObservation>,
}

impl WorldObservation {
    pub fn from_snapshot(snapshot: &WorldSnapshot) -> Self {
        Self {
            revision: 0,
            entities: snapshot
                .entities
                .iter()
                .map(|entity| EntityObservation { id: entity.id.0 })
                .collect(),
            component_types: snapshot
                .component_types
                .iter()
                .map(TypeObservation::from_snapshot)
                .collect(),
            resource_types: snapshot
                .resource_types
                .iter()
                .map(TypeObservation::from_snapshot)
                .collect(),
            relationship_types: snapshot
                .relationship_types
                .iter()
                .map(|relation| RelationshipObservation {
                    type_name: relation.type_name.to_owned(),
                    edges: relation
                        .edges
                        .iter()
                        .map(|(source, targets)| RelationshipEdgeObservation {
                            source: source.0,
                            targets: targets.iter().map(|target| target.0).collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntityObservation {
    pub id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TypeObservation {
    pub type_name: String,
    pub count: usize,
}

impl TypeObservation {
    fn from_snapshot(snapshot: &tokimu_core::world::TypeSnapshot) -> Self {
        Self {
            type_name: snapshot.type_name.to_owned(),
            count: snapshot.count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RelationshipObservation {
    pub type_name: String,
    pub edges: Vec<RelationshipEdgeObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RelationshipEdgeObservation {
    pub source: u64,
    pub targets: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagnosticsObservation {
    pub dropped_records: u64,
    pub records: Vec<DiagnosticObservation>,
}

impl DiagnosticsObservation {
    fn from_diagnostics(diagnostics: &Diagnostics) -> Self {
        Self {
            dropped_records: diagnostics.dropped_records(),
            records: diagnostics
                .records()
                .iter()
                .map(|record| DiagnosticObservation {
                    sequence: record.sequence(),
                    severity: diagnostic_severity_name(record.severity).to_owned(),
                    kind: diagnostic_kind_name(record.kind).to_owned(),
                    source: record.source.clone(),
                    message: record.message.clone(),
                    performance: record.performance.as_ref().map(|performance| {
                        PerformanceDiagnosticObservation {
                            metric: performance.metric.clone(),
                            observed: performance.observed,
                            budget: performance.budget,
                            unit: performance_unit_name(performance.unit).to_owned(),
                        }
                    }),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagnosticObservation {
    pub sequence: u64,
    pub severity: String,
    pub kind: String,
    pub source: String,
    pub message: String,
    pub performance: Option<PerformanceDiagnosticObservation>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PerformanceDiagnosticObservation {
    pub metric: String,
    pub observed: f64,
    pub budget: f64,
    pub unit: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProjectionFormat {
    #[default]
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellStatus {
    Success,
    ParseFailure,
    BudgetExceeded,
    Unsupported,
    OwnerFailure,
    SessionFailure,
    Unavailable,
    Unauthorized,
}

/// The authority a caller grants to one shell session.
///
/// Read-only sessions may discover registered mutations but never invoke their
/// handlers. Control sessions still require an invocation-local, caller-owned
/// handler; this enum never grants direct access to world state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellAuthority {
    ReadOnly,
    #[default]
    Control,
}

/// Fixed limits that protect one shell session from unbounded host input.
///
/// The source owner remains responsible for bounding the observations it
/// supplies. These limits bound the shell's own retained input, command
/// envelope, and projected transcript record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellBoundaryLimits {
    pub max_input_bytes: usize,
    pub max_arguments: usize,
    pub max_projection_bytes: usize,
    pub max_commands_per_sequence: usize,
}

impl Default for ShellBoundaryLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_arguments: DEFAULT_MAX_ARGUMENTS,
            max_projection_bytes: DEFAULT_MAX_PROJECTION_BYTES,
            max_commands_per_sequence: DEFAULT_MAX_COMMANDS_PER_SEQUENCE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellCommandKind {
    ShellMeta,
    SemanticQuery,
    Mutation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandAvailability {
    Available,
    Deferred,
    RegisteredWithoutHandler,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandResultKind {
    Catalog,
    Observation,
    Session,
    MutationReceipt,
}

/// The bounded source summary a shell watch may request.
///
/// Watches intentionally target copied observations rather than live engine
/// objects. More detailed application-defined watches remain owner adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchTarget {
    World,
    Diagnostics,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WatchSubscription {
    pub id: u64,
    pub target: WatchTarget,
    /// Logical observation-sequence interval, not wall-clock time.
    pub interval: u64,
    pub next_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WatchSummary {
    pub target: WatchTarget,
    pub revision: Option<u64>,
    pub entity_count: Option<usize>,
    pub diagnostic_count: Option<usize>,
    pub dropped_diagnostics: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WatchRefresh {
    pub watch_id: u64,
    pub sequence: u64,
    pub unchanged: bool,
    /// Watch summaries are fixed-size in v1 and therefore never truncate.
    pub truncated: bool,
    pub summary: WatchSummary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellSessionState {
    Open,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObservationContext {
    World,
    Entity { entity_id: u64 },
    Diagnostics,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ShellResponse {
    pub owner: String,
    pub command: String,
    pub status: ShellStatus,
    pub data: ShellData,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShellData {
    Help {
        commands: Vec<CommandDescription>,
    },
    World {
        world: WorldObservation,
    },
    Entities {
        entities: Vec<EntityObservation>,
    },
    Entity {
        entity: EntityObservation,
    },
    Relationships {
        entity_id: u64,
        relationships: Vec<RelationshipResult>,
    },
    Diagnostics {
        diagnostics: DiagnosticsObservation,
    },
    Format {
        format: String,
    },
    Context {
        state: ShellSessionState,
        current: ObservationContext,
        navigation_depth: usize,
    },
    Cleared {
        removed_records: usize,
    },
    Closed {
        released_history_records: usize,
        released_navigation_entries: usize,
        released_watches: usize,
    },
    WatchAdded {
        watch: WatchSubscription,
    },
    Watches {
        watches: Vec<WatchSubscription>,
    },
    WatchCancelled {
        watch: WatchSubscription,
    },
    ApplicationMutation {
        invocation: ApplicationCommandInvocation,
        receipt: ApplicationMutationReceipt,
    },
    ApplicationQuery {
        invocation: ApplicationCommandInvocation,
        result: ApplicationQueryResult,
    },
    Failure {
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommandDescription {
    pub command: String,
    pub owner: String,
    pub summary: String,
    pub kind: ShellCommandKind,
    pub arguments: Vec<String>,
    pub result_kind: CommandResultKind,
    pub availability: CommandAvailability,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApplicationCommandDescription {
    pub owner: String,
    pub command: String,
    pub summary: String,
    pub kind: ShellCommandKind,
    pub arguments: Vec<String>,
    pub result_kind: CommandResultKind,
}

impl ApplicationCommandDescription {
    pub fn query(
        owner: impl Into<String>,
        command: impl Into<String>,
        summary: impl Into<String>,
        arguments: Vec<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            command: command.into(),
            summary: summary.into(),
            kind: ShellCommandKind::SemanticQuery,
            arguments,
            result_kind: CommandResultKind::Observation,
        }
    }

    pub fn mutation(
        owner: impl Into<String>,
        command: impl Into<String>,
        summary: impl Into<String>,
        arguments: Vec<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            command: command.into(),
            summary: summary.into(),
            kind: ShellCommandKind::Mutation,
            arguments,
            result_kind: CommandResultKind::MutationReceipt,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandRegistrationError {
    EmptyOwner,
    EmptyCommand,
    Duplicate { owner: String, command: String },
}

/// A bounded, owner-qualified application invocation parsed by the shell.
///
/// The shell validates only this stable envelope. It deliberately does not
/// interpret application-specific arguments or invoke an application handler.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApplicationCommandInvocation {
    pub owner: String,
    pub command: String,
    pub arguments: Vec<String>,
}

/// A bounded mutation outcome supplied by an application-owned adapter.
///
/// The shell projects this receipt but never interprets its arguments, owns a
/// runtime, or mutates world truth itself.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApplicationMutationReceipt {
    pub accepted: bool,
    pub applied_tick: Option<u64>,
    pub resulting_revision: Option<u64>,
    pub message: String,
}

/// A compact, provider-owned read-only observation projected by the shell.
///
/// The shell limits this to named scalar fields so a caller cannot use the
/// command boundary to retain an unbounded private object graph. Applications
/// still choose the fields and own their meaning.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApplicationQueryResult {
    pub summary: String,
    pub fields: Vec<ApplicationQueryField>,
}

/// One named field in an application-owned query observation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApplicationQueryField {
    pub name: String,
    pub disclosure: ApplicationQueryFieldDisclosure,
}

impl ApplicationQueryField {
    /// Creates a field whose owner permits its scalar value to be projected.
    pub fn visible(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            disclosure: ApplicationQueryFieldDisclosure::Visible {
                value: value.into(),
            },
        }
    }

    /// Creates a field whose owner intentionally withholds its source value.
    ///
    /// The reason is evidence for callers; the omitted value never crosses the
    /// application-to-shell observation boundary.
    pub fn redacted(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            disclosure: ApplicationQueryFieldDisclosure::Redacted {
                reason: reason.into(),
            },
        }
    }
}

/// An owner-supplied decision about whether a query field can be projected.
///
/// The shell faithfully carries this decision but never classifies data or
/// chooses which fields should be hidden.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "visibility", rename_all = "snake_case")]
pub enum ApplicationQueryFieldDisclosure {
    Visible { value: String },
    Redacted { reason: String },
}

/// The bounded outcome an application supplies for one routed invocation.
///
/// This remains an invocation-local adapter value. The shell stores no owner
/// implementation or mutable owner state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApplicationCommandResult {
    Query { result: ApplicationQueryResult },
    Mutation { receipt: ApplicationMutationReceipt },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationCommandParseError {
    MissingOwner,
    MissingCommand,
}

pub fn parse_application_command(
    tokens: &[&str],
) -> Result<ApplicationCommandInvocation, ApplicationCommandParseError> {
    match tokens {
        ["application", owner, command, arguments @ ..] => Ok(ApplicationCommandInvocation {
            owner: (*owner).to_owned(),
            command: (*command).to_owned(),
            arguments: arguments
                .iter()
                .map(|argument| (*argument).to_owned())
                .collect(),
        }),
        ["application"] => Err(ApplicationCommandParseError::MissingOwner),
        ["application", _] => Err(ApplicationCommandParseError::MissingCommand),
        _ => Err(ApplicationCommandParseError::MissingCommand),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RelationshipResult {
    pub type_name: String,
    pub targets: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ShellRecord {
    pub input: String,
    pub response: ShellResponse,
    pub projection: String,
}

#[derive(Clone, Debug)]
pub struct ObservationShell {
    authority: ShellAuthority,
    boundary_limits: ShellBoundaryLimits,
    format: ProjectionFormat,
    history_limit: usize,
    history: Vec<ShellRecord>,
    state: ShellSessionState,
    current_context: ObservationContext,
    navigation_limit: usize,
    navigation: Vec<ObservationContext>,
    application_commands: Vec<ApplicationCommandDescription>,
    watch_limit: usize,
    watches: Vec<WatchSubscription>,
    next_watch_id: u64,
    watch_fingerprints: Vec<(u64, WatchSummary)>,
    current_sequence: u64,
    commands_in_current_sequence: usize,
}

impl Default for ObservationShell {
    fn default() -> Self {
        Self::new(DEFAULT_HISTORY_LIMIT)
    }
}

impl ObservationShell {
    pub fn new(history_limit: usize) -> Self {
        Self::with_limits(history_limit, DEFAULT_NAVIGATION_LIMIT)
    }

    pub fn with_limits(history_limit: usize, navigation_limit: usize) -> Self {
        Self::with_session_limits(history_limit, navigation_limit, DEFAULT_WATCH_LIMIT)
    }

    pub fn with_session_limits(
        history_limit: usize,
        navigation_limit: usize,
        watch_limit: usize,
    ) -> Self {
        Self::with_authority_and_boundary_limits(
            ShellAuthority::Control,
            history_limit,
            navigation_limit,
            watch_limit,
            ShellBoundaryLimits::default(),
        )
    }

    pub fn read_only(history_limit: usize) -> Self {
        Self::with_authority_and_boundary_limits(
            ShellAuthority::ReadOnly,
            history_limit,
            DEFAULT_NAVIGATION_LIMIT,
            DEFAULT_WATCH_LIMIT,
            ShellBoundaryLimits::default(),
        )
    }

    pub fn with_authority_and_boundary_limits(
        authority: ShellAuthority,
        history_limit: usize,
        navigation_limit: usize,
        watch_limit: usize,
        boundary_limits: ShellBoundaryLimits,
    ) -> Self {
        Self {
            authority,
            boundary_limits,
            format: ProjectionFormat::Text,
            history_limit,
            history: Vec::new(),
            state: ShellSessionState::Open,
            current_context: ObservationContext::World,
            navigation_limit,
            navigation: Vec::new(),
            application_commands: Vec::new(),
            watch_limit,
            watches: Vec::new(),
            next_watch_id: 1,
            watch_fingerprints: Vec::new(),
            current_sequence: 0,
            commands_in_current_sequence: 0,
        }
    }

    pub fn authority(&self) -> ShellAuthority {
        self.authority
    }

    pub fn boundary_limits(&self) -> ShellBoundaryLimits {
        self.boundary_limits
    }

    pub fn format(&self) -> ProjectionFormat {
        self.format
    }

    pub fn history(&self) -> &[ShellRecord] {
        &self.history
    }

    pub fn state(&self) -> ShellSessionState {
        self.state
    }

    pub fn current_context(&self) -> &ObservationContext {
        &self.current_context
    }

    pub fn navigation_depth(&self) -> usize {
        self.navigation.len()
    }

    pub fn watches(&self) -> &[WatchSubscription] {
        &self.watches
    }

    /// Refreshes due subscriptions from one caller-provided observation snapshot.
    ///
    /// The sequence belongs to the runtime/application caller. This method
    /// stores neither the source nor a live world reference, and skipped caller
    /// sequences coalesce into the next refresh instead of creating a queue.
    pub fn refresh_watches(
        &mut self,
        source: &ObservationSource,
        sequence: u64,
    ) -> Vec<WatchRefresh> {
        let mut refreshes = Vec::new();
        for watch in &mut self.watches {
            if sequence < watch.next_sequence {
                continue;
            }

            let summary = watch_summary(watch.target, source);
            let unchanged = self
                .watch_fingerprints
                .iter()
                .find(|(id, _)| *id == watch.id)
                .is_some_and(|(_, previous)| previous == &summary);
            if let Some((_, previous)) = self
                .watch_fingerprints
                .iter_mut()
                .find(|(id, _)| *id == watch.id)
            {
                *previous = summary.clone();
            } else {
                self.watch_fingerprints.push((watch.id, summary.clone()));
            }

            // Advance from the caller's current sequence. Missed sequences are
            // coalesced, so a slow terminal or browser never accumulates work.
            watch.next_sequence = sequence.saturating_add(watch.interval);
            refreshes.push(WatchRefresh {
                watch_id: watch.id,
                sequence,
                unchanged,
                truncated: false,
                summary,
            });
        }
        refreshes
    }

    /// Registers bounded application command metadata without attaching an executor.
    ///
    /// Discovery is intentionally separate from execution authority. A later
    /// application-owned adapter may bind a registered command to its own
    /// validated handler without changing this shell's observation parser.
    pub fn register_application_command(
        &mut self,
        command: ApplicationCommandDescription,
    ) -> Result<(), CommandRegistrationError> {
        let owner = command.owner.trim();
        if owner.is_empty() {
            return Err(CommandRegistrationError::EmptyOwner);
        }
        let name = command.command.trim();
        if name.is_empty() {
            return Err(CommandRegistrationError::EmptyCommand);
        }
        if self.application_commands.iter().any(|existing| {
            existing.owner.eq_ignore_ascii_case(owner)
                && existing.command.eq_ignore_ascii_case(name)
        }) {
            return Err(CommandRegistrationError::Duplicate {
                owner: owner.to_owned(),
                command: name.to_owned(),
            });
        }
        self.application_commands
            .push(ApplicationCommandDescription {
                owner: owner.to_owned(),
                command: name.to_owned(),
                summary: command.summary,
                kind: command.kind,
                arguments: command.arguments,
                result_kind: command.result_kind,
            });
        Ok(())
    }

    /// Returns the immutable, application-owned command declarations known to
    /// this shell session. Hosts may project this catalog for discovery, but
    /// must still route invocation through the shell boundary.
    pub fn application_commands(&self) -> &[ApplicationCommandDescription] {
        &self.application_commands
    }

    pub fn execute(&mut self, source: &ObservationSource, input: &str) -> ShellRecord {
        self.current_sequence = self.current_sequence.saturating_add(1);
        self.commands_in_current_sequence = 0;
        self.execute_at_sequence_inner(source, input, self.current_sequence, None)
    }

    /// Routes a command at a caller-owned logical sequence.
    ///
    /// Hosts that receive several commands in one frame, message batch, or
    /// browser turn should use this method so the shell can reject a bounded
    /// command flood deterministically without depending on wall-clock time.
    pub fn execute_at_sequence(
        &mut self,
        source: &ObservationSource,
        input: &str,
        sequence: u64,
    ) -> ShellRecord {
        self.execute_at_sequence_inner(source, input, sequence, None)
    }

    /// Routes one registered application command at a caller-owned logical
    /// sequence through an invocation-local owner adapter.
    ///
    /// This combines deterministic host sequencing with the same ownership
    /// boundary as [`Self::execute_with_application_handler`]. Browser and
    /// message-driven hosts can therefore retain their transport sequence
    /// without parsing application arguments or retaining runtime state.
    pub fn execute_at_sequence_with_application_handler<F>(
        &mut self,
        source: &ObservationSource,
        input: &str,
        sequence: u64,
        mut handler: F,
    ) -> ShellRecord
    where
        F: FnMut(&ApplicationCommandInvocation) -> ApplicationCommandResult,
    {
        self.execute_at_sequence_inner(source, input, sequence, Some(&mut handler))
    }

    /// Retains a bounded application-owned observation initiated by a host
    /// control rather than typed shell input.
    ///
    /// The host still owns its interaction affordance, while the shell owns
    /// the retained transcript record and its provider-qualified semantics.
    /// This keeps browser controls visible in terminal evidence without
    /// misrepresenting them as user-entered commands.
    pub fn record_application_query_at_sequence(
        &mut self,
        input: &str,
        sequence: u64,
        invocation: ApplicationCommandInvocation,
        result: ApplicationQueryResult,
    ) -> ShellRecord {
        if sequence != self.current_sequence {
            self.current_sequence = sequence;
            self.commands_in_current_sequence = 0;
        }
        self.commands_in_current_sequence = self.commands_in_current_sequence.saturating_add(1);

        let input = input.trim();
        let mut response = if input.len() > self.boundary_limits.max_input_bytes {
            budget_exceeded(
                "shell",
                "input",
                format!(
                    "input exceeds the shell byte limit ({} bytes)",
                    self.boundary_limits.max_input_bytes
                ),
            )
        } else if self.commands_in_current_sequence > self.boundary_limits.max_commands_per_sequence
        {
            budget_exceeded(
                "shell",
                "command rate",
                format!(
                    "logical sequence {sequence} exceeded the shell command budget ({})",
                    self.boundary_limits.max_commands_per_sequence
                ),
            )
        } else {
            let owner = invocation.owner.clone();
            let command = format!("application {} {}", invocation.owner, invocation.command);
            success(
                &owner,
                &command,
                ShellData::ApplicationQuery { invocation, result },
            )
        };
        let mut projection = project(&response, self.format);
        if projection.len() > self.boundary_limits.max_projection_bytes {
            response = budget_exceeded(
                "shell",
                "projection",
                format!(
                    "response projection exceeds the shell output limit ({} bytes)",
                    self.boundary_limits.max_projection_bytes
                ),
            );
            projection = project(&response, self.format);
        }
        let record = ShellRecord {
            input: input.to_owned(),
            response,
            projection,
        };

        if self.state != ShellSessionState::Closed {
            self.history.push(record.clone());
            if self.history.len() > self.history_limit {
                self.history.remove(0);
            }
        }
        record
    }

    /// Routes one registered mutation through a caller-supplied application adapter.
    ///
    /// The handler exists for this invocation only. The shell retains neither
    /// the handler nor a live runtime reference, so mutation authority remains
    /// with the application that explicitly supplied this boundary.
    pub fn execute_with_mutation_handler<F>(
        &mut self,
        source: &ObservationSource,
        input: &str,
        mut handler: F,
    ) -> ShellRecord
    where
        F: FnMut(&ApplicationCommandInvocation) -> ApplicationMutationReceipt,
    {
        self.current_sequence = self.current_sequence.saturating_add(1);
        self.commands_in_current_sequence = 0;
        self.execute_at_sequence_inner(
            source,
            input,
            self.current_sequence,
            Some(&mut |invocation| ApplicationCommandResult::Mutation {
                receipt: handler(invocation),
            }),
        )
    }

    /// Routes one registered application query or mutation through a
    /// caller-supplied owner adapter.
    ///
    /// The handler can project a bounded query result or mutation receipt, but
    /// the shell never retains it or receives the owner's implementation.
    pub fn execute_with_application_handler<F>(
        &mut self,
        source: &ObservationSource,
        input: &str,
        mut handler: F,
    ) -> ShellRecord
    where
        F: FnMut(&ApplicationCommandInvocation) -> ApplicationCommandResult,
    {
        self.current_sequence = self.current_sequence.saturating_add(1);
        self.commands_in_current_sequence = 0;
        self.execute_at_sequence_inner(source, input, self.current_sequence, Some(&mut handler))
    }

    fn execute_at_sequence_inner(
        &mut self,
        source: &ObservationSource,
        input: &str,
        sequence: u64,
        handler: Option<&mut dyn FnMut(&ApplicationCommandInvocation) -> ApplicationCommandResult>,
    ) -> ShellRecord {
        if sequence != self.current_sequence {
            self.current_sequence = sequence;
            self.commands_in_current_sequence = 0;
        }
        self.commands_in_current_sequence = self.commands_in_current_sequence.saturating_add(1);

        let input = input.trim();
        let mut response = if input.len() > self.boundary_limits.max_input_bytes {
            budget_exceeded(
                "shell",
                "input",
                format!(
                    "input exceeds the shell byte limit ({} bytes)",
                    self.boundary_limits.max_input_bytes
                ),
            )
        } else if self.commands_in_current_sequence > self.boundary_limits.max_commands_per_sequence
        {
            budget_exceeded(
                "shell",
                "command rate",
                format!(
                    "logical sequence {sequence} exceeded the shell command budget ({})",
                    self.boundary_limits.max_commands_per_sequence
                ),
            )
        } else {
            self.route(source, input, handler)
        };
        let mut projection = project(&response, self.format);
        if projection.len() > self.boundary_limits.max_projection_bytes {
            response = budget_exceeded(
                "shell",
                "projection",
                format!(
                    "response projection exceeds the shell output limit ({} bytes)",
                    self.boundary_limits.max_projection_bytes
                ),
            );
            projection = project(&response, self.format);
        }
        let record = ShellRecord {
            input: input.to_owned(),
            response,
            projection,
        };

        // Closing returns a final report but deliberately retains no session history.
        if self.state == ShellSessionState::Closed {
            return record;
        }

        self.history.push(record.clone());
        if self.history.len() > self.history_limit {
            self.history.remove(0);
        }
        record
    }

    fn route(
        &mut self,
        source: &ObservationSource,
        input: &str,
        handler: Option<&mut dyn FnMut(&ApplicationCommandInvocation) -> ApplicationCommandResult>,
    ) -> ShellResponse {
        let tokens = input
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>();
        let command = tokens.join(" ");

        if self.state == ShellSessionState::Closed {
            return session_failure(
                "shell",
                command,
                "session is closed; create a new shell session before issuing commands",
            );
        }

        let token_refs = tokens.iter().map(String::as_str).collect::<Vec<_>>();
        if matches!(token_refs.as_slice(), ["application", ..])
            && token_refs.len().saturating_sub(3) > self.boundary_limits.max_arguments
        {
            return budget_exceeded(
                "shell",
                "application arguments",
                format!(
                    "application command exceeds the shell argument limit ({} arguments)",
                    self.boundary_limits.max_arguments
                ),
            );
        }
        let permits_stale_context = matches!(
            token_refs.as_slice(),
            ["help"]
                | ["clear"]
                | ["close"]
                | ["select", ..]
                | ["watch", ..]
                | ["unwatch", ..]
                | ["list", "watches"]
        );
        if !permits_stale_context {
            if let Some(message) = self.stale_context_message(source) {
                return owner_failure("runtime-observation", &command, message);
            }
        }
        match token_refs.as_slice() {
            ["help"] => success(
                "shell",
                "help",
                ShellData::Help {
                    commands: command_catalog(&self.application_commands),
                },
            ),
            ["inspect", "world"] => success(
                "runtime-observation",
                "inspect world",
                ShellData::World {
                    world: source.world.clone(),
                },
            ),
            ["list", "entities"] => success(
                "runtime-observation",
                "list entities",
                ShellData::Entities {
                    entities: source.world.entities.clone(),
                },
            ),
            ["inspect", "entity", raw_id] => match parse_entity_id(raw_id) {
                Ok(entity_id) => match source
                    .world
                    .entities
                    .iter()
                    .find(|entity| entity.id == entity_id)
                {
                    Some(entity) => success(
                        "runtime-observation",
                        "inspect entity",
                        ShellData::Entity {
                            entity: entity.clone(),
                        },
                    ),
                    None => owner_failure(
                        "runtime-observation",
                        "inspect entity",
                        format!("entity {entity_id} is absent from the owner observation"),
                    ),
                },
                Err(message) => parse_failure("shell", "inspect entity", message),
            },
            ["list", "relationships", raw_id] => match parse_entity_id(raw_id) {
                Ok(entity_id) => {
                    if !source
                        .world
                        .entities
                        .iter()
                        .any(|entity| entity.id == entity_id)
                    {
                        return owner_failure(
                            "runtime-observation",
                            "list relationships",
                            format!("entity {entity_id} is absent from the owner observation"),
                        );
                    }
                    let relationships = source
                        .world
                        .relationship_types
                        .iter()
                        .filter_map(|relation| {
                            relation
                                .edges
                                .iter()
                                .find(|edge| edge.source == entity_id)
                                .map(|edge| RelationshipResult {
                                    type_name: relation.type_name.clone(),
                                    targets: edge.targets.clone(),
                                })
                        })
                        .collect();
                    success(
                        "runtime-observation",
                        "list relationships",
                        ShellData::Relationships {
                            entity_id,
                            relationships,
                        },
                    )
                }
                Err(message) => parse_failure("shell", "list relationships", message),
            },
            ["observe", "diagnostics"] => success(
                "diagnostics",
                "observe diagnostics",
                ShellData::Diagnostics {
                    diagnostics: source.diagnostics.clone(),
                },
            ),
            ["format", "text"] => {
                self.format = ProjectionFormat::Text;
                success(
                    "shell",
                    "format text",
                    ShellData::Format {
                        format: "text".to_owned(),
                    },
                )
            }
            ["format", "json"] => {
                self.format = ProjectionFormat::Json;
                success(
                    "shell",
                    "format json",
                    ShellData::Format {
                        format: "json".to_owned(),
                    },
                )
            }
            ["context"] => success(
                "shell",
                "context",
                ShellData::Context {
                    state: self.state,
                    current: self.current_context.clone(),
                    navigation_depth: self.navigation.len(),
                },
            ),
            ["select", "entity", raw_id] => match parse_entity_id(raw_id) {
                Ok(entity_id) => match source
                    .world
                    .entities
                    .iter()
                    .find(|entity| entity.id == entity_id)
                {
                    Some(_) => {
                        self.push_context(ObservationContext::Entity { entity_id });
                        self.context_response("select entity")
                    }
                    None => owner_failure(
                        "runtime-observation",
                        "select entity",
                        format!("entity {entity_id} is absent from the owner observation"),
                    ),
                },
                Err(message) => parse_failure("shell", "select entity", message),
            },
            ["select", "world"] => {
                self.push_context(ObservationContext::World);
                self.context_response("select world")
            }
            ["select", "diagnostics"] => {
                self.push_context(ObservationContext::Diagnostics);
                self.context_response("select diagnostics")
            }
            ["back"] => match self.navigation.pop() {
                Some(context) => {
                    self.current_context = context;
                    match self.stale_context_message(source) {
                        Some(message) => owner_failure("runtime-observation", "back", message),
                        None => self.context_response("back"),
                    }
                }
                None => {
                    session_failure("shell", "back", "no prior observation context is available")
                }
            },
            ["clear"] => {
                let removed_records = self.history.len();
                self.history.clear();
                success("shell", "clear", ShellData::Cleared { removed_records })
            }
            ["close"] => {
                let released_history_records = self.history.len();
                let released_navigation_entries = self.navigation.len();
                let released_watches = self.watches.len();
                self.history.clear();
                self.navigation.clear();
                self.watches.clear();
                self.watch_fingerprints.clear();
                self.state = ShellSessionState::Closed;
                success(
                    "shell",
                    "close",
                    ShellData::Closed {
                        released_history_records,
                        released_navigation_entries,
                        released_watches,
                    },
                )
            }
            ["watch", raw_target] => self.add_watch(raw_target, 1),
            ["watch", raw_target, raw_interval] => match parse_watch_interval(raw_interval) {
                Ok(interval) => self.add_watch(raw_target, interval),
                Err(message) => parse_failure("shell", "watch", message),
            },
            ["watch", ..] => parse_failure(
                "shell",
                "watch",
                "expected `watch world|diagnostics [sequence_interval]`",
            ),
            ["unwatch", raw_id] => match parse_watch_id(raw_id) {
                Ok(id) => self.cancel_watch(id),
                Err(message) => parse_failure("shell", "unwatch", message),
            },
            ["unwatch", ..] => parse_failure("shell", "unwatch", "expected `unwatch <watch_id>`"),
            ["list", "watches"] => success(
                "shell",
                "list watches",
                ShellData::Watches {
                    watches: self.watches.clone(),
                },
            ),
            ["application", ..] => match parse_application_command(&token_refs) {
                Ok(invocation) => self.route_registered_application_command(&invocation, handler),
                Err(ApplicationCommandParseError::MissingOwner) => parse_failure(
                    "shell",
                    "application",
                    "expected an owner-qualified command",
                ),
                Err(ApplicationCommandParseError::MissingCommand) => parse_failure(
                    "shell",
                    "application",
                    "expected `application <owner> <command> [arguments...]`",
                ),
            },
            [] => parse_failure("shell", "", "expected a command"),
            _ if command.starts_with("mutate") => unsupported(
                "shell",
                command,
                "mutation commands are deferred to a later corpus slice",
            ),
            _ => unsupported(
                "shell",
                command,
                "unknown command; run `help` for the bounded read-only catalog",
            ),
        }
    }

    fn push_context(&mut self, next: ObservationContext) {
        if self.current_context == next {
            return;
        }
        self.navigation.push(self.current_context.clone());
        if self.navigation.len() > self.navigation_limit {
            self.navigation.remove(0);
        }
        self.current_context = next;
    }

    fn add_watch(&mut self, raw_target: &str, interval: u64) -> ShellResponse {
        let target = match parse_watch_target(raw_target) {
            Ok(target) => target,
            Err(message) => return parse_failure("shell", "watch", message),
        };
        if self.watches.len() >= self.watch_limit {
            return unavailable(
                "shell",
                "watch",
                format!("active watch limit ({}) has been reached", self.watch_limit),
            );
        }
        let watch = WatchSubscription {
            id: self.next_watch_id,
            target,
            interval,
            next_sequence: 0,
        };
        self.next_watch_id = self.next_watch_id.saturating_add(1);
        self.watches.push(watch.clone());
        success("shell", "watch", ShellData::WatchAdded { watch })
    }

    fn cancel_watch(&mut self, id: u64) -> ShellResponse {
        match self.watches.iter().position(|watch| watch.id == id) {
            Some(index) => {
                let watch = self.watches.remove(index);
                self.watch_fingerprints
                    .retain(|(watch_id, _)| *watch_id != id);
                success("shell", "unwatch", ShellData::WatchCancelled { watch })
            }
            None => unavailable("shell", "unwatch", format!("watch {id} is not active")),
        }
    }

    fn context_response(&self, command: &str) -> ShellResponse {
        success(
            "shell",
            command,
            ShellData::Context {
                state: self.state,
                current: self.current_context.clone(),
                navigation_depth: self.navigation.len(),
            },
        )
    }

    fn stale_context_message(&self, source: &ObservationSource) -> Option<String> {
        match &self.current_context {
            ObservationContext::Entity { entity_id }
                if !source.world.entities.iter().any(|entity| entity.id == *entity_id) =>
            {
                Some(format!(
                    "current context entity {entity_id} is absent from the refreshed owner observation"
                ))
            }
            ObservationContext::World | ObservationContext::Entity { .. } | ObservationContext::Diagnostics => {
                None
            }
        }
    }

    fn route_registered_application_command(
        &self,
        invocation: &ApplicationCommandInvocation,
        handler: Option<&mut dyn FnMut(&ApplicationCommandInvocation) -> ApplicationCommandResult>,
    ) -> ShellResponse {
        let command = format!("application {} {}", invocation.owner, invocation.command);
        match self.application_commands.iter().find(|entry| {
            entry
                .owner
                .eq_ignore_ascii_case(&invocation.owner)
                && entry
                    .command
                    .eq_ignore_ascii_case(&invocation.command)
        }) {
            Some(entry) => match handler {
                Some(_) if entry.kind == ShellCommandKind::Mutation && self.authority == ShellAuthority::ReadOnly => {
                    unauthorized(
                        &entry.owner,
                        &command,
                        "this read-only shell session cannot invoke application mutations",
                    )
                }
                Some(handler) => match (entry.kind, handler(invocation)) {
                    (
                        ShellCommandKind::SemanticQuery,
                        ApplicationCommandResult::Query { result },
                    ) => success(
                        &entry.owner,
                        &command,
                        ShellData::ApplicationQuery {
                            invocation: invocation.clone(),
                            result,
                        },
                    ),
                    (ShellCommandKind::Mutation, ApplicationCommandResult::Mutation { receipt }) => {
                        success(
                            &entry.owner,
                            &command,
                            ShellData::ApplicationMutation {
                                invocation: invocation.clone(),
                                receipt,
                            },
                        )
                    }
                    (expected, actual) => owner_failure(
                        &entry.owner,
                        &command,
                        format!(
                            "application handler returned {} for a {} command",
                            application_result_name(&actual),
                            command_kind_name(expected),
                        ),
                    ),
                },
                None if entry.kind == ShellCommandKind::Mutation => unauthorized(
                    &entry.owner,
                    &command,
                    "catalog discovery does not grant mutation authority; no application handler is attached",
                ),
                None => unavailable(
                    &entry.owner,
                    &command,
                    "the application command is registered for discovery but has no attached handler",
                ),
            },
            None => unavailable(
                "shell",
                &command,
                "no registered application command matches this owner-qualified identity",
            ),
        }
    }
}

fn command_catalog(
    application_commands: &[ApplicationCommandDescription],
) -> Vec<CommandDescription> {
    let mut commands = vec![
        builtin_command(
            "help",
            "shell",
            "List the bounded read-only command catalog.",
            ShellCommandKind::ShellMeta,
            &[],
            CommandResultKind::Catalog,
        ),
        builtin_command(
            "inspect world",
            "runtime-observation",
            "Inspect copied world structure.",
            ShellCommandKind::SemanticQuery,
            &[],
            CommandResultKind::Observation,
        ),
        builtin_command(
            "list entities",
            "runtime-observation",
            "List copied entity identities.",
            ShellCommandKind::SemanticQuery,
            &[],
            CommandResultKind::Observation,
        ),
        builtin_command(
            "inspect entity <id>",
            "runtime-observation",
            "Inspect one owner-observed entity.",
            ShellCommandKind::SemanticQuery,
            &["id"],
            CommandResultKind::Observation,
        ),
        builtin_command(
            "list relationships <id>",
            "runtime-observation",
            "List copied outgoing relationships.",
            ShellCommandKind::SemanticQuery,
            &["id"],
            CommandResultKind::Observation,
        ),
        builtin_command(
            "observe diagnostics",
            "diagnostics",
            "Inspect copied diagnostic records.",
            ShellCommandKind::SemanticQuery,
            &[],
            CommandResultKind::Observation,
        ),
        builtin_command(
            "format text|json",
            "shell",
            "Select a projection without changing observations.",
            ShellCommandKind::ShellMeta,
            &["format"],
            CommandResultKind::Session,
        ),
        builtin_command(
            "context",
            "shell",
            "Inspect session-local observation context.",
            ShellCommandKind::ShellMeta,
            &[],
            CommandResultKind::Session,
        ),
        builtin_command(
            "select world|diagnostics|entity <id>",
            "shell + runtime-observation",
            "Move bounded session context after owner identity validation.",
            ShellCommandKind::ShellMeta,
            &["context"],
            CommandResultKind::Session,
        ),
        builtin_command(
            "back",
            "shell",
            "Return to the prior bounded observation context.",
            ShellCommandKind::ShellMeta,
            &[],
            CommandResultKind::Session,
        ),
        builtin_command(
            "clear",
            "shell",
            "Clear session-local transcript history.",
            ShellCommandKind::ShellMeta,
            &[],
            CommandResultKind::Session,
        ),
        builtin_command(
            "close",
            "shell",
            "Release session-local history and navigation state.",
            ShellCommandKind::ShellMeta,
            &[],
            CommandResultKind::Session,
        ),
        builtin_command(
            "watch world|diagnostics [sequence_interval]",
            "shell",
            "Subscribe to a fixed-size copied observation summary at a logical sequence cadence.",
            ShellCommandKind::ShellMeta,
            &["target", "sequence_interval"],
            CommandResultKind::Session,
        ),
        builtin_command(
            "list watches",
            "shell",
            "List bounded session-local watch subscriptions.",
            ShellCommandKind::ShellMeta,
            &[],
            CommandResultKind::Session,
        ),
        builtin_command(
            "unwatch <id>",
            "shell",
            "Cancel one session-local watch subscription.",
            ShellCommandKind::ShellMeta,
            &["id"],
            CommandResultKind::Session,
        ),
    ];
    commands.extend(
        application_commands
            .iter()
            .map(|command| CommandDescription {
                command: format!("application {} {}", command.owner, command.command),
                owner: command.owner.clone(),
                summary: command.summary.clone(),
                kind: command.kind,
                arguments: command.arguments.clone(),
                result_kind: command.result_kind,
                availability: CommandAvailability::RegisteredWithoutHandler,
            }),
    );
    commands
}

fn builtin_command(
    command: &str,
    owner: &str,
    summary: &str,
    kind: ShellCommandKind,
    arguments: &[&str],
    result_kind: CommandResultKind,
) -> CommandDescription {
    CommandDescription {
        command: command.to_owned(),
        owner: owner.to_owned(),
        summary: summary.to_owned(),
        kind,
        arguments: arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect(),
        result_kind,
        availability: CommandAvailability::Available,
    }
}

fn parse_entity_id(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("entity id `{value}` must be an unsigned integer"))
}

fn parse_watch_id(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("watch id `{value}` must be an unsigned integer"))
}

fn parse_watch_interval(value: &str) -> Result<u64, String> {
    match value.parse::<u64>() {
        Ok(0) => Err("watch sequence interval must be greater than zero".to_owned()),
        Ok(interval) => Ok(interval),
        Err(_) => Err(format!(
            "watch sequence interval `{value}` must be an unsigned integer"
        )),
    }
}

fn parse_watch_target(value: &str) -> Result<WatchTarget, String> {
    match value {
        "world" => Ok(WatchTarget::World),
        "diagnostics" => Ok(WatchTarget::Diagnostics),
        _ => Err(format!(
            "watch target `{value}` is unsupported; use `world` or `diagnostics`"
        )),
    }
}

fn watch_summary(target: WatchTarget, source: &ObservationSource) -> WatchSummary {
    match target {
        WatchTarget::World => WatchSummary {
            target,
            revision: Some(source.world.revision),
            entity_count: Some(source.world.entities.len()),
            diagnostic_count: None,
            dropped_diagnostics: None,
        },
        WatchTarget::Diagnostics => WatchSummary {
            target,
            revision: source
                .diagnostics
                .records
                .last()
                .map(|record| record.sequence),
            entity_count: None,
            diagnostic_count: Some(source.diagnostics.records.len()),
            dropped_diagnostics: Some(source.diagnostics.dropped_records),
        },
    }
}

fn success(owner: &str, command: &str, data: ShellData) -> ShellResponse {
    ShellResponse {
        owner: owner.to_owned(),
        command: command.to_owned(),
        status: ShellStatus::Success,
        data,
    }
}

fn parse_failure(owner: &str, command: &str, message: impl Into<String>) -> ShellResponse {
    response_failure(owner, command, ShellStatus::ParseFailure, message)
}

fn budget_exceeded(owner: &str, command: &str, message: impl Into<String>) -> ShellResponse {
    response_failure(owner, command, ShellStatus::BudgetExceeded, message)
}

fn unsupported(owner: &str, command: String, message: impl Into<String>) -> ShellResponse {
    response_failure(owner, &command, ShellStatus::Unsupported, message)
}

fn unavailable(owner: &str, command: &str, message: impl Into<String>) -> ShellResponse {
    response_failure(owner, command, ShellStatus::Unavailable, message)
}

fn unauthorized(owner: &str, command: &str, message: impl Into<String>) -> ShellResponse {
    response_failure(owner, command, ShellStatus::Unauthorized, message)
}

fn owner_failure(owner: &str, command: &str, message: impl Into<String>) -> ShellResponse {
    response_failure(owner, command, ShellStatus::OwnerFailure, message)
}

fn session_failure(
    owner: &str,
    command: impl Into<String>,
    message: impl Into<String>,
) -> ShellResponse {
    let command = command.into();
    response_failure(owner, &command, ShellStatus::SessionFailure, message)
}

fn response_failure(
    owner: &str,
    command: &str,
    status: ShellStatus,
    message: impl Into<String>,
) -> ShellResponse {
    ShellResponse {
        owner: owner.to_owned(),
        command: command.to_owned(),
        status,
        data: ShellData::Failure {
            message: message.into(),
        },
    }
}

fn project(response: &ShellResponse, format: ProjectionFormat) -> String {
    match format {
        ProjectionFormat::Json => serde_json::to_string_pretty(response)
            .expect("observation shell responses are always serializable"),
        ProjectionFormat::Text => project_text(response),
    }
}

fn project_text(response: &ShellResponse) -> String {
    let header = format!(
        "[{}] {} ({})",
        response.owner,
        response.command,
        status_name(response.status)
    );
    match &response.data {
        ShellData::Help { commands } => format!(
            "{header}\n{}",
            commands
                .iter()
                .map(|command| format!("{} [{}] - {}", command.command, command.owner, command.summary))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        ShellData::World { world } => format!(
            "{header}\nrevision: {}\nentities: {}\ncomponents: {}\nresources: {}\nrelationship types: {}",
            world.revision,
            world.entities.len(),
            world.component_types.len(),
            world.resource_types.len(),
            world.relationship_types.len()
        ),
        ShellData::Entities { entities } => format!(
            "{header}\n{}",
            entities
                .iter()
                .map(|entity| format!("entity {}", entity.id))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        ShellData::Entity { entity } => format!(
            "{header}\nentity: {}\ncomponent detail: unavailable from structural owner snapshot",
            entity.id
        ),
        ShellData::Relationships {
            entity_id,
            relationships,
        } => {
            let lines = relationships
                .iter()
                .map(|relation| {
                    format!(
                        "{} -> {}",
                        relation.type_name,
                        relation
                            .targets
                            .iter()
                            .map(u64::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect::<Vec<_>>();
            format!(
                "{header}\nentity: {entity_id}\n{}",
                if lines.is_empty() {
                    "no outgoing relationships in owner observation".to_owned()
                } else {
                    lines.join("\n")
                }
            )
        }
        ShellData::Diagnostics { diagnostics } => format!(
            "{header}\ndropped records: {}\n{}",
            diagnostics.dropped_records,
            if diagnostics.records.is_empty() {
                "no diagnostic records".to_owned()
            } else {
                diagnostics
                    .records
                    .iter()
                    .map(|record| format!(
                        "{} [{}] {}: {}",
                        record.severity, record.sequence, record.source, record.message
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        ),
        ShellData::Format { format } => format!("{header}\nprojection format: {format}"),
        ShellData::Context {
            state,
            current,
            navigation_depth,
        } => format!(
            "{header}\nsession: {}\ncurrent context: {}\nnavigation depth: {navigation_depth}",
            session_state_name(*state),
            context_name(current)
        ),
        ShellData::Cleared { removed_records } => {
            format!("{header}\ncleared session-local records: {removed_records}")
        }
        ShellData::Closed {
            released_history_records,
            released_navigation_entries,
            released_watches,
        } => format!(
            "{header}\nsession closed\nreleased history records: {released_history_records}\nreleased navigation entries: {released_navigation_entries}\nreleased watches: {released_watches}"
        ),
        ShellData::WatchAdded { watch } => format!(
            "{header}\nwatch {} added\ntarget: {}\nsequence interval: {}\nnext sequence: {}",
            watch.id,
            watch_target_name(watch.target),
            watch.interval,
            watch.next_sequence
        ),
        ShellData::Watches { watches } => format!(
            "{header}\n{}",
            if watches.is_empty() {
                "no active watches".to_owned()
            } else {
                watches
                    .iter()
                    .map(|watch| format!(
                        "watch {}: {} every {} sequence(s), next {}",
                        watch.id,
                        watch_target_name(watch.target),
                        watch.interval,
                        watch.next_sequence
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        ),
        ShellData::WatchCancelled { watch } => format!(
            "{header}\nwatch {} cancelled\ntarget: {}",
            watch.id,
            watch_target_name(watch.target)
        ),
        ShellData::ApplicationMutation {
            invocation,
            receipt,
        } => format!(
            "{header}\napplication: {} {}\naccepted: {}\napplied tick: {}\nresulting revision: {}\n{}",
            invocation.owner,
            invocation.command,
            receipt.accepted,
            receipt
                .applied_tick
                .map_or_else(|| "not applied".to_owned(), |tick| tick.to_string()),
            receipt
                .resulting_revision
                .map_or_else(|| "unchanged".to_owned(), |revision| revision.to_string()),
            receipt.message
        ),
        ShellData::ApplicationQuery { invocation, result } => format!(
            "{header}\napplication: {} {}\n{}{}",
            invocation.owner,
            invocation.command,
            result.summary,
            if result.fields.is_empty() {
                String::new()
            } else {
                format!(
                    "\n{}",
                    result
                        .fields
                        .iter()
                        .map(|field| match &field.disclosure {
                            ApplicationQueryFieldDisclosure::Visible { value } => {
                                format!("{}: {value}", field.name)
                            }
                            ApplicationQueryFieldDisclosure::Redacted { reason } => {
                                format!("{}: [redacted: {reason}]", field.name)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        ),
        ShellData::Failure { message } => format!("{header}\n{message}"),
    }
}

fn application_result_name(result: &ApplicationCommandResult) -> &'static str {
    match result {
        ApplicationCommandResult::Query { .. } => "query result",
        ApplicationCommandResult::Mutation { .. } => "mutation receipt",
    }
}

fn command_kind_name(kind: ShellCommandKind) -> &'static str {
    match kind {
        ShellCommandKind::ShellMeta => "shell meta",
        ShellCommandKind::SemanticQuery => "semantic query",
        ShellCommandKind::Mutation => "mutation",
    }
}

fn status_name(status: ShellStatus) -> &'static str {
    match status {
        ShellStatus::Success => "success",
        ShellStatus::ParseFailure => "parse failure",
        ShellStatus::BudgetExceeded => "budget exceeded",
        ShellStatus::Unsupported => "unsupported",
        ShellStatus::OwnerFailure => "owner failure",
        ShellStatus::SessionFailure => "session failure",
        ShellStatus::Unavailable => "unavailable",
        ShellStatus::Unauthorized => "unauthorized",
    }
}

fn session_state_name(state: ShellSessionState) -> &'static str {
    match state {
        ShellSessionState::Open => "open",
        ShellSessionState::Closed => "closed",
    }
}

fn context_name(context: &ObservationContext) -> String {
    match context {
        ObservationContext::World => "world".to_owned(),
        ObservationContext::Entity { entity_id } => format!("entity {entity_id}"),
        ObservationContext::Diagnostics => "diagnostics".to_owned(),
    }
}

fn watch_target_name(target: WatchTarget) -> &'static str {
    match target {
        WatchTarget::World => "world",
        WatchTarget::Diagnostics => "diagnostics",
    }
}

fn diagnostic_severity_name(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Info => "info",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Error => "error",
    }
}

fn diagnostic_kind_name(kind: DiagnosticKind) -> &'static str {
    match kind {
        DiagnosticKind::Message => "message",
        DiagnosticKind::BackendError => "backend_error",
        DiagnosticKind::PerformanceBudgetExceeded => "performance_budget_exceeded",
        DiagnosticKind::PerformanceRecovered => "performance_recovered",
    }
}

fn performance_unit_name(unit: PerformanceUnit) -> &'static str {
    match unit {
        PerformanceUnit::Seconds => "seconds",
        PerformanceUnit::Milliseconds => "milliseconds",
        PerformanceUnit::Count => "count",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokimu_core::{Diagnostics, World};

    #[derive(Debug)]
    struct Follows;

    fn fixture() -> ObservationSource {
        let mut world = World::default();
        let first = world.spawn();
        let second = world.spawn();
        assert!(world.add_relationship::<Follows>(first, second));

        let mut diagnostics = Diagnostics::default();
        diagnostics.record("fixture initialized");
        ObservationSource::from_world_and_diagnostics(&world, &diagnostics)
    }

    #[test]
    fn fixed_script_is_deterministic_and_read_only() {
        let source = fixture();
        let script = [
            "help",
            "inspect world",
            "list entities",
            "inspect entity 0",
            "list relationships 0",
            "observe diagnostics",
            "format json",
            "inspect world",
        ];
        let mut first = ObservationShell::default();
        let mut second = ObservationShell::default();
        for input in script {
            first.execute(&source, input);
            second.execute(&source, input);
        }

        assert_eq!(first.history(), second.history());
        assert_eq!(source, fixture());
        assert!(first
            .history()
            .last()
            .unwrap()
            .projection
            .contains("\"world\""));
    }

    #[test]
    fn parse_watch_and_owner_failures_are_distinct() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        assert_eq!(
            shell
                .execute(&source, "inspect entity none")
                .response
                .status,
            ShellStatus::ParseFailure
        );
        assert_eq!(
            shell.execute(&source, "watch world").response.status,
            ShellStatus::Success
        );
        assert_eq!(
            shell.execute(&source, "inspect entity 999").response.status,
            ShellStatus::OwnerFailure
        );
    }

    #[test]
    fn bounded_history_evicts_oldest_record() {
        let source = fixture();
        let mut shell = ObservationShell::new(2);
        shell.execute(&source, "help");
        shell.execute(&source, "inspect world");
        shell.execute(&source, "list entities");
        assert_eq!(shell.history().len(), 2);
        assert_eq!(shell.history()[0].input, "inspect world");
    }

    #[test]
    fn host_application_queries_are_retained_without_parsing_host_controls() {
        let mut shell = ObservationShell::default();
        let record = shell.record_application_query_at_sequence(
            "[ui] observe",
            7,
            ApplicationCommandInvocation {
                owner: "runtime".to_owned(),
                command: "toolbar-observe".to_owned(),
                arguments: Vec::new(),
            },
            ApplicationQueryResult {
                summary: "Captured the current runtime observation.".to_owned(),
                fields: vec![ApplicationQueryField::visible("source", "browser toolbar")],
            },
        );

        assert_eq!(record.input, "[ui] observe");
        assert_eq!(record.response.status, ShellStatus::Success);
        let ShellData::ApplicationQuery { invocation, result } = &record.response.data else {
            panic!("host controls must retain application-query evidence");
        };
        assert_eq!(invocation.owner, "runtime");
        assert_eq!(invocation.command, "toolbar-observe");
        assert_eq!(result.summary, "Captured the current runtime observation.");
        assert_eq!(shell.history(), [record]);
    }

    #[test]
    fn sessions_keep_independent_context_and_projection_over_one_source() {
        let source = fixture();
        let mut left = ObservationShell::default();
        let mut right = ObservationShell::default();

        left.execute(&source, "format json");
        left.execute(&source, "select entity 0");
        right.execute(&source, "select diagnostics");

        assert_eq!(left.format(), ProjectionFormat::Json);
        assert_eq!(right.format(), ProjectionFormat::Text);
        assert_eq!(
            left.current_context(),
            &ObservationContext::Entity { entity_id: 0 }
        );
        assert_eq!(right.current_context(), &ObservationContext::Diagnostics);
    }

    #[test]
    fn stale_context_is_reported_without_falling_back() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        shell.execute(&source, "select entity 0");

        let mut refreshed = source.clone();
        refreshed.world.entities.clear();
        let record = shell.execute(&refreshed, "context");

        assert_eq!(record.response.status, ShellStatus::OwnerFailure);
        assert_eq!(
            shell.current_context(),
            &ObservationContext::Entity { entity_id: 0 }
        );
        assert!(record.projection.contains("refreshed owner observation"));
    }

    #[test]
    fn close_releases_session_local_state_without_global_fallback() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        shell.execute(&source, "inspect world");
        shell.execute(&source, "select entity 0");
        shell.execute(&source, "watch world");
        let closed = shell.execute(&source, "close");

        assert_eq!(closed.response.status, ShellStatus::Success);
        assert_eq!(shell.state(), ShellSessionState::Closed);
        assert!(shell.history().is_empty());
        assert_eq!(shell.navigation_depth(), 0);
        assert!(shell.watches().is_empty());
        assert_eq!(
            shell.execute(&source, "context").response.status,
            ShellStatus::SessionFailure
        );
        assert!(shell.history().is_empty());
    }

    #[test]
    fn registered_application_commands_are_discoverable_without_execution_authority() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "summary",
                "Read a corpus-local summary through an application-owned adapter.",
                vec!["scope".to_owned()],
            ))
            .unwrap();
        shell
            .register_application_command(ApplicationCommandDescription::mutation(
                "fixture",
                "reset",
                "Reset corpus-local state through an application-owned adapter.",
                Vec::new(),
            ))
            .unwrap();

        let help = shell.execute(&source, "help");
        let ShellData::Help { commands } = help.response.data else {
            panic!("help must return a command catalog");
        };
        assert!(commands.iter().any(|entry| {
            entry.command == "application fixture summary"
                && entry.owner == "fixture"
                && entry.kind == ShellCommandKind::SemanticQuery
                && entry.arguments == ["scope"]
                && entry.result_kind == CommandResultKind::Observation
                && entry.availability == CommandAvailability::RegisteredWithoutHandler
        }));
        assert!(commands.iter().any(|entry| {
            entry.command == "application fixture reset"
                && entry.kind == ShellCommandKind::Mutation
                && entry.result_kind == CommandResultKind::MutationReceipt
                && entry.availability == CommandAvailability::RegisteredWithoutHandler
        }));

        assert_eq!(
            shell
                .execute(&source, "application fixture summary current")
                .response
                .status,
            ShellStatus::Unavailable
        );
        assert_eq!(
            shell
                .execute(&source, "application fixture reset")
                .response
                .status,
            ShellStatus::Unauthorized
        );
    }

    #[test]
    fn registered_mutations_cross_only_the_caller_owned_handler_boundary() {
        let source = fixture();
        let original = source.clone();
        let mut shell = ObservationShell::default();
        shell
            .register_application_command(ApplicationCommandDescription::mutation(
                "fixture",
                "set-enabled",
                "Apply a fixture-owned mutation through its adapter.",
                vec!["entity".to_owned(), "enabled".to_owned()],
            ))
            .unwrap();

        let record = shell.execute_with_mutation_handler(
            &source,
            "application fixture set-enabled 0 false",
            |invocation| {
                assert_eq!(invocation.owner, "fixture");
                assert_eq!(invocation.command, "set-enabled");
                assert_eq!(invocation.arguments, ["0", "false"]);
                ApplicationMutationReceipt {
                    accepted: true,
                    applied_tick: Some(4),
                    resulting_revision: Some(1),
                    message: "fixture adapter applied the command".to_owned(),
                }
            },
        );

        assert_eq!(record.response.status, ShellStatus::Success);
        assert_eq!(source, original);
        let ShellData::ApplicationMutation {
            invocation,
            receipt,
        } = record.response.data
        else {
            panic!("registered mutation must retain the caller receipt");
        };
        assert_eq!(invocation.arguments, ["0", "false"]);
        assert!(receipt.accepted);
        assert_eq!(receipt.applied_tick, Some(4));
        assert_eq!(receipt.resulting_revision, Some(1));
    }

    #[test]
    fn registered_queries_project_only_caller_owned_observation_fields() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "clips",
                "List the fixture-owned playback catalog.",
                Vec::new(),
            ))
            .unwrap();

        let record = shell.execute_with_application_handler(
            &source,
            "application fixture clips",
            |invocation| {
                assert_eq!(invocation.owner, "fixture");
                assert_eq!(invocation.command, "clips");
                assert!(invocation.arguments.is_empty());
                ApplicationCommandResult::Query {
                    result: ApplicationQueryResult {
                        summary: "2 fixture-owned clips".to_owned(),
                        fields: vec![
                            ApplicationQueryField::visible("clip 0", "idle; 1.000s"),
                            ApplicationQueryField::visible("clip 1", "launch; 2.000s"),
                        ],
                    },
                }
            },
        );

        assert_eq!(record.response.status, ShellStatus::Success);
        let ShellData::ApplicationQuery { invocation, result } = record.response.data else {
            panic!("registered query must retain the caller observation");
        };
        assert_eq!(invocation.owner, "fixture");
        assert_eq!(result.summary, "2 fixture-owned clips");
        assert_eq!(result.fields.len(), 2);
        assert_eq!(result.fields[1].name, "clip 1");
        assert_eq!(
            result.fields[1].disclosure,
            ApplicationQueryFieldDisclosure::Visible {
                value: "launch; 2.000s".to_owned(),
            }
        );
        assert!(record.projection.contains("clip 0: idle; 1.000s"));
    }

    #[test]
    fn owner_supplied_redaction_omits_the_value_from_text_projection() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "private-summary",
                "Project a fixture-owned observation with one withheld field.",
                Vec::new(),
            ))
            .unwrap();

        let record = shell.execute_with_application_handler(
            &source,
            "application fixture private-summary",
            |_| ApplicationCommandResult::Query {
                result: ApplicationQueryResult {
                    summary: "fixture disclosure test".to_owned(),
                    fields: vec![
                        ApplicationQueryField::visible("public count", "2"),
                        ApplicationQueryField::redacted(
                            "session credential",
                            "owner policy excludes secrets from shell observations",
                        ),
                    ],
                },
            },
        );

        assert!(record.projection.contains("public count: 2"));
        assert!(record.projection.contains(
            "session credential: [redacted: owner policy excludes secrets from shell observations]"
        ));
        assert!(!record.projection.contains("credential-value"));
    }

    #[test]
    fn caller_owned_sequence_can_route_an_application_handler() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "summary",
                "Read a fixture-owned summary.",
                Vec::new(),
            ))
            .unwrap();

        let record = shell.execute_at_sequence_with_application_handler(
            &source,
            "application fixture summary",
            41,
            |_| ApplicationCommandResult::Query {
                result: ApplicationQueryResult {
                    summary: "caller-sequenced result".to_owned(),
                    fields: Vec::new(),
                },
            },
        );

        assert_eq!(record.response.status, ShellStatus::Success);
        assert!(record.projection.contains("caller-sequenced result"));
        assert_eq!(shell.current_sequence, 41);
    }

    #[test]
    fn application_handler_kind_mismatches_are_explicit_owner_failures() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "summary",
                "Read the fixture summary.",
                Vec::new(),
            ))
            .unwrap();

        let record =
            shell.execute_with_application_handler(&source, "application fixture summary", |_| {
                ApplicationCommandResult::Mutation {
                    receipt: ApplicationMutationReceipt {
                        accepted: true,
                        applied_tick: None,
                        resulting_revision: None,
                        message: "wrong result kind".to_owned(),
                    },
                }
            });

        assert_eq!(record.response.status, ShellStatus::OwnerFailure);
        let ShellData::Failure { message } = record.response.data else {
            panic!("handler kind mismatch must retain a structured failure");
        };
        assert!(message.contains("returned mutation receipt for a semantic query command"));
    }

    #[test]
    fn unavailable_application_commands_never_reach_a_mutation_handler() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        let mut invoked = false;

        let record =
            shell.execute_with_mutation_handler(&source, "application fixture unknown", |_| {
                invoked = true;
                ApplicationMutationReceipt {
                    accepted: true,
                    applied_tick: None,
                    resulting_revision: None,
                    message: "must not be returned".to_owned(),
                }
            });

        assert_eq!(record.response.status, ShellStatus::Unavailable);
        assert!(!invoked);
    }

    #[test]
    fn read_only_sessions_reject_registered_mutations_before_the_handler() {
        let source = fixture();
        let mut shell = ObservationShell::read_only(8);
        shell
            .register_application_command(ApplicationCommandDescription::mutation(
                "fixture",
                "reset",
                "Reset fixture-owned state.",
                Vec::new(),
            ))
            .unwrap();
        let mut invoked = false;

        let record =
            shell.execute_with_mutation_handler(&source, "application fixture reset", |_| {
                invoked = true;
                ApplicationMutationReceipt {
                    accepted: true,
                    applied_tick: None,
                    resulting_revision: None,
                    message: "must not be returned".to_owned(),
                }
            });

        assert_eq!(shell.authority(), ShellAuthority::ReadOnly);
        assert_eq!(record.response.status, ShellStatus::Unauthorized);
        assert!(!invoked);
    }

    #[test]
    fn shell_budget_failures_preserve_the_open_session() {
        let source = fixture();
        let limits = ShellBoundaryLimits {
            max_input_bytes: 12,
            max_arguments: 1,
            max_projection_bytes: 256,
            max_commands_per_sequence: 2,
        };
        let mut shell = ObservationShell::with_authority_and_boundary_limits(
            ShellAuthority::Control,
            8,
            8,
            2,
            limits,
        );

        let oversized = shell.execute(&source, "inspect entity 999999");
        assert_eq!(oversized.response.status, ShellStatus::BudgetExceeded);
        assert_eq!(shell.state(), ShellSessionState::Open);

        let first = shell.execute_at_sequence(&source, "context", 10);
        let second = shell.execute_at_sequence(&source, "context", 10);
        let flooded = shell.execute_at_sequence(&source, "context", 10);
        assert_eq!(first.response.status, ShellStatus::Success);
        assert_eq!(second.response.status, ShellStatus::Success);
        assert_eq!(flooded.response.status, ShellStatus::BudgetExceeded);
        assert_eq!(shell.state(), ShellSessionState::Open);
    }

    #[test]
    fn command_rate_and_projection_limits_are_explicit_and_retained_safely() {
        let source = fixture();
        let limits = ShellBoundaryLimits {
            max_input_bytes: 128,
            max_arguments: 4,
            max_projection_bytes: 256,
            max_commands_per_sequence: 1,
        };
        let mut shell = ObservationShell::with_authority_and_boundary_limits(
            ShellAuthority::Control,
            8,
            8,
            2,
            limits,
        );
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "verbose",
                "Return a bounded fixture observation.",
                Vec::new(),
            ))
            .unwrap();

        let too_large =
            shell.execute_with_application_handler(&source, "application fixture verbose", |_| {
                ApplicationCommandResult::Query {
                    result: ApplicationQueryResult {
                        summary: "x".repeat(1024),
                        fields: Vec::new(),
                    },
                }
            });
        assert_eq!(too_large.response.status, ShellStatus::BudgetExceeded);
        assert!(too_large.projection.contains("output limit"));

        let allowed = shell.execute_at_sequence(&source, "context", 40);
        let rate_limited = shell.execute_at_sequence(&source, "context", 40);
        assert_eq!(allowed.response.status, ShellStatus::Success);
        assert_eq!(rate_limited.response.status, ShellStatus::BudgetExceeded);
        assert_eq!(shell.state(), ShellSessionState::Open);
    }

    #[test]
    fn application_argument_limits_and_catalogs_do_not_expose_handler_details() {
        let source = fixture();
        let limits = ShellBoundaryLimits {
            max_input_bytes: 128,
            max_arguments: 1,
            max_projection_bytes: 4 * 1024,
            max_commands_per_sequence: 4,
        };
        let mut shell = ObservationShell::with_authority_and_boundary_limits(
            ShellAuthority::Control,
            8,
            8,
            2,
            limits,
        );
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "details",
                "Read a public fixture summary.",
                Vec::new(),
            ))
            .unwrap();

        let catalog = shell.execute(&source, "help");
        assert_eq!(catalog.response.status, ShellStatus::Success);
        assert!(!catalog.projection.contains("private handler detail"));

        let too_many = shell.execute_with_application_handler(
            &source,
            "application fixture details public extra",
            |_| ApplicationCommandResult::Query {
                result: ApplicationQueryResult {
                    summary: "private handler detail".to_owned(),
                    fields: Vec::new(),
                },
            },
        );
        assert_eq!(too_many.response.status, ShellStatus::BudgetExceeded);
        assert!(!too_many.projection.contains("private handler detail"));
        assert_eq!(shell.state(), ShellSessionState::Open);
    }

    #[test]
    fn unusual_unicode_and_unknown_targets_fail_without_mutating_shell_context() {
        let source = fixture();
        let mut shell = ObservationShell::default();
        let before = shell.current_context().clone();

        let unicode = shell.execute(&source, "inspect\u{0000} world");
        let unknown = shell.execute(&source, "watch private-owner");

        assert_eq!(unicode.response.status, ShellStatus::Unsupported);
        assert_eq!(unknown.response.status, ShellStatus::ParseFailure);
        assert_eq!(shell.current_context(), &before);
        assert_eq!(shell.state(), ShellSessionState::Open);
    }

    #[test]
    fn duplicate_application_command_identity_is_rejected_deterministically() {
        let mut shell = ObservationShell::default();
        shell
            .register_application_command(ApplicationCommandDescription::query(
                "fixture",
                "summary",
                "First registration.",
                Vec::new(),
            ))
            .unwrap();

        assert_eq!(
            shell.register_application_command(ApplicationCommandDescription::query(
                "FIXTURE",
                "SUMMARY",
                "Duplicate registration.",
                Vec::new(),
            )),
            Err(CommandRegistrationError::Duplicate {
                owner: "FIXTURE".to_owned(),
                command: "SUMMARY".to_owned(),
            })
        );
    }

    #[test]
    fn application_invocations_are_typed_before_the_owner_boundary() {
        assert_eq!(
            parse_application_command(&["application", "fixture", "summary", "current"]),
            Ok(ApplicationCommandInvocation {
                owner: "fixture".to_owned(),
                command: "summary".to_owned(),
                arguments: vec!["current".to_owned()],
            })
        );
        assert_eq!(
            parse_application_command(&["application"]),
            Err(ApplicationCommandParseError::MissingOwner)
        );
        assert_eq!(
            parse_application_command(&["application", "fixture"]),
            Err(ApplicationCommandParseError::MissingCommand)
        );
    }

    #[test]
    fn watches_refresh_copied_summaries_at_caller_supplied_cadence() {
        let source = fixture();
        let mut shell = ObservationShell::with_session_limits(8, 8, 2);
        let added = shell.execute(&source, "watch world 2");
        let ShellData::WatchAdded { watch } = added.response.data else {
            panic!("watch must return its bounded subscription");
        };
        assert_eq!(watch.id, 1);
        assert_eq!(watch.target, WatchTarget::World);
        assert_eq!(watch.interval, 2);

        let first = shell.refresh_watches(&source, 0);
        assert_eq!(first.len(), 1);
        assert!(!first[0].unchanged);
        assert!(!first[0].truncated);
        assert_eq!(first[0].summary.entity_count, Some(2));
        assert!(shell.refresh_watches(&source, 1).is_empty());

        // A caller can skip sequences without queuing a refresh per missed tick.
        let coalesced = shell.refresh_watches(&source, 9);
        assert_eq!(coalesced.len(), 1);
        assert!(coalesced[0].unchanged);
        assert_eq!(coalesced[0].sequence, 9);
        assert!(shell.refresh_watches(&source, 10).is_empty());
    }

    #[test]
    fn watches_are_bounded_listed_and_cancelled_explicitly() {
        let source = fixture();
        let mut shell = ObservationShell::with_session_limits(8, 8, 1);
        assert_eq!(
            shell.execute(&source, "watch diagnostics").response.status,
            ShellStatus::Success
        );
        assert_eq!(
            shell.execute(&source, "watch world").response.status,
            ShellStatus::Unavailable
        );

        let listed = shell.execute(&source, "list watches");
        let ShellData::Watches { watches } = listed.response.data else {
            panic!("list watches must return current subscriptions");
        };
        assert_eq!(watches.len(), 1);
        assert_eq!(watches[0].target, WatchTarget::Diagnostics);

        let cancelled = shell.execute(&source, "unwatch 1");
        assert_eq!(cancelled.response.status, ShellStatus::Success);
        assert!(matches!(
            cancelled.response.data,
            ShellData::WatchCancelled { .. }
        ));
        assert!(shell.watches().is_empty());
        assert!(shell.refresh_watches(&source, 0).is_empty());
    }
}
