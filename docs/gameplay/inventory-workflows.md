# Bounded inventory moves and payments

`InventoryTransfer` and `Purchase::item_cost` compose with menus, interactions,
canonical inventory slots, item predicates and State. Both are registered
components invoked in player context. The maintained authoring example is
[`interaction_hub`](../../examples/interaction_hub/src/lib.rs): its merchant
accepts exactly three emeralds from hotbar slot 4, delivers a diamond into slot 5,
and offers a separate move from slot 5 to slot 6.

```rust,ignore
#[datapack_component]
fn move_reward() -> InventoryTransfer {
    InventoryTransfer::new(
        "shop:move_reward".parse().unwrap(),
        InventoryIndex::new(4).unwrap(),
        InventoryIndex::new(5).unwrap(),
    )
}
```

## Whole-stack moves

`InventoryTransfer::transfer()` moves the complete live source into an empty
numbered slot of the invoking player. Native copy preserves the count and every
component, and a failed destination write leaves the source untouched. Both full
live stacks are rechecked after destination mutation and before source removal.
If a native listener replaces the source, Sand preserves that replacement and
compensates its own still-unchanged destination write: an empty-slot copy is
cleared, or a merge returns to the prior destination count. If the listener also
changes the destination, Sand leaves that listener-owned stack untouched and
rejects without clearing the source. These checks do not undo arbitrary listener
effects or recursively compensate later listener changes. An empty
source rejects. Source and destination must differ; aliases fail export.

An occupied destination accepts a merge only when both full serialized item
compounds agree after removing the slot number and count. Comparison uses native
NBT equality through the success of a storage replacement; nested component
compounds are compared completely, rather than using a partial item predicate.
Order of compound keys does not matter; list order and NBT value types do.
Serialized distinctions such as omitted versus empty components conservatively
reject rather than risk merging different stacks.

Merging also requires an explicit live `minecraft:max_stack_size` component in
`1..=99`. The combined count must fit it. Sand's canonical item registry has IDs,
not default stack capacities, so ordinary stacks without this override can move
to empty slots but cannot merge through this increment. Sand does not guess
capacity from an item name. No partial moves, slot searching or dropped overflow
occur. Count bounds, component mismatch and full destinations reject before any
inventory mutation.

## Whole-slot payments

`Purchase::item_cost(slot, predicate, amount)` requires exactly `amount` items
in one numbered slot, distinct from the empty reward destination. The canonical
predicate defines identity and component matching. An item ID alone intentionally
accepts all components; explicit predicate filters narrow the accepted payment.
The exact quantity is additional to the predicate, and must be `1..=99`.
Insufficient and excess quantities both reject without delivery or removal.

Reward placement must succeed first. Sand then compares the complete payment
stack with its pre-placement snapshot and clears that whole slot. It never runs
an inventory-wide `clear` that could accidentally consume the new reward. If a
native placement listener changes the payment stack, the purchase rejects without
clearing the changed payment. Compensation clears the reward only when its full
serialized identity, count and components still equal the canonical expected
reward staged before placement. A listener replacement is preserved, including
the same item ID with a different count or components. Missing stacks and codec
representations whose exact equality cannot be proved are also preserved. This
is best-effort compensation: rejection can leave the delivered reward when
normalization prevents proof. It does not roll back arbitrary listener effects.
Score currency and claims commit after item removal; success callbacks run last.

## Execution and remaining boundaries

One export-pack-wide inventory mutation lock covers both APIs and Journal, even
when their definitions use different resource namespaces. Native listeners that
reenter either workflow while locked are silent no-ops, preventing recursive
rejection callbacks. Success/rejection callbacks execute after scratch storage is
removed and the lock released, so they may start subsequent gameplay. Every live
slot uses `@s`; two players cannot pay from or move each other's inventory.
Load clears ephemeral scratch and releases the lock.

This increment supports numbered player inventory slots only. It does not cover
containers, unloaded container failures, arbitrary entity targets, partial
quantities, general transactions, or rollback of other packs' commands. Existing
`ItemLocation::replace_from` and `EntityInventory::remove` retain their explicit
copy and remove-up-to semantics; appending them does not invent a safe move.

## Evidence

Export regressions in `sand-core/tests/gameplay_inventory.rs` cover deterministic
26.1/26.2/26.3 output, admission/mutation ordering, shared lock placement, exact
component comparison, source removal and payment compensation. Unit regressions
reject aliased slots and invalid quantities. `scripts/validate_inventory.py`
provides a dedicated 26.2 native two-player proof for count/component preservation,
merging and rejection, listener reentry, listener-replaced source/destination
compensation, whole-slot payment and owner isolation.
Native proof results should be recorded with the version actually executed.

The [2026-11-03 native evidence](../testing/inventory-workflows.md) records the
successful 26.2 two-player run, including source-replacement compensation and
preservation of listener-replaced destinations.
