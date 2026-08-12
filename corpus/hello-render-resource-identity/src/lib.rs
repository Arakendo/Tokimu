//! Corpus-local evidence for AR-0024/AR-0027 resource identity work.
//!
//! This crate models the current replace-on-upload identity behavior without a
//! GPU or source-format dependency. It is experimental evidence, not a public
//! allocator, registry, lifecycle contract, or recovery policy.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use tokimu_render::MeshHandle;

/// Logical identity retained only by this corpus fixture. The current renderer
/// sees a typed handle and mesh bytes; it does not receive this distinction.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LogicalMesh {
    StaticOpaque(u32),
    StaticCutout(u32),
    Dynamic(u32),
}

/// What replace-on-upload can observe when no lifecycle intent accompanies an
/// upload. A replacement may be either deliberate or accidental.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyUploadObservation {
    Created {
        handle: MeshHandle,
        resource: LogicalMesh,
    },
    Replaced {
        handle: MeshHandle,
        previous: LogicalMesh,
        replacement: LogicalMesh,
    },
}

/// Small evidence model of the current renderer mesh map: uploading a live
/// handle replaces the previous GPU mesh and increments replacement evidence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LegacyMeshLedger {
    live: BTreeMap<u64, LogicalMesh>,
}

impl LegacyMeshLedger {
    pub fn upload(&mut self, handle: MeshHandle, resource: LogicalMesh) -> LegacyUploadObservation {
        match self.live.insert(handle.0, resource) {
            Some(previous) => LegacyUploadObservation::Replaced {
                handle,
                previous,
                replacement: resource,
            },
            None => LegacyUploadObservation::Created { handle, resource },
        }
    }

