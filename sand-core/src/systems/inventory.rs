//! High-level typed inventory helpers.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::systems::inventory::InventorySystem;
//! use sand_core::cmd::{Selector, ItemSlot};
//!
//! // Check mainhand then run a command:
//! let cmd = InventorySystem::for_entity(Selector::self_())
//!     .has("minecraft:diamond_sword")
//!     .in_mainhand()
//!     .run("say has sword");
//!
//! // Replace a slot:
//! let cmd = InventorySystem::for_entity(Selector::self_())
//!     .replace(ItemSlot::MainHand, "minecraft:iron_sword");
//!
//! // Clear items:
//! let cmd = InventorySystem::for_entity(Selector::self_())
//!     .clear_item("minecraft:arrow")
//!     .amount(64);
//! ```

use std::fmt;

use sand_commands::selector::Selector;
use sand_commands::{Execute, ItemSlot};

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::inventory::InventorySystem",
    module = "sand::systems",
    summary = "High-level builder for inventory operations on a single entity.",
    context = "High-level builder for inventory operations on a single entity. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::inventory::InventorySystem;",
    availability = ["Cargo feature: systems-inventory"],
)]
/// High-level builder for inventory operations on a single entity.
#[derive(Debug, Clone)]
pub struct InventorySystem {
    selector: Selector,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::inventory::HasItemCheck",
    module = "sand::systems",
    summary = "Intermediate builder — holds the item string before the slot is specified.",
    context = "Intermediate builder — holds the item string before the slot is specified. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::inventory::HasItemCheck;",
    availability = ["Cargo feature: systems-inventory"],
)]
/// Intermediate builder — holds the item string before the slot is specified.
#[derive(Debug, Clone)]
pub struct HasItemCheck {
    selector: Selector,
    item: String,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::inventory::ClearBuilder",
    module = "sand::systems",
    summary = "Builder for `clear <selector> <item> [<count>]` commands.",
    context = "Builder for `clear <selector> <item> [<count>]` commands. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::inventory::ClearBuilder;",
    availability = ["Cargo feature: systems-inventory"],
)]
/// Builder for `clear <selector> <item> [<count>]` commands.
#[derive(Debug, Clone)]
pub struct ClearBuilder {
    selector: Selector,
    item: String,
}

