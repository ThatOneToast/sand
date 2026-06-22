# Typed Commands

Sand command builders live under `sand_core::cmd` and `sand_commands`.

```rust
use sand_core::prelude::*;

mcfunction! {
    cmd::tellraw(Selector::all_players(), Text::new("Quest complete").green());
    cmd::give(Selector::self_(), "minecraft:diamond");
    cmd::tag_add(Selector::self_(), "quest_complete");
}
```

For HUD output, use typed text builders:

```rust
Title::of(Selector::self_())
    .title(Text::new("Level Up").gold())
    .subtitle(Text::new("+1 skill point").aqua())
    .build();
```

For item-heavy datapacks, prefer `CustomItem` and item predicate builders over
manually assembled component strings.
