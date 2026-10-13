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

## Subject-scoped dispatch and composed evidence

A second live regression exposed #429: the composed tick parent's capture ran
in the server tick context, before `execute as @a`. The gate reset and handlers
ran, but parent, child and sibling attacker UUIDs were absent. Fresh damage did
not fix this execution-context error.

The compiler now retains the canonical participant plan separately from global
`EventSetup` commands. Capture runs inside the selected subject's dispatch.
Immediate descendants share that binding; staged graphs finish one subject's
cycle before advancing to another, resetting temporary evidence at both ends.
Global root lifecycle commands still run once per tick. The obsolete manual
`EventSetup::with_participants` endpoint is removed; declare the plan through
`SandEvent::participants` instead.

The audit compares the parent's, child's and sibling's captured UUIDs with the
independent attacker oracle for each player. A never-hit Beta following Alpha's
capture must produce no attacker evidence. Generated-output regressions also
check staged fan-in, absent-source scratch reset, global lifecycle placement,
and bounded per-subject snapshot persistence and expiry.

A real staged fan-in scenario places Alpha in the Overworld and Beta with its
attacker in the Nether, freezes game ticking, queues both players, and advances
exactly one tick. Both players must capture their own distinct attacker
UUID and main-hand compound. A second single tick fires only the alternate
parent: each named-source entity and item must then be absent, with no stale
field copied from either player's previous occurrence. Ticking is resumed in a
finally block, including on assertion failure, and Beta returns to the Overworld.
Temporary-tag cleanup is checked in all three vanilla dimensions. A real hit in
the Nether must succeed within the bounded setup timeout; the later death event
after returning must also fire. The passive client acknowledges protocol-776
position packets and loading after login/respawn. Without those acknowledgments,
vanilla leaves a dimension-changing player invulnerable; the original probe
failed the returning player's death assertion and the corrected client passes.
Teleport acknowledgment does not implement movement or combat input.

## Weapon and death timing

Each player damages and kills a separately summoned cow through `/damage ... by
<player>`. The real advancements must capture the equipped weapon, agree with
the full independent item oracle, and preserve their evidence after the live
hand is cleared. The other player's kill counter stays unchanged.

Real fatal damage then checks each player's killer and the same-cycle bridge's
inherited killer. The bridge also observes the victim's own main hand. With
`keep_inventory` enabled, the captured item is present and stable after a later
inventory edit. With normal drops, vanilla clears inventory before the death
advancement fires, so the snapshot is absent. Tests remove prior evidence first
and reject stale item data. These outcomes preserve the post-trigger snapshot
contract rather than claiming an exact pre-death item.

## Scope boundary

The hand snapshots above belong to a tracked health observation and have its
sampling time. Separate attacking-player scenarios verify weapon snapshots at
advancement dispatch; they do not reconstruct a pre-hit state vanilla already
changed. The audit does not manually invoke a generated handler or
grant an advancement, but the damage is server-directed rather than generated
by a client's attack packet.

The extended audit also exercises actual attacking-player damage/kill advancements,
real player deaths, and tick-composed parent/child/sibling participants. Rapid
mixed-attacker timing still requires its own assertions.
The two-player checks cover the controlled interleaving described above, not
every concurrency or dimension case. #265 remains open.
