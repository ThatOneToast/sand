# XP-level detector bookkeeping validation

This is a bounded part of #449. Other detectors, participant cleanup, and event
coordinator bookkeeping remain separate work.

## Runtime behavior

The detector retains one `experience query` into its writable current snapshot
at every existing observation point. It initializes the previous baseline and
seen flag once, silently. Only a changed sample computes the delta, invokes
callbacks for a positive increase, and advances the previous baseline.

Idle players execute three commands instead of seven, with one scoreboard write
(the query result) instead of five. The four namespace-owned dummy objectives
and callback snapshots retain their existing identities and criteria. There is
no criterion migration, new observation engine, or scheduler change.

After callbacks, previous advances from the sampled current value. A callback
that changes experience cannot change the snapshot seen by later callbacks;
its new level is visible at the next observation. Decreases advance the baseline
without firing. Existing baselines survive reload and reconnect unchanged.

## Why the native `level` criterion was rejected

An experimental implementation used vanilla's `level` criterion and three idle
guards with no writes. That signal cannot preserve the existing timing contract:
Minecraft 26.2 updates it in `ServerPlayer`'s player tick, while datapack tick
functions run earlier. `setExperienceLevels` changes the player value without
synchronously updating the criterion. A mutation in an earlier tick handler can
therefore be immediately visible to `experience query` but absent from the native
score when Sand observes it. Delaying the observation would change event timing.

The experiment also confirmed that creating the criterion does not backfill
already-online players and that writing it through `execute store` is invalid.
Those initialization problems were handled in the experiment, but did not solve
the timing mismatch. The final implementation deliberately retains the query.
Native debug command counts below are not MSPT, complete tick costs, or evidence
that the broader #449 acceptance criteria are complete.

## Reproduction

```sh
cargo test -p sand-core --test xp_level_bookkeeping --test xp_snapshot_ownership \
  --test builtin_detector_ownership --test builtin_detector_collisions
python3 scripts/validate_xp_level.py \
  --jar /Users/wb/.sand/cache/26.2/server.jar --output /tmp/sand-449-xp-final
python3 scripts/validate_detector_ownership.py \
  --jar /Users/wb/.sand/cache/26.2/server.jar --output /tmp/sand-449-ownership-final
scripts/check.sh
git diff --check
```

The focused script uses unmodified generated functions and a separate control
pack to suppress automatic observations during deterministic samples. It checks
an old retained baseline, silent initialization, a mutation immediately before
observation, mutating callbacks and typed snapshots, decreases, reload, real
reconnects, death/respawn with both `keep_inventory` settings, actual XP-orb
pickup after respawn, and preservation of an unrelated objective. The unchanged
ownership audit exercises normal ticking, both pack orders, and known namespace
collision pairs.

The cost workload invokes the generated check at level 5 for one and two players,
matching the baseline wrapper shape. Complete vanilla `debug function` traces
are retained and nested `[C]` evaluations counted, excluding the two wrapper
commands. Errors and missing completion markers fail the measurement.

The vanilla-reload workflow runs the focused regression on 26.2 and retains its
logs, player logs, generated datapacks, world, evidence JSON, and traces in the
failure artifact for 14 days.

## Recorded evidence

[Evidence JSON](xp-level-detector-evidence.json) records the baseline and final
native debug measurements, the generated-record and server-jar hashes, and the
rejected native-signal timing experiment. One-player idle evaluations fell from
7 to 3; two-player evaluations fell from 14 to 6 (57% fewer). Idle scoreboard
writes fell from 5 to 1 per player (80% fewer).

In the atomic timing counterexample, a 5→7 mutation produced one original
callback and zero native-candidate callbacks; the native score was still 5 while
the query returned 7. After a player update, the native candidate dispatched 7
and its callback changed XP to 9, while the native score within that same
function remained 7. This is why the final regression places the earlier XP
mutation and the observation together in one probe function. Frozen world ticks
alone do not establish an atomic boundary for player bookkeeping.
