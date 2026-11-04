# Owned presentation evidence

Validated on 2026-11-03. `Scene` and `Hud` compose the canonical `Lifetime`;
items use `ItemStack`, text uses `TextComponent`, and HUD progress uses the
canonical numeric/State lowering.

## Export and validation

`cargo test -p sand-core --test gameplay_presentation --test gameplay_lifetime`
passes on exports targeting 26.1, 26.2 and 26.3. The presentation tests verify
deterministic mixed block/item/text display output, component-bearing stacks,
rich text NBT, shared keyframes and instance-scoped cleanup. HUD output verifies
private per-instance native IDs, a shared per-player actionbar lease, compact
live-resource bookkeeping, offline-capable native removal and reload cleanup.
Private unit tests reject invalid channels, maxima, capacities, text, item
stacks, unrepresentable NBT values and transforms before runtime allocation.
Transform composition tests cover parent scale, yaw and translation and reject
shear instead of changing the authored geometry.

Scene and Journal share one internal component JSON-to-NBT conversion. Export
regressions retain multiline display text, item names and written-book pages
within one native merge command. Values and control-bearing object keys escape
line breaks instead of creating additional physical command lines; null and
integers outside signed NBT bounds remain rejected.

## Native Minecraft 26.3 scenes

```sh
python3 scripts/validate_gameplay.py \
  --jar "$SERVER_26_3_JAR" --output /tmp/sand-presentation-scenes
```

The existing entity-owner gameplay proof passes with additional mixed-scene
assertions: native diamond item displays retain count three and the gold rich
name `Gem\nShard`, including the line break. Their custom-data marker retains
the control-bearing key `mark\nline`. Native text displays retain `Ready\nNow`
and bold styling, and a written-book display retains the exact multiline page
`Page one\nPage two`. Two nearby
owners follow independently, cancellation removes only that owner's block,
item and text parts, and cleanup survives a user callback that returns early.
The full strict-marker gameplay proof passed after the PR505 regressions on
2026-11-04. Evidence, exported resources and server logs are retained under
`/tmp/sand-scene-controls-final`; the committed summary is
[gameplay-framework-evidence.json](gameplay-framework-evidence.json).

## Native Minecraft 26.2 player HUD

```sh
python3 scripts/validate_hud.py \
  --jar "$SERVER_26_2_JAR" --output /tmp/sand-presentation-hud
```

The protocol-776 proof passes with two real players. It checks independent
native bossbar values and one viewer per bar, State-driven updates and clamping,
capacity rejection before presentation, and literal `$(literal)` text without
accidental macro expansion. A HUD from another definition claims the newer
actionbar lease; cancelling the older instance preserves that lease. Winner
cancellation and expiry release it. Offline-owner cleanup removes the native
bar and its live-list entry. Reload removes a bar while its root chunk remains
unloaded and reclaims the persistent ledger. Vanilla runs `minecraft:load` on
the next server tick, which the frozen-world probe advances explicitly.

The namespace regression was rerun on 2026-11-04 with strict emitted-marker
assertions. It starts HUDs from `hud_import_a` and `hud_import_b`.
Both receive lifetime instance ID two, while their pack-wide actionbar claims
remain distinct. Instrumented entry into emitted actionbar bodies proves the
older HUD stops refreshing. Its cleanup preserves the newer claim and the other
player’s slot. Exhausting the claim sequence rejects admission before callbacks.
Reload preserves the claim sequence. A new claim survives cleanup of an old
instance whose root was unloaded during reload, then loaded again afterwards.

The final local run retained evidence at
`/tmp/sand-hud-cross-namespace-strict/evidence.json` and server logs under its
`runtime` directory.

## Bounds and unproven behavior

HUD capacity is at most 128 per definition. Bossbar IDs combine the definition
resource identity with its nonreused lifetime instance token. Actionbar claims
use an independent persistent counter shared across the entire export pack,
including definitions from other resource namespaces. Claims never wrap or
reuse, and exhaustion rejects new actionbar HUD starts. The persistent bossbar
list contains only live entries, not historical IDs. Ordinary stop cleanup
waits for an unloaded root to load; reload’s bossbar drain requires no loaded roots.

Actionbar evidence establishes server-side lease policy, not client-rendered
pixels or packet contents. Other packs can overwrite Minecraft's single native
actionbar slot. Superseded HUDs do not resume when the winner stops. No 26.3
real-player protocol or client rendering claim follows from the 26.2 proof.
Scene interpolation is native visual interpolation, with no physics collision,
forced chunk loading, animation blending or hierarchical bones. Item display
context and text display layout/billboard use native defaults.

The final [26.3 gameplay regression](gameplay-framework-evidence.json) also
passed with strict emitted-chat assertion matching. It includes the mixed scene
parts, ownership, follow/cleanup and the existing combat/task/spatial scenarios.
