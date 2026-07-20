# Sand

Sand is a strongly-typed Rust framework for authoring vanilla Minecraft Java
Edition datapacks (and optional resource packs) as ordinary Rust code. It
emits normal `data/<namespace>/...` files — functions, tags, advancements,
predicates, recipes, loot tables, item modifiers, and item components — via
an attribute-first authoring model (`#[function]`, `#[component(Load)]`,
`#[component(Tick)]`), typed state, typed conditions, typed execute chains,
and generated typed command builders. Raw commands and raw JSON/SNBT are
explicit, deliberate escape hatches, not the default authoring path.

> **Sand is pre-1.0 and volatile.** The public API can and does change
> between commits; there are no compatibility shims for removed APIs. Pin a
> commit if you need stability.

## Supported Minecraft versions

Minecraft Java **26.2** is the canonical export/profile target — it's what
`VersionProfile::resolve("latest")` resolves to and what canonical fixtures,
examples, and the book target by default. **1.21.4** is retained as an
explicit oldest-profile/compatibility boundary: it's the oldest version CI
verifies still codegens and renders correctly, not the implicit default.
Unknown/future Minecraft versions fall back to conservative capabilities
rather than erroring (`VersionProfile::resolve_strict()` is available when
you need a hard failure instead).

## Installing the CLI

Sand isn't published to crates.io yet — install the `sand` binary from a
clone of this repository:

```sh
git clone https://github.com/ThatOneToast/sand.git
cd sand
cargo install --path sand-cli
```

This installs the `sand-cli` package's binary, which is named `sand`.

## Adding the `sand` dependency

A datapack project depends on the single `sand` crate:

```toml
[dependencies]
sand = { git = "https://github.com/ThatOneToast/sand.git", branch = "main" }

[build-dependencies]
sand-build = { git = "https://github.com/ThatOneToast/sand.git", branch = "main" }
```

`sand new`/`sand init` generate this automatically (using a local path
dependency instead, if scaffolding inside the Sand workspace itself).

## A minimal datapack

```rust
use sand::prelude::*;

static VISITS: ScoreVar<i32> = ScoreVar::new("visits");

#[component(Load)]
pub fn load() {
    VISITS.define();
    cmd::tellraw(Selector::all_players(), Text::new("Pack loaded").green());
}

#[function]
pub fn greet() {
    VISITS.add(Selector::self_(), 1);
    cmd::tellraw(Selector::self_(), Text::new("Hello from Sand").gold().bold(true));
}
```

`#[component(Load)]` registers `load` into `minecraft:load`, so it runs on
every world load and `/reload`. `#[function]` exposes `greet` as a callable
function (`/function <namespace>:greet`). See
`examples/book_project/src/lib.rs` for a complete pack (state, events, items,
recipes, dialogs, conditions, and VFX) built the same way.

## Build and run

```sh
sand new my_pack       # scaffold a new project
cd my_pack
sand build              # compile to dist/
```

Copy the generated datapack from `dist/` into a world's `datapacks/`
directory and run `/reload`, or use `sand run` to download a server jar and
launch a local test server with the datapack already installed. `sand
build --release` zips the output for distribution. Run `sand --help` for the
full command list (`new`, `init`, `build`, `run`, `join`, `add`, `clean`).

## Documentation

The [Sand Guide](book/src/introduction.md) (mdBook) is the canonical user
guide: tutorials, a full manual, and reference pages. Build it locally with
`mdbook build` or `scripts/build-book.sh`.

## Architecture

Sand is split into a small public surface (`sand`, the façade crate you
depend on, plus the `sand-cli` binary) and several internal implementation
crates. See
[`docs/architecture/adr-001-crate-boundaries.md`](docs/architecture/adr-001-crate-boundaries.md)
for the full crate graph and rationale.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the validation checks, toolchain
policy, and local codegen contract.
