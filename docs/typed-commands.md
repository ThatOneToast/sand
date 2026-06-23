# Typed Commands

Sand command builders live under `sand_core::cmd` and `sand_commands`.

```rust
use sand_core::prelude::*;

#[function]
pub fn reward_player() {
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

```rust
let item = CustomItem::new("minecraft:diamond_sword")
    .custom_data("example_inferno_blade");

cmd::give(Selector::self_(), item);
```

Use `RawComponent` only as an explicit escape hatch for modded or future item
components that Sand does not model yet.

For typed positions use `tp_vec3`, `tp_with_rotation`, `summon_at`, and `summon_at_with_nbt`. `DamageAmount::hearts`, `.points`, and `.fixed` are fixed values and never use the removed panic-prone score/event variants. See the full [typed command guide](../book/src/typed-commands.md).

<div class="sand-warning"><strong>Experimental/version-sensitive.</strong> Typed APIs lower to vanilla command syntax. A raw command remains appropriate for unsupported snapshot, modded, or future syntax; keep it localized and label why it is raw.</div>
