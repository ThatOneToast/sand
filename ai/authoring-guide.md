# Authoring guide (agent-facing)

Decision rules for turning a user's datapack request into Sand code. This is
not an API manual — for API details follow the links each step provides.

## 1. Understand the request

Identify: what should happen in-game, what triggers it, what state (if any)
persists across ticks/players, and which Minecraft version(s) it must run
on. Most requests decompose into one or more `ai/capability-manifest.yaml`
capability IDs — name them before writing code.

## 2. Inspect or choose the target Minecraft version

- If working inside an existing project, read `sand.toml`'s `[pack]
  mc_version`. `"latest"` resolves to `sand_version::LATEST_KNOWN`.
- If scaffolding new, ask or default to `sand_build::latest_release_version()`
  (what `sand new` uses when no version is given).
- Resolve it: `MinecraftVersion::parse(...)` then `VersionProfile::resolve()`
  (never errors — unknown/future versions fall back to conservative
  capabilities, see `LIM-VER-001` in `known-limitations.md`) or
  `resolve_strict()` (errors on unknown versions — prefer this in
  generated/test code that must not silently under-target).

## 3. Map intent to capability IDs

Look up each piece of required behavior in `ai/capability-manifest.yaml`.
Read its `status`, `minecraft` range, `sand_limitations`,
`vanilla_limitations` (if listed), and `preferred_api` before writing any
code. Do not skip capabilities that seem "obviously fine" — e.g. dialogs and
worldgen registries look like ordinary datapack components but are partial.

## 4. Select typed APIs

- Default import: `use sand_core::prelude::*;` plus the proc macros needed
  (`sand_macros::{function, component}`).
- Use each capability's `preferred_api` field. If it names an `advanced_api`
  and the task genuinely needs framework-level control (custom dispatch,
  export hooks), use `sand_core::advanced` — otherwise stay in `prelude`.
- Never import from `sand_core::compat` in new code; it's a single
  deprecated alias for old call sites.

## 5. Enable optional systems

Systems capabilities (`cooldowns`, `player-data`, `lifecycle-events`,
`inventory`, `movement`, `entities-interactables`, `damage-tracking`) each
require a Cargo feature on `sand-core` (`systems-cooldowns`,
`systems-player-data`, `systems-lifecycle`, `systems-inventory`,
`systems-movement`, `systems-entities`, `systems-damage`, or the
`systems-all` umbrella). Check the consuming crate's `Cargo.toml` before
assuming a systems type is in scope — a missing feature flag is a compile
error, not a runtime gap.

## 6. Choosing between normal, advanced, compatibility, and raw APIs

```text
Is the behavior possible in vanilla?
├─ No → explain the vanilla limitation (check known-limitations.md's
│       "Vanilla Minecraft limitations" section first) and offer the
│       nearest approximation, stating clearly that it's an approximation.
└─ Yes
   ├─ capability status: implemented → use preferred_api directly.
   ├─ capability status: experimental or partial → use preferred_api,
   │   but tell the user which part is unverified/incomplete
   │   (sand_limitations field) so they aren't surprised later.
   ├─ capability status: raw_only, or the specific field/variant isn't
   │   covered by an otherwise-partial typed API → use the named
   │   raw_escape_hatch (RawJson / RawSnbt / RawComponent / cmd::raw),
   │   isolated to just that gap — don't drop the whole feature to raw
   │   when only one field needs it.
   └─ capability status: unknown, or no capability entry exists at all →
       say so explicitly and either search source yourself
       (registry_coverage.rs / trigger_coverage.rs / grep the crate) or
       report the gap. Do not guess an API exists.
```

## 7. Recognizing vanilla-impossible requirements

Cross-check any "this seems like it should just work" request against
`known-limitations.md`'s vanilla section: no free-form physics velocity
(`LIM-VAN-004`), no exact shield-block/axe-disable event (`LIM-VAN-006`), no
generic proximity trigger (`LIM-VAN-005`), no damage-event payload
(`LIM-VAN-003`), advancement triggers don't cover every action
(`LIM-VAN-002`). If a request needs one of these, say so up front and
propose the nearest vanilla-reachable approximation (e.g. tick-polling
distance instead of a proximity event) rather than silently building
something that only partially matches what was asked.

## 8. Adapting a request when Sand has only partial support

State the boundary precisely (e.g. "the typed loot table builder doesn't
cover this pool condition type"), then implement the covered portion with
the typed API and the uncovered portion with the matching raw escape hatch
in the same component, rather than abandoning the typed API entirely.

## 9. Validating the result

1. `cargo build` (workspace or touched crate).
2. `cargo test` for the touched crate, or `cargo test --workspace
   --all-features` for cross-crate changes.
3. `cargo run -p sand -- build` in the example/scaffolded project, then read
   the generated files under `dist/` — confirm the JSON/mcfunction actually
   contains what was intended, don't just trust that compilation succeeded.
4. If the change is version-sensitive, check output against the version's
   pack_format (`sand-core/src/version.rs` lookup table).
5. Vanilla-server reload validation (`scripts/validate-vanilla-reload.sh`) is
   opt-in and not part of default `cargo test` — only claim "verified against
   vanilla" if you actually ran it (network + Java 21+ required); otherwise
   say output was validated by golden tests only (see `LIM-VAL-001`).

## 10. Communicating limitations honestly

When reporting completed work, state plainly: which capabilities were
`implemented` vs. used a `partial`/`experimental`/`raw_only` path, which
vanilla limitations shaped the approach, and what validation was actually
performed (compiled + unit-tested vs. vanilla-reload-verified). Cite
capability IDs and limitation IDs so the user can look up the detail.

## Anti-hallucination rules

- Never infer that a method exists from naming conventions or "the pattern
  used elsewhere" — grep the actual crate or check the manifest.
- Never assume a type mentioned in `book/src/` or `docs/` prose is
  re-exported by `sand_core::prelude` — check `sand-core/src/prelude.rs`.
- Never assume an advancement trigger exposes rich event payload values;
  check `EventWrapperStatus` in `trigger_coverage.rs`
  (`Supported`/`Partial`/`None`) first.
- Never assume a capability supports every Minecraft version Sand targets —
  check its `minecraft.minimum`/`version_gate`.
- Never treat a passing `cargo test` run as equivalent to vanilla-server
  reload validation — they test different things (see `LIM-VAL-001`).
- Never hide an `experimental` or `partial` status from the user to make an
  answer sound more complete.
- Never reach for `cmd::raw`/`RawJson`/`RawSnbt`/`RawComponent` when a
  verified typed API already covers the operation — that defeats the point
  of the typed framework and should be treated as a last resort, not a
  shortcut.
