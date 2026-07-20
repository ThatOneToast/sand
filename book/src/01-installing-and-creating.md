# 1. Installing Sand And Creating A Project

## Requirements

- Rust 1.96+ with edition 2024 support.
- Java 21+ for Minecraft data generation during builds.
- Network access on the first build so Sand can download and cache the
  matching Minecraft server jar for codegen.

## Installing the CLI

The `sand` CLI (package `sand-cli`, binary name `sand`) scaffolds projects,
builds datapacks, and can run/join a local test server. Install it from the
workspace while it's being prepared for crates.io:

```sh
cargo install --path sand-cli
```

## Creating Trailforge

```sh
sand new trailforge
cd trailforge
```

`sand new <name>` creates a directory containing:

```text
trailforge/
├── Cargo.toml       # one dependency: sand
├── build.rs         # runs Minecraft data codegen for your pinned version
├── sand.toml         # pack metadata: namespace, description, mc_version
└── src/
    ├── lib.rs         # your datapack source
    └── bin/
        └── sand_export.rs   # thin binary the CLI invokes to render JSON
```

`sand.toml` is a standalone pack manifest — deliberately not embedded in
`Cargo.toml`, because pack metadata (namespace, target Minecraft version,
description) is a datapack concern, not a Cargo concern:

```toml
[pack]
namespace   = "trailforge"
description = "A Minecraft datapack built with Sand"
mc_version  = "26.2"
```

The rest of this book builds Trailforge chapter by chapter against
`examples/book_project`, the repository's own compile-tested copy of this
exact project (its `sand.toml` sets `namespace = "trail"` for brevity — the
snippets below use that shorter namespace). If you're following along by
hand, keep your own project's namespace (`trailforge`) instead; only the
namespace string differs.

## `sand init` for existing directories

If you already have a directory (for example, one under version control)
and want to turn it into a Sand project in place, use `sand init` instead of
`sand new` — it takes the directory name as the project name and refuses to
run if `sand.toml` already exists.

## What happens next

Every remaining chapter edits `src/lib.rs`. Chapter 17 covers `sand build`
and `sand run` once the pack has something worth loading.
