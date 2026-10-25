# Constant-assignment result boundaries

The first #488 candidate removed redundant clamps correctly for bare writes,
but changed an observable single-command composition boundary. A bounded global
write without dirty tracking previously expanded to a sequence, so
`Execute::store_success_score(...).run_actions(World::global().integer.set(7))`
called a generated function. After elision it directly ran `scoreboard players
set`. Minecraft distinguishes these commands' result/success callbacks.

The root's isolated vanilla 26.2 command probe observed a sentinel of `-999`
remaining unchanged for both stores around the legacy non-returning function,
but `7`/`1` for the direct set. A return-run wrapper produced `0`/`0` around the
function versus `7`/`1` around the direct set. This is correctness evidence,
not a performance measurement.

## Narrow correction

`lower_single_for_export` now retains the sequence function when constant clamp
elision would expose a terminal command that previously required that function.
The private numeric predicate is consulted only after normal validation and
lowering. Resource-retaining wrappers (`WithConditionResources` and
`WithScoreOperands`) forward the predicate without skipping their registration.
No variants, public APIs, or independent optimization pipeline are added.

The predicate distinguishes the original cardinalities:

- A non-selected bounded write with no dirty nodes retains its result boundary
  when at least one bound command would have existed.
- Full-i32 bounds on a single holder generated no clamps and remain direct,
  just like unbounded writes. One-sided bounds still require the old boundary.
- Wildcard bounds previously generated min/max operations even for full-i32
  bounds, so their sequence boundary remains.
- Dirty writes still have multiple commands and use the existing sequence path.
- Non-self selectors already lower to one execute-bound commit helper; their
  binding and result boundary remain unchanged, without another helper layer.
- Bare writes still benefit from clamp elision. The original static fixture
  remains 46 → 30 command lines; result-sensitive compositions retain the
  necessary function call while shrinking its body.

The only production change outside numeric.rs is the single-command boundary
and its recursive predicate in ir/mod.rs. This deliberately avoids the variant
work concurrently owned by #487.

## Regressions and native reproduction

`numeric_constant_results.rs` uses public derived global State accessors,
`Execute::run_actions`, both store modes, score/resource conditions, registered
function calls, and return-run components through the full pack exporter.
It checks shared helper provenance, setup replay, and A/B/A isolation.

The numeric unit fixture starts with the canonical FixedScore accessor's `.set`
action. Its 600 emitted combinations cover five bounds forms, self/fake/wildcard/
random selectors, dirty tracking, both resource wrappers, nested execute, and
return-run. The native fixture exports 112 representative compositions of those
actions, including direct ReturnRun nodes; the public export adds 16 true-path
observations and four false-gate controls. All 132 observations run both before
and after reload (264 per pack). The false gates must preserve an intentionally
out-of-bounds destination value of 17, and callback observations must match the
baseline and their pre-reload values. Random-owner checks pair value and dirty
scores on two eligible entities rather than assuming the same random entity wins in two independent runs.

Export the same test files on the integration base
`399b4175cc7e0232c4a03b0d3f0959234a485ae2` and candidate. The earlier proof used
pre-elision base `655e4f0cd2f1cc828eac724aabe045e3c47ffe0f`. On the base, add only the
`#[cfg(test)]` result_tests module declaration to numeric.rs. Keep production
lowering unchanged. Use each side's output paths below:

```sh
SAND_NUMERIC_RESULT_FIXTURE=/tmp/sand-488-candidate-numeric.json \
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/sand-448-constant-target \
  cargo test -p sand-core --lib export_numeric_result_fixture -- --ignored
SAND_NUMERIC_AUTHOR_FIXTURE=/tmp/sand-488-candidate-author.json \
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/sand-448-constant-target \
  cargo test -p sand-core --test numeric_constant_results \
  export_public_numeric_result_fixture -- --ignored
```

After root authorizes the focused native run, invoke the maintained harness on
a clean, frozen candidate commit. Use a new output directory and the cached
26.2 server JAR; no download or CLI native process is required:

```sh
python3 sand-core/tests/support/numeric_result_native.py \
  --commit "$(git rev-parse HEAD)" \
  --baseline-commit 399b4175cc7e0232c4a03b0d3f0959234a485ae2 \
  --baseline-numeric /tmp/sand-488-baseline-numeric.json \
  --baseline-author /tmp/sand-488-baseline-author.json \
  --candidate-numeric /tmp/sand-488-candidate-numeric.json \
  --candidate-author /tmp/sand-488-candidate-author.json \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --java /path/to/java25 --output /tmp/sand-488-result-differential
```

The runner reuses `scripts/vanilla_reload_harness.py`, binds loopback, allocates
a fresh port for each private server, and checks startup/reload/shutdown. A test
marker appended after the exported load-tag members proves that actual lifecycle
callbacks completed before the post-reload assertions; the console reload marker
alone is insufficient. Focused Python regressions verify the hook order and that
ordinary observation waits do not recursively rerun assertions. It requires a
clean matching commit and records source/tree, fixture, JAR, and pack hashes with all observations and log paths. Explicit known callback
expectations supplement baseline/candidate equality, preventing a vacuous
comparison of two broken exports. Full checks and shared example baseline
regeneration remain separate, root-coordinated gates.

The earlier 128-observation native proof at clean commit `7c641b13` is retained
in `/tmp/sand-488-results-7c641b13/`. That run checked semantic observations before
reload and checked reload completion separately; it did not claim semantic
assertions after reload. The expanded maintained harness closes that evidence gap.

## Portable frontend parity correction

The full check at `e43ad07c` failed the unchanged exact Rust/portable parity test:
portable `score_set` still encoded its literal and then emitted two redundant
bounds commands. That run is a failure, not completed full validation.

Portable assignment now constructs the same private `NumericWrite` model as Rust
State accessors and calls its canonical lowering. Only the method's crate-private
visibility changes. Portable v1 accepts i32 literals and player-self/global
holders, so this constant path neither registers generated helpers nor accesses
ambient export registries. Errors retain the operation's module, pointer and
origin; target-specific standalone command validation still follows lowering.
`score_add` retains its existing delta and runtime clamps. Defaults and lifecycle
initialization remain separate from assignment encoding.

The exact reachable-command/lifecycle parity assertion remains intact and now
also covers global writes, in-range values, both i32 extremes, and subsequent
runtime addition. Constructed/decoded tests cover 50 combinations of player and
global ownership, absent/full/equal/positive/negative bounds, and i32 literals.
An explicit branch regression retains the single decision and return-run helper
boundary while both branch bodies contain only their bounded literal write.
Portable v1 has no typed result/success-store operation; this correction adds no
new operation or raw escape hatch to imitate one.

The earlier 264-observation native evidence covers the Rust numeric path, not
this newly shared portable frontend. The maintained
`scripts/validate_portable_global.py` audit already compiles authored portable
input and asserts bounded assignment, conditional assignment, no-player global
ownership, and reload preservation. It is the focused native follow-up for this
change after a frozen candidate is reviewed. The player counter audit currently
exercises addition, not literal assignment; player assignment has exact emitted
Rust/portable parity coverage here, not a newly claimed live multiplayer proof.
No new native or full-check pass is claimed by this note.
