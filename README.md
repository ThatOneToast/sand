<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/readme/sand-header-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="assets/readme/sand-header-light.svg">
    <img alt="Sand — Rust gameplay, vanilla datapacks" src="assets/readme/sand-header-light.svg" width="100%">
  </picture>
  <p><strong>Build Minecraft gameplay in Rust. Ship a vanilla datapack.</strong></p>
  <p>
    <a href="https://github.com/ThatOneToast/sand/actions/workflows/rust.yml">CI</a> ·
    <a href="book/src/introduction.md">Guide</a> ·
    <a href="examples/README.md">Examples</a> ·
    <a href="ROADMAP.md">Roadmap</a> ·
    <a href="LICENSE">MIT</a>
  </p>
</div>

Sand is a Rust framework for Minecraft Java datapacks. Describe player state,
quests, encounters, interactions and owned tasks using typed, composable APIs.
Sand compiles them into ordinary Minecraft functions and resources. Players and
servers need no Sand mod or Rust runtime.

**Pre-1.0, under active development.** APIs can change between commits. The
bounded multiplayer game alpha is still being validated; see the
[alpha contract](docs/architecture/alpha-game-contract.md) for its release gates.
Pin your project and CLI to the same revision.

## Start here

You need Rust (the repository pins **1.96.0**), **Java 25+** for Minecraft data
generation and local servers, and network access for the first dependency and
Minecraft artifact downloads. Sand is not yet published to crates.io.

```sh
git clone https://github.com/ThatOneToast/sand.git
cd sand
cargo install --locked --path sand-cli

# Create your project outside the Sand checkout.
cd ..
sand new oasis --mc-version 26.3
cd oasis
sand build
```

`sand new` creates a project and performs its initial Cargo build. Edit
`src/lib.rs`; `sand build` exports to `dist/oasis/`. To make a distributable zip:

```sh
sand build --release
```

The scaffold depends on the `sand` façade and the `sand-build` build dependency
from Git `main`. For reproducible work, set both to the same `rev`, keep
`Cargo.lock`, and install the CLI from that revision. Run
`sand context --format json` inside the project to check the resolved dependency,
Minecraft target and installed API catalog match.

See [installation](book/src/01-installing-and-creating.md) for the project layout
and [packaging](book/src/17-packaging-and-running.md) for output and deployment.

## Write gameplay, keep the vanilla output

State belongs to the player, entity or global resource that owns it. Sand handles
its objective identities, initialization and bounded writes:

```rust
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "oasis", scope = player)]
struct Explorer {
    #[state(default = 0, min = 0, max = 100)]
    discoveries: Score,
}

#[function]
pub fn discover_oasis() {
    let player = Explorer::on(EntityContext::<PlayerKind>::default());
    player.discoveries.add(1);
    cmd::tellraw(
        Target::self_(),
        Text::new("Oasis discovered!").gold().bold(true),
    );
}
```

Place this alongside the scaffold's export hook. Invoke
`/function oasis:discover_oasis` as a player in a world with the pack installed.
The same State fields compose with events, conditions, systems, progression and
HUDs; no separate gameplay database is required.

Generated `.mcfunction` and JSON files remain inspectable and distributable.
Minecraft commands are the compilation target. Typed command/resource builders
and explicitly named raw escape hatches remain available for advanced edges.

## What you can build today

| Area | Authoring model | Start with |
| --- | --- | --- |
| Player and entity state | Scores, flags, timers, cooldowns, finite enums, typed storage, migrations, bundles and archetypes | [Unified State](examples/unified_state/) |
| Multiplayer rounds | Registered bounded rosters, typed roles, exclusive membership, owned sessions and saved rewards | [Cooperative arena](examples/cooperative_arena/) |
| Quests and abilities | State-backed objectives, guarded forward branches, resource costs, cooldowns and checkpoints | [Quest rounds](examples/quest_rounds/), [Aether tools](examples/aether_tools/) |
| Combat and encounters | Custom health/shields, attributed damage, owned projectiles, effects and child attacks | [Owned combat](examples/owned_combat/), [Animated boss](examples/animated_boss/) |
| Interactions and inventory | Player menus, object-bound conversations, journals, quantity-aware payments and guarded transfers | [Interaction hub](examples/interaction_hub/) |
| Presentation | Block/item/text scenes, transforms, lifetime-owned HUDs, particles and sounds | [Gameplay guide](book/src/23d-owned-gameplay.md) |
| Datapack resources | Typed recipes, advancements, loot, predicates, item modifiers, tags, dialogs, enchantments and worldgen builders | [Trailforge](examples/book_project/), [Examples index](examples/README.md) |
| Companion resource packs | Separate client-asset packaging, version metadata and deterministic archives | [Resource-pack guide](book/src/17a-companion-resource-packs.md) |

