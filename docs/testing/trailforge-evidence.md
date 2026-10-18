# Trailforge upgrade and ability evidence

The canonical example now exports a component-bearing Grapple Core recipe,
checks and consumes its custom identity when claiming Trail Striders, and
scopes ability activation to the requesting player. Its player entry points
initialize State before use. Boots are granted through the typed item API;
they do not replace existing armor. Vanilla's full-inventory drop behavior
still applies.

The Minecraft 26.2 / Java 25 run retained in
[trailforge-evidence.json](trailforge-evidence.json) passed 19 assertions with
two real passive players. These cover plain-item rejection and preservation,
another player's untouched inventory, exact consumption and reward quantity,
repeat-claim prevention, independent upgrades, stamina initialization, ability
isolation, cooldown gating, and reload preservation. The evidence records the
compiled pack digest; the harness rejects a server whose observed startup
version is not 26.2.

The harness reads the actual exported recipe result and gives that stack to
players, then reads the exported menu's `/trigger` commands and invokes them
as each player through the generated callback dispatcher. Automatic ticks
are frozen during assertions. This proves recipe-result identity and explicit
gameplay and callback dispatch behavior; it does **not** prove client-originated crafting, menu input,
automatic regeneration, death/respawn, or a general atomic inventory transaction.
Those require separate scenarios rather than broader claims from these checks.

Reproduce from the repository root with Java 25 on PATH:

```sh
(cd examples/book_project && CARGO_TARGET_DIR="$PWD/../../target" ../../target/debug/sand build)
python3 scripts/validate_trailforge.py \
  --pack examples/book_project/dist/trail \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --output target/vanilla-reload/26.2/trailforge
```

Build the CLI first with `cargo build -p sand-cli` if needed. The latest-version
vanilla workflow runs this audit and retains its server/client logs and evidence.
Export tests independently verify recipe components, the bounded single-subject
cost, and the absence of player fan-out in the reachable ability functions.

The menu uses survival-friendly callbacks rather than permission-gated function
commands. Export regressions verify both trigger actions and their dispatcher
targets. The acquisition event derives its predicate from the same custom item
definition used by crafting and upgrade consumption. Console execution of the
trigger commands does not prove client packet input or permission enforcement.
