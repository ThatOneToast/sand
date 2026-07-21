//! Deduplication of repeated copies of the same root diagnostic.
//!
//! Minecraft frequently logs the same underlying failure once per tick (a
//! function that fails every tick) or once per affected entity/chunk. The
//! [`Deduplicator`] tracks runs of consecutive diagnostics that share a
//! [`Diagnostic::fingerprint`], so only the first occurrence of a run is
//! rendered in full and the rest are folded into a trailing repeat count —
//! without merging diagnostics that merely happen to share a code.

use super::diagnostic::Diagnostic;

/// What to do with a diagnostic just observed by the deduplicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    /// First occurrence of this fingerprint (or the fingerprint differs
    /// from the currently-running one): render it, and if a previous run
    /// just ended, its repeat summary is returned alongside.
    New {
        diagnostic: Box<Diagnostic>,
        previous_repeat: Option<RepeatSummary>,
    },
    /// A repeat of the currently-running fingerprint: nothing new to
    /// render, the running count was simply incremented.
    Repeated,
}

/// How many extra times the just-finished run repeated beyond its first
/// occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatSummary {
    pub fingerprint: String,
    /// Total occurrences observed for the run, including the first.
    pub count: usize,
}

#[derive(Debug, Default)]
pub struct Deduplicator {
    running: Option<(String, usize)>,
}

impl Deduplicator {
    pub fn new() -> Self {
        Self { running: None }
    }

    /// Feed one diagnostic in and get back what the caller should do.
    pub fn observe(&mut self, diagnostic: Diagnostic) -> Observation {
        let fingerprint = diagnostic.fingerprint();

        if let Some((running_fp, count)) = &mut self.running
            && *running_fp == fingerprint
        {
            *count += 1;
            return Observation::Repeated;
        }

        let previous_repeat = self.running.take().and_then(|(fp, count)| {
            if count > 1 {
                Some(RepeatSummary {
                    fingerprint: fp,
                    count,
                })
            } else {
                None
            }
        });

        self.running = Some((fingerprint, 1));
        Observation::New {
            diagnostic: Box::new(diagnostic),
            previous_repeat,
        }
    }

    /// Call when the stream ends (or on a long quiet period) to get the
    /// trailing repeat summary for whatever run is still open.
    pub fn flush(&mut self) -> Option<RepeatSummary> {
        self.running.take().and_then(|(fp, count)| {
            if count > 1 {
                Some(RepeatSummary {
                    fingerprint: fp,
                    count,
                })
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::diagnostic::{DiagnosticCode, Severity};
    use crate::console::phase::RunPhase;

    fn diag(resource: &str, reason: &str) -> Diagnostic {
        Diagnostic {
            phase: RunPhase::Runtime,
            severity: Severity::Warning,
            code: DiagnosticCode::RuntimeCommandError,
            resource: Some(resource.to_string()),
            subsystem: Some("function".to_string()),
            file: None,
            position: None,
            context: None,
            reason: reason.to_string(),
            hint: None,
            raw_lines: vec![],
        }
    }

    #[test]
    fn first_occurrence_is_new_with_no_previous_repeat() {
        let mut dedup = Deduplicator::new();
        match dedup.observe(diag("arcane:tick", "boom")) {
            Observation::New {
                previous_repeat, ..
            } => assert!(previous_repeat.is_none()),
            Observation::Repeated => panic!("expected New"),
        }
    }

    #[test]
    fn consecutive_identical_diagnostics_are_folded() {
        let mut dedup = Deduplicator::new();
        assert!(matches!(
            dedup.observe(diag("arcane:tick", "boom")),
            Observation::New { .. }
        ));
        assert_eq!(
            dedup.observe(diag("arcane:tick", "boom")),
            Observation::Repeated
        );
        assert_eq!(
            dedup.observe(diag("arcane:tick", "boom")),
            Observation::Repeated
        );

        let summary = dedup.flush().expect("should have a pending repeat run");
        assert_eq!(summary.count, 3);
    }

    #[test]
    fn different_diagnostic_ends_the_running_repeat_and_reports_it() {
        let mut dedup = Deduplicator::new();
        assert!(matches!(
            dedup.observe(diag("arcane:tick", "boom")),
            Observation::New { .. }
        ));
        assert_eq!(
            dedup.observe(diag("arcane:tick", "boom")),
            Observation::Repeated
        );

        match dedup.observe(diag("arcane:other", "different")) {
            Observation::New {
                previous_repeat, ..
            } => {
                let summary = previous_repeat.expect("previous run repeated");
                assert_eq!(summary.count, 2);
            }
            Observation::Repeated => panic!("different fingerprint must not be folded"),
        }
    }

    #[test]
    fn single_occurrence_run_has_no_repeat_summary_on_flush() {
        let mut dedup = Deduplicator::new();
        assert!(matches!(
            dedup.observe(diag("arcane:tick", "boom")),
            Observation::New { .. }
        ));
        assert!(dedup.flush().is_none());
    }

    #[test]
    fn unrelated_diagnostics_with_shared_code_are_not_merged() {
        let mut dedup = Deduplicator::new();
        assert!(matches!(
            dedup.observe(diag("arcane:tick_a", "boom")),
            Observation::New { .. }
        ));
        // Same code, different resource: must not be treated as a repeat.
        match dedup.observe(diag("arcane:tick_b", "boom")) {
            Observation::New { .. } => {}
            Observation::Repeated => panic!("different resource must not dedupe together"),
        }
    }
}
