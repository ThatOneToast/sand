# Vanilla Expedition

A CLI-only Sand authoring experiment for vanilla Minecraft Java 26.3. This is an exportable adventure vertical slice, not a fully tested release. The original agent did not run a native server. After the parent repaired dialog
dismissal and added the required root background, the pack passed a 26.3
startup/reload/shutdown smoke test with zero players. Gameplay remains unverified.

Players receive a custom expedition compass on first join. A Quick Actions journal explains objectives and can cosmetically inscribe held equipment. Five entity kills award a Pathfinder sword and a five-second personal text-display celebration. Entering the Nether grants an advancement, experience, a golden apple, and randomized supplies. Nether explorers see periodic objective guidance. State is per player; the season banner is intentionally global.

Representative resource samples include a flat sanctuary dimension/type, a vanilla trail-ruins structure set, a typed custom enchantment, loot, predicate, modifier, item tag, recipe, dialog, and advancements. The operator function `vanilla_expedition:establish_camp` replaces air at the current position with a campfire. Sanctuary access and richer worldgen are deliberately deferred.

Build from this directory using a CLI installed from the same checkout:

```sh
CARGO_TARGET_DIR=../../../../target CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 sand build
sand check --agent
python3 verify_export.py
cargo fmt --check
```

Output: `dist/vanilla_expedition`. The [experiment record](../README.md),
[original assessment](../assessment.md) and [breadth inventory](../breadth-addendum.md)
separate the agent's baseline experience from subsequent parent repairs.
The original source is retained in `../author-source.tar.gz`.
