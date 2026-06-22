# Static Typing Repair Audit

This audit covers the event, selector, damage, command-lowering, and raw-command
surfaces that let Minecraft parser rules leak into Sand author code.

## Current Event API Limitations

- `Event<T>` is a zero-cost advancement context that exposes only
  `event.player()` / `event.subject()`, both as the untyped `Selector::self_()`.
- Damage-capable advancement triggers such as `EntityDamagePlayerEvent` and
  `PlayerDamageEntityEvent` are represented as ordinary `AdvancementEvent`
  markers. The handler type does not expose damage-specific operations.
- The event context does not model damage source, damage amount, or damage type.
- `#[event]` validates `Event<T>` against `T: AdvancementEvent`, but it has no
  capability-specific context like `DamageEvent<T>`.
- Vanilla advancement reward functions do not provide exact damage amount to the
  reward function. Sand must not imply exact reflected damage unless it adds a
  real tracker.

## Current Damage API Limitations

- `sand_commands::damage(target: Selector, amount, damage_type)` is a direct
  string builder for `/damage <target> ...`.
- The builder accepts any `Selector`, including `Selector::all_entities()`, even
  though vanilla `/damage` accepts exactly one entity target.
- There is no high-level damage builder that can accept many targets and lower
  safely through `execute as <targets> run damage @s ...`.
- Source attribution is not modeled. Authors must hand-write vanilla syntax and
  understand how executor changes affect `@s`.
- No typed amount model exists. Fixed amounts, score-backed amounts, and "same
  as event" are not distinguished.

## Current Selector Typing Limitations

- `Selector` is a single untyped value regardless of whether it denotes one
  entity, many entities, one player, or many players.
- `Selector::self_()`, `Selector::all_entities()`, and
  `Selector::all_players()` all have the same Rust type.
- `.limit(1)` and nearest-player selection do not communicate single-target
  semantics to command builders.
- Player-only and entity-only command targets are not separated in the type
  system.

## Current Raw-Command Escape Hatches

- `cmd::raw(...)` remains the explicit command escape hatch.
- `DialogAction::run_command(...)` remains the dialog JSON escape hatch.
- `Text::click_run_command(...)` is intentionally raw Minecraft UI command
  syntax.
- Examples and docs still contain raw `execute as`, raw `damage @s`, raw
  `function ...`, and raw advancement revoke patterns where typed builders are
  missing or were added after the example was written.

## Current Generated-Code Duplication

- `execute_when` registers anonymous branch functions under `sand/branches/N`.
- Anonymous `run_fn!` blocks register runtime functions under generated paths.
- Dynamic functions are appended to an in-memory registry and exported as-is; no
  body-based deduplication is currently applied.
- Branch counters are deterministic only when tests explicitly reset them.

## Parser Rules Leaked To Users

- `/damage` single-target arity: users can write `damage @e[...]` through the
  typed function and only discover the error when Minecraft parses the pack.
- Multi-target damage loop lowering: users currently need to know to write
  `execute as @e[...] run damage @s ...`.
- Source attribution under `execute as`: users need to reason about when `@s`
  changes.
- Selector arity for `attribute`, `tp` destinations, dialog player targets, and
  other command families is not expressed centrally.
- Local function sentinel resolution is internal, but raw `function ns:path`
  examples still teach string-level function references in some docs.

## Desired User-Facing APIs

Primary damage-event dogfood target:

```rust
#[event]
pub fn on_damaged_damage_nearby(event: DamageEvent<EnhancedCellsDamagedEvent>) {
    event
        .reflect_damage()
        .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
        .amount(DamageAmount::fixed(4.0))
        .damage_type(DamageType::Generic)
        .run();
}
```

General damage builder target:

```rust
Damage::new()
    .to(EntityTargets::nearby(5.0).excluding_players())
    .amount(DamageAmount::fixed(4.0))
    .damage_type(DamageType::Generic)
    .source(SingleEntity::self_())
    .run();
```

Lowering goal for many targets:

```mcfunction
execute as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic
```

The author-facing API should make invalid vanilla command shapes unrepresentable
or lower them automatically before Minecraft sees the generated datapack.
