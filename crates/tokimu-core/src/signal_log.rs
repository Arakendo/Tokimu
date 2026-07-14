use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignalLog {
    entries: Vec<String>,
}

impl SignalLog {
    pub fn record(&mut self, signal: impl Into<String>) {
        self.entries.push(signal.into());
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

impl fmt::Display for SignalLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "signal log")?;
        for (index, entry) in self.entries.iter().enumerate() {
            writeln!(f, "  {}: {}", index, entry)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_signals_in_arrival_order() {
        let mut log = SignalLog::default();
        log.record("enemy-stepped");
        log.record("dialogue-opened");
        log.record("enemy-stepped");

        assert_eq!(log.entries(), &["enemy-stepped", "dialogue-opened", "enemy-stepped"]);
        assert_eq!(format!("{log}"), "signal log\n  0: enemy-stepped\n  1: dialogue-opened\n  2: enemy-stepped\n");
    }
}