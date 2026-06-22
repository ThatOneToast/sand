# Typed Commands

Command builders return command-like values that attribute functions collect.

```rust
#[function]
pub fn reward() {
    cmd::tellraw(Selector::self_(), Text::new("Quest complete").green());
    cmd::give(Selector::self_(), "minecraft:diamond");
    cmd::tag_add(Selector::self_(), "quest_complete");
}
```

Prefer typed selectors, text, resources, items, and builders where Sand exposes
them. Use `cmd::raw(...)` only when the command is intentionally outside typed
coverage.
