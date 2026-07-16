# Known limitations

Consolidated, reviewed 2026-07-15. Each entry has a stable ID, so other files
and PRs can reference it (`[[LIM-XXX-NNN]]`). This file summarizes; it does
not replace the detailed source docs it cites — follow the evidence links for
depth.

Historical audit documents (`docs/*-audit.md`,
`docs/research/datapack-parity-audit.md`) are **not** treated as current
truth here. Where this file cites one, it has been reverified against
current source as of `last_reviewed`; unverified historical claims are
explicitly labeled as such below.

## Vanilla Minecraft limitations

These are constraints of vanilla Java Edition itself — no Sand typing change
can remove them.

- **LIM-VAN-001** — Command storage NBT is global, not namespaced per player
  by the game. Dynamic per-player storage keys are a Sand/authoring
  convention, not a vanilla guarantee.
  Affects: `storage`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

- **LIM-VAN-002** — Advancement triggers do not cover every gameplay action,
  and most triggers do not expose a rich event payload — they fire a
  condition check, not an event-with-data.
  Affects: `advancement-triggers`, `events-typed`.
  Workaround: combine a trigger with typed state/storage reads in the
  triggered function to recover context.
  Evidence: `book/src/reference/vanilla-limitations.md`,
  `sand-components/src/advancement/trigger_coverage.rs` (`EventWrapperStatus`).

- **LIM-VAN-003** — There is no vanilla "damage taken" event with a payload;
  damage tracking is a cumulative-scoreboard-stat approximation (total
  damage taken since last reset), not a per-hit event.
  Affects: `damage-tracking`.
  Evidence: `book/src/reference/vanilla-limitations.md`,
  `sand-core/src/systems/damage.rs`.

- **LIM-VAN-004** — Vanilla has no arbitrary entity physics/velocity API
  reachable from datapacks. Push/launch/speed-boost/slow helpers are built
  from teleports and potion-effect-like mechanics, not free-form physics.
  Affects: `movement`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

- **LIM-VAN-005** — Interaction entities (armor stands, interaction hitboxes)
  react to right-click/attack, not proximity. There is no vanilla
  "on approach" trigger without tick-polling distance checks.
  Affects: `entities-interactables`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

- **LIM-VAN-006** — There is no dedicated vanilla event for "successful
  shield block" or "axe disabled a shield" — these must be approximated
  (e.g. via damage-amount heuristics), not detected exactly.
  Affects: `advancement-triggers`, `damage-tracking`.
  Evidence: `book/src/reference/vanilla-limitations.md`.

## Sand API limitations

Vanilla supports the behavior; Sand's typed coverage is incomplete.

- **LIM-API-001** — Worldgen registries `configured_feature`, `structure`,
  `structure_set`, `processor_list`, `template_pool`, `density_function`,
  `noise`, `configured_carver`, and `dimension_type` have no typed Sand
  module at all.
  Affects: `worldgen-registries`.
  Workaround: `sand_components::raw::RawComponent`.
  Evidence: `sand-components/src/registry_coverage.rs`.

- **LIM-API-002** — Loot table pool conditions and several entry types are
  missing from the typed loot table builder.
  Affects: `loot-tables`.
  Workaround: `RawComponent` for the specific pool/entry.
  Tracking: issue #17.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:loot_table`).

- **LIM-API-003** — Predicate coverage for location, weather, and time
  predicate variants is partial.
  Affects: `predicates`.
  Workaround: `Predicate::raw(RawJson)`.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:predicate`).

- **LIM-API-004** — Item modifier registry covers `SetCount`,
  `SetComponents`, `EnchantRandomly`; the full vanilla modifier set is
  incomplete.
  Affects: `item-modifiers`.
  Workaround: `RawComponent`.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:item_modifier`).

- **LIM-API-005** — Dialog registry has a builder and well-known tag helpers
  (`pause_screen_additions`, `quick_actions`), but broader schema validation
  is partial and the API is still marked experimental.
  Affects: `dialogs`.
  Evidence: `sand-components/src/registry_coverage.rs` (`minecraft:dialog`),
  `ROADMAP.md` (Experimental Areas).

