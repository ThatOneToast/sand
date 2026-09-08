# 14. Timers And Cooldowns

`Timer` and `Cooldown` are field types inside a derived State schema. They do
not form a second, standalone state system.

```rust,ignore
#[derive(State)]
#[state(namespace = "trail", scope = player)]
struct Traversal {
    #[state(auto_tick)]
    grapple: Cooldown,
    #[state(auto_tick)]
    regen: Timer,
}
```

The `auto_tick` attribute asks State's generated lifecycle to decrement a
positive field once per tick. Authors do not define objectives or maintain a
separate timer loop.

## `Cooldown`: gate an action, then start it

Bind the State to the current player, test readiness, and start the cooldown at
the moment the action succeeds:

```rust,ignore
let traversal = Traversal::on(EntityContext::<PlayerKind>::default());

TypedExecute::as_players_at_self()
    .when(traversal.grapple.ready())
    .run(cmd::function(grapple_execute));

// Inside `grapple_execute`:
traversal.grapple.start(Ticks::seconds(4));
```

A cooldown is ready when its remaining score reaches zero. To clear one during
a reset, start it with `Ticks::new(0)`.

## `Timer`: restart continuously, drive system pacing

A timer exposes `elapsed()` and is restarted with an explicit duration:

```rust,ignore
let traversal = Traversal::on(EntityContext::<PlayerKind>::default());

traversal.stamina.add(10).into_iter().flat_map(|command| {
    TypedExecute::as_players()
        .when(all![
            traversal.regen.elapsed(),
            traversal.stamina.matches(..100).unwrap(),
        ])
        .run(command)
}).collect::<Vec<_>>();

traversal.regen.start(Ticks::seconds(2)).into_iter().flat_map(|command| {
    TypedExecute::as_players()
        .when(traversal.regen.elapsed())
        .run(command)
}).collect::<Vec<_>>();
```

The first statement acts on expiry only while stamina is below its cap. The
second restarts the timer whenever it expires, independently of whether the
first action ran.

Use `Cooldown` for an interval consumed by an action. Use `Timer` for a
repeating interval that drives ongoing behavior. Both remain ordinary fields
of the owning State.
