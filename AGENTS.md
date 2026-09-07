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

## API and framework changes

Sand has no stable public release and its API is volatile. Replace or delete
obsolete endpoints instead of preserving them through wrappers, aliases, or
deprecated duplicates.

Public-facing endpoints need useful Rustdoc explaining the abstraction, its
behavior, and important surrounding APIs without requiring authors to inspect
the implementation.

Compiler and export validation should prove Sand's observable Minecraft
commands, JSON, resources, runtime behavior, and framework invariants rather
than implementation details.

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
