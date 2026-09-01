# 31. Server Configuration

🖥️ **Server (host) only.** Everything in this chapter configures Sand's own
local dev server (`sand run`) or documents a `server.properties`-equivalent
value you set yourself on a real dedicated server. **None of it is
packaged into the exported datapack.** See
[chapter 24](./23-world-vs-server.md) if you haven't read the World vs.
Server explainer yet — it covers *why* these settings have no datapack
representation at all.

## `ServerConfig`

```rust
use sand::build::{Difficulty, ServerConfig, WorldResetPolicy};

let server = ServerConfig::new()
    .view_distance(12)
    .simulation_distance(8)
    .difficulty(Difficulty::Hard)
    .online_mode(false)
    .world_reset_policy(WorldResetPolicy::AlwaysReset);
```

Attach it to a build via `SandBuild::server(...)`:

```rust,ignore
SandBuild::new()
    .world(World::new())
    .server(ServerConfig::new().view_distance(6))
```

| Field | `server.properties` equivalent | Default |
|---|---|---|
| `view_distance` | `view-distance` | `10` |
| `simulation_distance` | `simulation-distance` | `10` |
| `difficulty` | `difficulty` (the world's *default*, not a gamerule) | `normal` |
| `online_mode` | `online-mode` | `true` |
| `world_reset_policy` | *(no equivalent — see below)* | `Keep` |

## How `sand run` applies it

When `sand build` runs a project's `sand.build.rs`, any `ServerConfig` is
written to `dist/.sand-server-config.json` — a sibling of `dist/<namespace>/`,
**not** inside it, so it is never picked up as part of the datapack or
synced into `dist/server/world/datapacks/`.

`sand run` reads that file (if present) and:

1. If `world_reset_policy` is `AlwaysReset`, deletes `dist/server/world/`
   before doing anything else, so the server starts from a clean local
   world every time.
2. On the **first** launch only (an existing `server.properties` is never
   overwritten — including one you've hand-edited), writes
   `view-distance`, `simulation-distance`, `difficulty`, and `online-mode`
   into `server.properties` from the resolved `ServerConfig`.

`sand run --offline` always wins over `ServerConfig::online_mode(true)` —
it's an explicit, one-off CLI override for local testing convenience.

## `WorldPreset`

`World::preset` (`WorldPreset`) is the one field that lives on the 🌍
`World` type but is actually 🖥️ Server (host) only: Minecraft only accepts
a world generation preset (superflat, large biomes, amplified, …) *at world
creation*, which has no datapack mechanism. `sand run` reads it only to
decide how to create/reset its local dev world; it's not written into the
exported datapack and has no effect on an existing world.

```rust
use sand::build::{World, WorldPreset};

let world = World::new().preset(WorldPreset::Flat);
```

## Deploying to a real dedicated server

If you're handing this datapack to someone running their own dedicated
server (or deploying it yourself outside `sand run`), **none of the above
travels with the datapack.** Reproduce the equivalent settings by hand in
that server's own `server.properties`:

```properties
view-distance=12
simulation-distance=8
difficulty=hard
online-mode=true
```

There is no `sand deploy`-style command that pushes `ServerConfig` to a
remote server — that would require SSH/RCON access Sand has no reason to
assume exists, and is out of scope for this feature.
