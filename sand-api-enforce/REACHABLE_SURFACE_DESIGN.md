# Reachable public API enforcement on stable Rust

## Finding

An attribute procedural macro cannot enforce complete coverage. It runs only
for an item that already has the attribute and cannot observe an unannotated
sibling, a re-export elsewhere, or another crate's inherent methods. Stable
Rust also has no supported compiler API that returns the fully resolved public
surface. `rustdoc` JSON would answer much of the question, but remains unstable
and therefore cannot be Sand's ordinary stable-toolchain build gate.

The maintainable stable design is a hybrid:

1. Parse the workspace-owned source graph before compiling the facade.
2. Resolve the items actually reachable through `sand`, including explicit,
   renamed, chained, and glob re-exports.
3. Discover child surface (inherent methods, trait items, associated items,
   public fields, and enum variants) at each underlying definition, then map
   it through every facade alias.
4. Ask controlled code generators for declarations from the same schema used
   to emit Rust. Never attempt to infer their expanded token stream.
5. Compare the resulting identities and paths with a central contract index.
   Require one contract per underlying identity, one reachable canonical path,
   and the exact discovered alias set.
6. Run the comparison from `sand/build.rs`, parameterized by Cargo's
   `CARGO_FEATURE_*` environment, so ordinary `cargo check` is the gate.

`reachable.rs` and `tests/reachable_surface.rs` are an executable proof of the
graph and comparison portions of this design.

## Alternatives considered

### Scan only the facade source

This catches a new `pub use`, but treats a whole imported module or type as one
opaque declaration. It misses new methods, variants, fields, and trait items in
the provider crate. It also cannot correctly distinguish aliases from distinct
APIs. This is the gap in the initial prototype.

### Generate the complete facade

A generated list of wrapper modules and re-exports makes top-level names easy
to audit, but inherent methods still belong to their defining types. Wrapping
every type to control method reachability is invasive, harms interoperability,
and duplicates implementation signatures. Generation is useful for registries
and other APIs already driven by schemas, not as the universal facade model.

### Source/re-export graph only

This is complete for ordinary local Rust declarations, but no source parser can
see arbitrary procedural-macro expansion. Treating macro output as absent would
create a bypass; treating it as guessed output would be brittle.

### Hybrid graph plus generator providers (recommended)

Ordinary APIs come from syntax and reachability. Generated APIs come from a
provider owned by the generator. The contract comparison is shared. For Sand's
derive macros, the provider must consume the same parsed declaration model as
the proc macro; for command/registry generation, it must consume the existing
command/registry schema. A generator that emits author-facing Rust without a
provider record is itself a build error.

## Production integration requirements

- Load every local crate that can feed the facade from an explicit crate map;
  do not follow arbitrary dependencies outside the supported boundary.
- Model Cargo target cfg values in addition to the feature evaluator proven
  here. Run each supported feature configuration, not merely `--all-features`
  when features are mutually exclusive.
- Add `#[path]`, raw identifiers, `extern crate` aliases, and macro namespace
  handling before replacing the current build gate.
- Preserve source spans on graph nodes so diagnostics point to both the
  reachable facade edge and underlying declaration.
- Make unresolved public re-exports a hard error. The proof currently ignores
  unknown external targets because its fixture graph is deliberately local.
- Keep exclusions structural and narrow: `pub(crate)`, `pub(super)`,
  `#[doc(hidden)]`, `__private`, and generator records explicitly marked as
  internal. Emit an auditable exclusion report.
- Have the contract index reference the stable underlying identity separately
  from its canonical facade path. Canonical paths and aliases are graph output,
  not free-form strings maintained independently.
- Serialize sorted maps/sets only; include the selected feature/cfg set in the
  exported catalog header.

## Proven fixture cases

The integration fixture executes coverage for:

- explicit renamed and glob re-exports;
- a chained re-export whose definition lives in a private module;
- inherent methods and associated constants;
- trait methods, associated types, and associated constants;
- public struct fields and enum variants;
- type aliases;
- enabled and disabled feature-gated functions;
- controlled generated types and generated methods;
- `pub(crate)`, private, `#[doc(hidden)]`, and `__private` exclusions;
- missing identities, incomplete alias sets, and duplicate canonical paths.
