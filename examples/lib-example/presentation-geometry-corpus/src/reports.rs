#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpusStage {
    Source,
    Xml,
    Outline,
    Vector,
    Mesh,
}

impl CorpusStage {
    pub const ALL: [Self; 5] = [
        Self::Source,
        Self::Xml,
        Self::Outline,
        Self::Vector,
        Self::Mesh,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Xml => "xml",
            Self::Outline => "outline",
            Self::Vector => "vector",
            Self::Mesh => "mesh",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageStatus {
    Ready,
    Failed,
    ExpectedFailure,
}

impl StageStatus {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::ExpectedFailure => "expected-failure",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageReport {
    pub stage: CorpusStage,
    pub status: StageStatus,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseReport {
    pub id: String,
    pub producer: String,
    pub selected_stages: Vec<CorpusStage>,
    pub stages: Vec<StageReport>,
    pub diagnostics: Vec<String>,
}

impl CaseReport {
    pub fn passed(&self) -> bool {
        self.diagnostics.is_empty()
            && self.stages.iter().all(|stage| {
                matches!(
                    stage.status,
                    StageStatus::Ready | StageStatus::ExpectedFailure
                )
            })
    }
}

pub(crate) fn failed_stage(stage: CorpusStage, message: &str) -> StageReport {
    StageReport {
        stage,
        status: StageStatus::Failed,
        summary: message.to_owned(),
    }
}