impl InventorySystem {
    /// Start an inventory operation for the given entity selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::for_entity",
        module = "sand::systems",
        kind = "method",
        summary = "Start an inventory operation for the given entity selector.",
        context = "Start an inventory operation for the given entity selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(selector = "`selector` provides the Minecraft target selection used to start an inventory operation for the given entity selector."),
        returns = "A newly constructed `InventorySystem` configured to start an inventory operation for the given entity selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Selector)  {\n    let inventory_system = sand::systems::inventory::InventorySystem::for_entity(selector);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn for_entity(selector: Selector) -> Self {
        Self { selector }
    }

    /// Begin an item-presence check.
    ///
    /// Chain with `.in_slot(slot)`, `.in_mainhand()`, `.in_any_slot()`, etc. to get
    /// an `Execute` builder that can be finished with `.run(cmd)`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::has",
        module = "sand::systems",
        kind = "method",
        summary = "Begin an item-presence check. Chain with `.in_slot(slot)`, `.in_mainhand()`, `.in_any_slot()`, etc. to get an `Execute` builder that can be finished with `.run(cmd)`.",
        context = "Begin an item-presence check. Chain with `.in_slot(slot)`, `.in_mainhand()`, `.in_any_slot()`, etc. to get an `Execute` builder that can be finished with `.run(cmd)`. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(item = "`item` provides the item value or item predicate used to begin an item-presence check. Chain with `.in_slot(slot)`, `.in_mainhand()`, `.in_any_slot()`, etc. to get an `Execute` builder that can be finished with `.run(cmd)`."),
        returns = "The `HasItemCheck` value produced to begin an item-presence check. Chain with `.in_slot(slot)`, `.in_mainhand()`, `.in_any_slot()`, etc. to get an `Execute` builder that can be finished with `.run(cmd)`.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(inventory_system_value: sand::systems::inventory::InventorySystem, item: impl fmt::Display)  {\n    let has = inventory_system_value.has(item);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn has(self, item: impl fmt::Display) -> HasItemCheck {
        HasItemCheck {
            selector: self.selector,
            item: item.to_string(),
        }
    }

    /// Shorthand for `.has(item).in_slot(slot)`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::has_in",
        module = "sand::systems",
        kind = "method",
        summary = "Shorthand for `.has(item).in_slot(slot)`.",
        context = "Shorthand for `.has(item).in_slot(slot)`. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(slot = "`slot` supplies the slot value used to shorthand for `.has(item).in_slot(slot)`.", item = "`item` provides the item value or item predicate used to shorthand for `.has(item).in_slot(slot)`."),
        returns = "The `Execute` value produced to shorthand for `.has(item).in_slot(slot)`.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(inventory_system_value: sand::systems::inventory::InventorySystem, slot: impl Into < sand::command::ItemSlot >, item: impl fmt::Display)  {\n    let has_in = inventory_system_value.has_in(slot, item);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn has_in(self, slot: impl Into<ItemSlot>, item: impl fmt::Display) -> Execute {
        Execute::new().if_items_entity(self.selector, slot.into(), item.to_string())
    }

    /// `item replace entity <selector> <slot> with <item>` — replace a slot's contents.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::replace",
        module = "sand::systems",
        kind = "method",
        summary = "`item replace entity <selector> <slot> with <item>` — replace a slot's contents.",
        context = "`item replace entity <selector> <slot> with <item>` — replace a slot's contents. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(slot = "`slot` supplies the slot value used to emit the documented `item replace entity <selector> <slot> with <item>` — replace a slot's contents form.", item = "`item` provides the item value or item predicate used to emit the documented `item replace entity <selector> <slot> with <item>` — replace a slot's contents form."),
        returns = "The string value produced to emit the documented `item replace entity <selector> <slot> with <item>` — replace a slot's contents form.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(inventory_system_value: sand::systems::inventory::InventorySystem, slot: impl Into < sand::command::ItemSlot >, item: impl fmt::Display)  {\n    let replace = inventory_system_value.replace(slot, item);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn replace(self, slot: impl Into<ItemSlot>, item: impl fmt::Display) -> String {
        format!(
            "item replace entity {} {} with {}",
            self.selector,
            slot.into(),
            item
        )
    }

    /// `item replace entity <selector> <slot> with <item> <count>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::replace_count",
        module = "sand::systems",
        kind = "method",
        summary = "`item replace entity <selector> <slot> with <item> <count>`.",
        context = "`item replace entity <selector> <slot> with <item> <count>`. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(slot = "`slot` supplies the slot value used to emit the documented `item replace entity <selector> <slot> with <item> <count>` form.", item = "`item` provides the item value or item predicate used to emit the documented `item replace entity <selector> <slot> with <item> <count>` form.", count = "`count` provides the requested numeric amount used to emit the documented `item replace entity <selector> <slot> with <item> <count>` form."),
        returns = "The string value produced to emit the documented `item replace entity <selector> <slot> with <item> <count>` form.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(inventory_system_value: sand::systems::inventory::InventorySystem, slot: impl Into < sand::command::ItemSlot >, item: impl fmt::Display, count: u32)  {\n    let replace_count = inventory_system_value.replace_count(slot, item, count);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn replace_count(
        self,
        slot: impl Into<ItemSlot>,
        item: impl fmt::Display,
        count: u32,
    ) -> String {
        format!(
            "item replace entity {} {} with {} {}",
            self.selector,
            slot.into(),
            item,
            count
        )
    }

    /// `item replace entity <selector> <slot> with air` — clear a single slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::clear_slot",
        module = "sand::systems",
        kind = "method",
        summary = "`item replace entity <selector> <slot> with air` — clear a single slot.",
        context = "`item replace entity <selector> <slot> with air` — clear a single slot. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(slot = "`slot` supplies the slot value used to emit the documented `item replace entity <selector> <slot> with air` — clear a single slot form."),
        returns = "The string value produced to emit the documented `item replace entity <selector> <slot> with air` — clear a single slot form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_system_value: sand::systems::inventory::InventorySystem, slot: impl Into < sand::command::ItemSlot >)  {\n    let clear_slot = inventory_system_value.clear_slot(slot);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn clear_slot(self, slot: impl Into<ItemSlot>) -> String {
        format!(
            "item replace entity {} {} with air",
            self.selector,
            slot.into()
        )
    }

    /// Begin a `clear` command. Call `.amount(n)` or use the returned builder
    /// as a `String` (via `Display`) to clear all matching stacks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::clear_item",
        module = "sand::systems",
        kind = "method",
        summary = "Begin a `clear` command. Call `.amount(n)` or use the returned builder as a `String` (via `Display`) to clear all matching stacks.",
        context = "Begin a `clear` command. Call `.amount(n)` or use the returned builder as a `String` (via `Display`) to clear all matching stacks. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(item = "`item` provides the item value or item predicate used to begin a `clear` command. Call `.amount(n)` or use the returned builder as a `String` (via `Display`) to clear all matching stacks."),
        returns = "The `ClearBuilder` value produced to begin a `clear` command. Call `.amount(n)` or use the returned builder as a `String` (via `Display`) to clear all matching stacks.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_system_value: sand::systems::inventory::InventorySystem, item: impl Into < String >)  {\n    let clear_item = inventory_system_value.clear_item(item);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn clear_item(self, item: impl Into<String>) -> ClearBuilder {
        ClearBuilder {
            selector: self.selector,
            item: item.into(),
        }
    }

    /// `give <selector> <item>` — give an item directly.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::InventorySystem::give",
        module = "sand::systems",
        kind = "method",
        summary = "`give <selector> <item>` — give an item directly.",
        context = "`give <selector> <item>` — give an item directly. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(item = "`item` provides the item value or item predicate used to emit the documented `give <selector> <item>` — give an item directly form."),
        returns = "The string value produced to emit the documented `give <selector> <item>` — give an item directly form.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(inventory_system_value: sand::systems::inventory::InventorySystem, item: impl fmt::Display)  {\n    let give = inventory_system_value.give(item);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn give(self, item: impl fmt::Display) -> String {
        format!("give {} {}", self.selector, item)
    }
}

