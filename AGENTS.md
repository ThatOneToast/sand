# Agent instructions for Sand

Sand is a strongly-typed Rust framework that generates **vanilla** Minecraft
Java Edition datapacks (and optional resource packs) — normal files under
`data/<namespace>/...`, not a mod or a runtime. It is **alpha** software
(`ai/project-status.yaml`): core attribute-first authoring is stable; events,
dialogs, and resource-pack generation are alpha; `mcfunction!` and
unreleased-version registry coverage are experimental. Not published to
crates.io — built from workspace source.

## Authority order

When sources disagree, prefer them in this order:

1. **Source and tests** — `sand-components/src/registry_coverage.rs` and
   `sand-components/src/advancement/trigger_coverage.rs` are the ground-truth
   coverage tables. Rust doc comments and `#[test]`s beat prose.
2. **`ai/*.yaml` manifests** — reviewed alongside source, updated with each
   PR that changes public behavior (see `ai/maintenance.md`).
3. **`ai/*.md` guides** (`authoring-guide.md`, `known-limitations.md`) —
   decision rules and consolidated gaps.
4. **`book/src/`** (mdBook) and **`docs/`** — tutorials and reference prose.
   Some `book/src/*.md` files are not linked from `book/src/SUMMARY.md` and
   drift from same-named `docs/*.md` files; see `ai/known-limitations.md`
   (`LIM-DOC-001`) before trusting either in isolation.
5. **`ROADMAP.md` / `RELEASE.md` / `CHANGELOG.md`** — directional intent, not
   proof of current behavior.
6. **`Milestones.md` / `Datapacks.md`** — historical only. `Milestones.md`
   self-labels as v0.1.0 history. `Datapacks.md` is generic vanilla
   1.21.11 reference notes, not Sand documentation. Do not cite either as
   current Sand behavior.

Never infer that a method, type, command, or capability exists from naming
conventions or "it would make sense." If you can't point to a source file,
test, or manifest entry, say so instead of guessing.

## Preferred authoring model

Sand is attribute-first. Normal pack code imports:

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};
```

- `sand_core::prelude` — the default surface: macros, typed state
  (`ScoreVar`, `Flag`, `Cooldown`, `Timer`, `StorageVar`), typed condition
  (`Condition`, `all!`, `any!`), typed execute, typed text, command builders,
  datapack component builders, version types, raw escape hatches, and
  feature-gated systems exports.
- `sand_core::advanced` — lower-level export/registry/dispatch internals for
  framework-level integration. Not the starting point for pack authoring.
- `sand_core::compat` — a single deprecated alias (`TypedEvent`) kept for
  source compatibility with older code. Don't use it in new code.

Enable optional systems only as needed via Cargo features on `sand-core`:
`systems-damage`, `systems-cooldowns`, `systems-lifecycle`,
`systems-player-data` (implies `systems-lifecycle`), `systems-movement`,
`systems-inventory`, `systems-entities`, or `systems-all`. Never assume a
systems module is available without checking the caller's `Cargo.toml`
feature list.

## Task workflow

1. **Read project configuration.** Find `sand.toml` (`[pack] namespace`,
   `description`, `mc_version`, optional `pack_format`; optional
   `[resourcepack]`). `mc_version = "latest"` resolves to
   `sand_version::LATEST_KNOWN`.
2. **Identify capabilities.** Map the user's request to one or more
   capability IDs in `ai/capability-manifest.yaml`.
3. **Confirm target-version support.** Check each capability's `minecraft`
   range and any `version_gate` in `registry_coverage.rs` /
   `trigger_coverage.rs` against the project's `mc_version`. Unknown/future
   versions fall back to conservative capabilities via
   `VersionProfile::resolve()` — use `resolve_strict()` when you need a hard
   failure instead of silent fallback.
4. **Select a verified recipe.** Start from `ai/recipes/` or an existing
   `examples/*.rs` / `book/src/recipes/*.md` file rather than inventing
   structure from scratch.
5. **Implement with typed APIs.** Use the capability's `preferred_api`. Only
   drop to `raw_escape_hatch` (`cmd::raw`, `RawJson`, `RawSnbt`,
   `RawComponent`) when the manifest marks the capability `partial`,
   `raw_only`, or the specific field/variant you need is documented as
   missing. Never use raw commands to bypass a typed API that already covers
   the operation.
6. **Build and test.** `cargo build --workspace`, `cargo test --workspace
   --all-features` (or the crate you touched), `cargo fmt --all -- --check`,
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
   `scripts/check.sh` runs the full pre-release set including mdBook.
7. **Inspect generated output.** Run `cargo run -p sand -- build` in a
   scaffolded/example project and read the emitted JSON/`.mcfunction` files
   under `dist/`. A passing Rust test is not proof the datapack loads in
   vanilla — see `cli-validate` in the capability manifest and
   `scripts/validate-vanilla-reload.sh`, which is opt-in and not run by
   default `cargo test`.
8. **State remaining limitations honestly.** If a capability is `partial`,
   `experimental`, `raw_only`, or `unknown`, say so in your response, citing
   the capability ID. Check `ai/known-limitations.md` for known vanilla vs.
   Sand boundaries before claiming something is impossible.

## Capability status vocabulary

From `ai/capability-manifest.yaml` (`status:` field):

| Status | Meaning |
|---|---|
| `implemented` | Typed Sand module exists, generates correct output, has test/example evidence. |
| `partial` | Typed module exists but covers only some fields/variants; check `sand_limitations`. |
| `experimental` | Works but the API surface is still changing; flag this to the user. |
| `raw_only` | No typed module; reachable only via `RawComponent`/`RawJson`/`cmd::raw`. |
| `planned` | No implementation yet, but a tracked issue/TODO/roadmap item exists as evidence. Never mark something `planned` without a citable source — use `unknown` instead. |
| `intentionally_unsupported` | Deliberately out of scope (documented reason), not a gap to fill. |
| `vanilla_impossible` | Not achievable in vanilla Minecraft at all, regardless of Sand. |
| `unknown` | Status not verified during the last review; do not assume either way. |

Distinguish **vanilla limitations** (no server-side command/mechanism can do
this, e.g. arbitrary client-side custom GUIs without resource-pack tricks)
from **Sand limitations** (vanilla supports it, Sand's typed API doesn't
cover it yet). The manifest's `vanilla_limitations` vs `sand_limitations`
fields make this explicit — read both before telling a user something can't
be done.

## Verification requirements before reporting a task done

- Code compiles: `cargo build` for the touched crate(s), or the workspace.
- Relevant tests pass: `cargo test -p <crate>` at minimum; `cargo test
  --workspace --all-features` for cross-crate changes.
- Generated output was actually inspected (`dist/` contents), not assumed.
- If public behavior changed (new/changed API, feature gate, CLI command,
  version support, experimental status, raw escape hatch, or validation
  coverage), update the relevant `ai/*.yaml`/`ai/*.md` file per
  `ai/maintenance.md`'s checklist in the same change.

## Directory map

| Need | Read |
|---|---|
| Current project maturity, CLI/test status | `ai/project-status.yaml` |
| Is a feature supported, and how well | `ai/capability-manifest.yaml` |
| How to implement a specific kind of pack | `ai/authoring-guide.md` |
| Known boundaries, contradictions, workarounds | `ai/known-limitations.md` |
| Worked, compilable examples | `ai/recipes/` |
| How to keep these files current | `ai/maintenance.md` |
| External-AI discovery entry point | `llms.txt` |

Do not duplicate explanations already in `ai/`, `docs/`, or `book/src/` —
link to them by path or capability ID instead.