- **LIM-API-006** — `minecraft:enchantment_provider` (1.21+) has no typed
  module.
  Affects: none (no capability entry yet — treat as `raw_only` via
  `RawComponent`).
  Evidence: `sand-components/src/registry_coverage.rs`.

- **LIM-API-007** — `sand join --local` is an unimplemented no-op: it parses
  `sand.toml` and returns without joining anything, despite CLI help text
  describing Prism Launcher integration.
  Affects: `cli-join`.
  Evidence: `sand/src/join_cmd.rs:17-19`.

- **LIM-API-008** — Only `BlockId`, `ItemId`, `EntityTypeId`, and
  `FunctionId` have a `TypedTag<T>`. Other taggable registries (fluid,
  damage_type, dimension_type, worldgen tags, enchantment) are reachable
  only through the raw `Tag` type or `RawComponent`.
  Affects: `tags`.
  Evidence: `sand-components/src/registry_coverage.rs` (`TAG_COVERAGE`).

- **LIM-API-009** — The profile-aware `RenderCommand` boundary currently has
  structural implementations for selectors/targets, coordinates, item slots,
  scoreboard holders/objectives/operations, and foundational `Execute`
  arguments. Other command families still migrate incrementally. The export
  boundary always checks collected function-line integrity and applies deeper
  fallback validation only to confidently recognized top-level commands and
  exact argument positions. It deliberately preserves unknown, macro, modded,
  and command-shaped literal JSON/SNBT content rather than speculatively
  parsing it.
  Affects: `command-validation`, `typed-commands`.
  Workaround: prefer typed builders and their `try_*`/`RenderCommand` paths;
  use `cmd::raw` only for advanced vanilla or modded syntax Sand does not model.
  Evidence: `sand-commands/src/render.rs`, `sand-core/src/component.rs`.

- **LIM-API-010** — Advancement-trigger coverage is partial across the
  verified 1.21.4 and 26.2 registries: 50 current registry IDs have typed
  variants, eight current IDs require `AdvancementTrigger::Custom`, and six legacy
  source-compatibility variants are rejected because their IDs exist in
  neither verified registry. Typed location, entity, damage, and item
  predicates are profile-rendered where verified; unverified legacy
  damage-source booleans and unsupported location `feature` filters fail with
  diagnostics instead of being silently ignored. Raw/custom compatibility is
  user-owned because Sand cannot semantically validate arbitrary JSON.
  Affects: `advancement-triggers`.
  Workaround: check `TRIGGER_COVERAGE`; use `Custom`/`RawJson` only with JSON
  verified for the exact target profile.
  Evidence: `sand-components/src/advancement/trigger_coverage.rs`,
  `sand-components/fixtures/trigger-coverage/`.

## Version-sensitive behavior

- **LIM-VER-001** — `VersionProfile::resolve()` never fails for an
  unknown/future Minecraft version; it silently returns a conservative
  fallback (`is_fallback: true`, all optional-feature flags `false`, pinned
  to the latest known pack formats). Code that only checks `Ok(profile)` can
  silently under-target a version. Use `resolve_strict()` when the caller
  needs a hard error instead.
  Affects: `version-profiles`.
  Evidence: `sand-core/src/version.rs` (`resolve`, `resolve_strict`,
  `VersionCaps::conservative`).

