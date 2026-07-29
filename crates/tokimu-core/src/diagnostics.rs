use std::fmt;

const DEFAULT_DIAGNOSTIC_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticKind {
    Message,
    PerformanceBudgetExceeded,
    PerformanceRecovered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerformanceUnit {
    Seconds,
    Milliseconds,
    Count,
}

impl fmt::Display for PerformanceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seconds => f.write_str("s"),
            Self::Milliseconds => f.write_str("ms"),
            Self::Count => f.write_str("count"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceObservation {
    pub metric: String,
    pub observed: f64,
    pub budget: f64,
    pub unit: PerformanceUnit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticRecord {
    sequence: u64,
    pub severity: DiagnosticSeverity,
    pub kind: DiagnosticKind,
    pub source: String,
    pub message: String,
    pub performance: Option<PerformanceObservation>,
}

impl DiagnosticRecord {
    fn message(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            sequence: 0,
            severity: DiagnosticSeverity::Info,
            kind: DiagnosticKind::Message,
            source: source.into(),
            message: message.into(),
            performance: None,
        }
    }

    fn performance(
        severity: DiagnosticSeverity,
        kind: DiagnosticKind,
        source: impl Into<String>,
        message: impl Into<String>,
        performance: PerformanceObservation,
    ) -> Self {
        Self {
            sequence: 0,
            severity,
            kind,
            source: source.into(),
            message: message.into(),
            performance: Some(performance),
        }
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl fmt::Display for DiagnosticRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            DiagnosticSeverity::Info => "info",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Error => "error",
        };
        write!(
            f,
            "{severity} [{}] {}: {}",
            self.sequence, self.source, self.message
        )
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostics {
    startup_messages: Vec<String>,
    records: Vec<DiagnosticRecord>,
    capacity: usize,
    next_sequence: u64,
    dropped_records: u64,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_DIAGNOSTIC_CAPACITY)
    }
}

impl Diagnostics {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            startup_messages: Vec::new(),
            records: Vec::with_capacity(capacity),
            capacity,
            next_sequence: 0,
            dropped_records: 0,
        }
    }

    pub fn record(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.startup_messages.push(message.clone());
        self.emit(DiagnosticRecord::message("startup", message));
    }

    pub fn emit(&mut self, mut record: DiagnosticRecord) {
        record.sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);

        if self.capacity == 0 {
            self.dropped_records = self.dropped_records.saturating_add(1);
            return;
        }
        if self.records.len() == self.capacity {
            self.records.remove(0);
            self.dropped_records = self.dropped_records.saturating_add(1);
        }
        self.records.push(record);
    }

    pub fn startup_messages(&self) -> &[String] {
        &self.startup_messages
    }

    pub fn records(&self) -> &[DiagnosticRecord] {
        &self.records
    }

    pub fn drain(&mut self) -> Vec<DiagnosticRecord> {
        std::mem::take(&mut self.records)
    }

    pub fn dropped_records(&self) -> u64 {
        self.dropped_records
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceBudget {
    pub source: String,
    pub metric: String,
    pub limit: f64,
    pub unit: PerformanceUnit,
    pub required_consecutive_violations: u32,
}

impl PerformanceBudget {
    pub fn new(
        source: impl Into<String>,
        metric: impl Into<String>,
        limit: f64,
        unit: PerformanceUnit,
    ) -> Self {
        Self {
            source: source.into(),
            metric: metric.into(),
            limit,
            unit,
            required_consecutive_violations: 3,
        }
    }

    pub fn with_required_consecutive_violations(mut self, required: u32) -> Self {
        self.required_consecutive_violations = required.max(1);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceMonitor {
    budget: PerformanceBudget,
    consecutive_violations: u32,
    degraded: bool,
}

impl PerformanceMonitor {
    pub fn new(budget: PerformanceBudget) -> Self {
        Self {
            budget,
            consecutive_violations: 0,
            degraded: false,
        }
    }

    pub fn observe(&mut self, observed: f64, diagnostics: &mut Diagnostics) {
        if !observed.is_finite() || !self.budget.limit.is_finite() || self.budget.limit < 0.0 {
            return;
        }

        if observed > self.budget.limit {
            self.consecutive_violations = self.consecutive_violations.saturating_add(1);
            if !self.degraded
                && self.consecutive_violations >= self.budget.required_consecutive_violations
            {
                self.degraded = true;
                diagnostics.emit(DiagnosticRecord::performance(
                    DiagnosticSeverity::Warning,
                    DiagnosticKind::PerformanceBudgetExceeded,
                    self.budget.source.clone(),
                    format!(
                        "{} exceeded its budget for {} consecutive observations: {:.3} {} > {:.3} {}",
                        self.budget.metric,
                        self.consecutive_violations,
                        observed,
                        self.budget.unit,
                        self.budget.limit,
                        self.budget.unit,
                    ),
                    self.observation(observed),
                ));
            }
            return;
        }

        self.consecutive_violations = 0;
        if self.degraded {
            self.degraded = false;
            diagnostics.emit(DiagnosticRecord::performance(
                DiagnosticSeverity::Info,
                DiagnosticKind::PerformanceRecovered,
                self.budget.source.clone(),
                format!(
                    "{} recovered within budget: {:.3} {} <= {:.3} {}",
                    self.budget.metric,
                    observed,
                    self.budget.unit,
                    self.budget.limit,
                    self.budget.unit,
                ),
                self.observation(observed),
            ));
        }
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded
    }

    pub fn consecutive_violations(&self) -> u32 {
        self.consecutive_violations
    }

    pub fn budget(&self) -> &PerformanceBudget {
        &self.budget
    }

    fn observation(&self, observed: f64) -> PerformanceObservation {
        PerformanceObservation {
            metric: self.budget.metric.clone(),
            observed,
            budget: self.budget.limit,
            unit: self.budget.unit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_monitor() -> PerformanceMonitor {
        PerformanceMonitor::new(
            PerformanceBudget::new("runtime", "frame time", 16.0, PerformanceUnit::Milliseconds)
                .with_required_consecutive_violations(2),
        )
    }

    #[test]
    fn performance_monitor_emits_after_sustained_violations_and_on_recovery() {
        let mut diagnostics = Diagnostics::default();
        let mut monitor = frame_monitor();

        monitor.observe(20.0, &mut diagnostics);
        assert!(diagnostics.records().is_empty());

        monitor.observe(21.0, &mut diagnostics);
        assert!(monitor.is_degraded());
        assert_eq!(diagnostics.records().len(), 1);
        assert_eq!(
            diagnostics.records()[0].kind,
            DiagnosticKind::PerformanceBudgetExceeded
        );

        monitor.observe(22.0, &mut diagnostics);
        assert_eq!(diagnostics.records().len(), 1);

        monitor.observe(15.0, &mut diagnostics);
        assert!(!monitor.is_degraded());
        assert_eq!(diagnostics.records().len(), 2);
        assert_eq!(
            diagnostics.records()[1].kind,
            DiagnosticKind::PerformanceRecovered
        );
    }

    #[test]
    fn diagnostics_retains_a_bounded_capture() {
        let mut diagnostics = Diagnostics::with_capacity(2);
        diagnostics.record("one");
        diagnostics.record("two");
        diagnostics.record("three");

        assert_eq!(diagnostics.startup_messages(), ["one", "two", "three"]);
        assert_eq!(diagnostics.records().len(), 2);
        assert_eq!(diagnostics.records()[0].sequence(), 1);
        assert_eq!(diagnostics.records()[1].sequence(), 2);
        assert_eq!(diagnostics.dropped_records(), 1);
    }
}
