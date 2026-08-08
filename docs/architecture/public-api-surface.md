# Supported public API surface

This document defines the compatibility boundary used by Sand's API contract
migration. It is the policy input to the machine-readable scope manifest; it
is not an item-by-item exemption list.

## Boundary

Sand supports author-facing Rust APIs reachable through a non-hidden path in
the `sand` facade. This includes root macros and types, canonical topic
modules, the curated prelude, feature-gated facade APIs, the explicitly tiered
`advanced` module, and author-callable APIs emitted by Sand-owned generators.

The following are not independently supported surfaces:

- public items in implementation crates that have no supported `sand::` path;
- compiler or macro wiring structurally contained by `sand::__private`;
- private, `pub(crate)`, and `pub(super)` declarations; and
- downstream application items that are not generated Sand API families.

`sand-core`, `sand-components`, `sand-commands`, and the other implementation
packages may need Rust `pub` visibility for composition. That visibility alone
does not create a compatibility promise. If direct use of one of those crates
is intentionally supported later, it requires a separate declared contract
surface.
`#[doc(hidden)]` changes Rustdoc presentation only and cannot remove a
reachable item from an enforced supported scope.

## Canonical identity

The identity of an API is its resolved underlying definition plus its member
identity. Textual re-export paths do not create new identities.

Canonical path precedence is:

1. Root macro entry points and `ResourceLocation` remain under `sand`.
2. A semantic topic module owns its APIs.
3. `sand::prelude` is alias-only and never wins canonical ownership.
4. `sand::command` is canonical; `sand::cmd` is its conventional alias.
5. `sand::predicate` owns predicate builders over component/prelude aliases.
6. `sand::inventory` owns live inventory locations over item/prelude aliases.
7. Curated `sand::text` and `sand::data` paths own their intentional overlaps.

A module re-export alias applies prefix substitution to all of its reachable
descendants. The source graph derives aliases from explicit, renamed, chained,
and glob re-exports. A contract records the canonical lookup path and useful
aliases for installed metadata, but an enforced scope requires that declaration
to equal the graph-derived path set. An identity exposed by multiple candidate
topic modules without an ownership rule is an error; two identities selecting
one canonical path is also an error.

Public re-export edges are fail-closed: named, glob, and chained edges must
resolve to an explicitly modeled source crate. A third-party facade export
cannot disappear from the contract graph merely because its dependency source
was not loaded.

`advanced` is a supported tier, not an exemption. `__private` is excluded with
a structural reason.

## Current all-feature baseline

The checked baseline is reproduced by `sand/build.rs` on stable Rust 1.96.0.
Every normal build enables the union of Sand's supported facade features for
the source audit, uses the current Cargo target cfg, reads declarations from
the explicit workspace crate map, and consumes provider artifacts generated
beside generated Rust. The measured aggregate is byte-compared with
`sand/api-surface-baseline.txt`; that file records kinds, origins, and
scope-level counts, never item exemptions.

The current installed static surface contains **11,835 unique API elements**:

| Kind | Count |
| --- | ---: |
| Modules | 80 |
| Attribute procedural macros | 8 |
| Derive procedural macros | 3 |
| Function-like procedural macros | 4 |
| Declarative macros | 3 |
| Structs | 972 |
| Enums | 170 |
| Traits | 38 |
| Type aliases | 16 |
| Constants | 18 |
| Statics | 1 |
| Free functions | 557 |
| Inherent methods | 2,859 |
| Trait methods | 56 |
| Associated constants | 21 |
| Associated types | 2 |
| Public fields | 1,120 |
| Enum variants | 5,907 |

Generated static families account for 6,398 identities:

- vanilla registries: 4 enums, 4 inherent functions, and 4,859 variants
  (4,867 total); and
- generated command builders: 486 structs and 769 functions/methods
  (1,255 total);
- typed registry-ID wrappers: 130 identities;
- effect registry enums: 95 identities;
- generated event marker types: 25 identities; and
- typed resource-reference wrappers: 26 identities.

The remaining 5,437 identities come from ordinary source declarations,
including the 15 exported procedural macros. Input-dependent items emitted
into downstream crates by attributes and derives are parametric families, so
they do not have an honest finite installed count. Each such generator is a
separate provider scope.

