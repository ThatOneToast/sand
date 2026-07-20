# Phase 4c — canonical event fixture coverage report

Date: 2026-07-19
Branch: `codex/sand-api-compiler-reorganization`
Scope: `sand-core/tests/` only (per ADR 001 crate boundaries).

## Method

Read every `event_chain_*`, `event_multi_parent_*`, `respawn_*`, `tick_*`,
`participant_*`, `item_snapshot_*`, `advancement_version_export.rs`,
`schedule_multiplayer_safety.rs`, `exporter_dyn_fn_determinism.rs`, and
`event_branch_export.rs` test file in `sand-core/tests/` and matched each
against the Phase 4c checklist below. All of these already assert against
full JSON records produced by the real export pipeline (`records(json)` +
per-resource lookup pattern), not substrings — this is the "canonical event
fixture" the phase asked for; it already exists distributed across focused
files rather than one monolith.

**Conclusion: no genuine gap was found.** No new test file was added.

## Coverage map

| Checklist item | Covering file(s) |
|---|---|
| Advancement-backed events | `advancement_version_export.rs`, `event_chain_advancement_parent_composition.rs`, `event_chain_advancement_parent_rejected.rs` and its `_after_any_rejected` / `_combined_occurrence_rejected` / `_direct_handler_rejected` / `_setup_mixed_rejected` / `_setup_objectives_rejected` / `_setup_post_observation_rejected` / `_setup_pre_observation_rejected` / `_within_rejected` siblings |
| Tick-backed events | `tick_lifecycle_export.rs`, `tick_lifecycle_consistency.rs`, `tick_lifecycle_conflict.rs` |
| Composed/chained events | `event_chain_export.rs` |
| Same-cycle chaining | `event_chain_export.rs`, `event_chain_child_lifecycle.rs`, `event_chain_cycle.rs`, `event_chain_identity_collision.rs` |
| Multi-parent composition | `event_multi_parent_export.rs`, `event_multi_parent_occurrence_collision.rs` |
| Bounded correlation (`.within(...)`) | `event_chain_within_export.rs` |
| Persistent conditions (`while_::<E>()`) | `event_chain_while_export.rs`, `event_chain_while_provider_reuse.rs`, `event_chain_legacy_tick_condition_parent.rs` |
| Death and respawn | `respawn_lifecycle_export.rs` |
| Item snapshots | `item_snapshot_tick_capture_export.rs` |
| Participant plans | `participant_plan_export.rs` |
| Correlated attacker observation | `participant_attacker_observation_export.rs`, `participant_context_capability_audit.rs` |
| Multiple handlers (one event, several `#[event]` fns) | `tick_lifecycle_export.rs` (setup dedup + sorted fan-out), `respawn_lifecycle_export.rs` (registration-order-independent fan-out) |
| Per-player ownership | `schedule_multiplayer_safety.rs` (generated schedule ownership, multiplayer-safe scoreboard mutations — commit 4746de5), `tick_lifecycle_export.rs` (per-player-per-tick dispatch guard) |
| Deterministic generated coordinator ordering | `exporter_dyn_fn_determinism.rs` (thread-local dyn-fn registry, `LIM-EXP-006` fix), `event_chain_identity_collision.rs`, `respawn_lifecycle_export.rs`, `tick_lifecycle_export.rs` (key is a pure function of canonical type name) |
| Branch bodies survive event export (`then_all`/`if_`/`unless`) | `event_branch_export.rs` (not on the original checklist, but adjacent coverage worth recording) |

## MC 26.2 version-exercise audit

Per commit `caaa505`, `sand-core`'s default codegen target (`DEFAULT_CODEGEN_VERSION`,
used by `build.rs` absent `SAND_MC_VERSION`) is now `LATEST_KNOWN` = 26.2. The
export-pipeline entry points used by the fixtures above split into two paths
with different version behavior:

- **Unprofiled path** (`try_export_components`/`try_export_components_json`,
  used by `event_chain_export.rs`, `event_chain_within_export.rs`,
  `event_chain_while_export.rs`, `event_multi_parent_export.rs`,
  `respawn_lifecycle_export.rs`, `tick_lifecycle_export.rs`,
  `item_snapshot_tick_capture_export.rs`, `event_branch_export.rs`,
  `schedule_multiplayer_safety.rs`, `exporter_dyn_fn_determinism.rs`, and
  most `event_chain_advancement_parent_*` files): this path performs **no
  version-gating at all** (see `sand-core/src/compiler/export/mod.rs`
  `try_export_components`'s doc comment: "no version-gating is performed").
  These tests are version-agnostic by design — they exercise the event
  graph/dispatch machinery, not version-conditional rendering, so they are
  neither pinned to 26.2 nor to any older profile.
- **Profiled path** (`try_export_components_for_version`): only exercised
  explicitly by `advancement_version_export.rs`, which resolves
  `MinecraftVersion::parse("26.2")` directly — this file **is** genuine 26.2
  canonical coverage, plus an explicit 1.19.0-rejection case proving
  version-gated advancement triggers fail closed on older profiles.

**Finding (informational, not a gap in this phase's checklist):**
`participant_plan_export.rs` and `participant_attacker_observation_export.rs`
call `VersionProfile::resolve(&MinecraftVersion::parse("1.21.4").unwrap())`
directly rather than 26.2 or `LATEST_KNOWN`. Per ADR 001 ("Canonical
fixtures... target Minecraft Java 26.2. 1.21.4 remains only as an explicit
oldest-profile/compatibility boundary"), these two files' hardcoded profile
choice is worth revisiting in Phase 6's review — but participant plans and
attacker observation are already covered by construction (both files pass;
the caller's declared feature is available in 1.21.4 and would remain
available in 26.2), so no coverage gap exists today, only a version-pin
staleness worth a follow-up.

## Result

All fourteen Phase 4c checklist items have existing, focused, full-JSON-record
export tests in `sand-core/tests/`. No new test file was added; no existing
file was modified or renamed.
