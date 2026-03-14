# Sand — Project Plan

> All milestones for v0.1.0 are complete. This document is retained as a
> historical reference for the design decisions made during development.

---

## Table of Contents

- [Milestone 0 — Workspace Bootstrap](#milestone-0--workspace-bootstrap)
- [Milestone 1 — sand-build: Data Pipeline](#milestone-1--sand-build-data-pipeline)
- [Milestone 2 — sand-core: Primitives](#milestone-2--sand-core-primitives)
- [Milestone 3 — sand CLI: Scaffolding](#milestone-3--sand-cli-scaffolding)
- [Milestone 4 — sand-macros: First Macros](#milestone-4--sand-macros-first-macros)
- [Milestone 5 — sand-core: Datapack Components](#milestone-5--sand-core-datapack-components)
- [Milestone 6 — sand CLI: Build Command](#milestone-6--sand-cli-build-command)
- [Milestone 7 — Polish & DX](#milestone-7--polish--dx)
- [Design Decisions (Resolved)](#design-decisions-resolved)

---

## Milestone 0 — Workspace Bootstrap

> Goal: Get a clean, compiling workspace with all crates wired together.

### Workspace

- [x] Create `Cargo.toml` workspace manifest listing all crates
- [x] Add `.gitignore` (target/, dist/)
- [x] Add top-level `README.md`

### Crates

- [x] Create `sand` — binary crate (CLI)
- [x] Create `sand-core` — library crate (core types and components)
- [x] Create `sand-macros` — proc macro crate (`proc-macro = true`)
- [x] Create `sand-build` — library crate (data pipeline and codegen)
- [x] Create `sand-example` — integration tests and reference examples

### Verify

- [x] `cargo build --workspace` compiles clean
- [x] `cargo test --workspace` runs with zero failures

---

## Milestone 1 — sand-build: Data Pipeline

> Goal: Given a Minecraft version string, download the server jar, run the data
> generator, parse the reports, and write Rust source files to `OUT_DIR`.

### Version Manifest

- [x] Define `VersionManifest` and `VersionEntry` serde structs
- [x] Fetch `version_manifest_v2.json` from Mojang meta URL
- [x] Resolve a version string (e.g. `"1.21.4"` or `"latest"`) to a `VersionEntry`
- [x] Cache manifest locally to avoid re-fetching on every build

### Server Jar Download

- [x] Fetch the version-specific `version.json` to get server jar URL + sha1
- [x] Download server jar to cache dir with progress bar
- [x] Verify sha1 checksum after download
- [x] Skip download if cached jar sha1 already matches

### Report Generation

- [x] Invoke `java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports`
- [x] Capture stdout/stderr and surface errors clearly
- [x] Detect if `generated/reports/` already exists and is up to date (skip re-run)
- [x] Handle missing Java gracefully with a useful error message

### Codegen — Registries

- [x] Parse `generated/reports/registries.json`
- [x] Generate `registries.rs` with enums for: `Item`, `Block`, `EntityType`, `Biome`, `Enchantment`, `SoundEvent`
- [x] Each enum variant: `snake_case` JSON key -> `PascalCase` Rust variant
- [x] Impl `resource_location() -> &'static str` on each enum
- [x] Impl `Display` on each enum (outputs resource location string)

### Codegen — Block States

- [x] Parse `generated/reports/blocks.json`
- [x] Generate `block_states.rs` with per-block state property types
- [x] Each block gets a typed properties struct

### Codegen — Commands

- [x] Parse `generated/reports/commands.json`
- [x] Generate typed command builder functions

### Integration

- [x] Expose a `sand_build::generate(mc_version: &str)` public fn for use in `build.rs`
- [x] Write generated files to `OUT_DIR`
- [x] Emit `cargo:rerun-if-changed` directives appropriately

---

## Milestone 2 — sand-core: Primitives

> Goal: Define the foundational types that everything else in Sand builds on.

### Types

- [x] `ResourceLocation` — `namespace:path` string wrapper, validated (`[a-z0-9_.-]+:[a-z0-9_./-]+`)
- [x] `Identifier` — type alias for `ResourceLocation`
- [x] `PackNamespace` — the user's declared namespace, validated
- [x] `McVersion` — parsed semver-like MC version struct with `Ord` impl

### Traits

- [x] `DatapackComponent` — trait for anything serializable into a datapack file
- [x] `IntoDatapack` — trait for collecting components into a full datapack output

### Error Types

- [x] Define `SandError` with `thiserror`
- [x] Variants for: invalid namespace, invalid path, serialization failure, IO error, invalid version

### Tests

- [x] Unit tests for `ResourceLocation` validation (valid, invalid namespace, invalid path)
- [x] Unit tests for `McVersion` parsing
- [x] Serde round-trip tests

---

## Milestone 3 — sand CLI: Scaffolding

> Goal: `sand new <name>` and `sand init` produce a valid, compiling Rust project.

### CLI Skeleton

- [x] Set up `clap` with `derive` feature
- [x] Define subcommands: `new`, `init`, `build`, `run`, `clean`, `version`
- [x] `sand --version` prints current Sand version

### Templates

- [x] Create `templates/default/` directory with embedded templates (`include_str!`)
- [x] `Cargo.toml.hbs`, `build.rs.hbs`, `sand.toml.hbs`, `src_lib_rs.hbs`, `sand_export_rs.hbs`

### `sand new`

- [x] Accept `<name>` positional arg
- [x] Accept `--mc-version <version>` flag (default: fetch latest from Mojang)
- [x] Validate project name (lowercase letters, digits, underscores, hyphens)
- [x] Create project directory, render all templates
- [x] Run `cargo build` to pre-warm cache
- [x] Print success message with next steps

### `sand init`

- [x] Same as `sand new` but uses current directory
- [x] Error if `sand.toml` already exists

### `sand.toml` Schema

- [x] `[pack]` section: `namespace`, `description`, `mc_version`, `pack_format` (optional override)

### Tests

- [x] Test scaffolding produces expected files
- [x] Test name validation (valid and invalid)
- [x] Test namespace conversion (hyphens to underscores)

---

## Milestone 4 — sand-macros: First Macros

> Goal: Proc macros for annotating Rust functions and having them emitted as
> datapack files.

### Setup

- [x] `proc-macro = true` in `sand-macros/Cargo.toml`
- [x] `proc-macro2`, `quote`, `syn` dependencies

### `#[function]`

- [x] Parse annotated `fn` with `syn`
- [x] Validate: free-standing (no `self`), no parameters
- [x] Emit `FunctionDescriptor` registration via `inventory::submit!`
- [x] Body transformation: `let` bindings pass through, `expr;` pushes to commands, trailing `expr` extends

### `#[component]`

- [x] Plain `#[component]` — returns `Box<dyn DatapackComponent>`
- [x] `#[component(Tick)]` — registers in `minecraft:tick` function tag
- [x] `#[component(Load)]` — registers in `minecraft:load` function tag
- [x] `#[component(Tag = "ns:name")]` — custom function tag

### `run_fn!`

- [x] With body: define inline function + register + return `cmd::function(...)` call
- [x] Without body: shorthand for `cmd::function(...)`

### `mcfunction!` (declarative macro in sand-core)

- [x] Semicolon-separated expressions
- [x] String literals and `Display` types both work

### Tests

- [x] `trybuild` compile-tests (pass and fail cases)

---

## Milestone 5 — sand-core: Datapack Components

> Goal: Typed representations for all main datapack component types.

### Components

- [x] `McFunction` — list of command strings, serializes to `.mcfunction`
- [x] `Advancement` — triggers, criteria, display, rewards, telemetry
- [x] `ShapedRecipe` / `ShapelessRecipe` — crafting recipes
- [x] `CookingRecipe` — smelting, blasting, smoking, campfire cooking
- [x] `StonecuttingRecipe` / `SmithingTransformRecipe` / `SmithingTrimRecipe`
- [x] `LootTable` — pools, entries, conditions, functions
- [x] `Predicate` — condition wrappers
- [x] `ItemModifier` — loot modification functions
- [x] `Tag` — block/item/entity/function tags
- [x] `CustomItem` — 1.21+ item component system (food, equipment, tools, etc.)

### For Each Component

- [x] Define Rust struct/enum matching the MC JSON schema
- [x] `impl DatapackComponent`
- [x] `impl Serialize` (serde)
- [x] Builder-pattern API with escape hatches for raw JSON
- [x] Unit tests for serialization output

---

## Milestone 6 — sand CLI: Build Command

> Goal: `sand build` compiles the user's project and writes a valid datapack to `dist/`.

### Build Flow

- [x] Read and validate `sand.toml`
- [x] Shell out to `cargo build --bin sand_export`
- [x] Run export binary to collect all registered components as JSON
- [x] Parse JSON and write files to `dist/<namespace>/data/<namespace>/<dir>/<path>.<ext>`
- [x] Write `pack.mcmeta` with `pack_format` and `description`
- [x] `sand build --release` zips output to `dist/<namespace>.zip`

### `sand run`

- [x] Build datapack, download server jar, create server directory
- [x] Accept EULA automatically, sync datapack
- [x] Launch Java with configurable heap (`--ram`)
- [x] `--offline` flag for `online-mode=false`
- [x] `--no-build` flag to skip rebuild

### `sand clean`

- [x] Remove `dist/`
- [x] `--cargo` flag also runs `cargo clean`

---

## Milestone 7 — Polish & DX

> Goal: Make Sand feel like a real, well-crafted tool.

- [x] Colorized CLI output (errors in red, success in green, etc.)
- [x] Progress bars for jar downloads (`indicatif`)
- [x] Clear, actionable error messages throughout
- [x] `sand build --release` flag for zip output
- [ ] `sand watch` — rebuild on `src/` changes (deferred to v0.2.0)
- [x] Doc comments on all public API items
- [x] `sand new` fetches actual latest MC version at scaffold time
- [ ] Published to crates.io (pending v0.1.0 release)
- [x] Top-level `README.md` with quickstart guide
- [x] `LICENSE` (MIT)
- [x] `CHANGELOG.md`
- [ ] `CONTRIBUTING.md` (deferred to v0.2.0)

---

## Design Decisions (Resolved)

| # | Question | Decision |
|---|----------|----------|
| 1 | Bundle pre-generated reports vs require Java? | Require Java — keeps generated types exactly matching the target MC version |
| 2 | Global cache or local project cache? | Global cache at `~/.sand/cache/<version>/` |
| 3 | `sand.toml` standalone or in `Cargo.toml`? | Standalone `sand.toml` |
| 4 | How are proc macro outputs collected? | `inventory` crate — link-time registration, no manual wiring |
| 5 | Builder pattern vs struct literals? | Builder pattern with escape hatches for raw JSON |
| 6 | Priority order for component types? | Functions, advancements, recipes, loot tables, tags, custom items |
| 7 | `sand build` output format? | Directory by default, zip with `--release` |
| 8 | Third-party loader support (Fabric, NeoForge)? | Not in v0.1.0 scope; escape hatches cover edge cases |
| 9 | `ResourceLocation` strict or permissive? | Strict validation at construction (`[a-z0-9_.-]+:[a-z0-9_./-]+`) |
| 10 | `mcfunction!` inline syntax timing? | Implemented in Milestone 4 as a declarative macro in sand-core |

---

## Future Work (v0.2.0+)

- `sand watch` — file watcher with auto-rebuild
- `CONTRIBUTING.md` guide
- Publish to crates.io
- Version-specific schema validation
- Fabric/NeoForge loader awareness
- More command builders (particle effects, NBT operations)
- `sand test` — datapack unit testing framework
