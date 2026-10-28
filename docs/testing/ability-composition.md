# Resource and ability composition validation

The public implementation is `sand::systems::ability`. It uses consumer-owned
State accessors, capacity StatCurve expressions, Conditions, Actions and ordinary
registration. Trailforge and `examples/aether_tools` import that implementation;
there is no parallel downstream copy. No keyed storage or lifecycle lowering is
changed by this workstream. Indexed ownership and reclamation are separate work.

## Contracts under test

- Eligibility and target acceptance are sampled before a grouped cost/cooldown/
  effects branch. Exactly-cost activation must still complete cooldown and effects.
- Missing/wrong equipment, insufficient resources, active cooldown and rejected
  targets leave cost/cooldown unchanged. Item checks use live ItemLocations.
- Two players can activate in one tick without sharing resources. Two abilities
  can share one cooldown/pool; different bound fields remain independent.
- Maintenance registers once per pool with a consumer function identity. Duplicate
  identity registration fails; a second ability adds no maintenance. Canonical
  State bounds, auto-ticking, initialization and persistence remain authoritative.
- A later native operation failure or callback early return does not undo payment.
  Effect-only callbacks are not safe activation entrypoints.
- Construction/discard adds no generated resources compared with the same State
  schema baseline. Registration and activation are separate observable changes.

`sand/tests/ability_composition.rs` covers validation, duplicate registrations,
complete export equality, cached Actions and repeat exports under two namespaces.
`scripts/check-ability-linking.sh` builds four independent sand-only executables:
State-only baseline, linked/constructed/discarded, registered but uninvoked, and
active. Every executable links the exact same shared State library. Baseline and
discarded JSON bytes must match in full, including helpers, objectives, predicates,
load/tick entries, provenance and runtime report. The other two exports must differ
and contain exactly one maintenance call. This check is part of `scripts/check.sh`.
The normal standalone example pipeline builds and records both gameplay consumers.

## Native reproduction

Build the examples through `sand build --format json`, then use the maintained
real-player server harnesses (Minecraft 26.2, Java 25):

```sh
python3 scripts/validate_trailforge.py --pack examples/book_project/dist/trail \
  --jar "$SERVER_JAR" --java "$JAVA_25" --source-root "$PWD" --output /tmp/trail-ability
python3 scripts/validate_ability_composition.py --pack examples/aether_tools/dist/aether \
  --jar "$SERVER_JAR" --java "$JAVA_25" --source-root "$PWD" --output /tmp/aether-ability
```

Use the actual pack path from the build result if configured differently. During
team validation, wrap each expensive build/server/check command with
`python3 /tmp/sand-milestone-validated-run.py` and set `CARGO_INCREMENTAL=0` for Cargo.
The wrapper is a session-local coordination aid, not a repository dependency.

The Aether harness reuses the maintained player connection, completed tick stepping
and actual load-tag observation machinery. It checks exact-cost activation,
shared cooldown rejection, missing/wrong/changed equipment, rejected/accepted
blocks, two players in a frozen tick, independent instances, one regeneration
pulse per cadence, State saturation, actual reload/reconnect retention, an unloaded
block mutation after acceptance, and an early-return effect callback. Trailforge
retains its bounded raycast, menu, equipment, lifecycle and two-player assertions.

These are correctness assertions with console-driven activation and passive real
clients. They do not establish client input behavior, death/respawn, MSPT, maximum
population, command-budget capacity, asynchronous ownership or rollback.
