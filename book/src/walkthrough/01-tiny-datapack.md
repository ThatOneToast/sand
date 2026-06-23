# A Tiny Datapack

## What you will build

Create the first callable function in the cumulative **Arcane Powers** datapack. This establishes the `arcane` namespace used by every later chapter.

## Concepts introduced

`#[component(Load)]`, `#[function]`, `Selector`, `Text`, generated functions, and the build/export loop.

## File changes

Create the project, then replace `arcane/src/lib.rs`:

```sh
cargo run -p sand -- new arcane
cd arcane
```

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

#[component(Load)]
pub fn arcane_load() {
    cmd::tellraw(Selector::all_players(), Text::new("Arcane Powers loaded").gold());
}

#[function("arcane:hello")]
pub fn hello() {
    cmd::tellraw(Selector::self_(), Text::new("Hello from Arcane Powers").aqua());
}
```

## How it works

`#[component(Load)]` registers a function in Minecraft's load tag. `#[function("arcane:hello")]` creates a stable function id. `@s` means the command executor, so `/function arcane:hello` responds to the player who ran it.

## What Sand generates

Sand writes a load function and `data/arcane/function/hello.mcfunction` containing a `tellraw @s ...` command. It also adds its load function to `data/minecraft/tags/function/load.json`.

## Try it in Minecraft

Run `cargo run -p sand -- build`. Copy the generated datapack directory into `<world>/datapacks/`, enter the world, run `/reload`, then run `/function arcane:hello`.

## Common mistakes

- Installing an older build output: replace the existing datapack directory.
- Using `/function hello`: function ids are namespaced.
- Ignoring `logs/latest.log` after a reload error.

## Deeper reading

[Functions](../manual/functions.md), [Components](../manual/components.md), and [Testing](../manual/testing.md).
