# Player equipment and participant evidence

The Minecraft 26.2 regression uses two real passive clients and the typed
`examples/participant_audit` pack. It passed locally on October 13, 2026 and
is included in the Vanilla reload workflow.

```sh
(cd examples/participant_audit && cargo run --manifest-path ../../sand-cli/Cargo.toml -- build)
python3 scripts/validate_player_participants.py \
  --version 26.2 \
  --jar ~/.sand/cache/26.2/server.jar \
  --pack examples/participant_audit/dist/paudit
```

The disposable offline server binds only to loopback. Server/client logs remain
in the printed directory; an explicit `--output` directory is replaced, as in
the base reload harness. Java 25 must be available.

## Reproduced bug and correction

Before the correction, a real player held a shield whose NBT appeared at
`equipment.offhand`, while Sand generated `Inventory[{Slot:-106b}]`. The
tracked health-loss handler ran and captured the main-hand sword, but reported
`tracked_offhand_present: 0b`. This was an absent-data bug, not a parser error.

Sand now uses named equipment compounds for player offhand/armor reads and
non-player hand/armor reads. Player main-hand reads retain `SelectedItem`.
The same canonical mapping serves item locations, typed equipment capabilities
and participant snapshots. The obsolete `ArmorItems`/`HandItems` lists and
player armor/offhand `Inventory` indices are no longer emitted by these paths.
No public endpoint or reliability label was added.

Independent vanilla probes verified all six equipment slots on a zombie in
26.1 and 26.2, plus the corresponding player paths in 26.2. The 26.1 probe did
not include a connected player. Body equipment remains unsupported.

## Automated assertions

The audit equips two players and two distinct, persistent, non-AI zombies.
Items have independent custom-data markers. The harness records the live item
compounds and UUIDs in its own oracle storage before triggering the scenarios.
It checks that both comparison operands exist before testing equality and logs
both values; missing data cannot pass as an unchanged copy.

For each player, a server-issued `/damage ... by <zombie>` fires the real
`entity_hurt_player` advancement and Sand's normal tick observation:

- Both advancement handlers capture the independently recorded attacker's UUID
  and the correct player UUID.
- The tracked native-health transition captures its declared attacker and
  mainhand/offhand participants. Its first sample is silent.
- Captured hand compounds match the independently recorded item contents.
  Changing the live hands afterward leaves the captured copies unchanged.
- Typed player-helmet and correlated-attacker helmet reads copy the expected
  equipment compounds.
- Interleaving Alpha and Beta does not increment the other player's counters
  or reuse that player's item/UUID evidence. Temporary participant tags are
  absent after each dispatch.
- Reload preserves baselines and counters. A subsequent empty-handed Alpha
  occurrence reports both hand participants absent; Beta's count stays fixed.

A roof prevents sunlight from changing zombie helmet durability between oracle
capture and observation. An initial uncovered run detected that change through
the exact-compound comparison, rather than treating it as successful evidence.

## Scope boundary

The hand snapshots above belong to a tracked health observation and have its
sampling time. They do not prove the separate attacking-player weapon event's
advancement timing. The audit does not manually invoke a generated handler or
grant an advancement, but the damage is server-directed rather than generated
by a client's attack packet.

Killer/bridge events, attacking-player weapon snapshots, composed participant
forwarding and rapid mixed-attacker timing still require their own assertions.
The two-player checks cover the controlled interleaving described above, not
every concurrency or dimension case. #265 remains open.
