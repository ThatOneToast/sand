# Quantity-aware inventory, branching progression and default-target proofs

The 2026-11-05 implementation adds exact-quantity player-slot transfers, payments
that preserve surplus items, effective native stack-capacity predicates, and
forward-only progression branches. `sand join` now invokes an explicitly selected
Prism instance and reports launcher failures. The isolated build suite also has
core, contract and codegen invalidation modes and opt-in test compilation.

## Native evidence

[Recorded results and exported-pack fingerprints](../../benchmarks/evidence/workflow-expansion-2026-11-05/evidence.json)
retain seven successful native runs and compressed complete logs. Inventory,
player ownership and HUD scenarios passed separately on 26.2 and 26.3 with two
real passive clients. Progression passed on 26.3 with two independent marker
hosts. The scripts retain the appropriate version-selected client protocol.

Inventory assertions cover native default capacities of 64, 16 and 1, an override
of 99, exact partial copies/merges, component-preserving remainders, insufficient
stock and overflow, listener replacement compensation with a fresh destination check before undo, reentry, cross-namespace
locking, payment ordering and player isolation. The capacity lookup asks vanilla's
component model rather than guessing defaults from item IDs. This changes the
previously conservative rejection of ordinary compatible stacks into a successful
merge. `Purchase::item_cost` now consumes only its requested quantity; compose an
exact count predicate if the complete payment stack must equal that amount.

Progression assertions cover objective eligibility before branching, first-match
branch priority, sequential fallback, owner isolation, skipped rewards, reentry,
terminal idempotence and explicit reset. Existing session cancellation, descendant
cleanup and reload-resume checks still run. These are forward branches, not loops.

Reproduce the player scenarios with their matching verified server jars:

```sh
python3 scripts/validate_inventory.py --version 26.3 --jar /path/to/server.jar --output /tmp/inventory-proof
python3 scripts/validate_owned_players.py --version 26.3 --jar /path/to/server.jar --output /tmp/ownership-proof
python3 scripts/validate_hud.py --version 26.3 --jar /path/to/server.jar --output /tmp/hud-proof
python3 scripts/validate_progression.py --jar /path/to/26.3/server.jar --output /tmp/progression-proof
```

Use `--version 26.2` with the 26.2 jar for the first three commands. Their default
is now 26.3. The CI matrix invokes each explicitly on both versions.

## Verification boundaries

Export regressions cover 26.1, 26.2 and 26.3. The launcher integration test uses a
stand-in executable and verifies exact argument boundaries and nonzero/missing
launcher errors; it does not claim an authenticated graphical Prism session.
The new benchmark modes are tested against controlled fake-Cargo fixtures for
source isolation, restoration, command selection and error evidence. No new
build-speed or server-MSPT result is claimed by adding these modes.

This increment does not implement container transfers, free-text or arbitrary
numeric form transport, dynamic matchmaking, pack reset/uninstall,
or population-aware performance comparisons. Those remain separate roadmap work. The subsequent [roster increment](../gameplay/arena-rosters.md)
adds explicit registered-arena roles and online membership; it does not supply
dynamic matchmaking or offline seats.
