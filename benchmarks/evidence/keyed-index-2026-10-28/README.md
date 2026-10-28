# Keyed owner index evidence

The final comparison is `baseline/` (50 assertions) and `candidate/`
(161 assertions), both with three completed historical-owner workloads and the
same harness hash. `summary.json` retains their complete stage-separated metrics.
The baseline runs production source
`8bfbfde322884e42612ae43ccc9b9dbcc1aeb65a`. Its input-pack identity matches the
retained #498 native pack exactly. That pack was reconstructed by removing the
old probe namespace and load sentinel from the retained world pack; changes
between its original production commit `067dab3b7c0ea7cf8b5641d593156e3fbb386cfb`
and the requested baseline are documentation/evidence only.

Each JSON records source, harness/dependency, server, Java, host and pack
identities. Server logs are retained losslessly as `latest.log.gz`, with both
compressed and uncompressed hashes in `server-log-identities.json`. Raw traces
and compressed vanilla storage snapshots remain beside
it. Pack hashing includes sorted relative paths and file contents with framing.
The probes live in a separate namespace in disposable server worlds.

## Earlier and rejected runs

Historical folders listed below are retained losslessly inside `history.zip`.
`history-identities.json` records every member hash and the archive hash.
The final `baseline/` and `candidate/` remain directly inspectable.

- `baseline-initial/` is an earlier completed workload. It predates the five-owner
  same-tick extension and is not the final comparison. Its metric replay covers
  its scale traces, not every correctness trace.
- `rejected-long-diagnostic/` failed native pack loading because the first
  diagnostic exceeded Minecraft's `say` length limit. Production now uses a
  short message plus detailed `tellraw`.
- `rejected-trace-counter/` exposed a counter that omitted reached guarded UUID
  copies. The corrected parser counts trace evidence of both changed and
  unchanged guarded copies, with a focused regression test.
- `rejected-baseline-probe-v6/` exposed the corresponding old player-setter
  expectation: its lifecycle initializer adds two real UUID reads. The exact
  assertion now includes them.
- `rejected-baseline-probe-v7/` passed absent-owner value creation but used an
  invalid appended match after a legacy list-filter path for the UUID oracle.
  The oracle now copies the complete record into independent probe storage.

`server-storage-access.json` records named method-reference evidence from the
exact vanilla runtime jar. It supports the direct-path design and preservation
of legacy lazy creation; it is not a whole-server allocation or timing profile.

Cleanup/reclamation is intentionally not measured here. Wall times include
console barriers, JIT and host noise. Baseline and candidate run in separate
processes with different correctness-test prefixes and no statistical warmup
normalization; they are single runs, not MSPT or latency percentiles. Migration checkpoints are after the first copied row, not peak
memory/disk measurements. Per-step work is bounded in rows and emitted commands,
not arbitrary payload bytes. Context reuse is deferred.

`candidate-invalid-scale/` passed 150 correctness assertions but its scaling
workload had no executor: a frozen-tick kill left the boundary UUID registered,
so summoning a benchmark entity with that UUID failed. Its timing numbers are
invalid. The final harness uses a separate UUID, requires a live executor,
checks changed values independently, and requires exact reached read/write
UUID counts plus one successful get. Both workloads were rerun successfully
after this fix. `baseline-pre-scale-fix/` retains the earlier valid baseline but is excluded from the final comparison.


The mutation counter covers positive native trace outcomes for `data modify`
and `data remove` commands, not NBT nodes. Migration's `execute store storage`
side effects are excluded, so its total storage mutations are unmeasured.
Steady read/write traces contain no such side effects. The source worktree had
pending documentation/evidence and generated runtime baselines during replay;
exact harness hashes are authoritative even when the dirty-worktree flag is true.

The first full gate stopped at a needless-borrow Clippy lint; the second passed
934 core unit tests but found the old 25-objective lifecycle expectation. Both
logs are retained compressed. The corrected integration test requires 26
objectives, exactly one migration objective, readiness guards, and the original
suppression behavior. Final full-gate status is recorded separately.

`baseline-before-standalone/`, `candidate-before-standalone/` and
`comparison-before-standalone.json` retain the valid earlier replay, before the
public standalone-handle provisioning correction. The final replay repeats both
sides with the same new harness and includes six standalone-storage assertions.
The interrupted third full gate was deliberately stopped to implement this
public-API correction; it is not a passed gate.

## Reproduction

Build `examples/unified_state` at each recorded production revision with the
maintained CLI (`sand build`, Minecraft 26.2). Use the final harness source for
both packs; its legacy/indexed recognition deliberately fails closed. Run each
pack separately with `SAND_KEYED_SCALE=1`, `SAND_SOURCE_REVISION` set to that
pack's actual source SHA, and:

```sh
python3 scripts/validate_keyed_literals.py --version 26.2 \
  --pack /absolute/path/to/unified_state \
  --jar /absolute/path/to/26.2/server.jar \
  --output /new/absolute/evidence/directory
```

The output directory must not already exist. Compare the recorded input-pack,
harness/dependency and server identities before comparing measurements. This
team ran every Cargo/Java/build/full-check workload through
`python3 /tmp/sand-milestone-validated-run.py <command> [args]`, with
`CARGO_INCREMENTAL=0` and the shared build target. Canonical runtime baselines
come only from `scripts/check-examples.sh --update-runtime-baselines`; full
validation is `scripts/check.sh` followed by `git diff --check`.

The unchanged baseline pack predates the new exported get/remove/presence probe
entry points. Its read workload replays the fail-closed extracted production
UUID resolver and equivalent legacy getter macro. The candidate invokes its
actual exported getter; setters invoke actual exported functions on both sides.
Independent observer functions supply value oracles rather than borrowing the
production operation's result as truth. This producer difference is a benchmark
limit, not a claim that the old pack contained the new probe entry points.

## Final validation

The complete `scripts/check.sh` and following `git diff --check` passed.
`validation.json` records the tested source, commands, results and log identities;
`full-validation.log.gz` retains the complete successful serialized run. All six
normal example baseline comparisons and all 13 portable/schema tests passed,
including generated player-initializer helper resolution. Native results are
50 baseline assertions and 161 candidate assertions. CI and external reviews
are separate from these local results.
