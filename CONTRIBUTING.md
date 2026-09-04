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
