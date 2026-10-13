# Guarded bounded State writes

A numeric State mutation includes its conversion, bounds enforcement and dirty
notifications. A condition around that mutation must be evaluated before the
write, once, so changing the tested value cannot skip the remaining work.

Pass the complete action to the execute builder:

```rust,ignore
TypedExecute::as_players()
    .when(state.energy.matches(..100).unwrap())
    .run(state.energy.add(10));
```

The compiler also preserves this invariant when an author iterates the returned
actions: `add` and `subtract` retain one owned action for the complete mutation.
This uses Sand's existing action/helper lowering and keeps generated helpers
within the exporting namespace.

## Reproducible server audit

```sh
python3 scripts/validate_bounded_state_writes.py --jar "$HOME/.sand/cache/26.2/server.jar"
```

The script exports `sand-core/tests/bounded_mutation_atomicity.rs` through Cargo
and writes the compiler records unchanged into a disposable Minecraft 26.2
datapack. Two passive clients connect to the shared loopback-only harness with
RCON disabled. After setting each player's public State field, it invokes the
generated functions and checks both scores:

| Operation | Initial Alpha / Beta | First call | Second call |
| --- | --- | --- | --- |
| Add 10 while below 100 | 95 / 40 | 100 / 50 | 100 / 60 |
| Subtract 10 while positive | 5 / 40 | 0 / 30 | 0 / 20 |
| Whole-action add 10 while below 100 | 95 / 99 | 100 / 100 | — |

The first two operations intentionally retain the previously unsafe iterator
composition. The final operation uses the recommended whole-action API. The
audit exercises synchronous generated operations directly; it does not measure
tick scheduling or numeric overflow behavior. The latest-version vanilla CI
workflow runs it and retains failure diagnostics.

Export tests assert one guarded entry call with reachable lower and upper clamps,
no repeated original guard inside the mutation, and deterministic isolated
exports. Existing fixed-point and entity-state tests inspect the owned helper's
actual command body to retain conversion and dirty-notification coverage. The
canonical book example also tests its exported regeneration helper, and the
repository check script runs that example's tests.
