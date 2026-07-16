# Sand

Sand is a strongly typed Rust framework for building vanilla Minecraft Java datapacks (and optional resource packs). It generates normal datapack files—functions, tags, advancements, predicates, recipes, loot, item modifiers, and components—rather than hiding Minecraft behind a runtime.

The full project guide lives in the [Sand mdBook](book/src/introduction.md). This README is the short orientation.

<div class="sand-warning"><strong>Experimental.</strong> Sand is evolving. Optional systems and Minecraft's command/data formats are version-sensitive; test generated output with your target Minecraft version.</div>

## Start a project

```sh
cargo run -p sand -- new my_pack
cd my_pack
cargo run -p sand -- build
```

Copy the generated datapack into a world's `datapacks/` directory and run `/reload`.

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[component(Load)]
pub fn load() { MANA.define(); }

#[function]
pub fn reward() {
    MANA.add(Selector::self_(), 10);
    cmd::tellraw(Selector::self_(), Text::new("+10 mana").aqua());
}
```

## What Sand covers

- Typed functions, load/tick components, selectors, conditions, execute chains, commands, text, state, storage schemas, and escape hatches.
- Typed custom items and advancement-backed events, including typed function references.
- Optional inventory, movement, entity/interactable, damage-tracking, lifecycle, cooldown, and player-data systems.
- Typed datapack components: advancements, predicates, recipes, loot, tags, item modifiers, dialogs, structure templates, resource-pack and HUD data.

Enable only the systems you need, for example `features = ["systems-inventory", "systems-movement"]`. Raw commands and raw JSON/SNBT are deliberate interop escape hatches, not the default authoring model.

Normal pack code should import `sand_core::prelude::*` plus the proc macros it
uses. Lower-level export hooks live under `sand_core::advanced`; compatibility
exports remain available for older code but are not the preferred starting
point.

Useful guide entry points: [getting started](book/src/getting-started.md), [custom items](book/src/manual/custom-items.md), [events](book/src/manual/events.md), [version capabilities](book/src/version-capabilities.md), and [full project tutorials](book/src/recipes/shockwave-shield.md).
