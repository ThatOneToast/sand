# Vanilla initial-load and reload validation

Sand's vanilla harness validates a generated datapack twice: during initial
server startup and during an actual `reload` command. It is intentionally a
manual/scheduled release-confidence check, not part of pull-request CI.

## Representative pack

The small `sand-vanilla-audit` workspace crate emits deterministic output for
functions, load/tick tags, automatic typed score/flag/timer/cooldown lifecycle,
the tracked sneaking transition backend, an advancement-backed event,
predicate, recipe, loot table, item modifier, scoreboard commands, and a
dialog on the latest 26.x target. It avoids broad legacy examples whose
runtime behavior is unrelated to this loader check.

`audit_placed_block_filtered`, `audit_item_used_on_block_filtered`, and the
profiled trigger-predicate matrix
(added for #231/#232) additionally cover the version-aware
`conditions.location` / `minecraft:location_check` / `minecraft:match_tool`
rendering for a block + custom-data-filtered item — this is the shape that
was previously silently ignored by vanilla (#231). Running this harness
against 1.21.4 and 26.2 with these fixtures confirms the exact generated JSON
parses and survives a `reload` on a real server. The matrix exercises direct
entity predicates, nested locations and damage-source entities, player-location
lowering, non-placement item predicates, and the allay location/tool consumer.

The default harness starts with no players. Vanilla parses every generated
function and component, and load functions run, but player-dependent
tick/event paths are not behaviorally exercised. This is loader/reload
validation, not gameplay simulation or exhaustive component parity.

Load/reload success alone cannot prove advancement criteria fire with the
correct semantics. `scripts/validate-vanilla-semantics.sh` adds a real
1.21.4 protocol client and executes actual placement, block-use, and player
sneaking state packets.
Its deterministic fixture verifies that unrelated blocks and ordinary base
items do not match, marked custom items do match, the final item in a stack
still matches, and reward-function revoke/reset permits a second firing. Both
`minecraft:placed_block` and `minecraft:item_used_on_block` are exercised.
Server-side commands prepare the arena and inventory only; they do not grant
the tested advancements or synthesize the triggering actions.

The same client also drives the Phase 3 persistent-composition fixture. A
score increase supplies the independent parent occurrence while real player
sneaking packets change the persistent state. The sequence proves a false
state rejects the child, becoming true without a parent does not fire, true
and remaining true admit later parent occurrences, becoming false stops
dispatch, and returning true permits another firing. This is single-player
semantic evidence; multiplayer isolation remains structural command-generation
evidence rather than a two-client runtime claim.

The Phase 4 fixture also drives two independent score-delta parents. Separate
A-only and B-only stimuli prove both `after_any` paths; repeated A proves one
parent cannot satisfy `after_all`; atomic A/B and B/A functions prove
order-independent `after_all` success and at-most-once `after_any` coalescing.
Idle checks between stimuli prove reset and no stale next-tick occurrence.
This is single-player 1.21.4 semantic evidence. Per-subject multiplayer safety
and 26.2 gameplay semantics remain structural and load/reload evidence,
respectively.

The semantic client currently supports 1.21.4 only. No semantic claim is made
for 26.2, for advancement triggers outside those two cases, or for persistent
providers other than current sneaking.
For advancement triggers,
`sand_components::advancement::trigger_coverage` tracks this distinction via
`vanilla_load_tested_profiles` and `semantic_runtime_tested_profiles`. The
persistent and multi-parent composition evidence boundaries are recorded under
capabilities `sandevent-persistent-conditions` and
`sandevent-multi-parent-composition`, plus `LIM-VAL-004` and `LIM-VAL-005`; do
not treat one evidence level as another.

## Synchronization and signals

The controller creates an isolated server/world directory, copies the pack
before startup, accepts the EULA only there, and keeps Java stdin writable.
Each phase has its own bounded timeout:

1. Initial startup completes only after vanilla logs `Done (...)! For help`.
2. The controller sends `say __SAND_RELOAD_SUBMITTED__` and requires its log
   marker, proving the command channel is live and defining a fresh log offset.
3. It sends `reload`, followed by
   `say __SAND_RELOAD_COMPLETE__`. Vanilla's `Reloading!` line proves reload
   began. Since console commands execute in order, the completion sentinel
   cannot appear until the preceding reload command finishes.
4. It sends `stop` and requires a zero-status shutdown.

Initial and reload log segments are scanned independently for focused
datapack/component parsing failures, missing functions/tags, incompatible pack
metadata, worker-thread `Couldn't parse data file` errors, server-thread errors,
and fatal exceptions. Failure diagnostics name
the Minecraft version and phase, print matched lines, and retain the log,
isolated server directory, and generated pack path. Cleanup always stops,
terminates, or kills and reaps the child process as needed.

## Local use

Build the audit pack for a verified target, populate Sand's SHA-verified jar
cache, then run the harness:

```sh
cd sand-vanilla-audit
SAND_MC_VERSION=1.21.4 SAND_STRICT_CODEGEN=1 cargo run -p sand -- build
cd ..
cargo run -p sand-build --bin ensure-server-jar -- 1.21.4
scripts/validate-vanilla-reload.sh \
  --version 1.21.4 \
  --pack sand-vanilla-audit/dist/sand_audit \
  --output target/vanilla-reload/1.21.4

# Optional gameplay semantics for placement, item use, persistent sneaking,
# and multi-parent same-cycle composition:
scripts/validate-vanilla-semantics.sh sand-vanilla-audit/dist/sand_audit
```

`SAND_JAR_CACHE` may override the default `~/.sand/cache` root. The stable
`1.21.4` server requires Java 21; the latest verified `26.2` server requires
Java 25. The optional semantic fixture additionally requires Node.js 22 or
newer and npm; it runs `npm ci` against the checked-in lockfile on every run.

## Scheduled/manual workflow

`.github/workflows/vanilla-reload.yml` runs weekly and through
`workflow_dispatch`. Its matrix is generated from the same Rust constants as
normal generated-API CI: stable `1.21.4`/Java 21 and
`sand_version::LATEST_KNOWN`/Java 25. It reuses Cargo and version-aware
`~/.sand/cache` entries; cold misses download through Sand's SHA-verifying
Mojang path. Failures upload only `latest.log` and the generated audit pack for
14 days, never the isolated server directory, libraries, or server jar.

Because GitHub only exposes a new manual workflow after it reaches the default
branch, the first hosted run is a post-merge verification step. Local real
server results do not imply that hosted run has occurred.
