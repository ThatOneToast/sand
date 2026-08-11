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

Type-alias targets use the same lexical resolver as impl ownership, including
relative, `self`, `super`, `crate`, import, extern-alias, alias-chain, and
generated-type targets. Sand-owned inherent members remain one underlying
identity when reached through an alias. Unknown third-party nominal targets
fail closed. Standard-library and primitive/container targets are a deliberate
language boundary: the Sand alias itself is supported, but the graph does not
claim the Rust standard library's inherent API as Sand-owned surface.

Public re-export edges are fail-closed: named, glob, and chained edges must
resolve to an explicitly modeled source crate. A third-party facade export
cannot disappear from the contract graph merely because its dependency source
was not loaded.

`advanced` is a supported tier, not an exemption. `__private` is excluded with
a structural reason.

## Version-keyed all-feature baselines

The checked baselines are reproduced by `sand/build.rs` on stable Rust 1.96.0.
Every normal build enables the union of Sand's supported facade features for
the source audit, uses the current Cargo target cfg, reads declarations from
the explicit workspace crate map, and consumes provider artifacts generated
beside generated Rust. Command and registry provider catalogs embed their
resolved Minecraft version. Both catalogs must agree, and that exact value
selects a reviewed entry in `sand/api-surface-profiles.toml`; unknown or mixed
versions fail compilation. Each profile binds its own exact item ceiling and
byte-for-byte report baseline. The reports record their Minecraft version,
kinds, origins, and scope-level counts, never item exemptions.

The verified profiles are:

| Minecraft version | Static identities | Commands | Registries | Baseline |
| --- | ---: | ---: | ---: | --- |
| 1.21.4 (compatibility) | 10,730 | 924 | 4,288 | `api-surface-baseline-1.21.4.txt` |
| 26.2 (latest/default) | 11,640 | 1,255 | 4,867 | `api-surface-baseline.txt` |

The handwritten source contribution is 5,262 identities in both profiles.
An explicit `SAND_ALLOW_PLACEHOLDER_CODEGEN=1` fallback uses a third,
source-only `placeholder-codegen` profile with 5,518 identities (5,262 source
identities plus 276 checked-in generator identities). The fallback writer
atomically replaces generated Rust and both provider catalogs; the catalogs
are machine-marked empty placeholders and must agree. The facade keeps the
contracted `sand::vanilla` module but cfg-disables its unavailable generated
re-exports. Placeholder mode therefore still runs the exact source/scope
ratchet, while real providers cannot select or weaken that profile. It is a
compile-only recovery mode: `sand-core` test targets reference unavailable
generated command symbols and therefore require real codegen. Provider tests
instead prove placeholder marking, empty catalogs, and exact source parity.

The following detailed kind count describes the latest/default 26.2 surface:

| Kind | Count |
| --- | ---: |
| Modules | 80 |
| Attribute procedural macros | 8 |
| Derive procedural macros | 3 |
| Function-like procedural macros | 4 |
| Declarative macros | 3 |
| Structs | 970 |
| Enums | 166 |
| Traits | 38 |
| Type aliases | 13 |
| Constants | 18 |
| Statics | 1 |
| Free functions | 556 |
| Inherent methods | 2,816 |
| Trait methods | 56 |
| Associated constants | 21 |
| Associated types | 2 |
| Public fields | 1,013 |
| Enum variants | 5,872 |

Generated static families account for 6,378 identities:

- vanilla registries: 4 enums, 4 inherent functions, and 4,859 variants
  (4,867 total); and
- generated command builders: 486 structs and 769 functions/methods
  (1,255 total);
- typed registry-ID wrappers: 136 identities (including the contracted
  resource-reference IDs);
- effect registry enums: 95 identities;
- generated event marker types: 25 identities.

The remaining 5,262 identities come from ordinary source declarations,
including the 15 exported procedural macros. Input-dependent items emitted
into downstream crates by attributes and derives are parametric families, so
they do not have an honest finite installed count. Each such generator is a
separate provider scope.

The migrated `sand::predicate` source scope owns 119 identities, all enforced
and contracted at their underlying definitions. Its four reachable
`PredicateId` identities are generated from the semantic `registry_id!`
declaration and form a separate enforced generator partition. Privatizing
builder state and validation plumbing, making `PredicateRoot` opaque, removing
obsolete compatibility paths, and typing domain identifiers reduced the
module from its 234-identity foundation baseline without losing an intentional
author operation.

The migrated `sand::execute_when` scope owns 22 supported identities, all
enforced and contracted at their `sand-core` definitions. Two Rust-visible
test/setup helpers became private, and the public if/else builder now lowers
through one generated dispatcher so a success arm that changes the tested
state cannot make the failure arm run afterward.

The migrated `sand::condition` scope owns 12 supported identities: an opaque
`Condition` plus 11 typed constructors and combinators. Its audit removed 75
accidental identities by hiding variants, payload fields, lowering plans,
range/comparison internals, and obsolete string-rendering compatibility APIs.
`ScoreOperand` moved to the pending state scope where the public score API that
uses it is actually owned.

## Intentional migration scopes

Static source scopes are: root, command, event, events, item, inventory,
predicate, state, entity, participant, component, condition, execute_when,
resource_ref, version, vfx, systems, text, data, vanilla, advanced, and
resourcepack. `prelude` is an alias projection rather than an owning scope.

Generator scopes separately cover generated commands, vanilla registries,
checked-in registry/effect/event macro families,
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
binds `registry_id!`, `vanilla_registry_enum!`,
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

After the resource-reference tranche, the exact 26.2 profile records 11,640
static identities: 180 enforced predicate, branch, condition, and resource-ID
identities and 11,460 identities across 34 pending scopes. The 1.21.4 and
explicit placeholder profiles enforce the same 180 source/generated identities
against their independently generated totals.

Every reachable identity must map to exactly one scope. Enforced scopes reject
missing contracts during ordinary compilation. Pending scopes remain in the
deterministic coverage report, and the pending baseline may only decrease.
Issue #327 completes only at zero pending source and generator scopes.

## Known canonical defects to resolve during migration

The current facade contains 396 source identities owned by the temporary
`prelude-unassigned-source` scope,
including component families that lack the promised canonical topic path.
Migration must add a canonical topic re-export or deliberately remove each
such promise; it must not make the prelude canonical by accident.

The remaining 126 registry-ID wrapper identities are likewise assigned to a
low-precedence pending prelude projection until their canonical topic modules
are established. `PredicateId` is no longer in that provisional partition.

Broad whole-module exports also expose graph, compiler, registry, and lowering
types that ordinary authors do not use. Each scope audit should narrow those
exports or move deliberate integration hooks to `advanced` before attaching a
compatibility contract.
