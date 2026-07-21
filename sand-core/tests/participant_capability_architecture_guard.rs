//! Architecture guard (#274): the orphaned participant capability-
//! propagation bookkeeping that #274 removed
//! (`EventContextCapabilities::for_event_with_participants`,
//! `capabilities::full::{propagate_after, merge_after_any, merge_after_all,
//! propagate_within}`, and the `EntityParticipantCapability`/
//! `ItemParticipantCapability`/`LocationParticipantCapability`/
//! `ResolvedEventContextCapabilities` types that only existed to feed it)
//! must not quietly come back. Real participant (entity/item) propagation
//! across event graph edges is
//! `EventParticipantPlan::inherit_entity`/`inherit_item` plus
//! `sand-core/src/compiler/export/participant_transport.rs`'s export-time
//! validation — see `sand-core/tests/event_chain_participant_inheritance*.rs`
//! and `sand-core/tests/event_chain_participant_inheritance_diag_*.rs` for
//! that real behavior's coverage, and
//! `sand-core/tests/participant_context_capability_audit.rs`/
//! `participant_plan_export.rs` for the still-live subject-capability model
//! this file does not duplicate.

use sand_core::participant::{EventContextCapabilities, SubjectCapability, SubjectScope};

/// `EventContextCapabilities` is exhaustively constructed with only a
/// `subject` field. If a future change re-adds `entities`/`items`/
/// `locations` fields (the shape the removed participant-capability
/// bookkeeping needed), this literal stops compiling — a visible, deliberate
/// diff here, not a silent reintroduction.
#[test]
fn event_context_capabilities_is_subject_only() {
    let caps = EventContextCapabilities {
        subject: SubjectCapability::NONE,
    };
    assert_eq!(caps.subject.scope, SubjectScope::NonPlayerUnsupported);
}

/// Defining occurrences of the removed symbols (not prose mentions — the
/// module docs legitimately reference these names in the historical
/// explanation of why they were removed) must not reappear anywhere under
/// `sand-core/src`.
#[test]
fn removed_participant_capability_symbols_do_not_reappear() {
    const FORBIDDEN_DEFINITIONS: &[&str] = &[
        "fn for_event_with_participants",
        "mod full",
        "struct EntityParticipantCapability",
        "struct ItemParticipantCapability",
        "struct LocationParticipantCapability",
        "struct ResolvedEventContextCapabilities",
        "enum CapabilityOccurrence",
        "type VersionFloor",
        "enum CapabilityDescriptorError",
    ];

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    assert!(!files.is_empty(), "no .rs files under {root:?}");

    for file in files {
        let source =
            std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        for (idx, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            // Doc comments are the sanctioned place for these names to
            // appear in prose (module docs explaining the #274 removal).
            if trimmed.starts_with("//") {
                continue;
            }
            for forbidden in FORBIDDEN_DEFINITIONS {
                if line.contains(forbidden) {
                    violations.push(format!("{}:{}: {}", file.display(), idx + 1, line.trim()));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "removed participant capability-propagation symbols reappeared:\n{}",
        violations.join("\n")
    );
}

fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {dir:?}: {e}")) {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}