impl HasItemCheck {
    /// `execute if items entity <selector> <slot> <item>` — check in a specific slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_slot",
        module = "sand::systems",
        kind = "method",
        summary = "`execute if items entity <selector> <slot> <item>` — check in a specific slot.",
        context = "`execute if items entity <selector> <slot> <item>` — check in a specific slot. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(slot = "`slot` supplies the slot value used to emit the documented `execute if items entity <selector> <slot> <item>` — check in a specific slot form."),
        returns = "The `Execute` value produced to emit the documented `execute if items entity <selector> <slot> <item>` — check in a specific slot form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck, slot: impl Into < sand::command::ItemSlot >)  {\n    let in_slot = has_item_check_value.in_slot(slot);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_slot(self, slot: impl Into<ItemSlot>) -> Execute {
        Execute::new().if_items_entity(self.selector, slot.into(), self.item)
    }

    /// Check in the `weapon.*` (mainhand or offhand) slots.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_any_weapon",
        module = "sand::systems",
        kind = "method",
        summary = "Check in the `weapon.*` (mainhand or offhand) slots.",
        context = "Check in the `weapon.*` (mainhand or offhand) slots. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to check in the `weapon.*` (mainhand or offhand) slots.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let in_any_weapon = has_item_check_value.in_any_weapon();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_any_weapon(self) -> Execute {
        self.in_slot(ItemSlot::AnyWeapon)
    }

    /// Check in the `weapon.mainhand` slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_mainhand",
        module = "sand::systems",
        kind = "method",
        summary = "Check in the `weapon.mainhand` slot.",
        context = "Check in the `weapon.mainhand` slot. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to check in the `weapon.mainhand` slot.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let in_mainhand = has_item_check_value.in_mainhand();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_mainhand(self) -> Execute {
        self.in_slot(ItemSlot::MainHand)
    }

    /// Check in the `weapon.offhand` slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_offhand",
        module = "sand::systems",
        kind = "method",
        summary = "Check in the `weapon.offhand` slot.",
        context = "Check in the `weapon.offhand` slot. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to check in the `weapon.offhand` slot.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let in_offhand = has_item_check_value.in_offhand();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_offhand(self) -> Execute {
        self.in_slot(ItemSlot::OffHand)
    }

    /// Check in any of the four `armor.*` slots.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_armor",
        module = "sand::systems",
        kind = "method",
        summary = "Check in any of the four `armor.*` slots.",
        context = "Check in any of the four `armor.*` slots. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to check in any of the four `armor.*` slots.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let in_armor = has_item_check_value.in_armor();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_armor(self) -> Execute {
        self.in_slot(ItemSlot::AnyArmor)
    }

    /// Check in any of the 9 `hotbar.*` slots.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_hotbar",
        module = "sand::systems",
        kind = "method",
        summary = "Check in any of the 9 `hotbar.*` slots.",
        context = "Check in any of the 9 `hotbar.*` slots. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to check in any of the 9 `hotbar.*` slots.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let in_hotbar = has_item_check_value.in_hotbar();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_hotbar(self) -> Execute {
        self.in_slot(ItemSlot::AnyHotbar)
    }

    /// Check in any `inventory.*` slot (the main 27-slot grid).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_inventory",
        module = "sand::systems",
        kind = "method",
        summary = "Check in any `inventory.*` slot (the main 27-slot grid).",
        context = "Check in any `inventory.*` slot (the main 27-slot grid). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to check in any `inventory.*` slot (the main 27-slot grid).",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let in_inventory = has_item_check_value.in_inventory();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_inventory(self) -> Execute {
        self.in_slot(ItemSlot::AnyInventory)
    }

    /// Check across all slots using `*` — any slot in the entity's full inventory.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::in_any_slot",
        module = "sand::systems",
        kind = "method",
        summary = "Check across all slots using `*` — any slot in the entity's full inventory.",
        context = "Check across all slots using `*` — any slot in the entity's full inventory. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to check across all slots using `*` — any slot in the entity's full inventory.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let in_any_slot = has_item_check_value.in_any_slot();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn in_any_slot(self) -> Execute {
        self.in_slot(ItemSlot::Raw("*".into()))
    }

    /// `execute unless items entity <selector> <slot> <item>` — the negated form.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::not_in_slot",
        module = "sand::systems",
        kind = "method",
        summary = "`execute unless items entity <selector> <slot> <item>` — the negated form.",
        context = "`execute unless items entity <selector> <slot> <item>` — the negated form. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(slot = "`slot` supplies the slot value used to emit the documented `execute unless items entity <selector> <slot> <item>` — the negated form form."),
        returns = "The `Execute` value produced to emit the documented `execute unless items entity <selector> <slot> <item>` — the negated form form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck, slot: impl Into < sand::command::ItemSlot >)  {\n    let not_in_slot = has_item_check_value.not_in_slot(slot);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn not_in_slot(self, slot: impl Into<ItemSlot>) -> Execute {
        Execute::new().unless_items_entity(self.selector, slot.into(), self.item)
    }

    /// Skip if the item is anywhere in the full inventory (`*`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::HasItemCheck::not_anywhere",
        module = "sand::systems",
        kind = "method",
        summary = "Skip if the item is anywhere in the full inventory (`*`).",
        context = "Skip if the item is anywhere in the full inventory (`*`). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The `Execute` value produced to skip if the item is anywhere in the full inventory (`*`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(has_item_check_value: sand::systems::inventory::HasItemCheck)  {\n    let not_anywhere = has_item_check_value.not_anywhere();\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn not_anywhere(self) -> Execute {
        Execute::new().unless_items_entity(self.selector, ItemSlot::Raw("*".into()), self.item)
    }
}

