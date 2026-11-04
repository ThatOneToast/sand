# Player inventory workflow native proof

The 2026-11-03 run of `scripts/validate_inventory.py` passed on vanilla **26.2**
with two connected survival players. It exercised the registered
`InventoryTransfer` and `Purchase::item_cost` APIs, using real live player slots:

- An empty-slot move preserved the exact count and custom components and emptied
  its source. A second player's inventory remained independent.
- Equal serialized component stacks with an explicit maximum size merged to the
  expected count. Different custom data, full capacity, an empty source and an
  absent explicit capacity rejected without changing either stack.
- A native `inventory_changed` advancement listener reentered a Sand transfer
  while the inventory lock was held; the inner attempt made no mutation.
- Listeners replaced the source during an empty-slot copy and during a merge.
  Both preserved the listener's replacement. The first cleared Sand's duplicate
  copy; the second restored the destination's prior count.
- A listener replaced the destination. Sand preserved that replacement and kept
  the original source instead of clearing it or overwriting listener-owned data.
- Whole-slot purchases rejected insufficient/excess quantities and a full reward
  slot, consumed the exact payment after successful delivery, rejected replay
  without payment, and isolated two players' payment inventories.
- A native placement listener replaced the payment stack. Compensation cleared
  the new reward and preserved the replacement payment item.
- A canonical component-filter payment rejected an unmarked emerald and accepted
  a matching custom-data marker even with an unrelated additional key.

Reproduce with a local 26.2 jar:

```sh
python3 scripts/validate_inventory.py --jar /path/to/server.jar --output /tmp/sand-inventory-proof
```

The [machine-readable evidence](../../benchmarks/evidence/inventory-workflows-2026-11-03/evidence.json)
and [compressed server log](../../benchmarks/evidence/inventory-workflows-2026-11-03/native-run.log.gz)
record the successful run. Export regressions separately cover deterministic
26.1, 26.2 and 26.3 output; this native proof does not claim 26.3 real-player
protocol validation or unloaded-container transfer support.

See the [authoring contract](../gameplay/inventory-workflows.md) for conservative
capacity handling and the remaining bounds. Compensation acts only on Sand's
own unchanged write; it cannot roll back arbitrary effects performed by a native
listener, or later changes triggered by the compensation itself.

## Resource namespace isolation regression

The 2026-11-04 native 26.2 rerun used strict emitted-chat marker matching and
additionally invoked an InventoryTransfer,
Purchase and Journal from three different resource namespaces inside one pack.
A native inventory listener attempted all three while the original move held
its lock: no foreign slot changed and neither success nor rejection callbacks
ran. After the outer operation unlocked, all three workflows completed normally.
This proves that importing a differently namespaced definition cannot bypass
the pack's shared inventory mutation lock.

The [result](../../benchmarks/evidence/inventory-lock-2026-11-04/evidence.json)
and [native log](../../benchmarks/evidence/inventory-lock-2026-11-04/native-run.log.gz)
retain the run. Export regressions separately assert one shared lock across all
three systems/resource namespaces and distinct locks for independent pack
exports on 26.1, 26.2 and 26.3.
