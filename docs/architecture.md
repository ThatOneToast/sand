# Architecture

Sand is split into focused crates:

- `sand`: CLI
- `sand-core`: framework APIs, state, conditions, version model, component export
- `sand-commands`: typed Minecraft command builders
- `sand-components`: typed datapack JSON builders
- `sand-macros`: proc macros
- `sand-build`: Minecraft data generation and codegen
- `sand-resourcepack`: optional resource-pack and HUD helpers
- `sand-example`: integration coverage

Build flow:

1. `sand-build` resolves Minecraft data and generates Rust types.
2. `sand-core` and `sand-commands` expose typed APIs over those generated types.
3. `sand-macros` registers functions and components.
4. `sand build` writes datapack/resource-pack output.
