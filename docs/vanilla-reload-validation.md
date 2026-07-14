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

`audit_placed_block_filtered` / `audit_item_used_on_block_filtered`
(added for #231/#232) additionally cover the version-aware
`conditions.location` / `minecraft:location_check` / `minecraft:match_tool`
rendering for a block + custom-data-filtered item — this is the shape that
was previously silently ignored by vanilla (#231). Running this harness
against 26.2 with these fixtures confirms the exact generated JSON parses and
survives a `reload` on a real server.

The harness starts with no players. Vanilla parses every generated function
and component, and load functions run, but player-dependent tick/event paths
are not behaviorally exercised. This is loader/reload validation, not gameplay
simulation or exhaustive component parity.

**This harness cannot prove advancement criteria fire with the correct
semantics.** It starts the server with `max-players=1` and never connects a
player (`enable-rcon=false`, no bot/protocol client exists in this
repository), so it can prove a document *loads*, never that
`minecraft:placed_block`/`minecraft:item_used_on_block` (or any other
trigger) actually fires only for matching in-game actions and not for
non-matching ones. Proving that requires a real Minecraft client capable of
issuing an actual block-placement/item-use packet — server-side commands
(`setblock`, `advancement grant`, RCON) cannot originate the internal event
these criteria hook into. `sand_components::advancement::trigger_coverage`
tracks this distinction explicitly via
`vanilla_load_tested_profiles` (what this harness proves) versus
`semantic_runtime_tested_profiles` (what it does not) — do not treat one as
evidence of the other.

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
metadata, server-thread errors, and fatal exceptions. Failure diagnostics name
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
```

`SAND_JAR_CACHE` may override the default `~/.sand/cache` root. The stable
`1.21.4` server requires Java 21; the latest verified `26.2` server requires
Java 25.

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
