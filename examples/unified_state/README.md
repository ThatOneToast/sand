# A small State-powered RPG

This is the kitchen-sink example, but it is still meant to be readable. It has
player progression, separate attack/defense/status components, nested bundles,
presence-aware queries, tick and attack-event systems, migrations, an
archetype, and one global resource with typed data.

Run it from this directory:

```sh
cargo test
cargo run --manifest-path ../../sand-cli/Cargo.toml --bin sand -- build
```

The build writes a server-ready datapack to `dist/unified_state`. To inspect the
intermediate component records separately, run `cargo run --quiet --bin sand_export`.

The repository's example check tests and builds this project. The latest-version
vanilla workflow also runs its real-server State audit; from the repository root:

```sh
python3 scripts/mc_validation/run_unified_state_audit.py --build \
  --jar "$HOME/.sand/cache/26.2/server.jar"
```

The audit installs the CLI-built datapack and checks two connected players,
independent component storage, ordered combat-system scan sharing, global and
per-owner reload retention, migrations, and selective detach. Its stability
checks wait for actual game ticks, so a slow server cannot pass by remaining
idle. Add `--evidence path/to/evidence.json` to retain check results and the
server log.
