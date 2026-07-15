# Migration Notes

Use `ItemSlot`/`Slot` in new code; `InventorySlot` and `SlotPattern` are deprecated compatibility APIs. Use `DamageAmount::hearts`, `points`, or `fixed`; removed score/event amount variants were not valid command generation paths. Replace raw normal effects, teleport, and summon commands with their typed builders where possible.

Selector narrowing is now fallible. Convert an unrestricted `Selector` with
`SingleEntity::try_from`, `SinglePlayer::try_from`, `EntityTargets::try_from`,
or `PlayerTargets::try_from`; only player-to-entity widening remains
infallible. `EntityTargets::limit` and `PlayerTargets::limit` accept only `1`
and return `Result`, because any other value cannot establish single-target
arity.

For scoreboard operations, use `ScoreHolder` and `ObjectiveName`. The source
holder must resolve to one score holder; lower collective work through
`execute as <targets>` and use `@s` as both operation target and source.
`DamageTracker::tick` therefore takes `SingleEntity`; use `tick_players()` for
all online players or the explicitly unchecked `tick_raw` only for syntax the
typed target model cannot represent.

Use `cmd::raw(...)`, `Selector::raw(...)`, or `ItemSlot::raw(...)` only for
syntax the typed model does not cover. Raw command lines bypass typed grammar
construction, but still must be safe single `.mcfunction` lines without a
leading slash. The export boundary performs additional checks only for
confidently recognized top-level commands and exact argument positions; it
preserves unknown, macro, and modded syntax and command-shaped literal
JSON/SNBT text.
