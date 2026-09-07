# Editor / LSP integration — maintainer notes

Sand does not currently ship a language server or editor extension. A future
integration can discover projects through adjacent `sand.toml` and
`Cargo.toml` files and reuse these existing boundaries:

- `cargo check` provides rustc and procedural-macro diagnostics;
- `sand context --format json` reports resolved project identity;
- `sand api ... --format json` exposes the enforced API contract catalog;
- `sand check --agent --format json` validates configuration and generated
  datapack state;
- `sand build --format json` exposes build diagnostics and output records.

The configuration schema contains a required `[pack]` table with namespace,
description, and `mc_version`, plus optional datapack format metadata. Sand
targets Minecraft Java 26.x and newer; editor validation should reject earlier
versions and avoid claiming unknown future schemas are verified.

Rust-analyzer already covers syntax, type checking, and macro expansion. Any
Sand-specific LSP should focus on structured export diagnostics and generated
Minecraft command/JSON validation rather than duplicating Rust tooling.
