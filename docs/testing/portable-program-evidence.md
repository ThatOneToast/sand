# Portable program runtime evidence (#396)

The separately runnable harness starts the cached Minecraft Java 26.2 server,
loads the complete compiled counter pack, and joins real offline-mode players
with the existing minimal join client. Automatic ticks are frozen. A separate
audit datapack invokes three lifecycle/tick pairs explicitly while both players
are online; it does not change the counter pack.

The run on 2026-09-27 passed direct first-entry initialization (value 1 and
presence revision 1), two-player isolation (values 4 and 3 after the direct call
and three tick pairs), reload preservation (4 and 3), and a new player's isolated
first increment (1, with the earlier scores still 4 and 3).

This proves explicitly invoked generated lifecycle/tick behavior. The clients'
short play-phase connections do **not** establish sustained automatic-tick
behavior. Independent implementation review and latest-head CI remain merge
requirements; these local results do not claim either has occurred.

```sh
python3 scripts/mc_validation/run_portable_program_audit.py \
  --binary target/debug/sand \
  --jar ~/.sand/cache/26.2/server.jar \
  --evidence /tmp/portable-program-evidence.json
```

The machine observations are in [portable-program-evidence.json](portable-program-evidence.json).

## Nonzero player State defaults

The [nonzero-default evidence](portable-defaults-evidence.json) records a
Minecraft 26.2 run starting each player's counter at 100. Direct first entry
produced 101 with presence revision 1; three explicit lifecycle/tick pairs
produced independent values 104 and 103. Reload preserved both values, and a
new player's first increment produced 101 without changing either existing
player. The run checked the observed server version and waited for both world
startup and the RCON listener before sending commands.

```sh
python3 scripts/mc_validation/run_portable_program_audit.py \
  --binary target/debug/sand --initial-value 100 \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --evidence target/portable-state-defaults/evidence.json
```

The latest-version vanilla workflow runs this scenario. It still proves
explicit invocation behavior with automatic ticks frozen, not sustained
automatic ticks. Rust construction, JSON decoding, and the Python frontend
also compare complete output packs for positive, negative, and i32-boundary
defaults. The lifecycle descriptor adapter is tested against canonical Rust
State initialization for the same values; existing values remain guarded
against reinitialization.

## Bounded and signed score mutations

The portable `bounds` field now uses the same clamp lowering as Rust State.
[`portable_state_bounds.rs`](../../sand-core/tests/portable_state_bounds.rs)
compares the reachable commands from a real Rust-authored State function with
an owned portable program, including canonical lifecycle initialization and
an out-of-bounds default. Portable regressions also compare complete output
maps for constructed and decoded programs. Helper names may differ between
frontends; these tests compare the expanded command sequence for unconditional
helper calls, without flattening conditional execution.

Minecraft 26.2 runs retain these independently checked scenarios:

| Scenario | Default | Bounds | Per-call delta | Evidence |
| --- | ---: | --- | ---: | --- |
| Upper bound | 100 | 0 through 102 | 1 | [JSON](portable-bounds-upper-evidence.json) |
| Lower bound | 0 | -2 through 2 | -1 | [JSON](portable-bounds-lower-evidence.json) |
| Minimum signed delta | 0 | -2 through 2 | -2147483648 | [JSON](portable-bounds-minimum-evidence.json) |

All three passed with two real player entities, automatic ticks frozen, three
explicit lifecycle/tick pairs, reload persistence, and an independently joining
player. These are bounded explicit-invocation tests, not sustained automatic
multiplayer tick or performance evidence. The third scenario also proves that
bounds apply after the complete two-command delta, rather than between its
parts: the established players finish at 2 and -2, respectively.

The first negative run exposed an existing invalid-command bug: Minecraft
refused to load the function containing `scoreboard players add ... -1` because
its amount must be nonnegative. Both Rust State and portable lowering now use
one signed mutation implementation. Fixed-point constants share that path.
Validated low-level command methods and score-command IR reject negative
Minecraft amounts. The audit now retains a sibling `.server.log` file alongside
its JSON evidence, and CI runs the upper, lower, and minimum-delta workloads.
