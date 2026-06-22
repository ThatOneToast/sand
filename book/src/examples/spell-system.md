# Spell System Example

The spell-system flow uses typed state, cooldowns, conditions, execute, text,
storage, and one explicit interop escape hatch.

Core loop:

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
static SETTINGS: StorageVar<i32> = StorageVar::new("spells:data", "settings.mana");

#[component(Load)]
pub fn load_spells() {
    MANA.define();
    DASH.define();
    MANA.set(Selector::all_players(), 100);
    SETTINGS.set_int(100);
}

#[component(Tick)]
pub fn tick_spells() {
    DASH.tick(Selector::all_players());
    TypedExecute::as_players()
        .when(all![
            MANA.of("@s").gte(25),
            any![DASH.ready("@s"), SETTINGS.exists()],
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua().bold(true),
        ));
}

#[function("spells:cast_dash")]
pub fn cast_dash() {
    MANA.remove(Selector::self_(), 25);
    DASH.start(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Dash cast").green());
}
```

Add a separate interop function when another datapack owns part of the spell
pipeline:

```rust
#[function]
pub fn after_dash_interop() {
    cmd::raw("function other_pack:api/after_dash");
}
```