impl ClearBuilder {
    /// `clear <selector> <item> <count>` — remove up to `count` items.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::inventory::ClearBuilder::amount",
        module = "sand::systems",
        kind = "method",
        summary = "`clear <selector> <item> <count>` — remove up to `count` items.",
        context = "`clear <selector> <item> <count>` — remove up to `count` items. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(count = "`clear <selector> <item> <count>` — remove up to `count` items."),
        returns = "The string value produced to emit the documented `clear <selector> <item> <count>` — remove up to `count` items form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(clear_builder_value: sand::systems::inventory::ClearBuilder, count: u32)  {\n    let amount = clear_builder_value.amount(count);\n}",
        availability = ["Cargo feature: systems-inventory"],
    )]
    pub fn amount(self, count: u32) -> String {
        format!("clear {} {} {}", self.selector, self.item, count)
    }
}

impl fmt::Display for ClearBuilder {
    /// `clear <selector> <item>` — remove all matching stacks.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clear {} {}", self.selector, self.item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_commands::selector::Selector;

    #[test]
    fn has_in_mainhand() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:diamond_sword")
            .in_mainhand();
        assert_eq!(
            exec.run("say armed"),
            "execute if items entity @s weapon.mainhand minecraft:diamond_sword run say armed"
        );
    }

    #[test]
    fn has_in_slot_explicit() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:shield")
            .in_slot(ItemSlot::OffHand);
        assert_eq!(
            exec.run("say blocking"),
            "execute if items entity @s weapon.offhand minecraft:shield run say blocking"
        );
    }

    #[test]
    fn has_in_any_slot() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:arrow")
            .in_any_slot();
        assert_eq!(
            exec.run("say has arrows"),
            "execute if items entity @s * minecraft:arrow run say has arrows"
        );
    }

    #[test]
    fn has_in_hotbar() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:totem_of_undying")
            .in_hotbar();
        assert_eq!(
            exec.run("say totem"),
            "execute if items entity @s hotbar.* minecraft:totem_of_undying run say totem"
        );
    }

    #[test]
    fn has_in_armor() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:elytra")
            .in_armor();
        assert_eq!(
            exec.run("say flying"),
            "execute if items entity @s armor.* minecraft:elytra run say flying"
        );
    }

    #[test]
    fn not_anywhere() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has("minecraft:arrow")
            .not_anywhere();
        assert_eq!(
            exec.run("say no arrows"),
            "execute unless items entity @s * minecraft:arrow run say no arrows"
        );
    }

    #[test]
    fn has_in_shorthand() {
        let exec = InventorySystem::for_entity(Selector::self_())
            .has_in(ItemSlot::MainHand, "minecraft:bow");
        assert_eq!(
            exec.run("say bow"),
            "execute if items entity @s weapon.mainhand minecraft:bow run say bow"
        );
    }

    #[test]
    fn replace_slot() {
        let cmd = InventorySystem::for_entity(Selector::self_())
            .replace(ItemSlot::MainHand, "minecraft:diamond_sword");
        assert_eq!(
            cmd,
            "item replace entity @s weapon.mainhand with minecraft:diamond_sword"
        );
    }

    #[test]
    fn replace_slot_with_count() {
        let cmd = InventorySystem::for_entity(Selector::self_()).replace_count(
            ItemSlot::Hotbar(0),
            "minecraft:arrow",
            64,
        );
        assert_eq!(
            cmd,
            "item replace entity @s hotbar.0 with minecraft:arrow 64"
        );
    }

    #[test]
    fn clear_slot() {
        let cmd = InventorySystem::for_entity(Selector::self_()).clear_slot(ItemSlot::OffHand);
        assert_eq!(cmd, "item replace entity @s weapon.offhand with air");
    }

    #[test]
    fn clear_item_display() {
        let builder = InventorySystem::for_entity(Selector::self_()).clear_item("minecraft:arrow");
        assert_eq!(builder.to_string(), "clear @s minecraft:arrow");
    }

    #[test]
    fn clear_item_amount() {
        let cmd = InventorySystem::for_entity(Selector::self_())
            .clear_item("minecraft:arrow")
            .amount(16);
        assert_eq!(cmd, "clear @s minecraft:arrow 16");
    }

    #[test]
    fn give_item() {
        let cmd = InventorySystem::for_entity(Selector::all_players()).give("minecraft:diamond");
        assert_eq!(cmd, "give @a minecraft:diamond");
    }

    #[test]
    fn custom_item_string_passthrough() {
        // CustomItem::to_string() produces "minecraft:shield[custom_data={...}]"
        // InventorySystem accepts impl fmt::Display so CustomItem works directly.
        let item_str = "minecraft:shield[custom_data={powers_shockwave:1b}]";
        let exec = InventorySystem::for_entity(Selector::self_())
            .has(item_str)
            .in_offhand();
        assert_eq!(
            exec.run("say shockwave"),
            "execute if items entity @s weapon.offhand minecraft:shield[custom_data={powers_shockwave:1b}] run say shockwave"
        );
    }
}
