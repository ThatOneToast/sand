# Entity archetype runtime evidence (issues #295 and #362)

Validation date: 2026-09-05.

Environment:

- Minecraft Java server: 26.2, Mojang server jar cached by
  `sand-build ensure-server-jar`
- Java: OpenJDK 25.0.3
- Pack: `examples/rpg_entity`, pack format 107
- Harness: `scripts/mc_validation/run_entity_archetype_audit.py`

Command:

```text
python3 scripts/mc_validation/run_entity_archetype_audit.py \
  --jar ~/.sand/cache/26.2/server.jar \
  --evidence /tmp/entity-archetype-evidence.json
```

The final run passed all 24 live checks. The server was frozen while the
reconciliation function was invoked directly for the health-causality cases,
making observation cadence and simultaneous-write ordering deterministic.

| Check | Live result |
|---|---|
| Real server startup and pack load | PASS |
| Real `/reload`, with zero server-log errors | PASS |
| Previously unmarked Zombie adopted once | PASS |
| Two Zombies received independent initial state | PASS |
| Initial max health 20 and attack damage 3 | PASS |
| Initial colored `Lv. 1 Plagued Zombie` name | PASS |
| Native damage observed into State | PASS |
| Explicit State heal applied to native health | PASS |
| Explicit State damage applied to native health | PASS |
| State heal clamped to maximum health | PASS |
| Simultaneous native damage and dirty State write: State wins | PASS |
| Interval 20 does not observe before its cadence | PASS |
| Interval 20 observes when due | PASS |
| Native observation does not become a self-sustaining refresh loop | PASS |
| Level mutation isolated to one Zombie | PASS |
| Dirty refresh produced max health 22 and attack damage 4 | PASS |
| Current health preserved from 10/20 to 11/22 | PASS |
| Dynamic name refreshed to level 2 | PASS |
| Version-one migration reached version two | PASS |
| Preserve reconciliation retained an unrelated external tag | PASS |
| Function-macro storage was empty after calls | PASS |
| State/native observation resumed after chunk unload/load | PASS |
| Explicit cleanup removed Sand lifecycle state | PASS |
| Both audit Zombies were cleanly removed | PASS |

The server log contained no `ERROR`, `Exception`, or `Failed to load`
entries. Live RCON output included State-to-native healing from 17 to 19,
clamping a 999-point State write to 20, deterministic State precedence at 18,
and:

```text
The value of attribute Max Health for entity Lv. 2 Plagued Zombie is 22.0
The value of attribute Attack Damage for entity Lv. 2 Plagued Zombie is 4.0
Lv. 2 Plagued Zombie has the following entity data: 11.0f
Storage rpg:__sand_entity has the following contents: {}
```

The checked-in `examples/rpg_entity` export test separately verifies that two
independent 26.2 component exports are byte-identical.

## Spawn-provenance boundary

The harness uses command-summoned Zombies with no archetype marker. They pass
through the exact type-constrained adoption scan used by naturally spawned
Zombies; this proves the scan and initialization behavior on real entities.
It does **not** prove that a random mob-spawning cycle occurred. A headless
26.2 server with no stable player session cannot force a genuine natural
hostile spawn deterministically. The harness records
`natural_spawn_provenance: false`, and this limitation must not be described
as a natural-spawn live proof.
