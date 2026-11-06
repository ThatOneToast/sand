//! Interaction composition using canonical State, inventory and world APIs.
use sand::component::{ItemComponent, ItemStack, WrittenBook};
use sand::inventory::InventoryIndex;
use sand::prelude::*;
use sand::systems::{Interactable, InventoryTransfer, Lifetime, Menu, Purchase};
use sand::world::WorldContext;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "interaction_hub", scope = player)]
struct Quest {
    #[state(default = 10)]
    coins: Score,
    key: Flag,
    gate: Flag,
    reward: Flag,
}

fn stack(item: &str) -> ItemStack {
    ItemStack::new(ItemId::minecraft(item).unwrap())
}

#[datapack_component]
fn journal() -> Purchase {
    Purchase::new(
        "interaction_hub:journal".parse().unwrap(),
        stack("written_book").component(ItemComponent::WrittenBook(
            WrittenBook::new("The locked gate", "Sand")
                .page(Text::new(
                    "Buy the key, use the gate, then claim your reward.",
                ))
                .page(Text::new(
                    "Your key and reward claims persist in player State.",
                )),
        )),
        InventoryIndex::new(1).unwrap(),
    )
    .on_rejected(|| cmd::say("Clear hotbar slot 2 to receive the journal."))
}

#[datapack_component]
fn key() -> Purchase {
    Purchase::new(
        "interaction_hub:key".parse().unwrap(),
        stack("tripwire_hook"),
        InventoryIndex::new(0).unwrap(),
    )
    .cost(Quest::coins, 3)
    .claim_once(Quest::key)
    .on_success(|| journal().buy())
    .on_rejected(|| {
        cmd::say("You need 3 coins, an empty first hotbar slot, and no prior key claim.")
    })
}

#[datapack_component]
fn reward() -> Purchase {
    Purchase::new(
        "interaction_hub:reward".parse().unwrap(),
        stack("diamond").count(3),
        InventoryIndex::new(2).unwrap(),
    )
    .claim_once(Quest::reward)
    .on_success(|| cmd::say("Quest complete!"))
    .on_rejected(|| cmd::say("Clear hotbar slot 3; this reward can be claimed once."))
}

// A quantity-aware barter: three emeralds from hotbar slot 4 pay for
// one diamond in slot 5. The reward never doubles as its own payment source.
#[datapack_component]
fn barter() -> Purchase {
    Purchase::new(
        "interaction_hub:barter".parse().unwrap(),
        stack("diamond"),
        InventoryIndex::new(4).unwrap(),
    )
    .item_cost(
        InventoryIndex::new(3).unwrap(),
        ItemId::minecraft("emerald").unwrap(),
        3,
    )
    .on_success(|| cmd::say("Barter complete."))
    .on_rejected(|| cmd::say("Put at least 3 emeralds in hotbar slot 4 and clear slot 5."))
}

#[datapack_component]
fn move_barter_reward() -> InventoryTransfer {
    InventoryTransfer::new(
        "interaction_hub:move_barter_reward".parse().unwrap(),
        InventoryIndex::new(4).unwrap(),
        InventoryIndex::new(5).unwrap(),
    )
    .on_rejected(|| cmd::say("Slot 5 must hold a reward and slot 6 must have room."))
}

#[datapack_component]
fn split_payment_stack() -> InventoryTransfer {
    InventoryTransfer::new(
        "interaction_hub:split_payment".parse().unwrap(),
        InventoryIndex::new(3).unwrap(),
        InventoryIndex::new(6).unwrap(),
    )
    .amount(3)
    .on_rejected(|| cmd::say("Slot 4 needs at least three items and slot 7 needs room."))
}

#[datapack_component]
fn shop_menu() -> Menu {
    Menu::new(
        "interaction_hub:shop_menu".parse().unwrap(),
        Text::new("Gatekeeper"),
        Ticks::new(600),
    )
    .choice(
        Text::new("Barter 3 emeralds in slot 4"),
        Condition::all([]),
        || barter().buy(),
    )
    .choice(
        Text::new("Move reward from slot 5 to slot 6"),
        Condition::all([]),
        || move_barter_reward().transfer(),
    )
    .choice(
        Text::new("Split 3 items from slot 4 into slot 7"),
        Condition::all([]),
        || split_payment_stack().transfer(),
    )
    .choice(
        Text::new("Buy key — 3 coins"),
        !Quest::key.bind().is_enabled(),
        || key().buy(),
    )
    .choice(Text::new("Get journal"), Condition::all([]), || {
        journal().buy()
    })
}

#[datapack_component]
fn shop() -> Interactable {
    Interactable::new(Lifetime::new(
        "interaction_hub:shop".parse().unwrap(),
        Ticks::new(6000),
    ))
    .bounds(1.0, 2.0, 5.0)
    .on_use(|| shop_menu().open())
}

#[datapack_component]
fn gate() -> Interactable {
    Interactable::new(Lifetime::new(
        "interaction_hub:gate".parse().unwrap(),
        Ticks::new(6000),
    ))
    .on_use(|| {
        when(Quest::key.bind().is_enabled()).then(mcfunction![
            Quest::gate.bind().enable();
            WorldContext::current().block(BlockPos::here()).replace(vanilla::Block::Air);
            cmd::say("The gate is unlocked.");
        ])
    })
}

#[datapack_component]
fn chest() -> Interactable {
    Interactable::new(Lifetime::new(
        "interaction_hub:chest".parse().unwrap(),
        Ticks::new(6000),
    ))
    .on_use(|| when(Quest::gate.bind().is_enabled()).then(reward().buy()))
}

#[function("interaction_hub:place_shop")]
fn place_shop() {
    shop().start();
}
#[function("interaction_hub:place_gate")]
fn place_gate() {
    gate().start();
}
#[function("interaction_hub:place_chest")]
fn place_chest() {
    chest().start();
}
#[function("interaction_hub:cleanup")]
fn cleanup() {
    shop().cancel();
    gate().cancel();
    chest().cancel();
    shop_menu().close();
}

#[doc(hidden)]
pub fn __sand_export(namespace: &str, version: &str) {
    println!(
        "{}",
        sand::advanced::try_export_pack_json(namespace, version).unwrap()
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_targets_export_deterministically() {
        for version in ["26.1", "26.2", "26.3"] {
            let first = sand::advanced::try_export_pack_json("interaction_hub", version).unwrap();
            assert_eq!(
                first,
                sand::advanced::try_export_pack_json("interaction_hub", version).unwrap()
            );
            assert!(first.contains("written_book_content"));
            assert!(first.contains("interaction_use"));
            assert!(first.contains("purchase_place"));
        }
    }
}
