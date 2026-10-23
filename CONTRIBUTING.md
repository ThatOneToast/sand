# Contributing

Sand is a Rust framework for building vanilla Minecraft Java datapacks. Changes
should move datapack authoring toward reusable gameplay abstractions rather than
toward more command-shaped APIs.

Read [AGENTS.md](AGENTS.md) for the project's design direction.

## Public API

The supported author-facing API is the `sand` crate. User-facing examples
should normally start with:

```rust
use sand::prelude::*;
```

Do not expose implementation crates as the recommended authoring surface.

Before adding public API, inspect the existing boundary:

```sh
sand api search <terms>
sand api show <path>
sand api module <module>
```

Prefer one canonical representation for a concept. Keep types small and
composable, and use type parameters when a meaningful subtype distinction is
needed instead of introducing several nearly identical wrappers.

Public API changes must keep API contracts and Rustdoc accurate.

## Development

`rust-toolchain.toml` is the toolchain authority for local development and CI.
Keep the workspace `rust-version` aligned with it.

Add focused regression coverage for behavior you change. Prefer exact generated
output tests where practical and trybuild coverage for macro/type-system
diagnostics.

Do not hand-edit generated output when a generator owns it. Network-heavy
Minecraft data regeneration should only be required for changes that actually
modify generated Minecraft data.

Run focused tests while iterating, then run the full repository validation:

```sh
scripts/check.sh
git diff --check
```

The full check covers formatting, strict Clippy, workspace and macro tests,
canonical façade-only example builds, Rustdoc, the book, and documentation
links.

Do not weaken tests, API-contract enforcement, or architecture guards to make a
change pass.

## Runtime performance review and example baselines

`scripts/check-examples.sh` tests and builds every standalone project under
`examples/`. Its existing `sand build --format json` invocation supplies the
canonical `RuntimeReport` for comparison with the committed files in
`benchmarks/example-runtime-baselines/`. The full local check and Rust CI already
run this script. Baseline checking does not export a second pack or analyze
command text through another estimator.

The baseline includes each function's explicit command evaluation range,
lexical selector sites, ownership and cadence provenance, ordered tick/load
entries, and limitations. Unknown upper bounds remain `null` with their reasons;
missing owners and truncated reasons remain visible. These are static report
fields, **not measured idle costs, runtime scan counts, MSPT, or Minecraft
command-chain budget units**. A lexical selector site may execute zero, one, or
many times. Population-dependent work with an unknown upper bound cannot be
turned into a safe player/entity capacity by multiplying the reported minimum.

To intentionally accept a reviewed change, run:

```sh
scripts/check-examples.sh --update-runtime-baselines
```

This uses the same example tests and single build per example, then regenerates
all baselines. Commit the generated JSON alongside the change and explain its
delta in the PR. Do not hand-edit counts, remove unknown reasons, or regenerate
just to silence a failure. Normal checks never update baselines and reject
missing/stale example files. Exact comparison catches decreases as well as
increases, new/removed functions, ownership/cadence drift, and changes in report
limitations. Build paths and incremental-write summaries are excluded; map/set
ordering is canonicalized. Tick/load ordering, selector-site lexical order
(including operations on the same execute line), and fused-system execution
order are preserved because they are semantic, not serialization noise.

For a runtime-feature PR, reviewers should ask:

- Which owners and cadence acquire work? Does anything run while idle?
- What changed in the example baselines, including unknown upper bounds? If an
  example does not exercise the feature, what focused regression covers it?
- Do tick paths add selector sites, entity NBT reads, macro calls, or unconditional
  writes? Inspect the lowering; the baseline is not a performance lint engine.
- Is runtime state namespace-owned, and are callback snapshots, lifecycle order,
  and shared ownership preserved?
- When claiming measured idle/active costs or a speedup, where are the native
  workload, version, before/after traces and pack hashes? Keep explicit command
  evaluations separate from actual server MSPT and command-budget measurements.

These gates enforce review of the existing report for examples. Performance
lints, population/command-budget planning, complete cost-class coverage, and
scheduled heavy-pack MSPT measurement remain separate work under #451 and its
related issues; an unchanged baseline does not prove those acceptance criteria.
