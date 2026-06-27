# API Cheat Sheet

Start normal pack code with `use sand_core::prelude::*;` plus the proc macros
you need from `sand_macros`. Use [`advanced`](api-tiers.md) only for lower-level
export hooks or custom integration work.

| Need | Start with |
|---|---|
| Callable commands | `#[function]` |
| Load/tick setup | `#[component(Load)]`, `#[component(Tick)]` |
| Score state | `ScoreVar`, `Flag`, `Cooldown` |
| JSON data | `#[component]` |
| Item identity | `CustomItem` + `CustomItemId` |
| Gameplay trigger | `AdvancementEvent` + `Event<T>` |
| Unsupported syntax | `cmd::raw`, `RawJson`, `RawSnbt` |

Start in the matching [Manual](../manual/functions.md) page for examples.
