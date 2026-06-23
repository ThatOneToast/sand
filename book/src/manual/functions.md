# Functions

`#[function]` exports a callable `.mcfunction`. Use it for reusable gameplay steps, event rewards, and command entry points.

```rust
#[function("arcane:spell/cast")]
pub fn cast() { cmd::say("cast"); }
cmd::call(cast);
```

Without an explicit path Sand derives one from the Rust item. Generated output is `data/arcane/function/spell/cast.mcfunction`; calling it lowers to `function arcane:spell/cast`. Prefer short functions that do one job. A large branch can be expressed with `when(...).then_all(...)`; Sand generates a private helper function for it. Event handlers are also reward functions generated behind an advancement.

Common mistake: using a string path for a local function when `cmd::call(cast)` preserves a typed reference. Related: [Components](components.md), [Function References](function-refs.md), [Events](events.md).
