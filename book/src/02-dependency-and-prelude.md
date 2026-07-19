# 2. The Sand Dependency And Prelude

## One dependency

Trailforge's `Cargo.toml` depends on exactly one Sand crate:

```toml
[dependencies]
sand = { path = "../../sand", features = ["systems-damage"] }

[build-dependencies]
sand-build = { path = "../../sand-build" }
```

(A project scaffolded with `sand new` depends on `sand` via a git dependency
instead of a path, since Sand has no crates.io release yet; pass
`--path-deps` when developing against a local Sand checkout.)

`sand` is a **façade crate**: everything a datapack author needs — attribute
macros, typed commands, state, events, components, conditions, text,
version handling — is re-exported from this one crate. Internally, `sand`
is built from several implementation crates (`sand-core`, `sand-commands`,
`sand-components`, `sand-version`, `sand-macros`), but authoring code never
names them directly. This book only ever writes `use sand::...` or
`use sand::prelude::*`.

The `[build-dependencies]` entry on `sand-build` is what makes `build.rs`
able to download and cache the Minecraft data your pinned `mc_version`
needs for accurate codegen (recipe/loot table schemas, block and item ID
validation, and so on):

```rust,ignore
fn main() {
    let strict = std::env::var("SAND_STRICT_CODEGEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    if let Err(err) = sand_build::generate("26.2") {
        if strict {
            panic!("book_project codegen failed: {err}");
        }
        println!(
            "cargo:warning=book_project codegen skipped: {err}. \
             Continuing because SAND_STRICT_CODEGEN is not enabled."
        );
    }
}
```

In restricted environments (no network), codegen fails softly by default so
`cargo build` still works from cached data; set `SAND_STRICT_CODEGEN=1` in
CI to make a codegen failure hard-fail the build instead.

## The prelude

Every chapter's Rust snippet in this book starts the same way:

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:imports}}
```

`sand::prelude::*` is the default authoring import — it covers the common
vocabulary of ordinary datapack development: the attribute macros
(`#[component]`, `#[event]`, `#[function]`, `#[item]`, …), typed commands
and selectors, conditions, state (scores, flags, timers, cooldowns,
storage), entities, the event context types, components, dialogs, and text.

Trailforge's remaining imports reach past the prelude for less common,
explicitly-named surfaces:

- `sand::event::AdvancementEvent` and `sand::event::trigger::InventoryChangedTrigger`
  — building a custom event backed by an advancement trigger (chapter 9).
- `sand::event::vanilla::{FirstJoin, OnDeath}` — typed vanilla event markers.
- `sand::events::{PlayerSprintEvent, SandEvent, SandEventDispatch}` — the
  event *graph* surface used to declare and compose custom events (as
  opposed to `sand::event`, the typed handler-context model).

This is deliberate: the façade's top-level module list —
`sand::{event, item, state, command, component, entity, data, text,
version, vfx}` — is the *full* supported surface for less common needs, so
you never have to guess whether a type exists somewhere unexported. A
`sand::advanced` module exists too, holding the low-level export entry
points that `__sand_export` calls into (chapter 17); ordinary gameplay code
never touches it directly.

## What you will never write

Nowhere in this book — or in `examples/book_project`, which is guarded by an
architecture test that fails CI if it regresses — does authoring code import
`sand_core`, `sand_macros`, `sand_commands`, or `sand_components` directly.
Those are internal implementation crates. If you ever see a tutorial or
older example doing that, it predates the `sand` façade and should be
updated.
