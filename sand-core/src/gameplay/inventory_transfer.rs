//! Bounded whole-stack moves within one player's canonical inventory.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    ir::{Actions, Cmd},
    item::{InventoryIndex, ItemLocation},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use sand_components::{ItemModifier, LootFunction, NumberProvider, ResourceLocation};
use std::sync::Arc;

/// Move a whole live stack between two numbered slots of the executing player.
///
/// Empty destinations receive the exact source count and components. Occupied
/// destinations merge only when their complete serialized identity/components
/// agree and the live stack explicitly specifies `minecraft:max_stack_size`.
/// Sand does not infer a vanilla default capacity. Both stacks must fit that
/// capacity together; there is no partial move, overflow drop, or slot search.
///
/// Register the definition before calling [`Self::transfer`]. A pack-wide lock
/// shared with [`super::Purchase`] rejects synchronous reentry during mutation.
/// Callbacks run after unlocking. These operations serialize Sand's own inventory
/// workflows, not arbitrary inventory changes made by other callbacks or packs.
/// After destination mutation, both complete live stacks are rechecked. A changed
/// source is preserved; Sand undoes its destination write only if that slot still
/// equals the expected delivery. Listener-replaced destinations are preserved
/// and the move rejects without clearing the source. Compensation restores only
/// Sand's own write, never arbitrary listener effects.
/// This is not a general transaction/rollback API. Only numbered player slots
/// are supported; entity/container addressing and partial quantities remain
/// available through lower-level canonical inventory primitives.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::InventoryTransfer", module = "sand::systems", summary = "Move a whole stack between two explicit player inventory slots.", context = "Register once, then invoke as the player. Reuses canonical slots and preserves live components.", minecraft = "Checked copy or exact-component merge followed by source clearing under a pack inventory lock.", use_when = ["Moving whole stacks between explicit player slots"], avoid_when = ["Partial quantities, container transfers, or arbitrary rollback"], example = "use sand::systems::InventoryTransfer;")]
pub struct InventoryTransfer {
    id: ResourceLocation,
    source: InventoryIndex,
    destination: InventoryIndex,
    success: Option<Arc<dyn Fn() -> Actions>>,
    rejected: Option<Arc<dyn Fn() -> Actions>>,
}
impl InventoryTransfer {
    /// Define a whole-stack move. Identical slots fail export.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::InventoryTransfer::new", module = "sand::systems", summary = "Define a bounded whole-stack player inventory move.", context = "The two slots belong to the executing player and must differ.", minecraft = "No commands run during construction.", use_when = ["Moving live item stacks without reconstructing components"], avoid_when = ["Selecting arbitrary entity or container owners"], params(id = "Unique registered definition identity.", source = "Numbered player slot to empty on success.", destination = "Numbered player slot receiving the whole stack."), returns = "The unregistered transfer definition.", example = "use sand::systems::InventoryTransfer;")]
    pub fn new(id: ResourceLocation, source: InventoryIndex, destination: InventoryIndex) -> Self {
        Self {
            id,
            source,
            destination,
            success: None,
            rejected: None,
        }
    }
    /// Run as the player after delivery and source clearing, with scratch state released.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::InventoryTransfer::on_success", module = "sand::systems", summary = "Run after a completed whole-stack move.", context = "Callbacks run unlocked and may start another transfer.", minecraft = "Runs isolated actions as the invoking player.", use_when = ["Continuing gameplay after a move"], avoid_when = ["Assuming callback rollback"], params(action = "Player-context success actions."), returns = "The configured definition.", example = "use sand::systems::InventoryTransfer;")]
    pub fn on_success<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.success = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Run after an empty source, incompatible/full destination, or native write failure.
    /// Reentrant attempts and non-player executors are silent no-ops.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::InventoryTransfer::on_rejected", module = "sand::systems", summary = "Run after rejected transfer admission or placement.", context = "Reentry while locked is a silent no-op, preventing rejection recursion.", minecraft = "Runs isolated actions after releasing scratch state and the inventory lock.", use_when = ["Explaining incompatible or full slots"], avoid_when = ["Treating every no-op as a rejection callback"], params(action = "Player-context rejection actions."), returns = "The configured definition.", example = "use sand::systems::InventoryTransfer;")]
    pub fn on_rejected<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.rejected = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Attempt the registered whole-stack move for the executing player.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::InventoryTransfer::transfer", module = "sand::systems", summary = "Attempt the registered whole-stack player move.", context = "Use from player-context functions, menus, or interaction callbacks.", minecraft = "Admission precedes checked destination mutation and source clearing.", use_when = ["Invoking a registered move"], avoid_when = ["Running outside player context"], returns = "Retained gameplay actions.", example = "use sand::systems::InventoryTransfer;")]
    pub fn transfer(&self) -> Actions {
        identity::call(&self.id, "inventory_transfer_move")
    }
}
impl IntoDatapack for InventoryTransfer {
    fn into_datapack(self) -> DatapackRegistration {
        let mut registration =
            DatapackRegistration::new().lifecycle(LifecycleContribution::load(format!(
                "function {}",
                identity::resource(&self.id, "inventory_transfer_load")
            )));
        for part in [
            "load",
            "move",
            "empty",
            "copy",
            "merge",
            "finish",
            "compensate",
            "success",
            "rejected",
        ] {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.id,
                &format!("inventory_transfer_{part}"),
                move || definition.build(part),
            ));
        }
        registration
    }
}
impl InventoryTransfer {
    fn build(&self, part: &str) -> CommandResult<Actions> {
        if self.source == self.destination {
            return Err(CommandError::new(
                "inventory transfer",
                "destination",
                "source and destination must be distinct numbered player slots",
            ));
        }
        let id = crate::function::resolve_export_location(&self.id)?;
        let lock = inventory_lock(&id)?;
        let scratch = identity::resource(&id, "inventory_transfer_scratch");
        let score = ObjectiveName::logical(format!("sand inventory {} scratch", id.namespace()));
        crate::function::request_numeric_objectives([score.clone()])?;
        let function = |part: &str| identity::resource(&id, &format!("inventory_transfer_{part}"));
        let source = ItemLocation::PlayerInventory(self.source);
        let destination = ItemLocation::PlayerInventory(self.destination);
        let source_path = source.nbt().as_str().to_owned();
        let dest_path = destination.nbt().as_str().to_owned();
        let rejected = || {
            Cmd::ReturnRun(Box::new(Cmd::Function(function("rejected").to_string())))
                .into_commands()
        };
        match part {
            "load" => Ok(format!(
                "scoreboard players set #busy {lock} 0\ndata remove storage {scratch} transfer"
            )
            .into_commands()),
            "empty" => Ok(format!("scoreboard players set #merged {score} 0\ndata modify storage {scratch} transfer.expected set from storage {scratch} transfer.before\ndata modify storage {scratch} transfer.expected.Slot set value {}b\nexecute unless function {} run return run function {}\nreturn run function {}", self.destination.get(), function("copy"), function("rejected"), function("finish")).into_commands()),
            "finish" => {
                // A native write may run arbitrary inventory listeners. Compare
                // complete live stacks before clearing anything. Compensation
                // only touches a destination that still equals our own write.
                let mut actions = format!("data modify storage {scratch} transfer.source_after set from entity @s {source_path}\ndata modify storage {scratch} transfer.destination_after set from entity @s {dest_path}\nscoreboard players set #source_changed {score} 1\nscoreboard players set #destination_changed {score} 1\nexecute if data storage {scratch} transfer.source_after store success score #source_changed {score} run data modify storage {scratch} transfer.source_after set from storage {scratch} transfer.before\nexecute if data storage {scratch} transfer.destination_after store success score #destination_changed {score} run data modify storage {scratch} transfer.destination_after set from storage {scratch} transfer.expected\nexecute if score #destination_changed {score} matches 1 run return run function {}\nexecute if score #source_changed {score} matches 1 run return run function {}", function("rejected"), function("compensate")).into_commands();
                actions.extend([source.clear().map_err(|e| CommandError::new("inventory transfer", "source", e.to_string()))?, format!("return run function {}", function("success")).into_commands()]);
                Ok(actions)
            }
            "compensate" => Ok(format!("execute if score #merged {score} matches 0 run {}\nscoreboard players operation #total {score} = #destination {score}\nexecute if score #merged {score} matches 1 run function {}\nreturn run function {}", clear_line(&destination)?, function("merge"), function("rejected")).into_commands()),
            "copy" => {
                let mut copy = destination
                    .replace_from(&source)
                    .map_err(|e| CommandError::new("inventory transfer", "copy", e.to_string()))?;
                Ok(Cmd::ReturnRun(Box::new(copy.0.remove(0))).into_commands())
            }
            "merge" => {
                let modifier = identity::resource(&id, "inventory_transfer_count");
                crate::function::register_generated_component(
                    ItemModifier::new(modifier.clone()).function(LootFunction::SetCount {
                        count: NumberProvider::Score {
                            target: serde_json::json!({"type":"minecraft:fixed","name":"#total"}),
                            score: score.to_string(),
                            scale: None,
                        },
                        add: false,
                    }),
                );
                let sand_commands::inventory::ItemCommandLocation::Entity { slot, .. } =
                    destination
                        .item_target_slot()
                        .expect("numbered player slot")
                else {
                    unreachable!()
                };
                let line = sand_commands::Inventory::of(sand_commands::Target::current_player())
                    .try_modify(slot, modifier.to_string())?;
                Ok(format!("return run {line}").into_commands())
            }
            "success" | "rejected" => {
                let mut actions = format!(
                    "data remove storage {scratch} transfer\nscoreboard players set #busy {lock} 0"
                )
                .into_commands();
                if let Some(callback) = if part == "success" {
                    &self.success
                } else {
                    &self.rejected
                } {
                    actions.extend([runtime::isolated(callback())]);
                }
                Ok(actions)
            }
            "move" => {
                let mut actions = format!("execute unless entity @s[type=minecraft:player] run return 0\nexecute if score #busy {lock} matches 1 run return 0\nscoreboard players set #busy {lock} 1\ndata remove storage {scratch} transfer").into_commands();
                actions.extend([source.exists().guard_actions(true, &[], rejected())]);
                actions.extend([format!("execute store result score #source {score} run data get entity @s {source_path}.count\nexecute unless score #source {score} matches 1..99 run return run function {}\ndata modify storage {scratch} transfer.before set from entity @s {source_path}", function("rejected")).into_commands()]);
                // The empty path uses the canonical checked copy. The occupied
                // path compares complete compounds, ignoring only count/slot.
                actions.extend([destination
                    .is_empty()
                    .expect("numbered slot")
                    .guard_actions(
                        false,
                        &[],
                        Cmd::ReturnRun(Box::new(Cmd::Function(function("empty").to_string())))
                            .into_commands(),
                    )]);
                actions.extend([format!("data modify storage {scratch} transfer.source set from storage {scratch} transfer.before\ndata modify storage {scratch} transfer.destination set from entity @s {dest_path}\ndata modify storage {scratch} transfer.expected set from storage {scratch} transfer.destination\nscoreboard players set #capacity {score} 0\nexecute store result score #capacity {score} run data get storage {scratch} transfer.source.components.\"minecraft:max_stack_size\"\nexecute unless score #capacity {score} matches 1..99 run return run function {}\nexecute store result score #total {score} run data get entity @s {dest_path}.count\nexecute unless score #total {score} matches 1..99 run return run function {}\nscoreboard players operation #destination {score} = #total {score}\nscoreboard players operation #total {score} += #source {score}\nexecute if score #total {score} > #capacity {score} run return run function {}\ndata remove storage {scratch} transfer.source.Slot\ndata remove storage {scratch} transfer.destination.Slot\ndata remove storage {scratch} transfer.source.count\ndata remove storage {scratch} transfer.destination.count\nexecute store success score #different {score} run data modify storage {scratch} transfer.destination set from storage {scratch} transfer.source\nexecute if score #different {score} matches 1 run return run function {}\nscoreboard players set #merged {score} 1\nexecute store result storage {scratch} transfer.expected.count int 1 run scoreboard players get #total {score}\nexecute unless function {} run return run function {}", function("rejected"), function("rejected"), function("rejected"), function("rejected"), function("merge"), function("rejected")).into_commands()]);
                actions.extend([format!("return run function {}", function("finish")).into_commands()]);
                Ok(actions)
            }
            _ => unreachable!(),
        }
    }
}

pub(super) fn inventory_lock(_definition: &ResourceLocation) -> CommandResult<ObjectiveName> {
    // Registered resources may use several namespaces inside one exported pack.
    // Native listeners must not bypass inventory serialization by invoking a
    // differently namespaced Purchase, InventoryTransfer, or Journal.
    let namespace = identity::pack_namespace()?;
    let lock = ObjectiveName::logical(format!("sand purchase {namespace} transaction lock"));
    crate::function::request_numeric_objectives([lock.clone()])?;
    Ok(lock)
}
fn clear_line(source: &ItemLocation) -> CommandResult<String> {
    use sand_commands::RenderCommand;
    sand_commands::inventory::ItemReplaceCommand {
        destination: source.item_target_slot().expect("numbered player slot"),
        source: sand_commands::inventory::ItemReplaceSource::Clear,
    }
    .render(&sand_commands::CommandProfile::unprofiled())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_aliased_numbered_slots_before_generating_a_move() {
        let slot = InventoryIndex::new(0).unwrap();
        let transfer = InventoryTransfer::new("proof:alias".parse().unwrap(), slot, slot);
        let error = transfer.build("move").unwrap_err().to_string();
        assert!(error.contains("distinct numbered player slots"), "{error}");
    }
}