    pub fn resolve(&self, handle: MeshHandle) -> Option<LogicalMesh> {
        self.live.get(&handle.0).copied()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutableOffsetFailureEvidence {
    pub original_cutout_handle: MeshHandle,
    pub dynamic_handle: MeshHandle,
    pub recomputed_cutout_handle: MeshHandle,
    pub dynamic_upload: LegacyUploadObservation,
    pub original_cutout_now_resolves_to: Option<LogicalMesh>,
    pub recomputed_cutout_resolves_to: Option<LogicalMesh>,
}

/// Reproduces the E1M1 defect independently of Doom:
///
/// 1. two opaque meshes make the cutout base `3`;
/// 2. a cutout is uploaded at `3`;
/// 3. appending a third opaque draw derives dynamic handle `3` and silently
///    replaces the cutout;
/// 4. recomputing the cutout base from the new opaque count yields unresolved
///    handle `4`.
pub fn reproduce_mutable_offset_alias() -> MutableOffsetFailureEvidence {
    let mut ledger = LegacyMeshLedger::default();
    ledger.upload(MeshHandle(1), LogicalMesh::StaticOpaque(0));
    ledger.upload(MeshHandle(2), LogicalMesh::StaticOpaque(1));

    let original_cutout_handle = MeshHandle(3);
    ledger.upload(original_cutout_handle, LogicalMesh::StaticCutout(0));

    let dynamic_handle = MeshHandle(3);
    let dynamic_upload = ledger.upload(dynamic_handle, LogicalMesh::Dynamic(0));
    let recomputed_cutout_handle = MeshHandle(4);

    MutableOffsetFailureEvidence {
        original_cutout_handle,
        dynamic_handle,
        recomputed_cutout_handle,
        dynamic_upload,
        original_cutout_now_resolves_to: ledger.resolve(original_cutout_handle),
        recomputed_cutout_resolves_to: ledger.resolve(recomputed_cutout_handle),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedRangeEvidence {
    pub cutout_handle: MeshHandle,
    pub dynamic_handle: MeshHandle,
    pub cutout_resolves_to: Option<LogicalMesh>,
    pub dynamic_resolves_to: Option<LogicalMesh>,
}

/// Retains the current E1M1 repair as a baseline: static opaque, cutout, and
/// dynamic identities occupy disjoint ranges and do not move when a dynamic
/// draw appears.
pub fn observe_fixed_disjoint_ranges() -> FixedRangeEvidence {
    let mut ledger = LegacyMeshLedger::default();
    ledger.upload(MeshHandle(1), LogicalMesh::StaticOpaque(0));
    ledger.upload(MeshHandle(2), LogicalMesh::StaticOpaque(1));
    let cutout_handle = MeshHandle(3);
    let dynamic_handle = MeshHandle(4);
    ledger.upload(cutout_handle, LogicalMesh::StaticCutout(0));
    ledger.upload(dynamic_handle, LogicalMesh::Dynamic(0));

    FixedRangeEvidence {
        cutout_handle,
        dynamic_handle,
        cutout_resolves_to: ledger.resolve(cutout_handle),
        dynamic_resolves_to: ledger.resolve(dynamic_handle),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LifecycleCounts {
    pub creates: u64,
    pub replacements: u64,
    pub retires: u64,
    pub rejections: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    AlreadyLive {
        handle: MeshHandle,
        current: LogicalMesh,
        requested: LogicalMesh,
    },
    Missing {
        handle: MeshHandle,
    },
    LogicalMismatch {
        handle: MeshHandle,
        current: LogicalMesh,
        requested: LogicalMesh,
    },
    StaleGeneration {
        slot: u32,
        requested_generation: u32,
        current_generation: u32,
    },
}

/// Alternative B: an application-owned monotonic allocator/registry. It keeps
/// renderer handles deterministic for one registry construction and requires
/// replacement to name the same retained logical identity.
#[derive(Clone, Debug, Default)]
pub struct ApplicationMeshRegistry {
    next: u64,
    live: BTreeMap<u64, LogicalMesh>,
    counts: LifecycleCounts,
}

impl ApplicationMeshRegistry {
    pub fn create(&mut self, resource: LogicalMesh) -> MeshHandle {
        self.next = self.next.saturating_add(1);
        let handle = MeshHandle(self.next);
        self.live.insert(handle.0, resource);
        self.counts.creates = self.counts.creates.saturating_add(1);
        handle
    }

    pub fn replace(
        &mut self,
        handle: MeshHandle,
        resource: LogicalMesh,
    ) -> Result<(), IdentityError> {
        match self.live.get(&handle.0).copied() {
            Some(current) if current == resource => {
                self.counts.replacements = self.counts.replacements.saturating_add(1);
                Ok(())
            }
            Some(current) => {
                self.counts.rejections = self.counts.rejections.saturating_add(1);
                Err(IdentityError::LogicalMismatch {
                    handle,
                    current,
                    requested: resource,
                })
            }
            None => {
                self.counts.rejections = self.counts.rejections.saturating_add(1);
                Err(IdentityError::Missing { handle })
            }
        }
    }

    pub fn retire(&mut self, handle: MeshHandle) -> Result<LogicalMesh, IdentityError> {
        self.live.remove(&handle.0).map_or_else(
            || {
                self.counts.rejections = self.counts.rejections.saturating_add(1);
                Err(IdentityError::Missing { handle })
            },
            |resource| {
                self.counts.retires = self.counts.retires.saturating_add(1);
                Ok(resource)
            },
        )
    }

    pub const fn counts(&self) -> LifecycleCounts {
        self.counts
    }

    pub fn resolve(&mut self, handle: MeshHandle) -> Result<LogicalMesh, IdentityError> {
        self.live.get(&handle.0).copied().ok_or_else(|| {
            self.counts.rejections = self.counts.rejections.saturating_add(1);
            IdentityError::Missing { handle }
        })
    }
}

/// Alternative D corpus handle. It is intentionally not convertible to the
/// current renderer handle without an explicit adapter decision.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GenerationalMeshHandle {
    pub slot: u32,
    pub generation: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GenerationalSlot {
    generation: u32,
    resource: Option<LogicalMesh>,
}

/// Alternative D: slot reuse increments a generation so a retired reference
/// cannot silently resolve to a later logical resource in the same slot.
#[derive(Clone, Debug, Default)]
pub struct GenerationalMeshRegistry {
    slots: Vec<GenerationalSlot>,
    free: Vec<u32>,
    counts: LifecycleCounts,
}

impl GenerationalMeshRegistry {
    pub fn create(&mut self, resource: LogicalMesh) -> GenerationalMeshHandle {
        let slot = self.free.pop().unwrap_or_else(|| {
            self.slots.push(GenerationalSlot {
                generation: 0,
                resource: None,
            });
            (self.slots.len() - 1) as u32
        });
        let entry = &mut self.slots[slot as usize];
        entry.resource = Some(resource);
        self.counts.creates = self.counts.creates.saturating_add(1);
        GenerationalMeshHandle {
            slot,
            generation: entry.generation,
        }
    }

    pub fn resolve(
        &mut self,
        handle: GenerationalMeshHandle,
    ) -> Result<LogicalMesh, IdentityError> {
        let Some(slot) = self.slots.get(handle.slot as usize) else {
            self.counts.rejections = self.counts.rejections.saturating_add(1);
            return Err(IdentityError::Missing {
                handle: MeshHandle(u64::from(handle.slot)),
            });
        };
        if slot.generation != handle.generation {
            self.counts.rejections = self.counts.rejections.saturating_add(1);
            return Err(IdentityError::StaleGeneration {
                slot: handle.slot,
                requested_generation: handle.generation,
                current_generation: slot.generation,
            });
        }
        slot.resource.ok_or_else(|| {
            self.counts.rejections = self.counts.rejections.saturating_add(1);
            IdentityError::Missing {
                handle: MeshHandle(u64::from(handle.slot)),
            }
        })
    }

    pub fn replace(
        &mut self,
        handle: GenerationalMeshHandle,
        resource: LogicalMesh,
    ) -> Result<(), IdentityError> {
        let current = self.resolve(handle)?;
        if current != resource {
            self.counts.rejections = self.counts.rejections.saturating_add(1);
            return Err(IdentityError::LogicalMismatch {
                handle: MeshHandle(u64::from(handle.slot)),
                current,
                requested: resource,
            });
        }
        self.counts.replacements = self.counts.replacements.saturating_add(1);
        Ok(())
    }

    pub fn retire(&mut self, handle: GenerationalMeshHandle) -> Result<LogicalMesh, IdentityError> {
        self.resolve(handle)?;
        let slot = &mut self.slots[handle.slot as usize];
        let resource = slot.resource.take().expect("resolve proved live slot");
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(handle.slot);
        self.counts.retires = self.counts.retires.saturating_add(1);
        Ok(resource)
    }

    pub const fn counts(&self) -> LifecycleCounts {
        self.counts
    }
}

/// Alternative E: caller-selected handles remain possible, but create,
/// replace, and retire are distinct operations validated against retained
/// logical ownership.
#[derive(Clone, Debug, Default)]
pub struct ExplicitLifecycleLedger {
    live: BTreeMap<u64, LogicalMesh>,
    counts: LifecycleCounts,
}

impl ExplicitLifecycleLedger {
    pub fn create(
        &mut self,
        handle: MeshHandle,
        resource: LogicalMesh,
    ) -> Result<(), IdentityError> {
        if let Some(current) = self.live.get(&handle.0).copied() {
            self.counts.rejections = self.counts.rejections.saturating_add(1);
            return Err(IdentityError::AlreadyLive {
                handle,
                current,
                requested: resource,
            });
        }
        self.live.insert(handle.0, resource);
        self.counts.creates = self.counts.creates.saturating_add(1);
        Ok(())
    }

    pub fn replace(
        &mut self,
        handle: MeshHandle,
        resource: LogicalMesh,
    ) -> Result<(), IdentityError> {
        match self.live.get(&handle.0).copied() {
            Some(current) if current == resource => {
                self.counts.replacements = self.counts.replacements.saturating_add(1);
                Ok(())
            }
            Some(current) => {
                self.counts.rejections = self.counts.rejections.saturating_add(1);
                Err(IdentityError::LogicalMismatch {
                    handle,
                    current,
                    requested: resource,
                })
            }
            None => {
                self.counts.rejections = self.counts.rejections.saturating_add(1);
                Err(IdentityError::Missing { handle })
            }
        }
    }

    pub fn retire(&mut self, handle: MeshHandle) -> Result<LogicalMesh, IdentityError> {
        self.live.remove(&handle.0).map_or_else(
            || {
                self.counts.rejections = self.counts.rejections.saturating_add(1);
                Err(IdentityError::Missing { handle })
            },
            |resource| {
                self.counts.retires = self.counts.retires.saturating_add(1);
                Ok(resource)
            },
        )
    }

    pub const fn counts(&self) -> LifecycleCounts {
        self.counts
    }

    pub fn resolve(&mut self, handle: MeshHandle) -> Result<LogicalMesh, IdentityError> {
        self.live.get(&handle.0).copied().ok_or_else(|| {
            self.counts.rejections = self.counts.rejections.saturating_add(1);
            IdentityError::Missing { handle }
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationOnlyObservation {
    pub upload: LegacyUploadObservation,
    pub unrelated_replacement: Option<IdentityError>,
}

/// Alternative F: preserve current replace-on-upload mechanics while emitting
/// an observation when retained logical ownership changes at a live handle.
#[derive(Clone, Debug, Default)]
pub struct ValidationOnlyLedger {
    legacy: LegacyMeshLedger,
    diagnostics: [Option<IdentityError>; 4],
    next_diagnostic: usize,
    total_diagnostics: u64,
}

impl ValidationOnlyLedger {
    pub fn upload(
        &mut self,
        handle: MeshHandle,
        resource: LogicalMesh,
    ) -> ValidationOnlyObservation {
        let upload = self.legacy.upload(handle, resource);
        let unrelated_replacement = match upload {
            LegacyUploadObservation::Replaced {
                previous,
                replacement,
                ..
            } if previous != replacement => {
                let diagnostic = IdentityError::LogicalMismatch {
                    handle,
                    current: previous,
                    requested: replacement,
                };
                let slot = self.next_diagnostic % self.diagnostics.len();
                self.diagnostics[slot] = Some(diagnostic);
                self.next_diagnostic = self.next_diagnostic.wrapping_add(1);
                self.total_diagnostics = self.total_diagnostics.saturating_add(1);
                Some(diagnostic)
            }
            _ => None,
        };
        ValidationOnlyObservation {
            upload,
            unrelated_replacement,
        }
    }

    pub const fn diagnostic_count(&self) -> u64 {
        self.total_diagnostics
    }

    /// Fixed-capacity retained observations. New diagnostics overwrite the
    /// oldest slot; callers retain the total count separately so bounded
    /// storage cannot masquerade as absence of failures.
    pub fn retained_diagnostics(&self) -> impl Iterator<Item = IdentityError> + '_ {
        self.diagnostics.iter().flatten().copied()
    }
}

/// Corpus-only comparison of a caller retaining the last known-good logical
/// resource *before* it asks the current replace-on-upload renderer mechanism
/// to mutate a handle. This is intentionally application-side: the renderer
/// does not infer whether replacement is desirable or keep an extra copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplacementRecoveryObservation {
    Replaced {
        handle: MeshHandle,
        previous: LogicalMesh,
        replacement: LogicalMesh,
    },
    RejectedPreservingLastKnownGood {
        handle: MeshHandle,
        retained: LogicalMesh,
        rejected: LogicalMesh,
    },
}

#[derive(Clone, Debug, Default)]
pub struct CallerStagedReplacement {
    renderer_view: LegacyMeshLedger,
}

impl CallerStagedReplacement {
    pub fn seed(&mut self, handle: MeshHandle, resource: LogicalMesh) {
        let observation = self.renderer_view.upload(handle, resource);
        debug_assert!(matches!(
            observation,
            LegacyUploadObservation::Created { .. }
        ));
    }

    /// The caller performs its own retained-identity check before submitting an
    /// upload. A rejected candidate therefore cannot replace the last
    /// known-good renderer resource as a side effect of merely being checked.
    pub fn replace_if_same_logical_identity(
        &mut self,
        handle: MeshHandle,
        candidate: LogicalMesh,
    ) -> Result<ReplacementRecoveryObservation, IdentityError> {
        let Some(retained) = self.renderer_view.resolve(handle) else {
            return Err(IdentityError::Missing { handle });
        };
        if retained != candidate {
            return Ok(
                ReplacementRecoveryObservation::RejectedPreservingLastKnownGood {
                    handle,
                    retained,
                    rejected: candidate,
                },
            );
        }

        let upload = self.renderer_view.upload(handle, candidate);
        match upload {
            LegacyUploadObservation::Replaced {
                previous,
                replacement,
                ..
            } => Ok(ReplacementRecoveryObservation::Replaced {
                handle,
                previous,
                replacement,
            }),
            LegacyUploadObservation::Created { .. } => unreachable!(
                "the successful resolve above proves that this caller seeded a live resource"
            ),
        }
    }

    pub fn resolved(&self, handle: MeshHandle) -> Option<LogicalMesh> {
        self.renderer_view.resolve(handle)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryComparisonEvidence {
    pub rejected_replacement: ReplacementRecoveryObservation,
    pub retained_after_rejection: Option<LogicalMesh>,
    pub allowed_replacement: ReplacementRecoveryObservation,
    pub retained_after_allowed_replacement: Option<LogicalMesh>,
    pub repeated_failure_total: u64,
    pub repeated_failure_retained: usize,
}

/// Retain the Slice 4 comparison without claiming a renderer recovery API:
/// the caller may reject an invalid candidate, retain the old logical mesh,
/// then later authorize a replacement with the same logical identity.
pub fn observe_caller_staged_recovery() -> RecoveryComparisonEvidence {
    let handle = MeshHandle(77);
    let retained = LogicalMesh::Dynamic(7);
    let mut replacements = CallerStagedReplacement::default();
    replacements.seed(handle, retained);

    let rejected_replacement = replacements
        .replace_if_same_logical_identity(handle, LogicalMesh::StaticCutout(7))
        .expect("seeded handle resolves");
    let retained_after_rejection = replacements.resolved(handle);
    let allowed_replacement = replacements
        .replace_if_same_logical_identity(handle, retained)
        .expect("same retained logical identity is an explicit replacement candidate");
    let retained_after_allowed_replacement = replacements.resolved(handle);

    let mut repeated_failures = BoundedFailureObservations::<3>::default();
    for index in 0..5 {
        repeated_failures.record(
            FailureObservationPhase::RendererResourceResolution,
            FailureObservationOperation::ResolveMesh,
            FailureObservationCategory::ResourceUnresolved,
            Some(MeshHandle(900 + index)),
            "slice-4-repeat-failure",
            FailureObservationContinuation::RejectOperationAndContinue,
        );
    }

    RecoveryComparisonEvidence {
        rejected_replacement,
        retained_after_rejection,
        allowed_replacement,
        retained_after_allowed_replacement,
        repeated_failure_total: repeated_failures.total_recorded(),
        repeated_failure_retained: repeated_failures.retained().count(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepresentationEvidence {
    pub renderer_mesh_handle_bytes: usize,
    pub generational_mesh_handle_bytes: usize,
    pub logical_mesh_label_bytes: usize,
    pub lifecycle_counts_bytes: usize,
}

pub fn representation_evidence() -> RepresentationEvidence {
    RepresentationEvidence {
        renderer_mesh_handle_bytes: std::mem::size_of::<MeshHandle>(),
        generational_mesh_handle_bytes: std::mem::size_of::<GenerationalMeshHandle>(),
        logical_mesh_label_bytes: std::mem::size_of::<LogicalMesh>(),
        lifecycle_counts_bytes: std::mem::size_of::<LifecycleCounts>(),
    }
}

/// Native-only timing observation for deterministic create/resolve/replace/
/// retire pressure. Durations are retained as corpus measurements, not as a
/// portable performance budget or proof of asymptotic cost.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChurnEvidence {
    pub cycles: u32,
    pub application_registry: LifecycleCounts,
    pub application_registry_elapsed: Duration,
    pub generational_registry: LifecycleCounts,
    pub generational_registry_elapsed: Duration,
    pub explicit_lifecycle: LifecycleCounts,
    pub explicit_lifecycle_elapsed: Duration,
}

pub fn observe_churn(cycles: u32) -> ChurnEvidence {
    let started = Instant::now();
    let mut application = ApplicationMeshRegistry::default();
    for index in 0..cycles {
        let resource = LogicalMesh::Dynamic(index);
        let handle = application.create(resource);
        assert_eq!(application.resolve(handle), Ok(resource));
        application
            .replace(handle, resource)
            .expect("same application-owned identity");
        assert_eq!(application.retire(handle), Ok(resource));
        assert!(matches!(
            application.resolve(handle),
            Err(IdentityError::Missing { .. })
        ));
    }
    let application_registry_elapsed = started.elapsed();

    let started = Instant::now();
    let mut generational = GenerationalMeshRegistry::default();
    for index in 0..cycles {
        let resource = LogicalMesh::Dynamic(index);
        let handle = generational.create(resource);
        assert_eq!(generational.resolve(handle), Ok(resource));
        generational
            .replace(handle, resource)
            .expect("same generational identity");
        assert_eq!(generational.retire(handle), Ok(resource));
        assert!(matches!(
            generational.resolve(handle),
            Err(IdentityError::StaleGeneration { .. })
        ));
    }
    let generational_registry_elapsed = started.elapsed();

    let started = Instant::now();
    let mut explicit = ExplicitLifecycleLedger::default();
    for index in 0..cycles {
        let resource = LogicalMesh::Dynamic(index);
        let handle = MeshHandle(u64::from(index) + 1);
        explicit
            .create(handle, resource)
            .expect("fresh caller-selected identity");
        assert_eq!(explicit.resolve(handle), Ok(resource));
        explicit
            .replace(handle, resource)
            .expect("same explicit identity");
        assert_eq!(explicit.retire(handle), Ok(resource));
        assert!(matches!(
            explicit.resolve(handle),
            Err(IdentityError::Missing { .. })
        ));
    }
    let explicit_lifecycle_elapsed = started.elapsed();

    ChurnEvidence {
        cycles,
        application_registry: application.counts(),
        application_registry_elapsed,
        generational_registry: generational.counts(),
        generational_registry_elapsed,
        explicit_lifecycle: explicit.counts(),
        explicit_lifecycle_elapsed,
    }
}

/// Corpus-local stages used to retain the distinction between a failed source
/// operation, renderer resource resolution, provider work, and platform
/// termination. These names are comparison evidence, not a public diagnostic
/// taxonomy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureObservationPhase {
    SourcePreparation,
    RendererResourceResolution,
    ProviderValidation,
    SurfacePresentation,
    ApplicationFrameHandler,
    PlatformTermination,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureObservationOperation {
    PrepareSourceGeometry,
    ResolveMesh,
    ValidateDeclaration,
    AcquireSurfaceFrame,
    ReturnFrameError,
    EndEventLoop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureObservationCategory {
    IntentionalSourceOmission,
    SourceUnavailable,
    ResourceUnresolved,
    DeclarationRejected,
    ProviderRejected,
    SurfaceUnavailable,
    HandlerReturnedError,
    EventLoopTerminated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureObservationContinuation {
    RejectOperationAndContinue,
    EndActiveComposition,
    FatalNoContinuationClaim,
}

/// A compact, source-correlatable failure record. `caller` is a corpus
/// identity, not a diagnostic message; presentation text remains outside this
/// observation model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FailureObservation {
    pub sequence: u64,
    pub phase: FailureObservationPhase,
    pub operation: FailureObservationOperation,
    pub category: FailureObservationCategory,
    pub resource: Option<MeshHandle>,
    pub caller: &'static str,
    pub continuation: FailureObservationContinuation,
}

/// A fixed-capacity failure log. It preserves the monotonic total so callers
/// can distinguish an empty log from an overwritten older observation without
/// creating an unbounded steady-state diagnostic history.
#[derive(Clone, Debug)]
pub struct BoundedFailureObservations<const CAPACITY: usize> {
    records: [Option<FailureObservation>; CAPACITY],
    next_sequence: u64,
}

impl<const CAPACITY: usize> Default for BoundedFailureObservations<CAPACITY> {
    fn default() -> Self {
        Self {
            records: [None; CAPACITY],
            next_sequence: 0,
        }
    }
}

impl<const CAPACITY: usize> BoundedFailureObservations<CAPACITY> {
    pub fn record(
        &mut self,
        phase: FailureObservationPhase,
        operation: FailureObservationOperation,
        category: FailureObservationCategory,
        resource: Option<MeshHandle>,
        caller: &'static str,
        continuation: FailureObservationContinuation,
    ) -> FailureObservation {
        assert!(CAPACITY > 0, "failure observation capacity must be nonzero");
        let observation = FailureObservation {
            sequence: self.next_sequence,
            phase,
            operation,
            category,
            resource,
            caller,
            continuation,
        };
        let slot = (self.next_sequence as usize) % CAPACITY;
        self.records[slot] = Some(observation);
        self.next_sequence = self.next_sequence.saturating_add(1);
        observation
    }

    pub const fn total_recorded(&self) -> u64 {
        self.next_sequence
    }

    pub fn retained(&self) -> impl Iterator<Item = FailureObservation> + '_ {
        let mut records = self.records.iter().flatten().copied().collect::<Vec<_>>();
        records.sort_by_key(|record| record.sequence);
        records.into_iter()
    }
}

/// Corpus-only presentation comparison for one retained observation. The same
/// source record can be formatted for a structured sink or a console, but only
/// an intentional source omission may truthfully request a visual stand-in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailurePresentationComparison {
    pub observation: FailureObservation,
    pub structured_record: String,
    pub console_line: String,
    pub allows_explicit_visual_standin: bool,
}

/// Renders the demonstrated provider-neutral facts without copying an error
/// message, WGPU details, or application recovery policy into this fixture.
pub fn compare_failure_presentation(
    observation: FailureObservation,
) -> FailurePresentationComparison {
    let structured_record = format!(
        "sequence={}; phase={:?}; operation={:?}; category={:?}; resource={:?}; caller={}; continuation={:?}",
        observation.sequence,
        observation.phase,
        observation.operation,
        observation.category,
        observation.resource,
        observation.caller,
        observation.continuation,
    );
    let console_line = format!(
        "failure #{}: {:?} / {:?} / {:?}; caller={}; resource={:?}; continuation={:?}",
        observation.sequence,
        observation.phase,
        observation.operation,
        observation.category,
        observation.caller,
        observation.resource,
        observation.continuation,
    );
    FailurePresentationComparison {
        allows_explicit_visual_standin: matches!(
            observation.category,
            FailureObservationCategory::IntentionalSourceOmission
        ),
        observation,
        structured_record,
        console_line,
    }
}

/// Retains the four materially different cases used by Slice 5. They share a
/// record shape, not a common recovery or presentation policy.
pub fn observe_diagnostic_presentation_fixture() -> BoundedFailureObservations<4> {
    let mut observations = BoundedFailureObservations::default();
    observations.record(
        FailureObservationPhase::SourcePreparation,
        FailureObservationOperation::PrepareSourceGeometry,
        FailureObservationCategory::IntentionalSourceOmission,
        None,
        "e1m1-sky-omission",
        FailureObservationContinuation::RejectOperationAndContinue,
    );
    observations.record(
        FailureObservationPhase::SourcePreparation,
        FailureObservationOperation::PrepareSourceGeometry,
        FailureObservationCategory::SourceUnavailable,
        None,
        "e1m1-door-refresh",
        FailureObservationContinuation::RejectOperationAndContinue,
    );
    observations.record(
        FailureObservationPhase::RendererResourceResolution,
        FailureObservationOperation::ResolveMesh,
        FailureObservationCategory::ResourceUnresolved,
        Some(MeshHandle(44)),
        "identity-fixture",
        FailureObservationContinuation::RejectOperationAndContinue,
    );
    observations.record(
        FailureObservationPhase::ProviderValidation,
        FailureObservationOperation::ValidateDeclaration,
        FailureObservationCategory::ProviderRejected,
        None,
        "hello-shader-backend-diagnostic",
        FailureObservationContinuation::EndActiveComposition,
    );
    observations
}

/// Replays the six failure boundaries already observed across the E1M1,
/// shader, native-platform, and browser corpus work. It does not inject a
/// real GPU or window failure; live target observations remain separately
/// retained by their owning corpus entries.
pub fn observe_failure_boundary_fixture() -> BoundedFailureObservations<8> {
    let mut observations = BoundedFailureObservations::default();
    observations.record(
        FailureObservationPhase::SourcePreparation,
        FailureObservationOperation::PrepareSourceGeometry,
        FailureObservationCategory::SourceUnavailable,
        None,
        "e1m1-door-refresh",
        FailureObservationContinuation::RejectOperationAndContinue,
    );
    observations.record(
        FailureObservationPhase::RendererResourceResolution,
        FailureObservationOperation::ResolveMesh,
        FailureObservationCategory::ResourceUnresolved,
        Some(MeshHandle(44)),
        "identity-fixture",
        FailureObservationContinuation::RejectOperationAndContinue,
    );
    observations.record(
        FailureObservationPhase::ProviderValidation,
        FailureObservationOperation::ValidateDeclaration,
        FailureObservationCategory::ProviderRejected,
        None,
        "hello-shader-backend-diagnostic",
        FailureObservationContinuation::EndActiveComposition,
    );
    observations.record(
        FailureObservationPhase::SurfacePresentation,
        FailureObservationOperation::AcquireSurfaceFrame,
        FailureObservationCategory::SurfaceUnavailable,
        None,
        "browser-webgpu-readiness",
        FailureObservationContinuation::EndActiveComposition,
    );
    observations.record(
        FailureObservationPhase::ApplicationFrameHandler,
        FailureObservationOperation::ReturnFrameError,
        FailureObservationCategory::HandlerReturnedError,
        None,
        "e1m1-door-refresh-pre-containment",
        FailureObservationContinuation::EndActiveComposition,
    );
    observations.record(
        FailureObservationPhase::PlatformTermination,
        FailureObservationOperation::EndEventLoop,
        FailureObservationCategory::EventLoopTerminated,
        None,
        "native-window-loop",
        FailureObservationContinuation::FatalNoContinuationClaim,
    );
    observations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_logical_identity_is_a_deliberate_replacement_candidate() {
        let mut ledger = LegacyMeshLedger::default();
        let handle = MeshHandle(7);
        assert_eq!(
            ledger.upload(handle, LogicalMesh::Dynamic(4)),
            LegacyUploadObservation::Created {
                handle,
                resource: LogicalMesh::Dynamic(4),
            }
        );
        assert_eq!(
            ledger.upload(handle, LogicalMesh::Dynamic(4)),
            LegacyUploadObservation::Replaced {
                handle,
                previous: LogicalMesh::Dynamic(4),
                replacement: LogicalMesh::Dynamic(4),
            }
        );
    }

    #[test]
    fn mutable_offset_reproduces_alias_and_unresolved_reference() {
        let evidence = reproduce_mutable_offset_alias();
        assert_eq!(evidence.original_cutout_handle, evidence.dynamic_handle);
        assert_eq!(
            evidence.dynamic_upload,
            LegacyUploadObservation::Replaced {
                handle: MeshHandle(3),
                previous: LogicalMesh::StaticCutout(0),
                replacement: LogicalMesh::Dynamic(0),
            }
        );
        assert_eq!(
            evidence.original_cutout_now_resolves_to,
            Some(LogicalMesh::Dynamic(0))
        );
        assert_eq!(evidence.recomputed_cutout_resolves_to, None);
    }

    #[test]
    fn fixed_ranges_preserve_existing_live_identity() {
        let evidence = observe_fixed_disjoint_ranges();
        assert_ne!(evidence.cutout_handle, evidence.dynamic_handle);
        assert_eq!(
            evidence.cutout_resolves_to,
            Some(LogicalMesh::StaticCutout(0))
        );
        assert_eq!(evidence.dynamic_resolves_to, Some(LogicalMesh::Dynamic(0)));
    }

    #[test]
    fn application_registry_never_reuses_live_identity_and_rejects_wrong_owner() {
        let mut registry = ApplicationMeshRegistry::default();
        let cutout = registry.create(LogicalMesh::StaticCutout(0));
        let dynamic = registry.create(LogicalMesh::Dynamic(0));
        assert_ne!(cutout, dynamic);
        assert!(registry.replace(dynamic, LogicalMesh::Dynamic(0)).is_ok());
        assert!(matches!(
            registry.replace(cutout, LogicalMesh::Dynamic(0)),
            Err(IdentityError::LogicalMismatch { .. })
        ));
        registry.retire(dynamic).expect("dynamic remains live");
        assert!(matches!(
            registry.resolve(dynamic),
            Err(IdentityError::Missing { .. })
        ));
        assert_eq!(
            registry.counts(),
            LifecycleCounts {
                creates: 2,
                replacements: 1,
                retires: 1,
                rejections: 2,
            }
        );
    }

    #[test]
    fn generational_registry_rejects_reference_after_slot_reuse() {
        let mut registry = GenerationalMeshRegistry::default();
        let first = registry.create(LogicalMesh::Dynamic(0));
        registry.retire(first).expect("first live resource");
        let second = registry.create(LogicalMesh::Dynamic(1));
        assert_eq!(first.slot, second.slot);
        assert_ne!(first.generation, second.generation);
        assert!(matches!(
            registry.resolve(first),
            Err(IdentityError::StaleGeneration { .. })
        ));
        assert_eq!(registry.resolve(second), Ok(LogicalMesh::Dynamic(1)));
    }

    #[test]
    fn explicit_lifecycle_distinguishes_create_replace_and_retire() {
        let mut ledger = ExplicitLifecycleLedger::default();
        let handle = MeshHandle(12);
        ledger
            .create(handle, LogicalMesh::Dynamic(2))
            .expect("fresh create");
        assert!(matches!(
            ledger.create(handle, LogicalMesh::StaticCutout(0)),
            Err(IdentityError::AlreadyLive { .. })
        ));
        ledger
            .replace(handle, LogicalMesh::Dynamic(2))
            .expect("same-owner replacement");
        ledger.retire(handle).expect("live retire");
        assert!(matches!(
            ledger.replace(handle, LogicalMesh::Dynamic(2)),
            Err(IdentityError::Missing { .. })
        ));
    }

    #[test]
    fn validation_only_observes_unrelated_replacement_without_changing_mechanics() {
        let mut ledger = ValidationOnlyLedger::default();
        let handle = MeshHandle(3);
        ledger.upload(handle, LogicalMesh::StaticCutout(0));
        let observation = ledger.upload(handle, LogicalMesh::Dynamic(0));
        assert!(matches!(
            observation.upload,
            LegacyUploadObservation::Replaced { .. }
        ));
        assert!(matches!(
            observation.unrelated_replacement,
            Some(IdentityError::LogicalMismatch { .. })
        ));
        assert_eq!(ledger.diagnostic_count(), 1);
    }

    #[test]
    fn validation_only_retains_a_fixed_number_of_recent_mismatch_observations() {
        let mut ledger = ValidationOnlyLedger::default();
        for index in 0..5 {
            let handle = MeshHandle(u64::from(index) + 1);
            ledger.upload(handle, LogicalMesh::StaticOpaque(index));
            ledger.upload(handle, LogicalMesh::Dynamic(index));
        }
        assert_eq!(ledger.diagnostic_count(), 5);
        assert_eq!(ledger.retained_diagnostics().count(), 4);
    }

    #[test]
    fn unrelated_dynamic_addition_preserves_live_owner_identity() {
        let mut application = ApplicationMeshRegistry::default();
        let application_cutout = application.create(LogicalMesh::StaticCutout(0));
        let application_dynamic = application.create(LogicalMesh::Dynamic(0));
        assert_eq!(
            application.resolve(application_cutout),
            Ok(LogicalMesh::StaticCutout(0))
        );
        assert_ne!(application_cutout, application_dynamic);

        let mut generational = GenerationalMeshRegistry::default();
        let generational_cutout = generational.create(LogicalMesh::StaticCutout(0));
        let generational_dynamic = generational.create(LogicalMesh::Dynamic(0));
        assert_eq!(
            generational.resolve(generational_cutout),
            Ok(LogicalMesh::StaticCutout(0))
        );
        assert_ne!(generational_cutout, generational_dynamic);

        let mut explicit = ExplicitLifecycleLedger::default();
        let explicit_cutout = MeshHandle(40);
        let explicit_dynamic = MeshHandle(41);
        explicit
            .create(explicit_cutout, LogicalMesh::StaticCutout(0))
            .expect("first explicit resource");
        explicit
            .create(explicit_dynamic, LogicalMesh::Dynamic(0))
            .expect("unrelated explicit resource");
        assert_eq!(
            explicit.resolve(explicit_cutout),
            Ok(LogicalMesh::StaticCutout(0))
        );
    }

    #[test]
    fn representation_sizes_are_retained_as_evidence_not_budgets() {
        let evidence = representation_evidence();
        assert_eq!(evidence.renderer_mesh_handle_bytes, 8);
        assert_eq!(evidence.generational_mesh_handle_bytes, 8);
        assert!(evidence.logical_mesh_label_bytes > 0);
        assert_eq!(evidence.lifecycle_counts_bytes, 32);
    }

    #[test]
    fn repeated_churn_retains_expected_operation_and_rejection_counts() {
        let evidence = observe_churn(128);
        let expected = LifecycleCounts {
            creates: 128,
            replacements: 128,
            retires: 128,
            rejections: 128,
        };
        assert_eq!(evidence.application_registry, expected);
        assert_eq!(evidence.generational_registry, expected);
        assert_eq!(evidence.explicit_lifecycle, expected);
    }

    #[test]
    fn bounded_observations_preserve_phase_resource_and_continuation() {
        let mut observations = BoundedFailureObservations::<4>::default();
        let source = observations.record(
            FailureObservationPhase::SourcePreparation,
            FailureObservationOperation::PrepareSourceGeometry,
            FailureObservationCategory::SourceUnavailable,
            None,
            "e1m1-door-refresh",
            FailureObservationContinuation::RejectOperationAndContinue,
        );
        let resource = observations.record(
            FailureObservationPhase::RendererResourceResolution,
            FailureObservationOperation::ResolveMesh,
            FailureObservationCategory::ResourceUnresolved,
            Some(MeshHandle(44)),
            "identity-fixture",
            FailureObservationContinuation::RejectOperationAndContinue,
        );
        let provider = observations.record(
            FailureObservationPhase::ProviderValidation,
            FailureObservationOperation::ValidateDeclaration,
            FailureObservationCategory::ProviderRejected,
            None,
            "hello-shader",
            FailureObservationContinuation::EndActiveComposition,
        );

        assert_eq!(source.sequence, 0);
        assert_eq!(resource.resource, Some(MeshHandle(44)));
        assert_eq!(provider.phase, FailureObservationPhase::ProviderValidation);
        assert_eq!(observations.total_recorded(), 3);
        assert_eq!(observations.retained().count(), 3);
    }

    #[test]
    fn failure_observations_are_bounded_without_erasing_total_count() {
        let mut observations = BoundedFailureObservations::<2>::default();
        for index in 0..3 {
            observations.record(
                FailureObservationPhase::ApplicationFrameHandler,
                FailureObservationOperation::ReturnFrameError,
                FailureObservationCategory::HandlerReturnedError,
                Some(MeshHandle(index)),
                "frame-handler-fixture",
                FailureObservationContinuation::EndActiveComposition,
            );
        }
        assert_eq!(observations.total_recorded(), 3);
        assert_eq!(observations.retained().count(), 2);
        assert!(observations.retained().any(|record| record.sequence == 2));
    }

    #[test]
    fn bounded_observations_are_presented_in_chronological_order_after_wrap() {
        let mut observations = BoundedFailureObservations::<2>::default();
        for index in 0..3 {
            observations.record(
                FailureObservationPhase::ApplicationFrameHandler,
                FailureObservationOperation::ReturnFrameError,
                FailureObservationCategory::HandlerReturnedError,
                Some(MeshHandle(index)),
                "frame-handler-fixture",
                FailureObservationContinuation::EndActiveComposition,
            );
        }

        assert_eq!(
            observations
                .retained()
                .map(|record| record.sequence)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn diagnostic_presentation_keeps_one_record_but_separates_visual_claims() {
        let comparisons = observe_diagnostic_presentation_fixture()
            .retained()
            .map(compare_failure_presentation)
            .collect::<Vec<_>>();
        assert_eq!(comparisons.len(), 4);
        assert!(comparisons.iter().all(|comparison| comparison
            .structured_record
            .contains(comparison.observation.caller)));
        assert!(comparisons.iter().all(|comparison| comparison
            .console_line
            .contains(comparison.observation.caller)));
        assert!(comparisons.iter().all(|comparison| comparison
            .console_line
            .contains(&format!("{:?}", comparison.observation.phase))));
        assert!(comparisons.iter().all(|comparison| comparison
            .console_line
            .contains(&format!("{:?}", comparison.observation.operation))));
        assert!(comparisons.iter().all(|comparison| comparison
            .console_line
            .contains(&format!("{:?}", comparison.observation.category))));
        assert_eq!(
            comparisons
                .iter()
                .filter(|comparison| comparison.allows_explicit_visual_standin)
                .count(),
            1
        );
        assert_eq!(
            comparisons
                .iter()
                .find(|comparison| comparison.allows_explicit_visual_standin)
                .unwrap()
                .observation
                .category,
            FailureObservationCategory::IntentionalSourceOmission
        );
    }

    #[test]
    fn boundary_fixture_preserves_each_observed_failure_layer() {
        let observations = observe_failure_boundary_fixture();
        let retained = observations.retained().collect::<Vec<_>>();
        assert_eq!(observations.total_recorded(), 6);
        assert_eq!(retained.len(), 6);
        assert!(retained
            .iter()
            .any(|record| record.phase == FailureObservationPhase::SourcePreparation));
        assert!(retained
            .iter()
            .any(|record| record.phase == FailureObservationPhase::RendererResourceResolution));
        assert!(retained
            .iter()
            .any(|record| record.phase == FailureObservationPhase::ProviderValidation));
        assert!(retained
            .iter()
            .any(|record| record.phase == FailureObservationPhase::SurfacePresentation));
        assert!(retained
            .iter()
            .any(|record| record.phase == FailureObservationPhase::ApplicationFrameHandler));
        assert!(retained
            .iter()
            .any(|record| record.phase == FailureObservationPhase::PlatformTermination));
    }

    #[test]
    fn caller_staged_recovery_rejects_before_mutating_last_known_good_resource() {
        let evidence = observe_caller_staged_recovery();
        assert!(matches!(
            evidence.rejected_replacement,
            ReplacementRecoveryObservation::RejectedPreservingLastKnownGood {
                handle: MeshHandle(77),
                retained: LogicalMesh::Dynamic(7),
                rejected: LogicalMesh::StaticCutout(7),
            }
        ));
        assert_eq!(
            evidence.retained_after_rejection,
            Some(LogicalMesh::Dynamic(7))
        );
        assert!(matches!(
            evidence.allowed_replacement,
            ReplacementRecoveryObservation::Replaced {
                handle: MeshHandle(77),
                previous: LogicalMesh::Dynamic(7),
                replacement: LogicalMesh::Dynamic(7),
            }
        ));
        assert_eq!(
            evidence.retained_after_allowed_replacement,
            Some(LogicalMesh::Dynamic(7))
        );
    }

    #[test]
    fn repeat_failures_are_bounded_but_the_total_remains_visible() {
        let evidence = observe_caller_staged_recovery();
        assert_eq!(evidence.repeated_failure_total, 5);
        assert_eq!(evidence.repeated_failure_retained, 3);
    }
}