The current `sand::predicate` source scope owns 234 identities. Only nine of
those identities currently resolve to pilot contracts, so the prototype
catalog did not cover that module. The measured surface also preserves legal
but undesirable field/method lookup collisions as distinct identities; the
predicate migration must remove those collisions before it can be enforced.

## Intentional migration scopes

Static source scopes are: root, command, event, events, item, inventory,
predicate, state, entity, participant, component, condition, execute_when,
resource_ref, version, vfx, systems, text, data, vanilla, advanced, and
resourcepack. `prelude` is an alias projection rather than an owning scope.

Generator scopes separately cover generated commands, vanilla registries,
checked-in registry/effect/event/resource-reference macro families,
function/component/event/item/armor-event/schedule/entity-archetype attribute
macros, State/EntityStateEnum/SandStorage derives, and resource-pack macro
output. A generator provider consumes the same parsed input or schema that
emits its Rust API.

Reachable item-position `include!` calls are fail-closed. Literal file paths
are parsed into the source graph. Build-output expressions must be explicitly
bound to a named provider that owns generated declarations in that exact
module; Sand currently binds the command and vanilla-registry output modules.
For those modules the facade build separately parses the emitted Rust and
requires its public identity/kind set to equal the provider exactly. Editing a
generated file to add a public item, or declaring provider metadata for an item
that was not emitted, therefore stops normal compilation.
An unbound opaque include stops an ordinary build before it can conceal a new
public declaration.

Checked-in item macro families use the same exact-edge rule. The facade build
binds `resource_ref!`, `registry_id!`, `vanilla_registry_enum!`,
`gamemode_transition!`, and `status_effect_marker!` by both their defining
module and lexical macro path to their structural providers. A same-named
macro in another module remains unclassified, and an unbound item macro that
can emit facade declarations stops reachability extraction.

Item macros that intentionally emit no facade identity are classified on an
equally exact module/path edge. Local `macro_rules!` families are accepted only
when every transcriber arm passes a structural audit that rejects public
declarations, inherent impls, nested item macros, and repetition. This covers
the conversion and sealed/trait-implementation families in NBT, tags, item
predicates, and events. The one tuple-arity event family that relied on
repetition is written as explicit trait impls so it does not require a weaker
exception. External inert classifications are limited to the exact
`inventory::collect!` linker-registration and `thread_local!` storage spellings
used by compiler/export wiring.

Reachability audits the full lexical ancestor chain of every exposed
declaration, including items reached by re-export from a private module. Module
attributes and API producers, opaque includes, item and associated-position
macros, and unsupported foreign syntax in those ancestors must all be modeled.
Macro namespace checks likewise include ancestor imports, modules, extern
aliases, and resolvable globs, so a trusted derive or attribute name cannot be
shadowed above its defining module. Custom helper attributes are accepted only
on the declaration forms and under the derive that defines them.

The static providers are connected to the facade build. Input-dependent
attribute/derive scopes are marked `consumer_build`; they remain pending and
cannot be changed to enforced until a consuming-crate build connects the
corresponding provider audit. The foundation proves this mechanism with the
real `SandStorage` derive, but does not claim every downstream generator has
been migrated.

The foundation baseline records 39 pending architectural scopes and 11,835
pending static identities. No scope is marked enforced by the foundation;
predicate becomes the first enforced source scope only after its complete
migration tranche.

Every reachable identity must map to exactly one scope. Enforced scopes reject
missing contracts during ordinary compilation. Pending scopes remain in the
deterministic coverage report, and the pending baseline may only decrease.
Issue #327 completes only at zero pending source and generator scopes.

## Known canonical defects to resolve during migration

The current facade contains 404 source identities owned by the temporary
`prelude-unassigned-source` scope,
including component families that lack the promised canonical topic path.
Migration must add a canonical topic re-export or deliberately remove each
such promise; it must not make the prelude canonical by accident.

Broad whole-module exports also expose graph, compiler, registry, and lowering
types that ordinary authors do not use. Each scope audit should narrow those
exports or move deliberate integration hooks to `advanced` before attaching a
compatibility contract.
