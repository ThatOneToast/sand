# Text And UI

Use `Text`, `Actionbar`, and `Title` instead of handwritten JSON.

```rust
#[function]
pub fn notify() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("Quest complete").green().bold(true),
    );

    Title::of(Selector::self_())
        .title(Text::new("Level Up").gold())
        .subtitle(Text::new("+1 skill point").aqua())
        .build();
}
```
