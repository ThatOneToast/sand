# Authoring Model

The primary path is attribute-first:

- `#[function]` creates a named `.mcfunction`.
- `#[component(Load)]` registers a function in `minecraft:load`.
- `#[component(Tick)]` registers a function in `minecraft:tick`.
- Typed state, commands, conditions, dialogs, storage, and resources are plain
  Rust expressions in the function body.

The advanced path is `mcfunction!`. Use it for command grouping, migration,
generated command fragments, and explicit raw interop. Raw commands should be
wrapped in `cmd::raw(...)` so they are visible in code review.
