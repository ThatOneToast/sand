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

| Minecraft version | Enforced identities | Commands | Registries | Baseline |
| --- | ---: | ---: | ---: | --- |
| 1.21.4 (compatibility) | 10,234 | 902 | 4,288 | `api-surface-baseline-1.21.4.txt` |
| 26.2 (latest/default) | 11,144 | 1,233 | 4,867 | `api-surface-baseline.txt` |

The handwritten source contribution is 4,776 identities in both profiles.
An explicit `SAND_ALLOW_PLACEHOLDER_CODEGEN=1` fallback uses a third,
`placeholder-codegen` profile with 5,044 identities (4,776 source identities
plus 268 checked-in generator identities). The fallback writer
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
| Modules | 36 |
| Attribute procedural macros | 10 |
| Derive procedural macros | 6 |
| Function-like procedural macros | 4 |
| Declarative macros | 3 |
| Structs | 954 |
| Enums | 160 |
| Traits | 43 |
| Type aliases | 4 |
| Constants | 2 |
| Free functions | 523 |
| Inherent methods | 2,793 |
| Trait methods | 71 |
| Associated constants | 17 |
| Associated types | 3 |
| Public fields | 725 |
| Enum variants | 5,790 |

Generated static families account for 6,368 identities:

- vanilla registries: 4 enums, 4 inherent methods, and 4,859 variants
  (4,867 total); and
- generated command builders: 1,233 identities;
- typed registry-ID wrappers: 148 identities (including the contracted
  resource-reference IDs);
- effect registry enums: 95 identities;
- generated event marker types: 25 identities.

The remaining 4,776 identities come from ordinary source declarations,
including the 20 exported procedural macros. Input-dependent items emitted
into downstream crates by attributes and derives are parametric families, so
they do not have an honest finite installed count and are not represented as
zero-item static scopes. API-producing proc macros validate their actual
generated surface and contract Rustdoc while expanding in the consumer crate.

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
`ScoreOperand` is owned and enforced by the state scope where the public score
API that uses it lives.

The generated `sand::vanilla` scope is enforced as one versioned family rather
than 4,859 handwritten variant contracts. Its registry schema emits the enum,
`resource_location` method, exact vanilla variants, Rustdoc, and deterministic
contracts together. The 26.2 profile owns 4,867 identities and the 1.21.4
profile owns 4,288. The explicit placeholder profile owns zero registry
identities but must still connect and validate its empty provider/source parity;
an empty catalog cannot silently claim a real Minecraft profile.

The root-facade source scope owns 37 direct identities and is enforced. Its
contracts cover the curated root modules, `ResourceLocation`, declarative
macros, derives, and every supported procedural macro. The former
`#[component]`, `#[event]`, and `#[item]` attributes were renamed to
`#[datapack_component]`, `#[on_event]`, and `#[custom_item]`: each old name
also named a topic module in Rust's separate macro namespace, which made a
single collision-free CLI identity impossible. The renamed attributes are the
only supported paths; `sand::component`, `sand::event`, and `sand::item`
remain the canonical topic modules.

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
attribute and derive output remains outside the finite installed catalog;
isolated downstream fixtures exercise the same expansion-time guards used by
ordinary consumer builds. The static report therefore cannot be made complete
by blessing a named zero-item consumer scope.

The exact 26.2 profile records 11,258 enforced static identities across 29
scopes, with zero pending scopes and zero pending items. The 1.21.4 and explicit
placeholder profiles enforce the same source and checked-in-generator boundary
against their independently generated totals. Every reachable identity maps to
exactly one scope; an uncontracted addition, ownership collision, provider
drift, or enforced-to-pending regression stops an ordinary build.

## Canonical ownership after migration

No semantic API is canonically owned by a prelude descendant. Registry-ID
wrappers live under `sand::registry`, topic-specific wrappers retain their
stronger domain owners, and `sand::prelude` contains aliases only. Compiler
graphs, exporter plans, coverage bookkeeping, lowering records, and event
transport structures were removed from the supported facade or made private;
the retained public surface is the author-facing semantic layer.
