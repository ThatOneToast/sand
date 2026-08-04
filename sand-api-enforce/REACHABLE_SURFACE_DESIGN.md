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
6. Run the comparison from `sand/build.rs` so ordinary `cargo check` is the
   gate. The source audit enables the union of supported facade features on
   every build (preventing default-build headroom from hiding a new gated API)
   and uses Cargo's current target cfg. The foundation has no enforced
   repository scope; changing one to `enforced` activates the same
   provider-backed missing-contract and canonical/alias comparison proven by
   the fixtures.

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

The foundation connects installed command/registry and checked-in declarative
providers. Parametric proc-macro/derive scopes declare `consumer_build`
enforcement and fail closed if marked enforced without a named consuming-build
audit. The `SandStorage` acceptance fixture executes the real derive, shares
its generated-member model with its provider, and proves a missing generated
accessor stops normal compilation.

Item-position `include!` is part of the same boundary. A literal path is parsed
as source in its containing module. An opaque expression such as an `OUT_DIR`
include fails when that module becomes facade-reachable unless the build graph
explicitly binds the module to a named provider that owns declarations beneath
it. A binding covers exactly one opaque include, preventing a second generated
file from silently borrowing the first provider's audit.

## Production integration requirements

- Load every local crate that can feed the facade from an explicit crate map;
  do not follow arbitrary dependencies outside the supported boundary.
- Populate `CfgSet` with all supported facade features and Cargo's target
  environment. If mutually exclusive features are introduced later, split the
  checked baselines into the complete supported matrix. Unknown cfg predicates
  fail closed.
- The graph handles `#[path]`, raw identifiers, `extern crate` aliases, and
  `macro_export`'s crate-root namespace. Extend the same normalization if Sand
  adopts additional namespace-affecting syntax.
- Unresolved public re-exports into the explicit workspace crate map now fail
  with source line and facade-edge context. Preserve definition spans too when
  the proof is connected to user-facing diagnostics.
- Keep exclusions structural and narrow: Rust-private visibility, the named
  `__private` module boundary, and generator records explicitly marked as
  internal. `#[doc(hidden)]` only controls Rustdoc presentation and does not
  remove a reachable item from enforcement.
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
- `pub(crate)`/private and `__private` exclusions, plus proof that an arbitrary
  `#[doc(hidden)] pub` item remains enforced;
- missing identities, incomplete alias sets, and duplicate canonical paths.
