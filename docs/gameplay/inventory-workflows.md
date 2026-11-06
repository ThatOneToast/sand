# Quantity-aware inventory moves and payments

`InventoryTransfer` and `Purchase::item_cost` compose with menus, interactions,
canonical inventory slots, item predicates and State. Both are registered
components invoked in player context. The maintained authoring example is
[`interaction_hub`](../../examples/interaction_hub/src/lib.rs): its merchant
consumes three emeralds from hotbar slot 4, delivers a diamond into slot 5,
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

## Stack moves and splitting

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

Merging queries the native effective `minecraft:max_stack_size` component, including
ordinary item defaults and explicit overrides. It never guesses capacity from an
item name or treats a missing serialized override as zero. Common capacities
64, 16 and 1 are queried first; custom capacities have a finite 99-query bound.
`ItemPredicate::max_stack_size` exposes this same typed component constraint.

Use `.amount(3)` on an `InventoryTransfer` to move exactly three items and retain
the source remainder. Omit it to move the whole stack. Insufficient stock or
capacity rejects before mutation. A typed copy modifier writes only the requested
quantity to an empty destination in one native operation. Source debit happens
only after the complete source/destination snapshots are checked; it preserves
all remaining components. No slot search or dropped overflow occurs.

## Exact-quantity payments

`Purchase::item_cost(slot, predicate, amount)` requires at least `amount` items
in one numbered slot, distinct from the empty reward destination. The canonical
predicate defines identity and component matching. An item ID alone intentionally
accepts all components; explicit predicate filters narrow the accepted payment.
The exact quantity is additional to the predicate, and must be `1..=99`.
Insufficient quantities reject without delivery or removal; excess quantities remain in the payment slot with their components.

Reward placement must succeed first. Sand then compares the complete payment
stack with its pre-placement snapshot and removes only the requested quantity using a native count modifier (or clears an exact-size payment). It never runs
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
containers, unloaded container failures, arbitrary entity targets, general transactions, or rollback of other packs' commands. Existing
`ItemLocation::replace_from` and `EntityInventory::remove` retain their explicit
copy and remove-up-to semantics; appending them does not invent a safe move.

## Evidence

Export regressions in `sand-core/tests/gameplay_inventory.rs` cover deterministic
26.1/26.2/26.3 output, admission/mutation ordering, shared lock placement, exact
component comparison, source removal and payment compensation. Unit regressions
reject aliased slots and invalid quantities. `scripts/validate_inventory.py`
provides version-selected 26.2 and 26.3 native two-player proofs for count/component preservation,
merging and rejection, listener reentry, listener-replaced source/destination
compensation, exact-quantity payment and owner isolation.
Native proof results should be recorded with the version actually executed.

The [2026-11-03 native evidence](../testing/inventory-workflows.md) records the
successful 26.2 two-player run, including source-replacement compensation and
preservation of listener-replaced destinations.

To require an exact whole-stack payment after the quantity-aware change, compose
`item_cost` with `ItemPredicate::count(IntRange::exact(amount))`. The quantity
policy consumes only the requested amount; the predicate remains the explicit
authoritative restriction on which stacks qualify.
