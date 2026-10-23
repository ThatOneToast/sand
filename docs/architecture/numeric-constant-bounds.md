# Bounded constant-assignment lowering (#448)

This implements only redundant destination-bound removal after literal numeric
assignment encoding. It does not complete [#448](https://github.com/ThatOneToast/sand/issues/448),
introduce an optimizer pipeline, fold compound curves, or change dynamic writes.

## Semantic boundary

`NumericWrite::lower` already validates the holder, destination objective, dirty
objectives, positive scale, and ordered bounds. For `StatCurve::Constant`, it
rejects NaN and then calls the canonical `encode_fixed`. That encoder multiplies
by the destination scale, rounds exact halves away from zero, saturates to i32,
and clamps to inclusive destination bounds. Thus the emitted integer cannot
require another destination clamp. Literal infinities and finite scaling overflow
already saturate; this slice preserves that behavior. General curve arithmetic
and failed runtime reads retain their existing validation/overflow policies.

The constant branch now emits the encoded write followed by the original dirty
writes. Both field and component invalidation remain ordered after the write.
Dynamic writes still apply bounds and then call the same private dirty helper.
Selectors retain the existing single binding around the complete commit, with
`@s` inside the helper. A random or plural destination cannot change owners
between its value and dirty writes. Fake, global, and wildcard holders retain
their original holders; wildcard constants no longer allocate bound objectives
or introduce a `#value` holder just to clamp an already-bounded value.

This change does not alter author function IDs, runtime reporting, the generated
identity allocator, public APIs, API profiles, or lifecycle registration. Generated
commit IDs change with their smaller bodies and retain their semantic owners.

## Measured static result

The same maintained fixture was run against base
`655e4f0cd2f1cc828eac724aabe045e3c47ffe0f` and the candidate lowering, using
Minecraft profile 26.2. The [captured reports](evidence/numeric-constant-bounds.json)
include fixture/source hashes, provenance, unknowns, and reporting limitations.
The fixture uses bounded integer and fixed-point fields, two entity dirty
objectives, a global field without dirty tracking, and six destination forms.

RuntimeReport explicit command evaluations per invocation:

- Single `@s` and fake holder: **5 → 3** each (40% fewer).
- Wildcard: **7 → 3** (57.1% fewer), with **2 → 0** generated bound objectives.
- Global: **3 → 1** (66.7% fewer).
- Random and plural destination commit helpers: **5 → 3** each. Their entrypoints
  retain the report's minimum of 1 and unknown maximum due to selector
  multiplicity; no whole-entrypoint bound is inferred.
- State lifecycle load remains **12 → 12**. The separate two-command numeric
  objective initializer disappears because no numeric scratch is needed.

Across the complete fixture, nonblank, noncomment command lines fall from
**46 to 30** (16 fewer, 34.8%). Function definitions fall from **10 to 9**.
These are static generated-command counts and conservative report estimates,
not MSPT, selector search cost, terminal execution multiplicity, Minecraft
command-budget units, or a measured server speedup. Result-sensitive composition and its separate
[native validation workflow](numeric-constant-results.md) are documented independently.

To reproduce the probe, copy `sand-core/tests/numeric_constant_assignment.rs`
from this change into a clean base checkout and run the same command in each
checkout, with a separate absolute target directory for each concurrent build:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/sand-448-constant-target \
  cargo test -p sand-core --test numeric_constant_assignment \
  report_constant_assignment_costs -- --ignored --nocapture
```

The ignored probe emits `CONSTANT_REPORT` JSON and a separate
`CONSTANT_STATIC_COMMAND_LINES` count. Its candidate regression assertions are
in another test, so the probe runs unchanged against the baseline lowering.
Shared example runtime baselines were neither regenerated nor hand-edited.
Their coordinated refresh must use `scripts/check-examples.sh --update-runtime-baselines`.

Result-sensitive nested execute/return composition retains the original function
boundary even when its body shrinks to one write; see the
[result-boundary correction](numeric-constant-results.md).

## Focused validation and remaining gates

Regression coverage checks in-range and out-of-range integers/fixed-point
values, positive/negative half rounding, finite and infinite saturation, exact
and exceeded i32 boundaries, equal bounds, NaN rejection, invalid destination
metadata, dirty tracking on/off, and no constant bound-storage requests. Export
coverage checks fake/global/wildcard/self/random/plural ownership, both dirty
dependencies, retained State initialization, helper provenance, and cached A/B/A
namespace isolation. Existing dynamic wildcard/collision tests remain active;
a strengthened ordering assertion puts failed-read guards before the dynamic
write, bounds, and dirty marking. Native-source export regressions also remain
part of the focused run; these inspect generated code, not a live server.

Focused checks passed: 12 `ir::numeric::tests` unit tests; the
`numeric_constant_assignment`, `numeric_state_assignment`,
`native_numeric_sources`, `native_attribute_values`, and
`inventory_numeric_sources` integration targets; the explicit before/after
probe; and Clippy for the library plus those five targets with `-D warnings`.
Formatting and `git diff --check` also passed. All Cargo invocations used two
jobs and `/tmp/sand-448-constant-target`.

Full `scripts/check.sh`, native validation, and shared example baseline refresh
are held for root coordination after pending merges. This is a focused source
candidate for independent review and CI, not a claim that those gates passed.
