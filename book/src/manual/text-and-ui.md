# Text And UI

Use `Text`, `Actionbar`, `Title`, `Bossbar`, sound, and particle builders for structured Minecraft output.

```rust
cmd::tellraw(Selector::self_(), Text::new("Quest complete").gold().bold(true));
Actionbar::show(Selector::self_(), Text::new("Mana +10").aqua());
```

These builders serialize Minecraft's text JSON and command syntax. Use `RawJson` only for a component field not modeled yet.