- **LIM-VER-002** — Dialogs, chat types, jukebox songs, enchantment
  data-drivenness, damage types, trim material/pattern, and wolf variants
  are each gated to a specific minimum Minecraft version (see each
  capability's `minecraft.minimum` in `capability-manifest.yaml`). Using a
  capability against an older `mc_version` in `sand.toml` than its gate will
  not automatically downgrade gracefully — check the gate before authoring.
  Affects: `dialogs`, `damage-type-registry`, and others.
  Evidence: `sand-components/src/registry_coverage.rs` (`version_gate` field).

## Experimental areas

- **LIM-EXP-002** — `mcfunction!` macro is explicitly positioned as advanced
  tooling, not the beginner authoring path; prefer `#[function]` +
  `#[component]` for new pack code.
  Affects: `functions`.
  Evidence: `ROADMAP.md`, `RELEASE.md` (Stability levels).

- **LIM-EXP-003** — Resource pack generation and HUD workflows are alpha and
  require manual asset setup; do not assume `sand add resourcepack` produces
  a complete, playable resource pack without further authoring.
  Affects: `resource-pack`, `hud-workflows`.
  Evidence: `ROADMAP.md`, `book/src/resource-packs-and-hud.md`.

- **LIM-EXP-004** — Same-cycle single- and multi-parent `SandEvent` dispatch
  and explicit persistent `while_<E>()` conditions are the implemented
  composition phases of #240. Multi-parent groups support two through eight
  typed parents, at most one `after_any` and one `after_all` group per child,
  conjunctive clauses, deterministic per-subject occurrence staging, and
  at-most-once any-group coalescing. Persistent conditions are currently
  player-scoped, directly queryable states; they are evaluated live at the
  child boundary and do not invoke another event detector or transition
  lifecycle. Advancement-backed `SandEvent` parents are explicitly rejected —
  their reward-function codegen path does
  not yet provide a player execution context compatible with same-cycle
  child dispatch. Bounded `within::<E>(TickWindow)` cross-tick correlation
  (Phase 5 of #240) is now implemented — see `sandevent-bounded-correlation`.
  Participant-rich execution contexts (#230) and arbitrary non-player entity
  execution scopes are not implemented and are not exposed as partial APIs.
  Affects: `sandevent-chained-dispatch`, `sandevent-persistent-conditions`,
  `sandevent-multi-parent-composition`, `sandevent-bounded-correlation`.
  Evidence: `sand-core/src/events/graph.rs`, `sand-core/src/component.rs`,
  `book/src/manual/events.md`
  (Same-cycle and persistent composition).

## Validation gaps

- **LIM-VAL-001** — Passing `cargo test --workspace` (including golden
  export tests) is evidence the generated JSON/mcfunction *shape* is
  correct, not that a vanilla server successfully loads the datapack.
  Vanilla-reload validation (`scripts/validate-vanilla-reload.sh`) is not part
  of the default `cargo test` run. Local runs require cached server jars and
  Java 21/25; the checked-in scheduled/workflow-dispatch action covers the
  verified profiles separately from pull-request CI.
  Affects: `cli-validate`.
  Evidence: `scripts/validate-vanilla-reload.sh`, `docs/vanilla-reload-validation.md`.

- **LIM-VAL-002** — `docs/research/datapack-parity-audit.md` claims a last
  local vanilla-reload run of 2026-07-12, but this claim is inside a
  document already shown (LIM-DOC-002) to contain stale per-registry
  counts from the same review cycle. Treat that specific claim as
  unverified until re-run and cross-checked against `registry_coverage.rs`.
  Affects: `cli-validate`.
  Evidence: `docs/research/datapack-parity-audit.md`.

- **LIM-VAL-003** — Real-server load/reload evidence proves that advancement
  JSON parses, not that a filtered criterion has the requested gameplay
  semantics. A connected protocol-client fixture provides positive, negative,
  final-item-in-stack, and revoke/reset re-fire evidence for `placed_block`
  and `item_used_on_block` on 1.21.4. Other triggers and the 26.2 profile have
  no automated semantic-runtime evidence and must not inherit that claim from
  snapshots or reload success.
  Affects: `advancement-triggers`, `cli-validate`.
  Evidence: `docs/vanilla-reload-validation.md`,
  `sand-components/src/advancement/trigger_coverage.rs`.

- **LIM-VAL-004** — Persistent `while_<E>()` has real single-player runtime
  evidence only for current sneaking on Minecraft 1.21.4. The other supported
  persistent providers, Minecraft 26.2 semantics, and two-player isolation
  remain structural export/unit evidence; reload success does not upgrade
  those claims.
  Affects: `sandevent-persistent-conditions`, `cli-validate`.
  Evidence: `sand-vanilla-audit/src/lib.rs`,
  `scripts/vanilla-semantic-client/client.cjs`.

- **LIM-VAL-005** — Multi-parent same-cycle composition has real single-player
  1.21.4 protocol-client evidence for either-parent matching, both-parent
  at-most-once coalescing, all-parent positive/negative matching, reverse
  atomic parent order, repeated-one-parent rejection, reset, and no stale
  next-tick occurrence. Per-subject scoreboard structure proves isolation, but
  no two-client runtime test currently verifies multiplayer behavior. 26.2 has
  real load/reload evidence only; that does not prove gameplay semantics.
  Affects: `sandevent-multi-parent-composition`, `cli-validate`.
  Evidence: `sand-core/tests/event_multi_parent_export.rs`,
  `sand-vanilla-audit/src/lib.rs`,
  `scripts/vanilla-semantic-client/client.cjs`,
  `docs/vanilla-reload-validation.md`.

- **LIM-VAL-006** — Bounded `.within(...)` cross-tick correlation (Phase 5 of
  #240) has deterministic, exact boundary evidence only from `sand-core`
  unit/export tests (the generated `matches ..N-1` condition, age-counter
  ordering, and objective dedup) — those tests assert exact generated command
  text, not live server timing. A real 1.21.4 protocol-client fixture
  (`SemanticWithin`) is prepared and exercises the "clearly within,"
  "refreshes," and "clearly expired" cases, but has not been run against a
  live server as part of landing this feature (no server available in the
  authoring environment); do not treat it as executed runtime evidence until
  someone runs `scripts/validate-vanilla-semantics.sh` against it. 26.2 has no
  semantic-runtime claim at all. Two-client multiplayer isolation remains
  structural (per-`@s` command generation), not a two-client runtime test.
  Affects: `sandevent-bounded-correlation`, `cli-validate`.
  Evidence: `sand-core/src/events/graph.rs`,
  `sand-core/tests/event_chain_within_export.rs`,
  `sand-vanilla-audit/src/lib.rs`,
  `scripts/vanilla-semantic-client/client.cjs`,
  `docs/vanilla-reload-validation.md`.

## Documentation and status contradictions found during audit (2026-07-12)

- **LIM-DOC-001** — `book/src/SUMMARY.md` does not link
  `book/src/version-capabilities.md`,
  `book/src/getting-started.md`, `book/src/typed-state.md`, and roughly a
  dozen other top-level `book/src/*.md` files, even though they are actively
  edited (some as recently as the same day as this review) and are linked
  from `README.md` and other docs. Several of these orphan files have a
  same-named counterpart under `docs/` with materially different content
  (confirmed for `typed-state.md` and `version-capabilities.md`), so two
  independently maintained versions of some topics exist and only one
  (`docs/`) is reachable from the rendered mdBook navigation. The former
  orphan `book/src/events.md` now points to the navigation-linked
  `book/src/manual/events.md`; do not assume the same repair has been made for
  other top-level pages. Do not assume
  `book/src/*.md` content is current or canonical solely because the file
  exists — check whether it's reachable from `SUMMARY.md`, and prefer
  `docs/` for these overlapping topics until reconciled.
  Evidence: `book/src/SUMMARY.md` vs `ls book/src/*.md`; diffed
  `book/src/typed-state.md` against `docs/typed-state.md`.
  Status: unresolved, out of scope for this review to rewrite the book.

- **LIM-DOC-002** — `docs/research/datapack-parity-audit.md`'s own summary
  tables lag behind `sand-components/src/registry_coverage.rs` and
  `.../trigger_coverage.rs`, despite the file having a recent mtime (it gets
  re-touched by unrelated feature PRs without its counts being
  regenerated). Confirmed discrepancies as of 2026-07-12:
  - Doc claims registry status counts `FullyImplemented: 12,
    PartiallyImplemented: 9, Missing: 10, RawOnly: 19,
    IntentionallyUnsupported: 3`; live source has `16, 9, 10, 20, 4`.
  - Doc marks loot tables and damage types `❌ Not implemented` /
    `Missing`; live source marks `minecraft:loot_table`
    `PartiallyImplemented` and `minecraft:damage_type`
    `FullyImplemented`.
  - Doc's trigger status breakdown (`FullyImplemented: 51,
    PartiallyImplemented: 0, Missing: 0, RawOnly: 0`) doesn't match live
    source (`50 FullyImplemented, 1 IntentionallyUnsupported`), even though
    the total row count (51) happens to match.
  Treat `registry_coverage.rs`/`trigger_coverage.rs` as ground truth over
  this document's prose tables. `ai/capability-manifest.yaml` was built from
  the source tables, not from this audit doc.
  Evidence: direct comparison of `docs/research/datapack-parity-audit.md`
  against `sand-components/src/registry_coverage.rs` and
  `.../advancement/trigger_coverage.rs` on 2026-07-12.

- **LIM-DOC-003** — `Datapacks.md` (repo root) is generic vanilla
  Minecraft 1.21.11 datapack reference notes, not Sand documentation, and
  has not been touched since the initial commit. It does not self-label as
  historical (unlike `Milestones.md`, which does). An agent should not cite
  it as describing Sand's current version support, which spans 1.18–26.2
  per `sand-core/src/version.rs`.
  Evidence: `git log --follow Datapacks.md` (initial commit only);
  `sand-core/src/version.rs` version table.

- **LIM-DOC-005** — `examples/custom_items.rs` is stale and does not compile
  against current source: it calls `CustomItem::new(id, base_item)` with two
  arguments, but `CustomItem::new` takes exactly one (`base: impl
  fmt::Display`) — identity is set separately via `.custom_data("key")`. It
  also passes raw JSON strings to `.custom_name(...)`/`.lore_line(...)`,
  which take `TextComponent`, not strings, and wraps builder functions in
  `#[component]`, but `CustomItem` does not implement `DatapackComponent`
  (it has no resource location; it's an inline item-component string passed
  to `cmd::give`). `git log` shows this file has not been touched since the
  initial commit while `sand-components/src/item/mod.rs` has changed since.
  `examples/README.md` itself labels this file (and `advancements.rs`,
  `recipes.rs`, `loot_tables.rs`, `particle_effects.rs`, `player_join.rs`)
  "legacy reference files," and none of the top-level `examples/*.rs` files
  are wired into any `[[example]]`/`[[bin]]` target or compiled by CI — they
  are copy-paste reference text only. Verified correct usage (compiled
  against current source during this review) is the doc-comment example at
  the top of `sand-components/src/item/mod.rs` and mirrored in
  `ai/recipes/custom-item.md`. `player_join.rs`, by contrast, does compile
  correctly against current source — it was spot-checked and its API usage
  matches; only `custom_items.rs`'s constructor/text-component/`#[component]`
  usage was confirmed broken during this review. The other four "legacy"
  files were not individually re-verified — treat them as unverified pending
  a compile check, not as confirmed-correct or confirmed-broken.
  Affects: `custom-items`.
  Evidence: `sand-components/src/item/mod.rs:1004-1044` (`CustomItem::new`),
  `:419,431` (`custom_name`/`lore_line` take `TextComponent`),
  `examples/README.md`, `git log -- examples/custom_items.rs`.

- **LIM-DOC-004** — `sand-core/Cargo.toml`'s `systems-all` feature does not
  explicitly list `systems-lifecycle`; it is only pulled in transitively
  through `systems-player-data`. This is currently correct (no missing
  functionality), but is easy to misread as a gap — noted here so it isn't
  re-flagged as a bug without checking the transitive dependency.
  Evidence: `sand-core/Cargo.toml` (`[features]`).
