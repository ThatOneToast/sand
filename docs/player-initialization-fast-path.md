# Player initialization fast path

Scoreboard-only player State schemas with at least two fields, no custom lifecycle
hooks, and no migrations use a conditional call to their existing initializer.
The call is skipped only when the presence score equals the current schema version and **all** field
scores exist. Existence accepts the entire signed 32-bit score range; zero,
negative values, and values outside authored bounds are not treated as missing.
This optimization does not clamp or normalize existing values.

The slow path retains declaration order, per-command detach suppression, missing
field defaults, and presence publication last. A score removed after
initialization is still repaired on the next lifecycle initialization. An explicitly detached player remains suppressed;
reattachment continues to use the existing State API. Reload needs no new marker
or invalidation state. Storage-backed schemas, custom lifecycle hooks, migrations,
and smaller schemas keep their previous output. Entity declarations without
ticking work still produce no tick work; player schemas retain their existing
automatic observation policy.

The wrapper contains no explicit return. It does not introduce a return value or
short-circuit another schema's initialization. Tick-tag order, countdown ticks,
dirty propagation, event dispatch and StateFlow hooks are unchanged. The same
lowering is used by registered Rust and portable State declarations.

For an already initialized five-field schema such as Trailforge's Traversal,
the initializer previously evaluated six command lines. It now evaluates one
conditional call, whose score-filter condition still checks presence and every
field. The cold/repair path evaluates the wrapper plus the original six lines.
The helper adds a function and a lexical selector site. These are explicit
command evaluations and static output changes, not measured MSPT, actual scan
counts, or vanilla command-sequence budget savings.

## Milestone validation

The unchanged `examples/book_project` fixture at base `6f77dcb6` was exported
before and after this change on Minecraft 26.2. The new slow helper is byte-for-byte
the previous initializer body. The ordered tick tag and canonical tick report
are unchanged. The initializer's static evaluation range changes from `6..6`
to `1..7`, with no unknown bounds. The helper retains the `trail:traversal` owner.
No objectives are added. Direct callers inherit the reduced minimum and the
one-evaluation increase on cold/repair paths.

A native probe with two real passive players passed the same 13 assertions for
both packs: fresh initialization, existing-score preservation, missing-score
repair, explicit suppression, and other-player isolation. Result and success
stores retain their sentinel values for ordinary initializer calls in all four
cases; no explicit return callback was introduced.

Completed native debug traces, excluding the identical two-evaluation probe
wrapper, recorded these initializer command evaluations:

| Scenario | Before | After |
| --- | ---: | ---: |
| Healthy, zero selected players | 0 | 0 |
| Healthy, one player | 6 | 1 |
| Healthy, two players | 12 | 2 |
| One missing field among two players | 12 | 8 |
| One suppressed player with a missing field, two players | 12 | 8 |

These are debug `[C]` evaluations, not writes, MSPT, or vanilla budget units.
The selector still evaluates score conditions. The missing stamina score repairs
to 100, the other player's 61 remains unchanged, and suppression leaves the
missing score absent.

The milestone coordinator retains the unchanged packs, canonical reports,
normalized baseline comparisons, raw traces, probe hashes, and native assertion
results under `/tmp/sand-runtime-evidence/`. Boundary results are in
`before-native-final/evidence.json` and `after-native-final/evidence.json`;
trace results are in `before-traces/evidence.json` and `after-traces/evidence.json`.
The frozen probes are `/tmp/sand-initializer-probe.py` and
`/tmp/sand-initializer-trace.py`; each accepts a pack path and a fresh output
path. The lead owns permanent evidence archival and integrated baseline updates.

Focused validation passed 78 Rust tests across lifecycle, portable compiler,
StateFlow, entry ordering and report ownership, plus strict Clippy for the changed
core/test targets, formatting, documentation links and whitespace checks. Two
fixture-export tests remain intentionally ignored. The lead runs the complete
integrated repository gate; the child did not run a concurrent full check.
