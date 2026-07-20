# Participant runtime-validation tooling (#265)

Real Minecraft Java 26.2 runtime-validation tooling for the #230
participant backends merged in #266. Run with:

```bash
cargo run -p sand-build --bin ensure-server-jar -- 26.2   # downloads a real server jar
python3 scripts/mc_validation/run_audit.py --build --jar ~/.sand/cache/26.2/server.jar
```

This starts a **real** Minecraft Java 26.2 server (downloaded from Mojang's
own version manifest — not a mock, not a simulation), loads the
`examples/participant_audit` datapack (built with the actual merged #266
`sand` facade, `EntityDamagePlayerEvent`/`PlayerKillEvent`/
`PlayerDamageEntityEvent`/`EntityKillEvent` participant plans), and reports
results by category.

## What is proven, and how

| Category | Status | Evidence |
|---|---|---|
| Real server startup | ✅ Proven | `run_audit.py`'s `startup` check: actual `java -jar server.jar`, waits for the real `Done (...)! For help` line, fails on any logged exception/datapack error. |
| Real datapack load | ✅ Proven | `datapack list` via real RCON shows `file/paudit` enabled; zero load errors in the server log. |
| Real `/reload` | ✅ Proven | `reload` command issued over real RCON; `datapack list` re-checked afterward; the actual merged-#266 generated functions (including the automatic participant-plan splicing) are what gets reloaded. |
| Generated functions execute without error | ✅ Proven | `function paudit:init` run over real RCON; the audit storage schema (`paudit:audit`) is inspected afterward via `data get storage paudit:audit` and shows the expected initialized shape. |
| A real player *can* join a real 26.2 server | ✅ Proven | `minimal_join_client.py` performs a real (not simulated) Handshake → Login → Configuration → Play handshake; the server log shows `<name> logged in with entity id N` and `<name> joined the game` on every run. |
| Player-triggered combat scenarios (attacker/killer/weapon capture) | ❌ **Not proven** | See "What is not proven" below. |
| Two independent player sessions | ❌ **Not proven** | Blocked by the same gap — see below. |

## What is not proven, and exactly why

The custom minimal client (`minimal_join_client.py`) reliably completes a
real login and enters the Play state — but the server disconnects it
(`lost connection: Timed out`, or occasionally a decode error while this
tooling's packet-id guesses were still being corrected) within roughly one
server tick of entering Play, consistently too fast for a scripted
follow-up (e.g. an RCON-issued `/damage` targeting that player) to land
before the connection drops.

Root cause was **not** conclusively identified in the time available:

- Minecraft Java 26.2 (protocol version 776) is recent enough that no
  official protocol documentation exists yet, and `minecraft-data`'s
  published protocol definitions do not yet cover it either (only the
  version *number* is catalogued there; the closest full reference
  available was `pc/1.21.11/protocol.json`, used to derive every packet id
  in `minimal_join_client.py` — cross-checked empirically against the real
  server's actual byte-level responses, not assumed).
- Every packet id this tooling relies on (compression threshold, login
  success, configuration select-known-packs/finish-configuration,
  play-phase keep-alive/login) was independently confirmed correct by
  observing the real server's behavior. What remains missing is very
  likely one additional serverbound acknowledgement introduced in a
  version this recent (a plausible candidate is a "player loaded"-style
  packet some newer server implementations require shortly after Play
  starts) that this tooling does not yet send.
- A real end-to-end RCON-triggered damage attempt against a joined bot was
  attempted (see PR history) and did not reliably land before disconnect;
  it is not included in `run_audit.py` because it does not yet produce
  trustworthy results, and this PR's scope is validation tooling, not
  protocol-client redesign.

**Do not extend the claims in `docs/testing/participant-role-evidence.md`
beyond what this README documents.** When the Play-phase connection is made
stable (tracked as follow-up), extend `run_audit.py` with the full combat
scenario matrix from the #265 issue (repeated attacks, two concurrent
subjects, weapon/custom-data snapshots, empty-hand behavior, `/reload`
before/after) and update the evidence doc accordingly.

## Manual validation procedure (until the above is resolved)

Until an automated client can hold a stable Play-phase connection, the
combat/weapon scenarios require a **real Minecraft 26.2 client** (the
official game, not a bot):

1. `python3 scripts/mc_validation/run_audit.py --build --jar <server.jar>`
   confirms structural readiness first.
2. Start the same server manually (`cargo run -p sand-cli --bin sand -- run
   --offline` from `examples/participant_audit/`, or point a real client at
   the server started by `run_audit.py` before it auto-stops — increase
   `--timeout` and remove the final `stop` call for a manual session).
3. Join with two real Minecraft Java 26.2 clients (or one client plus
   controlled entities via `/summon`).
4. Player A: get hit by a summoned hostile mob (`/summon zombie ~ ~1 ~
   {Tags:[audit_x]}`, let it attack, or `/damage @s 4 mob_attack by
   @e[tag=audit_x]`). Check `/data get storage paudit:audit` — expect
   `attacker.present: 1b` and a real `attacker.uuid` matching the summoned
   mob (`/data get entity @e[tag=audit_x] UUID`).
5. Repeat rapidly (multiple hits within ~1 second) — confirm each
   occurrence gets a fresh, correct attacker binding
   (`paudit_seq`/`paudit_att1` increments each time; no stale UUID from a
   previous hit).
6. Player B joins concurrently; both players get hit by different mobs in
   an overlapping window; confirm each player's own last-known values in
   `paudit:audit` reflect *their own* attacker, not the other player's
   (note: the current audit pack uses one shared storage path per role, not
   per-player — for genuine concurrent-player isolation evidence, tag the
   storage write with the player's own UUID first, or capture immediately
   after each individual hit before the next player's hit overwrites it).
7. Player attacks an entity with a custom-data weapon in hand
   (`PlayerDamageEntityEvent`/`EntityKillEvent`) — confirm
   `weapon.item`/`kill_weapon.item` in storage contains the correct item id
   and custom data, and that switching held items immediately afterward
   does not retroactively change the stored snapshot.
8. Empty mainhand: confirm `weapon.present: 0b` and `pwpn_pres` reads `0`.
9. `/reload` mid-session (after step 4) — confirm subsequent hits still
   produce correct attacker/weapon evidence.

Record the actual observed values (not just pass/fail) in
`docs/testing/participant-role-evidence.md` once this manual pass runs.
