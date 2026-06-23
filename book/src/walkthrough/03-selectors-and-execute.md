# 3. Selectors And Execute

## What you will build

Create a nearby-mob scan that explains target (`as`) versus position (`at`) context.

## Concepts introduced

`Selector::self_`, `all_players`, `EntityTargets::nearby`, `TypedExecute`, `as`, `at`, and multiplayer scope.

## File changes

```rust
#[function("arcane:nearby_demo")]
pub fn nearby_demo() {
    let nearby_mobs = EntityTargets::nearby(6.0).excluding_players();
    TypedExecute::as_players_at_self().run(cmd::tellraw(
        Selector::self_(), Text::new("Scanning around each player").aqua(),
    ));
    PushAway::new().source(Selector::self_()).targets(nearby_mobs).strength(0.25).build();
}
```

## How it works

`as` changes who `@s` is; `at` changes where relative/local coordinates are evaluated. Source position matters for movement: `PushAway` faces its declared source. Broad selectors are a multiplayer hazard—exclude players, constrain distance, and avoid assuming a tag matches exactly one entity.

## What Sand generates

`TypedExecute::as_players_at_self()` lowers to an `execute as @a at @s run ...` chain. The push uses each target's execution context and faces the source.

## Try it in Minecraft

Spawn a mob nearby, `/reload`, then run `/function arcane:nearby_demo` as one player. Test with two players before relying on `@a` logic.

## Common mistakes

- Using `at` when only target identity should change.
- Using `as` then expecting `@s` to remain the original player.
- Targeting all entities without a type/distance bound.

## Deeper reading

[Selectors](../manual/selectors.md), [Execute](../manual/execute.md), and [Movement](../manual/movement.md).
