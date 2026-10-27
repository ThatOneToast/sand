# Keyed owner resolution (#442 bounded slice)

`KeyedData` first runs `execute unless entity @s run data remove storage …
__sand_owner.uuid`, then copies the executor's complete UUID array once and
invokes a macro with `$(uuid)`. Only a missing executor clears the argument.
The persistent layout remains `owners[{uuid:[I;…]}].components.…`. No identity,
layout, or migration changes are involved.

Each get, set, remove, presence callback, or field initialization requires at
most one entity UUID NBT read. A missing default reuses the initialization's
resolved arguments for its conditional staged write. That staging executes
only ordinary data commands, so it cannot change the executor or call another
owner. Authored payloads never enter macro source; strings resembling macro
parameters remain literal, and write staging is removed afterwards.

Without an executor, the UUID copy fails after the old argument was cleared.
Macro instantiation then fails rather than accessing the previous owner. No
owner value is read, written, or removed, and presence callbacks do not run.
The enclosing ordinary function can continue, including literal staging cleanup.
This is not a transaction or an early return from the caller.

Vanilla 26.2 probes of the final get macro call observe result/success stores
remaining at their sentinel for a valid owner (an ordinary function with no
explicit return). Missing UUID arguments fail that call with result 0 and
success 0. The implementation adds no return command or success guard that
would change the valid-owner contract. The macro is never gated on copy
success: copying the same UUID reports no change but must still run the macro.
Repeated same-owner resolution performs zero successful storage mutations.
Switching to another valid owner changes the UUID once; a missing executor
removes the prior UUID before the failed copy and macro call.

Nested callbacks resolve their own executor. Macro paths are instantiated
before executing the callback; nested calls can replace the scratch UUID
without rewriting an outer macro's instantiated paths. Subsequent accesses
resolve again.

Owner lookup still scans the owner list. This slice does not provide constant
time lookup, general per-body caching, owner reclamation, or closure of #442.
Removing a field retains its owner record. Static NBT read counts and native
executed UUID-copy command counts are not total evaluations, Minecraft's
internal command-budget units, or MSPT measurements.

## Focused validation

Run from the repository root:

```sh
CARGO_TARGET_DIR=examples/unified_state/target cargo test -p sand-core --lib keyed_
python3 -m unittest discover -s scripts -p test_validate_keyed_literals.py
(cd examples/unified_state && sand build)
SAND_SOURCE_REVISION="$(git rev-parse HEAD)" python3 scripts/validate_keyed_literals.py \
  --version 26.2 --pack examples/unified_state/dist/unified_state \
  --jar "$HOME/.sand/cache/26.2/server.jar" --output /tmp/keyed-owner-unique-run
```

Use a new output directory for every attempt; the harness refuses to overwrite
existing evidence. It records the input pack hash, supplied source revision,
harness hash, assertions, failures, and trace hashes. Native metrics separately
record command evaluations, entity UUID reads, successful storage mutation
commands, and successful storage gets. Preserve failures too.
`SAND_SOURCE_REVISION` must identify the revision used to build the input pack,
which need not be the current checkout when comparing an older pack.

The existing unified_state fixture proves two online players and two entities,
literal strings and nested compounds, actual reload, and selective detach.
The stale-write regression invokes the real compiled `set_entity_literals`
without an executor after a valid owner call. Get/remove/presence and nested
probes reuse the exact production write frame's resolver with small synthetic
macro bodies; they are probe functions, not extra exported author APIs. The
Rust regressions separately verify the corresponding generated command shapes.
An independent UUID-array oracle checks stored values. Native debug traces
exercise the actual compiled player/entity writes and attachment functions.

Expected native UUID-copy command counts for the unchanged fixture:

- Two-field entity write: 8 before, 2 after.
- Two-field player write: 8 before, 2 after.
- Fresh two-field status attachment: 12 before, 1 after.
- Existing status attachment: 4 before, 1 after.

The attachment builder already deduplicates identical inline initialization
commands. These attachment counts describe that particular exported fixture;
they do not establish a general per-body cache. Each standalone resolver emits
four UUID reads before this change and one afterwards.

## Conditional-clear correction (independent review P2)

The unconditional-clear implementation at `4ac33f9590ddac26acde04b990fa64e64da3d38f`
is a rejected candidate: its existing-attachment trace performs two successful
storage mutations (remove then copy) even for the same owner. Preserve its
original evidence at `/tmp/sand442-after-final` and the stricter regression run
at `/tmp/sand442-p2-rejected-run`; do not relabel either as corrected evidence.

The corrected fixture requires zero successful storage mutation commands in
both consecutive existing-attachment traces and a same-owner get trace. It
also requires a successful storage get after the unchanged UUID copy, proving
that copy failure does not skip the macro. Existing attachment evaluates 22
commands including one probe wrapper, reads UUID once, and mutates storage zero
times; the rejected candidate has the same 22 evaluations and one UUID read,
but two successful storage mutations. A same-owner get evaluates five commands
including its wrapper, reads UUID once, and has zero successful storage mutations.

These mutation counts count positive vanilla `[R]` results for `data modify`
or `data remove` targeting `rpg:state`, including guarded commands. They count
successful commands, not the numeric result magnitude or affected NBT nodes.
Evaluations count `[C]` records (including the separately identified wrapper),
and UUID reads count executed UUID-copy commands regardless of success. An
unchanged copy is an evaluation and read, but not a successful mutation. These
metrics make no MSPT or internal command-budget claim.

The lead integration owns `scripts/check.sh` and final API/runtime baselines.
This slice runs focused checks and `git diff --check` only.
