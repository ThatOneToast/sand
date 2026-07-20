# 18. Extending Trailforge

The previous seventeen chapters covered every system Trailforge's core
gameplay loop uses. This closing chapter looks at the one system left —
an *optional*, feature-gated one — and at how to add your own systems
following the same shape.

## Optional systems: `systems-damage`

Trailforge's `Cargo.toml` opts into one optional system:

```toml
sand = { path = "../../sand", features = ["systems-damage"] }
```

`systems-damage` is a feature-gated `DamageTracker` — cumulative-stat-based
damage tracking, used in both `load` and `tick`:

```rust,ignore
DamageTracker::define();       // in load
DamageTracker::tick_players(); // in tick
```

```rust,ignore
TypedExecute::as_players()
    .when(DamageTracker::hurt_within("@s", Ticks::seconds(3)))
    .run(Actionbar::show(
        Selector::self_(),
        Text::new("Catch your breath...").red(),
    ));
```

Like every state primitive in chapter 7, `DamageTracker` needs a `.define()`
in `load` and, because it derives "was recently hurt" from tracked deltas
rather than an instantaneous event, a per-tick update
(`DamageTracker::tick_players()`) to keep its cumulative stats current.
`hurt_within(selector, duration)` is then a plain condition, usable in a
`.when(...)` exactly like any `Flag` or `ScoreVar` comparison. This is
called out explicitly as an approximation, not an event payload — vanilla
has no "player took N damage from X" event Sand can subscribe to directly;
`DamageTracker` polls the `minecraft.damage_taken`-style cumulative
scoreboard criteria and infers "recently hurt" from the delta. See
[Vanilla Limitations](reference/vanilla-limitations.md).

Optional systems like this exist specifically so packs that don't need
damage tracking don't pay for it — no feature flag, no compiled code, no
exported components. Enable only what your pack actually uses.

## Adding your own function, item, or event

Every system in this book followed the same three-step shape when it was
introduced:

1. **Declare state** (chapter 7) if the new feature needs to remember
   anything across ticks or reloads — pick `ScoreVar`, `Flag`, `Cooldown`,
   `Timer`, or `StorageVar` by asking what *kind* of fact it is (a
   quantity, a boolean, a rate-limited action, a repeating pulse, or
   configuration/structured data).
2. **Define it in `load`**, idempotently (chapter 3).
3. **Wire the behavior** as a `#[function]` (imperative, callable), a
   `#[component(Tick)]` guard (continuous, re-evaluated every tick), or a
   `#[event]` handler (reactive, fires once per occurrence) — chapters 8,
   3, and 9 respectively cover when each is the right choice.

If the new feature needs a custom item, decide whether it needs a
generated predicate (`#[item]`, chapter 5) or is only ever granted, never
matched against (a plain builder function). If it needs to be craftable,
add a `ShapedRecipe` (chapter 6) referencing the item by its base ID. If it
needs passive, always-on-while-equipped behavior, reach for an
`AttributeModifier` (chapter 11) before writing procedural tick logic. If
it needs to *feel* like something happened, package a `Vfx` sequence
(chapter 13) rather than inlining particle/sound commands at every call
site.

## The escape hatch: `sand::advanced`

Every builder this book covered has a typed, validated surface. Real packs
occasionally need to emit something Sand doesn't yet model as a typed
builder — raw JSON for a component Sand hasn't wrapped yet, or a
hand-written command string. `sand::advanced` (the same module
`__sand_export` uses for the export entry point, chapter 17) documents
these deliberate, supported low-level hooks. Reach for `cmd::raw(...)`
(already used once in this book, in `trail:claim_striders`'s `give`
command) for one-off raw command strings, and consult `sand::advanced`'s
own documentation before reaching further — it exists precisely so escape
hatches are a documented, intentional choice rather than an undocumented
workaround.

## Where to go next

You've now seen every system `examples/book_project` demonstrates. The
[Vanilla Limitations](reference/vanilla-limitations.md) page collects, in
one place, every gameplay signal this book noted vanilla Minecraft simply
doesn't expose — read it before assuming a missing feature is a Sand gap
rather than an engine constraint. From here, the best next step is
building your own pack the same way this book built Trailforge: start with
`sand new`, add one system at a time, and let `cargo test` catch mistakes
before `sand build` ever runs.
