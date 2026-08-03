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
- `sand::__private` and `#[doc(hidden)]` compiler or macro wiring;
- private, `pub(crate)`, and `pub(super)` declarations; and
- downstream application items that are not generated Sand API families.

`sand-core`, `sand-components`, `sand-commands`, and the other implementation
packages may need Rust `pub` visibility for composition. That visibility alone
does not create a compatibility promise. If direct use of one of those crates
is intentionally supported later, it requires a separate declared contract
surface.

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
and glob re-exports. Contracts do not maintain a second hand-written alias
list. An identity exposed by multiple candidate topic modules without an
ownership rule is an error; two identities selecting one canonical path is
also an error.

`advanced` is a supported tier, not an exemption. `__private` is excluded with
a structural reason.

## Current all-feature baseline

The baseline was measured at prototype commit `827c259` with rustc/rustdoc
1.96.0 and all facade features enabled. Rustdoc JSON was used as an independent
audit oracle: the audit traversed the public `sand` module/use graph, resolved
cross-crate re-exports, counted each underlying definition once, and separately
enumerated public fields, enum variants and variant fields, inherent items, and
trait items.

The current installed static surface contains **11,663 unique API elements**:

| Kind | Count |
| --- | ---: |
| Modules | 80 |
| Procedural macros | 15 |
| Declarative macros | 3 |
| Structs | 970 |
| Enums | 170 |
| Traits | 38 |
| Type aliases | 16 |
| Constants | 12 |
| Statics | 1 |
| Functions and methods | 3,333 |
| Associated constants | 18 |
| Associated types | 2 |
| Public fields | 1,099 |
| Enum variants | 5,906 |

10,513 identities have two or more facade paths; only 1,150 have one path.
Counting Rustdoc pages therefore gives the wrong answer: alias modules create
duplicate pages, while members are embedded in their owner's page.

Generated static families account for 6,113 identities:

- vanilla registries: 4 enums, 4 inherent functions, and 4,859 variants
  (4,867 total); and
- generated command builders: 486 structs and 760 functions/methods
  (1,246 total).

The remaining static surface is 5,535 handwritten identities plus 15 proc
macro entry points. Input-dependent items emitted into downstream crates by
attributes and derives are parametric families, so they do not have an honest
finite installed count. Each such generator is a separate enforced provider
scope.

The current `sand::predicate` facade reaches 195 identities: one module, 12
structs, three enums, 61 methods, 88 public fields, and 30 variants. The
prototype catalog's 14 linked entries therefore did not cover that scope.

## Intentional migration scopes

Static source scopes are: root, command, event, events, item, inventory,
predicate, state, entity, participant, component, condition, execute_when,
resource_ref, version, vfx, systems, text, data, vanilla, advanced, and
resourcepack. `prelude` is an alias projection rather than an owning scope.

Generator scopes separately cover generated commands, vanilla registries,
function/component/event/item/armor-event/schedule/entity-archetype attribute
macros, State/EntityStateEnum/SandStorage derives, and resource-pack macro
output. A generator provider consumes the same parsed input or schema that
emits its Rust API.

The foundation baseline records 35 pending architectural scopes and 11,663
pending static identities. No scope is marked enforced by the foundation;
predicate becomes the first enforced source scope only after its complete
migration tranche.

Every reachable identity must map to exactly one scope. Enforced scopes reject
missing contracts during ordinary compilation. Pending scopes remain in the
deterministic coverage report, and the pending baseline may only decrease.
Issue #327 completes only at zero pending source and generator scopes.

## Known canonical defects to resolve during migration

The current facade contains 598 identities reachable only through the prelude,
including component families that lack the promised canonical topic path.
Migration must add a canonical topic re-export or deliberately remove each
such promise; it must not make the prelude canonical by accident.

Broad whole-module exports also expose graph, compiler, registry, and lowering
types that ordinary authors do not use. Each scope audit should narrow those
exports or move deliberate integration hooks to `advanced` before attaching a
compatibility contract.
