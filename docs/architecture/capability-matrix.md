# Versioned framework capability matrix

This matrix separates emitted resources, typed authoring, and native evidence.
A generated command builder is not a tested high-level game system. An export
regression proves deterministic output and validation, not every runtime behavior.
The default target is 26.3; compatible 26.1 and 26.2 exports remain supported.

| Capability | Typed authoring / exported targets | Maintained native evidence | Remaining boundary |
|---|---|---|---|
| Commands and registries | Version-selected generated contracts; exact provider counts in `sand/api-surface-profiles.toml` and baseline reports | Representative load/reload and scenario tests | No claim that every generated command is exhaustively exercised |
| State and lifecycle | Canonical schemas, numeric/storage fields, migrations, keyed owner storage | 26.2 player/state/storage proofs; 26.3 gameplay State | Save semantics follow declared schemas; arbitrary tasks do not resume after reload |
| Owned execution | Lifetime, child tasks, sequences, jobs; exports 26.1/26.2/26.3 | 26.2 real-player lifecycle and 26.3 entity gameplay | Loaded-tick work; no forced chunks or durable coroutine scheduler |
| Numeric expressions | Checked lowering; compute backend on 26.3 where semantics match | 26.3 provider/rounding/failure and gameplay checks | Not every numeric provider has a high-level curve abstraction |
| Combat and encounters | Canonical custom health/shields, attributed damage, child-attack policies | 26.2 real-player combat; 26.3 combat and encounters | No universal native-damage interception or general pathfinding |
| Menus and conversations | Guarded typed views, pagination, one-shot routing, object-bound transitions; exports all three targets | Two-player 26.2 and 26.3 session/conversation proofs | Free text and arbitrary numeric dialog submission lack verified transport |
| World interactions | Owned hitboxes and typed world/block operations | 26.2 and 26.3 actors, nearby-object isolation, dimension changes and cleanup | Native click records can coalesce; no generic container GUI interception |
| Inventory workflows | Empty-slot rewards, exact whole-slot item payment, numbered player-slot transfer | 26.2 live-item/payment and listener-interference proof | No general distributed transaction; implicit vanilla stack limits are not guessed |
| Quests and sessions | Ordered guarded objectives over State, owned session control; exports all three targets | 26.3 independent entity-host progression/session proof | Host scope is explicit; matchmaking, spectators and team policy remain composition work |
| Scenes and HUD | Block/item/text parts, checked transform composition, player actionbar/bossbar lifetime | 26.3 native mixed scenes; 26.2 two-player HUD proof | No skeletal rig, animation blending or independent simultaneous actionbar slots |
| Written books and journals | Canonical text/item snapshots and explicit State-driven Journal refresh | 26.2 book delivery; 26.2 and 26.3 two-player Journal delivery, replacement and literal safety | No forced opening or automatic book tracking |
| Checkpoints | Explicit canonical integer State projection, migration and task preparation | 26.2 and 26.3 two-player bounds, reentry, detach, reload and reconnect proofs | No arbitrary world/inventory snapshot or durable task continuation |
| Resource packs | Opt-in companion asset packaging, exact independent format metadata, safe separate output | CLI deterministic rebuild/zip, source safety and grouped rollback tests; Mojang version.json provenance | Asset bytes and JSON syntax are validated; client codecs, rendering, delivery and installation remain separate |

Evidence entry points are the version-filtered jobs in
[vanilla-reload.yml](../../.github/workflows/vanilla-reload.yml), their maintained
`scripts/validate_*.py` fixtures, and [BENCHMARKS.md](../../BENCHMARKS.md). The
[roadmap](../../ROADMAP.md) records which increments have landed and remaining
acceptance work. 26.1 is export-covered here; it is not a third native scenario
lane. The [real-player protocol proof](../testing/player-protocols.md) covers
26.2 and 26.3 connection/keepalive, dimensions, controlled damage and reconnect.
This passive-client proof does not establish every gameplay scenario or scripted
client controls on 26.3.

## Measuring coverage without a misleading percentage

Track four independent inventories:

1. **Native emit coverage:** version-specific command/resource identities whose
   syntax can be emitted, including explicit raw escape hatches.
2. **Typed coverage:** those identities with a validated canonical API. Raw text
   and JSON emission do not count toward this inventory.
3. **Gameplay coverage:** concrete authoring workflows such as buying a reward,
   cancelling a child task, or confirming an NPC conversation.
4. **Evidence coverage:** export, negative/compile checks, native single-actor,
   native multiplayer, reload/disconnect and measured cost evidence per workflow.

The API baseline is an enforced public-surface inventory, not a denominator of
all vanilla behavior. Minecraft's unavailable signals must remain explicit gaps,
not be counted as implemented because a related command exists. A percentage is
only meaningful for a named, versioned inventory and acceptance criteria.

## Runtime-cost guidance

`RuntimeReport::advice(threshold)` identifies tick entries with unknown maximum
work and those whose known maximum reaches the caller's review threshold. The
CLI emits these as `runtime_advice` beside the unchanged `runtime_report`, using
10,000 explicit evaluations as a review threshold. This is not a server command
limit or a safe MSPT budget. Advice preserves the original unknown causes and
never substitutes guessed player/entity populations. Results are deterministic,
deduplicated by tick reference, and capped at 64 with an explicit omission marker.
Use native workload traces and representative populations before changing cadence,
participant bounds or batching policies.
