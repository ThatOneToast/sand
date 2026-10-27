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

The maintained native harness with two real passive players passed 21 assertions
for each pack, including: fresh initialization, existing-score preservation, missing-score
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

The [archived evidence](../benchmarks/evidence/player-initialization-2026-10-27/summary.json)
contains both runs, ten complete traces, source-file hashes, pack digests, and
provenance. It records the original candidate capture as uncommitted and
separately verifies that a clean rebuild of `0e47c6c1` produces the same pack.
The summary also distinguishes the two historical file-ordering conventions
used for pack hashing; unlike digests are not compared as if they were identical.

Reproduce against the identified unchanged-fixture packs with Java 25:

```sh
python3 scripts/validate_player_initialization.py \
  --pack /path/to/after-pack \
  --provenance benchmarks/evidence/player-initialization-2026-10-27/after/provenance.json \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --output /tmp/player-initialization-after
```

Use the before pack and before provenance in a separate output directory for
the baseline. The harness rejects mismatching pack digests and incomplete traces.
These probes isolate initialization; gameplay reload/reconnect validation is
covered separately by the [Trailforge audit](testing/trailforge-evidence.md).
Observed gameplay score changes include one repaired default, preservation of
the other player's value, and no suppressed write. A complete mutation-count
or MSPT benchmark was not measured.

Focused validation covers lifecycle, portable compilation, StateFlow, entry
ordering, report ownership, authored return boundaries, and the maintained
native harness. The required full gate is `scripts/check.sh` plus
`git diff --check` on the integrated revision.
