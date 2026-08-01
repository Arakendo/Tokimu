use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PerformanceCorpusMetadata {
    pub schema: u32,
    pub producer: String,
    pub case_id: String,
    pub build_profile: String,
    pub target: String,
    pub workload_revision: String,
    pub monitor: String,
    pub renderer_counter_policy: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BudgetArtifact {
    pub source: String,
    pub metric: String,
    pub limit: f64,
    pub unit: String,
    pub required_consecutive_violations: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseExpectation {
    Silence,
    Warning,
    WarningThenRecovery,
    BoundedOverflow,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum MeasurementSupport {
    Supported,
    Unsupported { reason: String },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ObservationArtifact {
    pub sequence: usize,
    pub value: f64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticTransition {
    BudgetExceeded,
    Recovered,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagnosticArtifact {
    pub sequence: u64,
    pub severity: String,
    pub kind: String,
    pub source: String,
    pub message: String,
    pub metric: Option<String>,
    pub observed: Option<f64>,
    pub budget: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RenderFrameArtifact {
    pub sequence: usize,
    pub draw_calls: u32,
    pub submit_calls: u32,
    pub binding_allocations: u32,
    pub uniform_buffer_writes: u32,
    pub mesh_uploads: u32,
    pub mesh_replacements: u32,
    pub texture_allocations: u32,
    pub texture_replacements: u32,
    pub texture_writes: u32,
    pub lifetime_binding_allocations: u64,
    pub lifetime_uniform_buffer_writes: u64,
    pub lifetime_mesh_uploads: u64,
    pub lifetime_mesh_replacements: u64,
    pub lifetime_texture_allocations: u64,
    pub lifetime_texture_replacements: u64,
    pub lifetime_texture_writes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResourceLifecycleArtifact {
    pub sequence: u64,
    pub resource_kind: String,
    pub resource_id: u64,
    pub generation: u64,
    pub transition: String,
    pub source: Option<String>,
    pub measured_bytes: Option<u64>,
    pub measured_duration_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NumericSummaryArtifact {
    pub sample_count: usize,
    pub window_size: usize,
    pub cadence: String,
    pub reset_behavior: String,
    pub last: f64,
    pub total: f64,
    pub average: f64,
    pub peak: f64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResourceLifecycleSummaryArtifact {
    pub event_count: usize,
    pub reset_behavior: String,
    pub allocated: usize,
    pub prepared: usize,
    pub replaced: usize,
    pub released: usize,
    pub final_active_resources: usize,
    pub last_generation: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PerformanceCaseArtifact {
    pub metadata: PerformanceCorpusMetadata,
    pub description: String,
    pub measurement: MeasurementSupport,
    pub expected: CaseExpectation,
    pub actual: CaseExpectation,
    pub budget: Option<BudgetArtifact>,
    pub observations: Vec<ObservationArtifact>,
    pub render_frames: Vec<RenderFrameArtifact>,
    pub resource_lifecycle: Vec<ResourceLifecycleArtifact>,
    pub numeric_summary: Option<NumericSummaryArtifact>,
    pub resource_lifecycle_summary: Option<ResourceLifecycleSummaryArtifact>,
    pub transitions: Vec<DiagnosticTransition>,
    pub diagnostics: Vec<DiagnosticArtifact>,
    pub diagnostic_capacity: usize,
    pub dropped_records: u64,
}
