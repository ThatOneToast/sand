# Cooperative arena

A small reference game that composes canonical State, `Roster<Role>`, `Session`,
`Progression`, `Combat`, `Scene`, `Hud` and `Menu`. Two arenas have independent
rounds and up to four online members each, including spectators. Players share a
custom-health enemy; each surviving fighter receives one persistent win on victory.

Use a disposable world. With both arena chunks loaded in the overworld, run
`/function cooperative_arena:setup`. This replaces two 13×13 floor regions at
Y=100, centered on X=0/Z=0 and X=32/Z=0, and creates their marker hosts. It does not
force-load chunks or prepare an arbitrary production world.

Build with `sand build`, run with `sand run`, and join your existing Prism instance
with `sand join --instance INSTANCE --local`. As a player, sneak to open the menu:
join A or B, then reopen the menu and start that arena. Menu choices close after
one action. During the active round, sneaking deals five damage to your arena's
core when within eight blocks, with a ten-tick cooldown. The core pulses one
custom-health damage per second; zero health makes a player a gameplay spectator.
The spectator role does not change native Minecraft gamemode or grant flight.

Destroy the core within 60 seconds. Successful fighters earn a saved win; timeout
or no remaining fighters ends the round without victory rewards. After results,
sneak to open a new lobby, join again and start another round. Use
`/function cooperative_arena:menu` to open the menu explicitly during a fight.

`cooperative_arena:reset` stops both loaded controllers, closes rosters and cancels
online player HUDs while preserving wins and platforms. It is not pack uninstall.
Reload closes membership and enters results; it does not resume a running round.
Disconnect discards membership, while death explicitly leaves. Members in another
arena are excluded from damage and rewards. Initial setup/reset are operator
entry points; normal menu and sneak gameplay is intended for survival players.

Current verification is recorded alongside the implementation. Deterministic
export tests alone do not prove rendered UI, real input, all integrated lifecycle
paths, or a server performance envelope. The alpha release requires the native
integration and actual-client checks described in
[the alpha game contract](../../docs/architecture/alpha-game-contract.md).

## Reproduce the headless proof

From the repository root, run the maintained scenario in a new directory:

```sh
python3 scripts/validate_alpha.py --scenario arena \
  --jar /path/to/26.3/server.jar --output /tmp/cooperative-arena-proof
```

The [scenario workflow](../../docs/testing/alpha-workflow.md) explains artifacts,
failure reporting and the remaining input/rendering and performance boundaries.
