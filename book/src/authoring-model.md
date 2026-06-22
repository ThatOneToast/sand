# Authoring Model

The primary path is attribute-first:

- `#[function]` creates a named `.mcfunction`.
- `#[component(Load)]` registers a function in `minecraft:load`.
- `#[component(Tick)]` registers a function in `minecraft:tick`.
- `#[event]` wires a handler to a Minecraft advancement trigger. The handler
  parameter is `Event<T>` where `T: AdvancementEvent`. Use `event.player()`
  to access the triggering player.
- `#[component]` returning a `Dialog` registers a typed dialog component.
- Typed state, commands, conditions, dialogs, storage, and resources are plain
  Rust expressions in the function body.

## Calling functions

Use `cmd::call(local_fn)` to call a `#[function]` by Rust pointer — Sand
resolves the resource location at compile time without requiring a string:

```rust
#[function]
pub fn reward() { cmd::say("Reward!"); }

#[function]
pub fn on_event() {
    cmd::call(reward);   // preferred: typed pointer
}
```

For functions in other datapacks, use `FunctionRef::external("ns:path")`:

```rust
cmd::call(FunctionRef::external("other_pack:api/run").unwrap());
```

## Showing dialogs

Use `cmd::show_dialog(selector, DialogRef::local("path"))` to show a dialog
registered in this pack. For dialogs in other packs, use `DialogRef::external`.

## Escape hatches

The advanced path is `mcfunction!`. Use it for command grouping, migration,
generated command fragments, and explicit raw interop. Raw commands should be
wrapped in `cmd::raw(...)` so intent is visible in code review. See
[Escape Hatches](advanced/escape-hatches.md) for the full list.
