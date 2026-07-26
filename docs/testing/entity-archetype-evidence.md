# Entity archetype runtime evidence (issue #295)

Validation date: 2026-07-26.

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

The final run passed all 16 live checks:

| Check | Live result |
|---|---|
| Real server startup and pack load | PASS |
| Real `/reload`, with zero server-log errors | PASS |
| Previously unmarked Zombie adopted once | PASS |
| Two Zombies received independent initial state | PASS |
| Initial max health 20 and attack damage 3 | PASS |
| Initial colored `Lv. 1 Plagued Zombie` name | PASS |
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
entries. Live RCON output included:

```text
The value of attribute Max Health for entity Lv. 2 Plagued Zombie is 22.0
The value of attribute Attack Damage for entity Lv. 2 Plagued Zombie is 4.0
Lv. 2 Plagued Zombie has the following entity data: 11.0f
Storage rpg:__sand_entity has the following contents: {}
```

Two independent 26.2 component exports were byte-identical at
`b431d77e8eefd6baae491e0156635b77492a7410a36c2727079eb1c0f35a1d9c`
(SHA-256). Two complete `sand build` runs produced the same 46-file pack-tree
hash,
`99f9d16efb6862a21a5d28f7ab3f4b340a7740c51ba63afa8c2e64639bee7d97`.
The resulting unpacked datapack occupied 188 KiB on the validation host.

## Spawn-provenance boundary

The harness uses command-summoned Zombies with no archetype marker. They pass
through the exact type-constrained adoption scan used by naturally spawned
Zombies; this proves the scan and initialization behavior on real entities.
It does **not** prove that a random mob-spawning cycle occurred. A headless
26.2 server with no stable player session cannot force a genuine natural
hostile spawn deterministically. The harness records
`natural_spawn_provenance: false`, and this limitation must not be described
as a natural-spawn live proof.
