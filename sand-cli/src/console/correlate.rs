//! Correlates a load-time root failure (e.g. a function parse error) with a
//! downstream consequence that references the same resource (e.g. a
//! `Couldn't load tag ... missing following references` failure caused by
//! it), so the consequence is presented as a related note on the root
//! diagnostic instead of an unrelated second top-level failure.
//!
//! Real Minecraft always logs the root cause before its consequence (a
//! function fails to parse, *then* the tag that references it fails to
//! resolve), so a single-slot, one-step lookahead is enough: hold the most
//! recent correlation-eligible root diagnostic, and either fold the very
//! next diagnostic into it (if it's a matching consequence) or flush it
//! unmodified and start considering the new one. [`Correlator::flush`] must
//! be called on a quiet period/EOF so a trailing root is never held forever.

use super::diagnostic::Diagnostic;

#[derive(Debug, Default)]
pub struct Correlator {
    pending_root: Option<Diagnostic>,
}

impl Correlator {
    pub fn new() -> Self {
        Self { pending_root: None }
    }

    /// Feed one diagnostic in. Returns the diagnostics ready to hand
    /// downstream, in order: empty (this diagnostic was folded into the
    /// still-pending root as a consequence and the root keeps waiting —
    /// this only happens when this diagnostic completed the picture and
    /// the merged root is returned instead; see below), one (the common
    /// case), or two (an unrelated pending root gets flushed, followed by
    /// this diagnostic starting a new pending root or passing straight
    /// through).
    pub fn observe(&mut self, diagnostic: Diagnostic) -> Vec<Diagnostic> {
        if let Some(missing) = diagnostic.missing_reference_target()
            && let Some(root) = &self.pending_root
            && root.resource.as_deref() == Some(missing)
        {
            let mut root = self.pending_root.take().expect("checked Some above");
            diagnostic.attach_as_related(&mut root);
            return vec![root];
        }

        let mut out = Vec::new();
        if let Some(root) = self.pending_root.take() {
            out.push(root);
        }

        if diagnostic.is_correlation_root() {
            self.pending_root = Some(diagnostic);
        } else {
            out.push(diagnostic);
        }
        out
    }

