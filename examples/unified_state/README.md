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
  --jar "$(cargo run --quiet -p sand-build --bin ensure-server-jar -- 26.2)"
```

The audit installs the CLI-built datapack and checks two connected players,
independent component storage, ordered combat-system scan sharing, global and
per-owner reload retention, migrations, and selective detach. Its stability
checks wait for actual game ticks, so a slow server cannot pass by remaining
idle. Add `--evidence path/to/evidence.json` to retain check results and the
server log.

The `template` fields and `set_*_literal*` functions demonstrate that strings such
as `$(name)`, quotes and backslashes remain ordinary data in player, entity and
global State. `scripts/validate_keyed_literals.py` checks natural player defaults,
explicit entity attachment, nested values, independent owners, scratch cleanup,
reload retention and selective detach against the CLI-built pack:

```sh
python3 scripts/validate_keyed_literals.py --version 26.2 \
  --pack examples/unified_state/dist/unified_state \
  --jar "$(cargo run --quiet -p sand-build --bin ensure-server-jar -- 26.2)"
```

The latest-version vanilla CI job runs this audit and retains failure evidence.

`count_diamonds` demonstrates a player-context function receiving the same
`EntityContext<PlayerKind>` used by `Event<OnJoinEvent>::player()` and player
queries. The join handler calls it through `player.call`; its semantic inventory
count writes directly to `Supplies::on(player).diamonds`. The handle is the
current execution's `@s`, not a persistent player identity.
