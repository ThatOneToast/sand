# 18. Extending Trailforge

The previous seventeen chapters covered Trailforge's core gameplay loop. This
closing chapter shows how to add your own behavior using Sand's canonical
State, ECS/archetype, event, and `#[system]` architecture.

## Adding your own function, item, or event

Every system in this book followed the same three-step shape when it was
introduced:

1. **Declare state** (chapter 7) with `#[derive(State)]` if the new feature
   needs to remember anything across ticks or reloads. Choose a field marker
   such as `Score`, `Flag`, `Cooldown`, or `Data<T>` inside that schema.
2. **Let the derived lifecycle provision it**; application code does not
   manually register objectives.
3. **Wire the behavior** as a `#[function]` (imperative, callable), a
   `#[datapack_component(Tick)]` guard (continuous, re-evaluated every tick), or a
   `#[on_event]` handler (reactive, fires once per occurrence) — chapters 8,
   3, and 9 respectively cover when each is the right choice.

If the new feature needs a custom item, decide whether it needs a
generated predicate (`#[custom_item]`, chapter 5) or is only ever granted, never
matched against (a plain builder function). If it needs to be craftable,
add a `ShapedRecipe` (chapter 6) referencing the item by its base ID. If it
needs passive, always-on-while-equipped behavior, reach for an
`AttributeModifier` (chapter 11) before writing procedural tick logic. If
it needs to *feel* like something happened, package a `Vfx` sequence
(chapter 13) rather than inlining particle/sound commands at every call
site.

## Custom export integration: `sand::advanced`

Every builder this book covered has a typed, validated surface. Most packs
should use `sand build` and never call `sand::advanced`. The module exists
only for a custom build integration that must collect Sand's registered
components as version-validated JSON; chapter 17 shows the generated
`__sand_export` hook that calls it. Reach for `cmd::raw(...)` (already used
once in this book, in `trail:claim_striders`'s `give` command) for a
deliberate one-off raw command string. Raw JSON/SNBT values keep their
canonical prelude paths rather than becoming an implicit compiler API.

## Where to go next

You've now seen every system `examples/book_project` demonstrates. The
[Vanilla Limitations](reference/vanilla-limitations.md) page collects, in
one place, every gameplay signal this book noted vanilla Minecraft simply
doesn't expose — read it before assuming a missing feature is a Sand gap
rather than an engine constraint. From here, the best next step is
building your own pack the same way this book built Trailforge: start with
`sand new`, add one system at a time, and let `cargo test` catch mistakes
before `sand build` ever runs.