    /// Flush any diagnostic still held waiting for a possible consequence.
    /// Call on a quiet period/stream EOF so a trailing root is never lost.
    pub fn flush(&mut self) -> Option<Diagnostic> {
        self.pending_root.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::classify::classify;
    use crate::console::diagnostic::{DiagnosticCode, Severity, build_diagnostic};
    use crate::console::diagnostic::{GroupedEvent, Grouper};
    use crate::console::log_record::{Stream, parse_line};
    use crate::console::phase::RunPhase;

    fn group_lines(lines: &[&str]) -> Vec<GroupedEvent> {
        let mut grouper = Grouper::new();
        let mut out = Vec::new();
        for line in lines {
            let record = parse_line(line);
            let category = classify(Stream::Stdout, &record);
            out.extend(grouper.feed(Stream::Stdout, record, category));
        }
        if let Some(finished) = grouper.flush() {
            out.push(finished);
        }
        out
    }

    fn diagnostics(lines: &[&str], phase: RunPhase) -> Vec<Diagnostic> {
        group_lines(lines)
            .iter()
            .filter_map(|g| build_diagnostic(g, phase))
            .collect()
    }

    #[test]
    fn links_missing_tag_consequence_to_its_root_function_failure() {
        let diags = diagnostics(
            &[
                "[10:15:32] [Server thread/ERROR]: Failed to load function vanilla_plus:on_load",
                "java.util.concurrent.CompletionException: java.lang.IllegalArgumentException: Whilst parsing command on line 6: Unknown or incomplete command. See below for error at position 0: <--[HERE]",
                "\tat java.base/java.util.concurrent.CompletableFuture.wrapInCompletionException(CompletableFuture.java:323)",
                "Caused by: java.lang.IllegalArgumentException: Whilst parsing command on line 6: Unknown or incomplete command. See below for error at position 0: <--[HERE]",
                "\t... 8 more",
                "[10:15:32] [Server thread/WARN]: Couldn't load tag minecraft:load as it is missing following references: vanilla_plus:on_load (from file/vanilla_plus)",
            ],
            RunPhase::DatapackDiscovery,
        );
        assert_eq!(diags.len(), 2);

        let mut correlator = Correlator::new();
        let mut out = Vec::new();
        for d in diags {
            out.extend(correlator.observe(d));
        }
        if let Some(f) = correlator.flush() {
            out.push(f);
        }

        assert_eq!(
            out.len(),
            1,
            "consequence must fold into root, not stand alone"
        );
        let root = &out[0];
        assert_eq!(root.code, DiagnosticCode::CommandParseError);
        assert_eq!(root.resource.as_deref(), Some("vanilla_plus:on_load"));
        assert_eq!(root.related.len(), 1);
        let related = &root.related[0];
        assert_eq!(related.code, DiagnosticCode::MissingReference);
        assert_eq!(related.resource, "minecraft:load");
        assert_eq!(related.missing, "vanilla_plus:on_load");
        assert_eq!(related.source.as_deref(), Some("file/vanilla_plus"));
    }

    #[test]
    fn standalone_missing_tag_reference_still_classifies_independently() {
        let diags = diagnostics(
            &[
                "[10:15:32] [Server thread/WARN]: Couldn't load tag minecraft:load as it is missing following references: vanilla_plus:on_load (from file/vanilla_plus)",
            ],
            RunPhase::DatapackDiscovery,
        );
        assert_eq!(diags.len(), 1);

        let mut correlator = Correlator::new();
        let mut out = correlator.observe(diags.into_iter().next().unwrap());
        if let Some(f) = correlator.flush() {
            out.push(f);
        }

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].code, DiagnosticCode::MissingReference);
        assert!(out[0].related.is_empty());
    }

    #[test]
    fn unrelated_root_is_flushed_before_a_non_matching_diagnostic() {
        let root = diagnostics(
            &["[10:15:32] [Server thread/ERROR]: Failed to load function pack_a:broken"],
            RunPhase::DatapackDiscovery,
        )
        .remove(0);
        let unrelated = diagnostics(
            &["[10:15:33] [Server thread/WARN]: Couldn't load tag minecraft:load as it is missing following references: pack_b:other (from file/pack_b)"],
            RunPhase::DatapackDiscovery,
        )
        .remove(0);

        let mut correlator = Correlator::new();
        assert!(correlator.observe(root).is_empty());
        let out = correlator.observe(unrelated);

        assert_eq!(out.len(), 2, "both must be emitted, unmerged");
        assert!(out[0].related.is_empty());
        assert!(out[1].related.is_empty());
    }

    #[test]
    fn flush_returns_a_root_with_no_consequence() {
        let root = diagnostics(
            &["[10:15:32] [Server thread/ERROR]: Failed to load function pack_a:broken"],
            RunPhase::DatapackDiscovery,
        )
        .remove(0);

        let mut correlator = Correlator::new();
        assert!(correlator.observe(root).is_empty());
        let flushed = correlator.flush().expect("root should flush");
        assert!(flushed.related.is_empty());
        assert!(correlator.flush().is_none());
    }

    #[test]
    fn a_json_component_diagnostic_with_no_consequence_still_flushes_cleanly() {
        // A JsonComponentError with a resource is itself a correlation
        // root candidate (in case a consequence follows), so it's held
        // for one step rather than emitted immediately — but it must
        // still come out, unmodified, on flush.
        let events = group_lines(&[
            "[12:00:00] [Server thread/WARN]: Skipping loading recipe minecraft:foo",
        ]);
        let diag = build_diagnostic(&events[0], RunPhase::ServerStartup).unwrap();
        assert_eq!(diag.severity, Severity::Warning);
        let mut correlator = Correlator::new();
        assert!(correlator.observe(diag).is_empty());
        let flushed = correlator.flush().expect("should flush the held root");
        assert!(flushed.related.is_empty());
    }
}
