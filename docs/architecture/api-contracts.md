# Sand public API contracts

Status: phased migration in progress ([#327](https://github.com/ThatOneToast/sand/issues/327))

Sand's API contract work is an umbrella migration. The infrastructure may be
merged before every API is contracted, but partial coverage must remain
machine-visible and must never be described as complete.

## Supported boundary

The compatibility boundary is the author-facing API reachable through the
`sand` crate's non-hidden facade:

- root macros, functions, types, and constants;
- the curated `sand::prelude` aliases;
- canonical topic modules such as `sand::predicate` and `sand::event`;
- the explicitly supported `sand::advanced` tier;
- feature-gated facade APIs; and
- public Rust APIs emitted by Sand-owned generators.

An implementation crate's `pub` item is not supported merely because it is
Rust-visible. It becomes supported only when a `sand` facade path reaches it.
`sand::__private` compiler wiring and private or crate-private implementation
items are outside the boundary. `#[doc(hidden)]` alone is not an exclusion: a
reachable item in a supported topic module remains subject to the ratchet.

One underlying item has one canonical identity. Topic-module paths take
precedence over prelude paths; the latter are aliases. Other deliberate
alternate paths, such as the long and short command module names, are aliases
to the same identity rather than duplicate API entries.

## Stable-Rust enforcement architecture

An attribute procedural macro cannot detect an unannotated sibling and cannot
observe re-exports or arbitrary generated code. Sand therefore uses a hybrid:

1. A stable source graph follows the workspace-owned facade and resolves
   explicit, renamed, chained, and glob re-exports.
2. The graph maps public fields, enum variants, inherent methods, trait items,
   and associated items through every reachable facade path.
3. Sand-owned generators provide their public declarations from the same
   schema or parsed model that emits their Rust. Generated declarations are
   compared by the same contract audit; they are never guessed from source.
4. A central contract index selects one reachable canonical path per
   underlying identity. The graph derives the complete alias set and rejects
   missing contracts, unreachable paths, alias drift, and duplicate canonical
   identities.
5. `sand/build.rs` runs that graph and scope audit during ordinary
   `cargo check` and `cargo build`, regardless of the build's selected facade
   features. It consumes generated command/registry artifacts, discovers
   checked-in macro families, resolves contracts from the mapped source
   crates, partitions all 11,632 static identities in the latest profile, and byte-compares the
   deterministic aggregate baseline. A migration may mark a scope enforced
   only in the same change that supplies every contract for that scope.

Static installed generators are connected now: command and vanilla-registry
providers emit deterministic JSON beside their Rust, while checked-in
declarative families derive their shape from the generator body and
invocations. Input-dependent procedural macros and derives use
`consumer_build` scopes. Those scopes cannot be marked enforced until their
consumer-side provider audit is explicitly connected; zero-item enforcement
is rejected rather than passing vacuously. The real `SandStorage` fixture
shares its generated-member model between macro expansion and the build
provider and proves a missing generated accessor fails ordinary `cargo check`.
Producer connections are declaration-specific and exact: the provider must
claim the same owner and the complete `(identity, kind)` set derived from that
source declaration. A partial, wrong-owner, or wrong-kind claim is rejected.
Unknown, qualified-untrusted, or suspiciously aliased custom attributes and
derives on reachable declarations, modules, re-exports, traits, and impls fail
closed. Qualified macro roots are checked for local-module, import, and extern
alias shadowing. Unsupported producer templates remain pending consumer-build
work rather than accepting a fabricated family claim.

Rustdoc JSON is useful as an independent audit oracle, but it is not the build
gate because its JSON format is not a stable Rust interface.

## Migration ratchet

The checked-in surface manifest is scope-based. Each canonical facade module
or generator family is either `pending` or `enforced`, with its tier, aliases,
and feature availability recorded. It does not list uncovered items.

Promoted scope ids are also appended to `enforced_scope_baseline`. Validation
requires that every recorded id remain enforced and that every enforced scope
be recorded. This per-scope ledger prevents an enforced-to-pending regression
from being concealed by promoting a different scope while leaving the
aggregate pending-scope and pending-item ceilings unchanged.

For an enforced scope, every reachable item must have a valid contract and an
uncontracted addition fails the normal build. Pending scopes remain visible in
reports and exported coverage metadata. Migration lowers the pending baseline;
the ratchet rejects a later increase. Completion of #327 requires zero pending
supported scopes.

The executable `reachable-enforced-missing` fixture exercises the same
provider-backed comparison for migrated boundaries: an undocumented predicate
method, public field, enum variant, execute-branch builder method, or condition
method causes an ordinary `cargo check` to fail. The normal Sand build enforces
the 119-item `predicate-source` scope, the four-item generated `PredicateId`
partition, the 22-item `execute-when-source` scope, the 12-item opaque
`condition-source` scope, the one-item `resource-ref-source` module, and the
22-item generated resource-ID partition. Parametric `consumer_build` boundaries
additionally require their named provider-audit connection.

`#[api]` defaults to the facade's hidden registration transport. Definitions
in lower implementation crates use `registry = ::sand_api_contract`, which
keeps the dependency graph acyclic while feeding the same runtime inventory
and build-time facade identity audit. A two-crate compile/runtime fixture
verifies the lower definition, inherent method, facade alias, and installed
catalog entry together.

The predicate tranche places handwritten contracts on the underlying
`sand-components` definitions. Consequently the attribute derives signatures
and parameter names, emits Rustdoc, and registers installed metadata from the
same declaration. `PredicateId` follows the equivalent generator rule: its
semantic `registry_id!` input emits both the wrapper API and four deterministic
contract entries. The same declaration model now contracts the
resource-reference family: existing registry IDs are canonically grouped under
`sand::resource_ref`, while `DialogId` adds its explicit local-namespace
capability from that generator input. The facade forwarding table no longer
owns those signatures.

## Contract and catalog layers

The implementation separates:

1. the versioned serializable contract and surface models;
2. `#[api]` parsing, validation, and Rustdoc generation;
3. reachable-surface extraction and generator providers;
4. deterministic catalog construction and validation; and
5. local CLI query and rendering behavior.

Contract prose is intentional. Kind, signature, parameter names, cfg state,
canonical paths, and aliases may be derived when the tool has authoritative
information. Summary, context, Minecraft behavior, use/avoid guidance,
parameter meaning, return meaning, and examples may not be filled with generic
identifier-derived prose.

Generated command and registry surfaces are versioned profiles, not one loose
aggregate allowance. Their authoritative provider catalogs embed the resolved
Minecraft version. The facade build requires every provider to agree, selects
that exact version from `sand/api-surface-profiles.toml`, and byte-compares the
measured report with the profile's baseline. A new Minecraft codegen target is
therefore unsupported until its exact report has been audited and registered;
an unknown version or a directory containing mixed-version providers fails
closed during ordinary compilation.

The explicitly opted-in codegen-failure path emits empty, marked provider
catalogs from the same helper that emits its placeholder Rust. Those catalogs
select a separate exact 5,510-item source-only profile; they cannot be mixed
with real catalogs or contain declarations. The facade's source and checked-in
generator scopes remain fully audited in this recovery mode, while unavailable
vanilla re-exports are cfg-disabled. This is a compile-only recovery mode:
`sand-core` test targets reference generated command symbols and are unavailable
until real codegen succeeds. Provider tests validate the empty marker and exact
source parity instead.

## Prototype audit

The original eight-commit prototype was retained as a starting point and
classified before further implementation:

| Prototype component | Classification | Disposition |
| --- | --- | --- |
| `sand-api-contract` data model, sorting, search, and JSON | Reusable after architectural changes | Keep deterministic primitives; add surface coverage, missing API kinds, provenance, structured availability, and build-time identity validation. Link-time inventory is a transport, not proof of completeness. |
| `sand-api-enforce` missing-attribute fixture | Experimental but useful as a test fixture | Keep the normal-`cargo check` negative-test pattern. |
| `sand-api-enforce` single-file syntax scanner | Reusable after architectural changes | Replace its root-only view with reachable source/re-export analysis and generator providers. |
| `sand-macros/src/api_contract.rs` parser and Rustdoc renderer | Reusable after architectural changes | Keep strict key/parameter validation and prose-driven Rustdoc; support every contractable item kind and remove reliance on manually typed canonical relationships. |
| macro duplicate-identity compile fixture | Experimental but useful as a test fixture | The generated const catches same-module collisions only. Global duplicate identities must be rejected by the central build audit. |
| `sand-cli/src/api_cmd.rs` renderers, lexical search, suggestions, module grouping, and byte-stable export tests | Reusable after architectural changes | Keep query behavior; feed it the installed generated catalog and expose partial-coverage state until migration reaches zero pending scopes. |
| predicate contract prose and `sand::predicate` facade | Reusable after architectural changes | Preserve useful domain prose, but move contracts to the authoritative underlying identities and complete the whole predicate scope before enforcing it. |
| `sand/src/api_contracts.rs` forwarded registrations | Experimental but useful as migration material | Reuse prose only. Handwritten signatures and aliases cannot remain authoritative. |
| `sand/api-surface.txt` | Invalid because it assumes incomplete public-surface discovery | Remove. Tokenized `pub use` lines neither enumerate reachable children nor define intentional migration scopes. |
| prototype `sand/build.rs` hook | Invalid because it assumes incomplete public-surface discovery | Replace. Scanning only `lib.rs` and `prelude.rs` misses re-exported members, traits, fields, variants, cfg APIs, and generated APIs. |
| installed-inventory registration test | Experimental but useful as a test fixture | Keep as transport coverage only; do not infer full supported-surface coverage from the number of linked registrations. |
| prototype “implemented for #327” documentation claim | Should be removed | Replaced by this phased status and explicit completion criteria. |

At the commit level, `54ed04e` supplied the reusable negative-test harness,
`8315df2`, `75fbfc6`, and `827c259` supplied reusable CLI/catalog query work,
and `129206c` supplied the reusable schema/parser core. `e91e4de` and
`c416770` remain focused fixtures. `9f142b5` contains useful predicate prose
but its surface allowlist and complete-enforcement claim are invalid.

## Completion criteria

The installed catalog becomes authoritative only when every supported scope is
enforced, every generated family has a provider, all aliases resolve to one
canonical identity, and the pending-scope count is zero. Until then, foundation
and migration pull requests reference #327 without closing it.
