# Trailforge gameplay evidence

The native harness exercises the exported Trailforge upgrade and traversal loop
with two real passive players on Minecraft 26.2 / Java 25. The retained
[JSON evidence](trailforge-evidence.json) identifies the gameplay source revision,
source tree and example-file hashes, exported pack digest, harness revision and
script digest, Java version, and server jar digest. A passing record applies to
that pack, not to later edits or an untested integrated build.

The retained run passed 120 assertions (including dispatch-clock checks)
on gameplay commit `41b91e5d87ddb296f75bfe9a884bddbbfc1531d1` with
harness commit `c0905c6f4497a012879ffd17eeb228401dc62bc9`. Both
source checkouts were clean for the recorded run. This gameplay pack predates
integration of the separate State initializer optimization.

The gameplay contract requires the claimed upgrade and canonical custom Trail
Striders in the feet slot. Grapple accepts the first gold-block collision within
8 blocks from the player's eyes, spends 30 stamina, and starts an 80-tick
cooldown. Scout reuses the eligibility path with an iron anchor, a 4-block range,
10 stamina, and a 20-tick shared cooldown. Both rays sample every 0.25 blocks;
these tests do not establish exact voxel collision for thin obstacles.

The canonical gameplay-only runtime baseline grows from 37 to 58 emitted
functions, including the two registered raycasts and their callbacks. The
authored `tick` function now uses one positioned player pass instead of four
separate passes, and groups regeneration behind one timer decision. Raycasts
run only on eligible requests; they add no tick observer. The report retains
unknown upper bounds for recursive ray helpers and population-dependent calls.
The 32/16 sample bounds come from the ray definitions and their tests, not a
whole-tick budget inferred from that report. This is an intentional static
output delta for new behavior, not a measured speedup over the old untargeted
dash. The separate initialization optimization has its own unchanged-fixture
before/after evidence.

The harness covers:

- Recipe-result custom identity, plain-item rejection, exact upgrade consumption
  and reward quantity, independent upgrades, and repeat-claim prevention.
- Successful grapple effects and resource/cooldown changes, insufficient
  stamina, active cooldown, unequipped or ordinary boots, and exhaustion policy.
- Missing and invalid targets, a wall before an anchor, last-block/out-of-range
  range cases, and external execution context in another dimension. Failed
  attempts preserve stamina/cooldown and grant no speed effect.
- Both players activating in one frozen game tick, independent cost/effects,
  scout's smaller range and cost, and cooldown shared across both abilities.
- Missing-state default repair, exact-cost depletion without underflow,
  regeneration at 40-tick boundaries, clamping 95 + 10 to 100, regeneration
  while unequipped, and natural cooldown expiration.
- Actual resource reload and client disconnect/reconnect, preserving unlocks,
  equipment and stamina, with cooldown decrements matching the exact game ticks
  advanced by the harness.

Automatic game ticks are frozen during synchronous assertions. Both trigger
requests are queued before one callback dispatcher invocation, and the harness
checks that game time stayed unchanged. Regeneration and cooldown scenarios use
`tick step` and wait for the requested game-time advance before checking state.
Player effects may still tick while the world is frozen; effect-duration timing
is not an assertion of this audit.

The exported gameplay pack is copied unchanged into a disposable server. A
separate observation-only pack adds a load-tag marker. Reload validation waits
for a fresh marker and completion of the game tick that ran the load tag, so it
observes completion of all synchronous production load callbacks regardless of
marker ordering. A console command submitted after `reload` alone would not
establish that those callbacks had run. Scheduled work after the load tick is
outside this synchronization guarantee.

The harness grants the actual exported recipe result, copies the exported
reward stack for equipment setup, and invokes exported menu triggers or the
scout function from the console. It does **not** prove client crafting, dialog
packet input, survival permission enforcement, death/respawn, full-inventory
transactions, or rollback if an individual effect command fails. Successful
activation means anchor acceptance and observed effects, not a transactional
world mutation. Unloaded/out-of-world ray behavior belongs to the separate
raycast audit; this run makes no new native claim for those cases.

Reproduce from a checkout with Java 25, Python 3, Cargo, and the cached vanilla
26.2 server jar. The passive client uses the standard library and protocol 776;
no graphical Minecraft client is required. The disposable local server accepts
the Minecraft EULA and uses offline mode on a private loopback port.

```sh
cargo build -p sand-cli
(cd examples/book_project && CARGO_TARGET_DIR="$PWD/../../target" ../../target/debug/sand build)
python3 scripts/validate_trailforge.py \
  --pack examples/book_project/dist/trail \
  --source-root "$PWD" \
  --jar "$HOME/.sand/cache/26.2/server.jar" \
  --java java \
  --output target/vanilla-reload/26.2/trailforge
python3 -m unittest scripts.tests.test_trailforge
```

`--java` can be omitted when Java 25 is on PATH. `--source-root` identifies the
build checkout when the pack was copied outside Git. The output directory is
disposable and replaced on each run; choose a new directory to retain failed
attempts. Evidence includes partial passed checks and the failure phase/message
when a runtime or fixture assertion fails. The script rejects an observed server
startup version other than 26.2.
