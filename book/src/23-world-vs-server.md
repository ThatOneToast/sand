# 23. What Ships In The Datapack vs. What's Local To Your Dev Server

Sand's shipped artifact is a **datapack**: files under `data/<namespace>/...`,
portable to singleplayer, LAN, realms, or any vanilla-compatible server, with
no mod required. Everything in this chapter and the ones that follow is
badged consistently so you always know which bucket a setting falls into:

- 🌍 **World (datapack)** — packaged into the exported datapack. Works
  identically in singleplayer, on a friend's LAN game, or on any dedicated
  server you drop the pack into. Dimension definitions, world generation,
  gamerules/time/weather set via pack functions, world border, and spawn
  configuration are all 🌍.
- 🖥️ **Server (host)** — controls Sand's own local dev server via `sand run`,
  or documents a `server.properties`-equivalent value you set yourself on a
  real dedicated server. **Not** part of the datapack, and has no effect in
  singleplayer.

## Why the split exists

A datapack is data Minecraft loads *into* a running world. Several settings
people think of as "world configuration" are not data the world loads —
they're host process configuration:

- **View distance** and **simulation distance** are server-process settings
  (`server.properties`). A singleplayer world doesn't have a separate
  server process to configure this way, and a dedicated server operator sets
  these independently of any datapack they install.
- **Difficulty as a server default** is the value a fresh world starts with
  before anyone runs `/difficulty`. This is distinct from a **difficulty
  gamerule override** set via a pack function — that one *does* travel with
  the datapack because it's an ordinary command Minecraft runs on load.
- **Online mode** controls whether the server verifies Mojang sessions. It's
  meaningless in singleplayer and has no datapack representation.
- **World reset policy** — whether `sand run` wipes and regenerates its local
  world directory between runs — is entirely about Sand's own dev tooling,
  not something a datapack can express.

Sand's typed API keeps this split structural, not just documented:
[`sand::build::World`](https://github.com/ThatOneToast/sand) and everything
reachable from it lowers into datapack resources, while
[`sand::build::ServerConfig`](https://github.com/ThatOneToast/sand) is a
completely separate type that `sand run` reads and `sand build` never writes
into `dist/<namespace>/`.

## What this means for you

- If you're building a datapack you plan to **distribute** (put it on
  Modrinth, hand it to a friend, drop it in a server you don't administer),
  only 🌍-badged settings travel with it. Anything 🖥️ is Sand's own
  convenience for local iteration — reproduce the equivalent by hand
  (usually by editing `server.properties`) on whatever server actually hosts
  the datapack.
- If you're only ever testing locally with `sand run`, both badges matter to
  you, but only 🖥️ settings need `--offline`/profile-specific tuning; 🌍
  settings behave the same in `sand run` as they will everywhere else.

The rest of this section (chapters 24–34) uses these badges throughout —
look for 🌍 or 🖥️ next to every config item.
