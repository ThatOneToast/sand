# Escape Hatches

Sand's default path is typed APIs. Escape hatches exist for cases where typed
builders do not cover the required syntax.

## When to use an escape hatch

- Another datapack's documented public API (external function call by string)
- Modded commands not in Sand's command tree
- Snapshot syntax not yet modeled by Sand
- Future Minecraft features not yet in Sand
- Focused debugging of generated output

## `cmd::raw(...)`

Prefer `cmd::raw(...)` over bare string literals so the intent is explicit:

```rust
#[function]
pub fn interop() {
    // Explicit escape hatch — calls an external pack's public function.
    cmd::raw("function other_pack:api/run_quest");
}
```

`cmd::raw(...)` accepts any `impl Into<String>` and returns a `String` that is
included verbatim in the generated `.mcfunction` output. There is no validation.

For local function calls, use `cmd::call(local_fn)` instead — it resolves the
resource location from the `#[function]` registry without string literals.

## `FunctionRef::external(...)`

Use `FunctionRef::external(...)` for typed references to functions in other
datapacks:

```rust
use sand_core::resource_ref::FunctionRef;

#[function]
pub fn call_external() {
    cmd::call(FunctionRef::external("other_pack:api/do_thing").unwrap());
}
```

## `DialogAction::run_command(...)`

Prefer `DialogAction::run_function(local_fn)` for datapack functions in dialog
buttons. Use `run_command` only when the action is not a function call:

```rust
// Preferred: typed function ref
DialogButton::new(Text::new("Start").green())
    .action(DialogAction::run_function(start_function))

// Escape hatch: raw command for non-function actions
DialogButton::new(Text::new("Suggest"))
    .action(DialogAction::run_command("/trigger quest_select set 1"))
```

`run_command` accepts any raw command string verbatim, including `/say` and
other non-function commands that have no typed builder.

## Summary

| Escape hatch | Use when |
|---|---|
| `cmd::raw("...")` | Raw command string for commands with no typed builder |
| `FunctionRef::external("ns:path")` | Typed reference to another datapack's function |
| `DialogAction::run_command("...")` | Non-function dialog button action |
| `#[function("explicit:path")]` | Override the auto-derived resource location |
