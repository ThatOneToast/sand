# Scripted player melee: bounded candidate for #265

## Validation status

The source candidate adds an explicit two-player melee command using the existing
`participant_audit` export and passive client's opt-in controls. Focused Python
tests pass. **Native validation has not run for this candidate:** Cargo, native
server runs and full checks are held for the coordinated resource slot. The
assertions below describe the intended proof, not acquired runtime evidence.
No participant reliability label changes. #265 remains open.

After exporting `examples/participant_audit` through the normal build, run:

```sh
python3 scripts/validate_player_melee.py \
  --version 26.2 \
  --jar ~/.sand/cache/26.2/server.jar \
  --pack examples/participant_audit/dist/paudit
```

The existing disposable server harness chooses a port via loopback bind-zero,
binds Minecraft to `127.0.0.1`, and uses offline mode with RCON disabled. Its
normal private temporary output directory retains server/client logs and local
command/status files. An explicit `--output` directory is replaced by the base
harness; use a dedicated disposable path. Do not upload server jars or world
directories as evidence. No credentials are used by these controls.

## Intended native assertions

Two players remain connected throughout sequential reciprocal, nonfatal attacks.
Server commands establish a flat arena, survival mode, health and distinct
custom-data unbreakable swords. Each client acknowledges its server teleport and
sends one small grounded position/rotation update. The server's own position NBT
must reflect that movement. Initial attack cooldown readiness requires 21
observed game ticks; this is not a rapid-hit or same-tick claim.

Each attacker sends exactly one native attack packet against the other player's
currently tracked UUID. The server independently verifies the expected offline
UUID and records its full UUID/item NBT in harness-owned oracle storage. Wire
IDs come from vanilla add-entity packets, not from Sand's participant output.
Removal, ID reuse and respawn invalidate obsolete mappings.

The command acknowledgment only means bytes were sent. A positive vanilla
`minecraft.custom:minecraft.damage_dealt` statistic and a nonfatal victim health
decrease establish native acceptance independently of Sand. Only then do the
generated damage advancement counters, both attacker captures, victim identity,
weapon presence and full-item snapshot comparisons establish participant behavior.
The other player's event counters and native statistic must remain unchanged.
Clearing the attacking hand afterward must leave its captured item unchanged;
temporary participant tags must be absent in all vanilla dimensions.

The harness never supplies damage through `/damage`, grants advancements, or
calls generated handlers. The existing initialization function only prepares the
fixture. No framework or fixture API changes are needed.

## Controls and failure boundary

Without `--control-dir`, the client retains passive behavior. With it, one session
loop owns **all** outbound packet writes: controls, keep-alives, loading and
teleport acknowledgments cannot interleave bytes. Incoming packets are drained
before mailbox processing. A partial-frame receive timeout fails the opt-in
session rather than restarting framing at an arbitrary byte.

The harness atomically replaces `command.json` in a per-player private directory
and waits for `status.json`. There is no listening control port or shell command
execution in this interface. Command IDs must increase from 1 through at most
32; each command is at most 4096 bytes, has a monotonic deadline at most 30
seconds away, and supports exactly `attack` or `move_pos_rot`. The latter accepts
finite bounded x/y/z/yaw/pitch values and sends grounded/no-horizontal-collision
flags. The unchanged acknowledged mailbox is not sent again. The harness uses
at most a 20-second command deadline and never retries an attack.

Status reports client loading readiness, teleport acknowledgments, the current
UUID-to-wire-ID map, last **sent** command ID, and control errors. It never reports
accepted hits. A healthy unmet readiness, position or native-hit condition at the
deadline exits **2 / INCONCLUSIVE**. Every polling stage checks server/client
process liveness and both clients' published protocol errors. Server send/marker
failures, exited processes, protocol errors or a mismatch after native acceptance
exit **1 / FAILED**.
Both outcomes are non-success. Process cleanup uses the existing harness's
finally-protected termination/reaping on every exit. Normal completion disconnects
players from the server before terminating their subprocesses.

The slice does not implement inventory input, mouse aiming/raycast selection,
continuous movement physics, projectiles, simultaneous combat, client tick
simulation or generalized bots. Server setup and one explicit target attack are
deliberate limits. Mixed melee/projectile and broader timing acceptance remain open.

## Primary protocol evidence: Minecraft Java 26.2 / 776

Inspected locally with `javap -c -p` against the cached Mojang server jar,
not an earlier-version protocol wiki. SHA-256 of the inspected files:

| File | SHA-256 |
|---|---|
| Bundled `26.2/server.jar` | `cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5` |
| Inner `26.2/versions/26.2/server-26.2.jar` | `183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8` |

`net.minecraft.network.protocol.game.GameProtocols` registers the following
ordered IDs (the clientbound bundle delimiter occupies ID zero):

| Direction / ID | Class and observed codec |
|---|---|
| Serverbound `0x01` | `ServerboundAttackPacket`: record `entityId`; composite codec is `ByteBufCodecs.VAR_INT` only. **Not** the old interact-action encoding. |
| Serverbound `0x1f` | `ServerboundMovePlayerPacket$PosRot`: three doubles, two floats, one unsigned flags byte. `packFlags` uses bit 0 for grounded and bit 1 for horizontal collision. |
| Clientbound `0x01` | `ClientboundAddEntityPacket`: begins with VarInt wire ID and UUID (two longs); the control client only needs this prefix. |
| Clientbound `0x4d` | `ClientboundRemoveEntitiesPacket`: VarInt count and VarInt IDs. |
| Clientbound `0x31`, `0x48`, `0x52` | Login, player-position and respawn; existing session loop acknowledges loading/teleport, and controls clear their map across login/respawn. |

`net.minecraft.server.network.ServerGamePacketListenerImpl.handleAttack` requires
a loaded, non-spectator player, resolves the entity in the player's current level,
checks world border and attack range, rejects invalid targets and incompatible
items, then calls `ServerPlayer.attack`. Sending the bytes alone cannot establish
that those gates passed; this is why native acceptance is a separate assertion.

Reproduce the relevant codec inspection without launching a server:

```sh
javap -classpath ~/.sand/cache/26.2/versions/26.2/server-26.2.jar -c -p \
  net.minecraft.network.protocol.game.GameProtocols \
  net.minecraft.network.protocol.game.ServerboundAttackPacket \
  'net.minecraft.network.protocol.game.ServerboundMovePlayerPacket$PosRot' \
  net.minecraft.network.protocol.game.ServerboundMovePlayerPacket \
  net.minecraft.network.protocol.game.ClientboundAddEntityPacket \
  net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket \
  net.minecraft.network.FriendlyByteBuf \
  net.minecraft.server.network.ServerGamePacketListenerImpl
```

Focused tests run in the normal lightweight client/harness test groups:

```sh
python3 -m unittest scripts.mc_validation.test_minimal_join_client \
  scripts.tests.test_player_validation_harness -v
```