The [capability matrix](docs/architecture/capability-matrix.md) records the
boundary between typed authoring, export tests and native evidence. Resource and
signal coverage varies; the API catalog's item count is not a percentage of
vanilla support.

## Discover the API locally

The CLI describes the public API compiled into your installation. It does not
scrape online documentation or infer methods that do not exist.

```sh
sand api search "player damage" --all-terms
sand api show sand::systems::Progression
sand api module sand::systems::Progression
sand api alternatives "damage entity"
```

`search` finds entry points; `show` explains signatures, behavior and examples;
`module` lists a module or a type's members. `alternatives` searches for typed
ways to express Minecraft behavior. Add `--format json` for structured results,
matching evidence, compatibility information and truncation indicators. Use
`--all`, `--limit`, `--module`, `--kind` and `--field` to narrow searches.

For coding agents and automation:

```sh
sand context --format json
sand api search "quest" --format json
sand build --format json
sand check --agent --format json
```

A complete catalog means the configured public surface is inventoried. It does
not mean every example is a complete game or that every vanilla capability has
a typed equivalent. Compiler errors, exported resources and native behavior
remain separate checks. See [CLI for coding agents](book/src/reference/agent-cli.md).

## Build, play and iterate

```sh
sand run                  # build and start a local vanilla server
sand dev                  # watch, rebuild and reload during development
sand join --instance INSTANCE_ID --local
```

Run the join command in another terminal. It requires an existing **Prism
Launcher instance folder ID**, not its display name. The launcher and instance
must already be installed; use `--launcher /path/to/prismlauncher` if needed.
Join launches a client; it is not a gameplay validation command.

The [dev workflow](docs/dev-workflow.md) documents reload behavior, failure
preservation and server ownership. `sand add worldbuild` adds typed world/server
configuration; `sand add resourcepack` opts into companion client assets.
`sand clean` removes generated pack output, with explicit options for Cargo and
server artifacts. Use `sand --help` for the complete command list.

## Versions and current limits

**26.3 is the default target.** Exact 26.1 and 26.2 profiles also exist; versions
below 26 are rejected. Export support does not imply equal native proof across
versions. Unknown future schemas are handled conservatively, and generated
registries require a known profile. See [version support](book/src/reference/version-support.md).

Keep these boundaries in mind when choosing a game:

- Owned tasks run for eligible loaded owners; there is no durable coroutine
  restoration or implicit force-loading.
- Rosters model explicitly registered arenas. Dynamic matchmaking and offline
  seat reservations are not implemented.
- Menus and conversations do not intercept arbitrary container-slot clicks.
  Free-text and arbitrary numeric form transport remain unverified.
- Collision is sampled; encounters do not provide general pathfinding or native
  damage interception. Scenes do not provide skeletal animation blending.
- Resource-pack packaging does not install client packs or prove asset rendering.
  Structure assets are copied and checked as files, not semantically parsed NBT.
- Round reset is not pack uninstall. Offline/unloaded cleanup and upgrade policy
  need explicit design; removing a datapack cannot invoke an unload callback.

The cooperative arena has retained 26.3 headless multiplayer evidence. Actual
player-input/rendering acceptance and a measured operating envelope are still
release work. Static runtime estimates are not MSPT or safe population limits.

## Quality and contributing

Sand checks deterministic exports, versioned API contracts, architecture
boundaries, downstream examples, compile-fail diagnostics and documentation.
Native server scenarios separately test gameplay and multiplayer isolation.
The [alpha scenario workflow](docs/testing/alpha-workflow.md) runs the roster and
reference-game proofs with one command and retains failure artifacts.

```sh
scripts/check.sh
git diff --check
```

See [Contributing](CONTRIBUTING.md), the
[architecture](docs/architecture/adr-001-crate-boundaries.md),
[benchmarks and limitations](BENCHMARKS.md), and [roadmap](ROADMAP.md).
For a bug report, include your Sand revision, Minecraft version, a minimal
project and the failing command/output. Distinguish compile/export failures
from observed server behavior.

Sand is available under the [MIT License](LICENSE). It is an independent project
and is not affiliated with or endorsed by Mojang Studios or Microsoft.
