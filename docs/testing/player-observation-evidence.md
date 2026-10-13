# Real-player observation evidence

Minecraft Java 26.2, protocol 776. The local audit passed on October 13, 2026.
The Vanilla reload workflow runs the same audit against its latest-version pack.
The client deliberately rejects other protocol versions until their packet
registries and runtime behavior have been checked.

## Reproduce

Build the representative pack, then run the harness with Java 25 available:

```sh
(cd sand-vanilla-audit && cargo run --manifest-path ../sand-cli/Cargo.toml -- build)
python3 scripts/validate_player_observations.py \
  --version 26.2 \
  --jar ~/.sand/cache/26.2/server.jar \
  --pack sand-vanilla-audit/dist/sand_audit
```

The harness copies the pack into a disposable world, binds its offline server
to loopback, connects two minimal clients, and shuts down its own processes.
Server and per-client logs are retained in the printed output directory. An
explicit `--output` directory is replaced, as with the base reload harness.

## Verified behavior

- Both subjects are real connected Minecraft players, selected through `@a`.
- Native health and effect observations initialize without dispatching handlers.
- Damaging Alpha dispatches one health-change and one health-loss event for
  Alpha; Beta's counters remain zero.
- Applying speed to Beta dispatches one effect-start event for Beta and does
  not change Alpha's counters.
- A real `/reload` retains both players' observation baselines: no duplicate
  health or effect transitions occur during 35 subsequent presence checks,
  spaced one second apart for both players.
- Removing Beta's speed and damaging Beta produces the corresponding stop,
  health-change and health-loss events, while Alpha's counts remain unchanged.

The assertions observe counters written by the exported audit handlers.
Dispatch comes from normal Minecraft ticks. The harness neither invokes the
samplers/handlers manually nor replaces the read-only player-health criterion.
It disables natural health regeneration so healing cannot confound the counts.

## Client correction and evidence boundary

The old client listened for Play keep-alive packet `0x4c`. Inspecting the actual
26.2 server's `GameProtocols.CLIENTBOUND_TEMPLATE.details().listPackets(...)`
shows `clientbound/minecraft:keep_alive` at `0x2c`; the serverbound template
assigns its reply `0x1c`. With that correction, a separate one-player probe
passed 60 consecutive one-second server-side presence assertions. The
regression test feeds the actual `0x2c` challenge and verifies its exact echo.
The client now rejects a connection that closes early, even after login.

This is a passive protocol client, not a complete gameplay bot. The audit
proves the listed observation behaviors only. It does not establish item
snapshot correctness, real advancement-driven combat participant correlation,
composed participant forwarding, player movement input, or every scenario in
#265. Those still require their own fixtures and assertions. Server-directed
`/damage` here tests native health observation, not an attacker's input packet.
