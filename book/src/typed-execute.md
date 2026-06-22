# Typed Execute

Use `TypedExecute` to turn typed conditions into complete commands.

```rust
#[function]
pub fn show_ready() {
    TypedExecute::as_players_at_self()
        .when(all![MANA.of("@s").gte(25), DASH.ready("@s")])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua(),
        ));
}
```

When an `any!` condition expands to multiple execute plans, Sand emits one
command per plan.
