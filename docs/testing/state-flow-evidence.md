# StateFlow transition dispatch evidence

StateFlow now selects the current state's outgoing transitions through a
balanced tree. Within that state, guards remain sorted by descending priority
and then declaration order. A matching guard returns through the private
selection frame, so changing the state cannot trigger another transition in
the same cycle. The caller continues to the authored tick hooks afterward.

This removes the transition-lock objective and its per-subject reset/set writes.
For 64 sparse source states, a compile regression checks six binary dispatch
levels plus an exact-state leaf check, including i32 boundary values. Each
internal node contains a guarded return and an unconditional fallback; this is
an output-shape bound, not an MSPT measurement. Unrelated transition guards are
never evaluated. Unknown values fall through without becoming another state.

The [Minecraft 26.2 evidence](state-flow-evidence.json) retains 25 passing checks
with two real passive players and automatic ticks frozen. Assertions cover:

- Higher-priority and equal-priority declaration ordering, overlapping OR guards,
  and one transition per cycle.
- Exit/write/enter order, same-tick hooks in the newly entered state, and
  independent players in different states.
- An ordered tick hook changing the state, with later hooks seeing that change
  while enter/exit hooks remain bypassed.
- Interval cadence, reset after observing a departure, low-level state writes,
  signed and unknown states, missing-score defaults, and reload persistence.

Reproduce with Java 25 on PATH:

```sh
python3 scripts/validate_state_flow.py \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --output target/state-flow-audit
```

The script exports the same Rust fixture used by integration tests and writes
its compiler records unchanged. It retains the fixture, generated pack, server
and client logs, assertion conditions, and pack digest. CI runs the scenario on
the latest supported vanilla target.

This completes the transition-selection portion of #441. The missing-score
initialization check remains necessary for arbitrary subject selectors. Tick
hooks retain their ordered live-state checks and existing interval counters;
hoisting them into one initial-state dispatch would change behavior when an
earlier hook changes state. Counter/clock optimization, lifecycle integration,
idle-subject scan filtering and shared scans remain open. No sustained tick
throughput or complete StateFlow performance target is claimed here. This does
not add portable StateFlow authoring; that remains part of #468.
