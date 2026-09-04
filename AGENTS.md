# AGENTS.md

## What Sand is

Sand is a Rust framework for building vanilla Minecraft Java datapacks.

Its goal is to make datapack development easier by providing typed, reusable
abstractions over Minecraft commands and datapack resources. Authors should be
able to think in terms of gameplay concepts, state, entities, events, resources,
and systems rather than manually assembling commands.

Minecraft commands are the compilation target, not the ideal authoring API.

## Design direction

Design for the framework, not for one command or one call site.

Before adding a type or abstraction, look for the canonical model already used
by Sand and extend it when possible. Avoid parallel representations of the same
concept.

Keep public types small and composable. If one type can safely represent several
closely related concepts, prefer one type over several wrappers. When a real
subtype distinction is needed, type parameters such as `Thing<T>` are generally
preferable to duplicating `PlayerThing`, `EntityThing`, `LivingThing`, and
similar APIs.

Prefer, in order:

```text
gameplay abstraction
→ typed Sand model
→ typed Minecraft command/resource
→ explicit raw escape hatch
```

Raw commands, JSON, SNBT, identifiers, and other escape hatches are for
unsupported edges and interoperability, not the default implementation path.

## Public API and documentation

The supported author-facing API is the `sand` crate. Normal examples should
use:

```rust
use sand::prelude::*;
```

Implementation crates are not user-facing APIs merely because an item is
`pub`.

Use the API boundary to understand and document supported behavior:

```sh
sand api search <terms>
sand api show <path>
sand api module <module>
```

When changing public API, keep its API contract and Rustdoc accurate. Document
what the abstraction means to a datapack author rather than exposing compiler
implementation details.

Do not bypass API-contract enforcement to make a build pass.

## Implementation

Preserve deterministic generated output, collision safety, export isolation,
and multiplayer safety where runtime state is involved.

Do not hand-edit generated code when a generator or schema owns it.

Sand is pre-1.0. Prefer consolidating around the best API instead of retaining
obsolete alternatives through compatibility wrappers.

## Validation

Add focused tests for behavior you change. Bug fixes should include a regression
test.

Before considering work complete, run:

```sh
scripts/check.sh
git diff --check
```

Do not weaken tests, API enforcement, or architecture guards just to make CI
pass.
