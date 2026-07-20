# 17. Packaging And Running

## The export hook

Every Sand project needs exactly one export hook, called by the generated
`sand_export` binary:

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:export_hook}}
```

The doc comment above it in the source explains why this function has to
exist and be called from a binary at all: Rust's linker will discard object
files it can't prove are reachable from `main`, and Sand's component
registration relies on the `inventory` crate's link-time collection
(`inventory::submit!`, run as a constructor before `main`) to gather every
`#[component]`, `#[item]`, `#[function]`, and `#[event]` in the crate.
Calling into the library from `sand_export`'s `main` — even just to invoke
`__sand_export` — forces the linker to keep every object file whose
registrations you want collected. If you add new components in a separate
module, `pub use my_module::*;` it from `lib.rs` (as the scaffold template
comment says) so its registrations link in too.

`try_export_components_json_for_version` (from `sand::advanced` — the
supported low-level export surface, chapter 2) does the real work: it walks
every registered component, validates it against the resolved version
profile (chapter 16), and renders the full datapack JSON tree, printed to
stdout as one JSON document.

## `sand build`

```sh
sand build
```

Runs `cargo build` for the `sand_export` binary, executes it, and writes
the resulting datapack under `dist/` — the `.mcfunction` files, advancement
and recipe JSON, function tags, and dialogs this book has walked through,
laid out exactly as vanilla expects under `data/trail/...`. Pass
`--release` to also zip the output for distribution, or `--resourcepack`
if the project has resource-pack support enabled (`sand add resourcepack`;
Trailforge itself doesn't use this).

## `sand run`

```sh
sand run
```

Builds the datapack, downloads (and caches) the matching vanilla server
jar for `sand.toml`'s `mc_version`, and starts a local server with the
datapack loaded — the fastest way to playtest without a separate server
setup. Useful flags: `--ram 4G` (JVM heap size), `--offline` (sets
`online-mode=false` for easier local testing without a premium account
check), `--no-build` (skip rebuilding and use whatever's already in
`dist/`), and `--verbose` (stream the server's raw log instead of Sand's
filtered console).

## `sand join`

```sh
sand join --local
```

**Requires Prism Launcher.** Joins the local dev server started by `sand
run` (with `--local`), or joins a preconfigured `sand-dev` world with the
datapack (and resource pack, if built) already attached — useful for
testing without manually managing a Minecraft installation's world/mods
folder.

## `/reload` during a play session

With the server running, edit `src/lib.rs`, run `sand build` again in
another terminal, then run `/reload` in-game (or restart the server if
you'd rather). `load` (chapter 3) re-runs on reload, redefining objectives
and re-seeding storage — safe by design, since `load` was written to be
idempotent from the start.

## Cleaning up

```sh
sand clean            # remove dist/
sand clean --cargo    # also cargo clean
sand clean --server   # also remove dist/server/ (the sand run server files)
```
