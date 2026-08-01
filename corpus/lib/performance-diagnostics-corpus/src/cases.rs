use tokimu_core::PerformanceUnit;
use tokimu_render::{RenderFrameStats, RenderLifetimeStats, RenderStats};

use crate::{CaseExpectation, MeasurementSupport};

#[derive(Clone, Copy, Debug)]
pub struct PerformanceCase {
    pub id: &'static str,
    pub description: &'static str,
    pub workload_revision: &'static str,
    pub expected: CaseExpectation,
    pub measurement: PerformanceCaseMeasurement,
    pub diagnostic_capacity: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum PerformanceCaseMeasurement {
    Observations {
        source: &'static str,
        metric: &'static str,
        limit: f64,
        unit: PerformanceUnit,
        required_consecutive_violations: u32,
        values: &'static [f64],
    },
    RenderBindingAllocations {
        required_consecutive_violations: u32,
        frames: &'static [RenderStats],
    },
    RenderMeshUploads {
        required_consecutive_violations: u32,
        frames: &'static [RenderStats],
    },
    AssetLifecycle,
    Unsupported {
        reason: &'static str,
    },
}

impl PerformanceCase {
    pub fn support(&self) -> MeasurementSupport {
        match self.measurement {
            PerformanceCaseMeasurement::Unsupported { reason } => MeasurementSupport::Unsupported {
                reason: reason.into(),
            },
            _ => MeasurementSupport::Supported,
        }
    }
}

const HEALTHY: &[f64] = &[1.0, 2.0, 1.0];
const TRANSIENT_SPIKE: &[f64] = &[1.0, 4.0, 1.0];
const SUSTAINED: &[f64] = &[4.0, 5.0, 6.0];
const RECOVERY: &[f64] = &[4.0, 5.0, 2.0];
const OVERFLOW: &[f64] = &[2.0, 0.0, 2.0, 0.0, 2.0];

const fn stats(frame: RenderFrameStats, lifetime: RenderLifetimeStats) -> RenderStats {
    RenderStats { frame, lifetime }
}

const fn frame_stats(
    binding_allocations: u32,
    mesh_uploads: u32,
    mesh_replacements: u32,
) -> RenderFrameStats {
    RenderFrameStats {
        draw_calls: 4,
        submit_calls: 1,
        binding_allocations,
        mesh_uploads,
        mesh_replacements,
        ..RenderFrameStats::EMPTY
    }
}

const fn lifetime_stats(
    binding_allocations: u64,
    mesh_uploads: u64,
    mesh_replacements: u64,
) -> RenderLifetimeStats {
    RenderLifetimeStats {
        binding_allocations,
        mesh_uploads,
        mesh_replacements,
        ..RenderLifetimeStats::EMPTY
    }
}

const STATIC_RESOURCES: &[RenderStats] = &[
    stats(frame_stats(3, 2, 0), lifetime_stats(3, 2, 0)),
    stats(frame_stats(0, 0, 0), lifetime_stats(3, 2, 0)),
    stats(frame_stats(0, 0, 0), lifetime_stats(3, 2, 0)),
    stats(frame_stats(0, 0, 0), lifetime_stats(3, 2, 0)),
];

const REPEATED_BINDING_ALLOCATION: &[RenderStats] = &[
    stats(frame_stats(4, 0, 0), lifetime_stats(4, 2, 0)),
    stats(frame_stats(4, 0, 0), lifetime_stats(8, 2, 0)),
    stats(frame_stats(4, 0, 0), lifetime_stats(12, 2, 0)),
];

const REPEATED_MESH_UPLOAD: &[RenderStats] = &[
    stats(frame_stats(0, 2, 2), lifetime_stats(3, 2, 2)),
    stats(frame_stats(0, 2, 2), lifetime_stats(3, 4, 4)),
    stats(frame_stats(0, 2, 2), lifetime_stats(3, 6, 6)),
];

const CASES: &[PerformanceCase] = &[
    PerformanceCase {
        id: "diagnostics/healthy",
        description: "Controlled observations remain within budget.",
        workload_revision: "diagnostic-transitions-v1",
        expected: CaseExpectation::Silence,
        measurement: PerformanceCaseMeasurement::Observations {
            source: "corpus.synthetic",
            metric: "controlled work units",
            limit: 3.0,
            unit: PerformanceUnit::Count,
            required_consecutive_violations: 2,
            values: HEALTHY,
        },
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "diagnostics/transient-spike",
        description: "One spike does not satisfy the sustained-pressure policy.",
        workload_revision: "diagnostic-transitions-v1",
        expected: CaseExpectation::Silence,
        measurement: PerformanceCaseMeasurement::Observations {
            source: "corpus.synthetic",
            metric: "controlled work units",
            limit: 3.0,
            unit: PerformanceUnit::Count,
            required_consecutive_violations: 2,
            values: TRANSIENT_SPIKE,
        },
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "diagnostics/sustained-pressure",
        description: "Sustained pressure emits one latched warning.",
        workload_revision: "diagnostic-transitions-v1",
        expected: CaseExpectation::Warning,
        measurement: PerformanceCaseMeasurement::Observations {
            source: "corpus.synthetic",
            metric: "controlled work units",
            limit: 3.0,
            unit: PerformanceUnit::Count,
            required_consecutive_violations: 2,
            values: SUSTAINED,
        },
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "diagnostics/recovery",
        description: "A degraded monitor emits one recovery after pressure clears.",
        workload_revision: "diagnostic-transitions-v1",
        expected: CaseExpectation::WarningThenRecovery,
        measurement: PerformanceCaseMeasurement::Observations {
            source: "corpus.synthetic",
            metric: "controlled work units",
            limit: 3.0,
            unit: PerformanceUnit::Count,
            required_consecutive_violations: 2,
            values: RECOVERY,
        },
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "diagnostics/bounded-overflow",
        description: "Alternating pressure and recovery overflow a bounded capture.",
        workload_revision: "diagnostic-transitions-v1",
        expected: CaseExpectation::BoundedOverflow,
        measurement: PerformanceCaseMeasurement::Observations {
            source: "corpus.synthetic",
            metric: "controlled work units",
            limit: 1.0,
            unit: PerformanceUnit::Count,
            required_consecutive_violations: 1,
            values: OVERFLOW,
        },
        diagnostic_capacity: 2,
    },
    PerformanceCase {
        id: "renderer/stable-resources",
        description: "A static scene creates resources during warm-up and stays quiet afterward.",
        workload_revision: "static-presentation-v1",
        expected: CaseExpectation::Silence,
        measurement: PerformanceCaseMeasurement::RenderBindingAllocations {
            required_consecutive_violations: 2,
            frames: STATIC_RESOURCES,
        },
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "renderer/repeated-binding-allocation",
        description: "A deliberately regressed scene allocates bindings every frame.",
        workload_revision: "repeated-binding-allocation-v1",
        expected: CaseExpectation::Warning,
        measurement: PerformanceCaseMeasurement::RenderBindingAllocations {
            required_consecutive_violations: 2,
            frames: REPEATED_BINDING_ALLOCATION,
        },
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "renderer/repeated-mesh-upload",
        description: "A deliberately regressed scene replaces stable meshes every frame.",
        workload_revision: "repeated-mesh-upload-v1",
        expected: CaseExpectation::Warning,
        measurement: PerformanceCaseMeasurement::RenderMeshUploads {
            required_consecutive_violations: 2,
            frames: REPEATED_MESH_UPLOAD,
        },
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "assets/registered-lifecycle",
        description: "A stable asset identity is allocated, prepared, replaced, and released.",
        workload_revision: "asset-lifecycle-v1",
        expected: CaseExpectation::Silence,
        measurement: PerformanceCaseMeasurement::AssetLifecycle,
        diagnostic_capacity: 8,
    },
    PerformanceCase {
        id: "renderer/gpu-completion-time",
        description: "GPU completion time remains unsupported without timestamp queries.",
        workload_revision: "unsupported-measurements-v1",
        expected: CaseExpectation::Unsupported,
        measurement: PerformanceCaseMeasurement::Unsupported {
            reason: "no GPU timestamp-query mechanism is active",
        },
        diagnostic_capacity: 8,
    },
];

pub fn all_cases() -> &'static [PerformanceCase] {
    CASES
}

pub fn find_case(id: &str) -> Option<&'static PerformanceCase> {
    CASES.iter().find(|case| case.id == id)
}
