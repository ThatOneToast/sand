# Keyed owner index evidence

The final comparison is `baseline/` (50 assertions) and `candidate/`
(155 assertions), both with three completed historical-owner workloads and the
same harness hash. `summary.json` retains their complete stage-separated metrics.
The baseline runs production source
`8bfbfde322884e42612ae43ccc9b9dbcc1aeb65a`. Its input-pack identity matches the
retained #498 native pack exactly. That pack was reconstructed by removing the
old probe namespace and load sentinel from the retained world pack; changes
between its original production commit `067dab3b7c0ea7cf8b5641d593156e3fbb386cfb`
and the requested baseline are documentation/evidence only.

Each JSON records source, harness/dependency, server, Java, host and pack
identities. Raw traces and compressed vanilla storage snapshots remain beside
it. Pack hashing includes sorted relative paths and file contents with framing.
The probes live in a separate namespace in disposable server worlds.

## Earlier and rejected runs

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
