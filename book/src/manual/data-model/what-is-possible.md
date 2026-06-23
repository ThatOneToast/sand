# What Vanilla Can Reliably Do

| Goal | Best Sand API | Vanilla mechanism | Reliable? | Notes |
|---|---|---|---|---|
| Per-player number | `ScoreVar` | scoreboard | Yes | normal datapack state |
| Per-player boolean | `Flag` | scoreboard 0/1 | Yes | use typed conditions |
| Cooldown | `Cooldown` | scoreboard timer | Yes | tick it consistently |
| Global config | `StorageSchema` | `data storage` NBT | Yes | global, static paths |
| Custom item identity | `CustomItemId` | `custom_data` component | Yes | stable marker key |
| Detect item use | item/advancement trigger | advancement JSON | Usually | only exposed vanilla actions |
| Exact shield block | none | no direct trigger payload | No | use closest held/use/damage pattern |
| Push entities | `PushAway` | local teleport | Mostly | displacement, not velocity |
| True directional velocity | raw score/NBT design | physics/NBT math | Limited | no simple general vanilla abstraction |
| Inventory check | `InventorySystem` | `execute if items` | Yes | scope selectors carefully |
| Interactable entity | `Interactable` | interaction entity + advancement | Yes | right-click only |
| Store location | marker/scores/storage | several mechanisms | Yes | choose representation |
| Dynamic per-player NBT | manual scheme | runtime UUID/name keying | Limited | not static Sand fields |
| Custom GUI | `Dialog` where supported | version-gated dialog JSON | Version-sensitive | not arbitrary mod UI |
| Block-attached data | NBT/data commands | block entity NBT | Limited | only block entities have NBT |
| Entity-attached data | tags/scores/data | entity runtime data | Mostly | avoid unsafe player-NBT mutation |

<div class="sand-generated"><strong>Rule of thumb.</strong> Sand gives typed authoring around vanilla mechanisms. If vanilla has no stable representation or event, Sand documents the closest honest pattern instead of inventing one.</div>
