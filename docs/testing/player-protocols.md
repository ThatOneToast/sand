# Versioned passive-player protocol evidence

The maintained minimal offline-mode client supports **26.2 / protocol 776** and
**26.3 / protocol 777** for passive native gameplay audits. `PlayerHarness`
selects the protocol from its exact `args.version`; unknown versions fail before
server preparation or client creation. This is a tested transport prerequisite,
not a claim that all gameplay scenarios have run on both versions.

The 2026-11-03 dedicated runs passed on both official server artifacts with two
connected survival players. Each run proved:

- Login, configuration, Play login and initial player-loaded acknowledgments.
- Native keepalive challenge responses from both clients.
- Independent score and inventory commands for each real ServerPlayer.
- Nether and Overworld transitions, teleport/loading responses and accepted
  native damage after transition. The fixture requires full initial health,
  a successful native damage command result, and its 20-to-19 health change;
  environmental damage alone cannot satisfy the assertion.
- Disconnect/reconnect under the same player identity, retained score/inventory,
  and the other player's uninterrupted independent connection.

## Authoritative packet data

The [26.2 registry metadata](../../scripts/mc_validation/protocol_metadata/26.2.json)
and [26.3 registry metadata](../../scripts/mc_validation/protocol_metadata/26.3.json)
come from each cached official bundled server jar. They pin the outer SHA-1 and
SHA-256, `version.json`, verified nested-jar digest, relevant class hashes and
full login/configuration/Play registration tables.

`scripts/mc_validation/extract_protocol_metadata.py` uses `javap -c -p` on the
actual `LoginProtocols`, `ConfigurationProtocols` and `GameProtocols` classes.
`ProtocolInfoBuilder` appends codec entries in registration order, and
`ProtocolCodecBuilder` assigns sequential IDs. The clientbound bundle entry
occupies the delimiter's ID zero. Both builder bytecodes are retained in the
compressed evidence. IDs are extracted independently; no uniform offset from
an older protocol is assumed.

| Packet | 26.2 | 26.3 |
|---|---:|---:|
| Configuration clientbound known packs | `0x0e` | `0x0f` |
| Play clientbound keepalive | `0x2c` | `0x2d` |
| Play clientbound login | `0x31` | `0x32` |
| Play clientbound player position | `0x48` | `0x49` |
| Play clientbound combat kill | `0x44` | `0x45` |
| Play clientbound respawn | `0x52` | `0x54` |

26.3 also changes the serverbound teleport acknowledgment **payload**, despite
retaining packet ID zero. The old reply contains only a VarInt teleport ID;
777 requires that ID followed by three doubles and two floats for absolute
position and rotation. The client decodes the server's position/velocity/
rotation/relative-flags packet, follows `PositionMoveRotation.calculateAbsolute`
and the `Relative` bitmask, clamps pitch, and sends the required fields. Invalid
or truncated 777 teleports fail rather than emitting a fabricated acknowledgment.
The native proof exercises same-dimension teleports, both dimension transitions
and reconnect; unit tests separately exercise relative position/rotation flags.

Regenerate a profile from the exact local artifact with the JDK required by that
server version:

```sh
python3 scripts/mc_validation/extract_protocol_metadata.py \
  --jar /path/to/official-server.jar \
  --output /tmp/protocol-metadata.json \
  --bytecode-output /tmp/protocol-bytecode.txt
```

## Reproduction and evidence

```sh
python3 scripts/validate_player_protocol.py --version 26.3 \
  --jar /path/to/26.3-server.jar --output /tmp/sand-player-protocol-26.3
python3 scripts/validate_player_protocol.py --version 26.2 \
  --jar /path/to/26.2-server.jar --output /tmp/sand-player-protocol-26.2
```

The proof rejects a jar whose digest or `version.json` disagrees with the
profile. Use a new output directory. Every completed assertion and any failure
are recorded in `evidence.json`; server and client logs remain available.

- **26.3:** [result](../../benchmarks/evidence/player-protocols-2026-11-03/26.3/evidence.json),
  [native server log](../../benchmarks/evidence/player-protocols-2026-11-03/26.3/native-run.log.gz),
  [client session](../../benchmarks/evidence/player-protocols-2026-11-03/26.3/SandAlpha.log.gz),
  [reconnect](../../benchmarks/evidence/player-protocols-2026-11-03/26.3/SandAlpha-2.log.gz),
  [second client](../../benchmarks/evidence/player-protocols-2026-11-03/26.3/SandBeta.log.gz),
  [authoritative bytecode](../../benchmarks/evidence/player-protocols-2026-11-03/26.3/protocol-bytecode.txt.gz).
- **26.2 regression:** [result](../../benchmarks/evidence/player-protocols-2026-11-03/26.2/evidence.json),
  [native server log](../../benchmarks/evidence/player-protocols-2026-11-03/26.2/native-run.log.gz),
  [authoritative bytecode](../../benchmarks/evidence/player-protocols-2026-11-03/26.2/protocol-bytecode.txt.gz).

Scripted movement/melee through `--control-dir` remains explicitly restricted to
776; its active-input codecs and native acceptance have separate evidence. This
client does not render GUIs, submit arbitrary text, emulate physical mouse input,
implement online authentication or provide a general Minecraft client. The new
proof does not itself broaden interaction/gameplay capability claims; maintained
scenario suites must run on 26.3 before their version-specific evidence broadens.
