//! Validated score purchases and one-time player rewards.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    entity::{EntityFlag, EntityScore, EntityStateField},
    ir::{Actions, Cmd},
    item::{InventoryIndex, ItemLocation},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use sand_components::{ItemStack, ResourceLocation};
use std::sync::Arc;

/// A player reward delivered into one explicitly empty inventory slot.
///
/// Optional currency and claim flags are canonical registered player/entity or global
/// State. Admission checks State membership, balance, prior claim and empty
/// destination. Native item placement must succeed before charging or claiming;
/// state commits before callbacks. A pack-owned lock rejects synchronous
/// reentrant purchases during native placement; normal callbacks run unlocked. Rejection changes neither balance nor claim.
/// This deliberately does not merge stacks, drop overflow, search for arbitrary
/// payment items or promise rollback for user callbacks. Attach referenced State
/// through its normal lifecycle before invoking the purchase.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase", module = "sand::systems", summary = "Validated player purchases and one-time item rewards.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or inventory-wide item-payment removal"],
example = "use sand::systems::Purchase;")]
pub struct Purchase {
    id: ResourceLocation,
    reward: ItemStack,
    slot: InventoryIndex,
    cost: Option<(EntityScore<i32>, u32)>,
    claim: Option<EntityFlag>,
    item_cost: Option<(InventoryIndex, sand_components::ItemPredicate, u32)>,
    success: Option<Arc<dyn Fn() -> Actions>>,
    rejected: Option<Arc<dyn Fn() -> Actions>>,
}
impl Purchase {
    /// Define a reward and its destination player inventory slot (0–35).
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::new", module = "sand::systems", summary = "Define a reward delivered to an empty player slot.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or inventory-wide item-payment removal"],
params(id = "Unique registered purchase identity.", reward = "Canonical item stack.", slot = "Player inventory slot which must be empty."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn new(id: ResourceLocation, reward: ItemStack, slot: InventoryIndex) -> Self {
        Self {
            id,
            reward,
            slot,
            cost: None,
            claim: None,
            item_cost: None,
            success: None,
            rejected: None,
        }
    }
    /// Charge canonical integer currency only after successful delivery.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::cost", module = "sand::systems", summary = "Set the canonical score currency cost.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or inventory-wide item-payment removal"],
params(balance = "Registered player/entity integer State field.", amount = "Nonnegative currency amount fitting i32."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn cost(mut self, balance: EntityScore<i32>, amount: u32) -> Self {
        self.cost = Some((balance, amount));
        self
    }
    /// Pay an exact quantity from one matching stack in a distinct player slot.
    ///
    /// The slot must contain at least `amount` items and satisfy the canonical
    /// predicate, including its component filters. An ID-only predicate accepts
    /// any components; use an explicit predicate when payment identity requires
    /// them. The remainder retains its complete live components; no slot search occurs. The reward slot cannot also be
    /// the payment slot. This cost composes with score currency and claims.
    ///
    /// Placement is checked before the payment quantity is removed. Sand snapshots
    /// and rechecks the complete payment stack after native placement. Changes
    /// reject; compensation clears the reward only when its complete serialized
    /// stack still equals the canonical expected reward staged before placement.
    /// Listener replacements and unprovable codec-normalized representations are
    /// preserved. This is best-effort compensation, not rollback of arbitrary
    /// native listeners or other packs' mutations.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::item_cost", module = "sand::systems", summary = "Require an exact payment quantity from a separate player slot.", context = "Combines canonical ItemPredicate identity/components with a minimum available quantity and explicit slot exclusion.", minecraft = "Checks the payment slot before placement, rechecks its snapshot afterward, then removes only the requested quantity before State commits.", use_when = ["Shops accepting a quantity from an explicitly selected payment stack"], avoid_when = ["Searching an inventory or arbitrary transaction rollback"], params(slot = "Numbered payment slot, distinct from the reward slot.", item = "Canonical item identity and component predicate.", amount = "Exact quantity to consume, from 1 through 99."), returns = "The configured purchase.", example = "use sand::systems::Purchase;")]
    pub fn item_cost(
        mut self,
        slot: InventoryIndex,
        item: impl Into<sand_components::ItemPredicate>,
        amount: u32,
    ) -> Self {
        self.item_cost = Some((slot, item.into(), amount));
        self
    }
    /// Require an unset claim flag and set it before success callbacks.
    /// Player/entity flags create personal claims; global flags create shared claims.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::claim_once", module = "sand::systems", summary = "Make the reward claimable once per canonical claim holder.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or inventory-wide item-payment removal"],
params(claimed = "Canonical State flag; global flags produce shared claims."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn claim_once(mut self, claimed: EntityFlag) -> Self {
        self.claim = Some(claimed);
        self
    }
    /// Run as the purchasing player after item delivery and state commits.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::on_success", module = "sand::systems", summary = "Run after delivery and committed payment.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or inventory-wide item-payment removal"],
params(action = "Player-context success actions."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn on_success<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.success = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Run as the player when admission or delivery fails, with no charge.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::on_rejected", module = "sand::systems", summary = "Run after admission or placement rejection.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or inventory-wide item-payment removal"],
params(action = "Player-context failure actions."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn on_rejected<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.rejected = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Attempt the registered purchase for the executing player.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::buy", module = "sand::systems", summary = "Attempt a purchase for the executing player.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or inventory-wide item-payment removal"],
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn buy(&self) -> Actions {
        identity::call(&self.id, "purchase_buy")
    }
}
impl IntoDatapack for Purchase {
    fn into_datapack(self) -> DatapackRegistration {
        let mut registration = DatapackRegistration::new().lifecycle(LifecycleContribution::load(
            format!("function {}", identity::resource(&self.id, "purchase_load")),
        ));
        for part in [
            "load",
            "buy",
            "place",
            "payment_changed",
            "debit",
            "success",
            "rejected",
        ] {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.id,
                &format!("purchase_{part}"),
                move || definition.build(part),
            ));
        }
        registration
    }
}
impl Purchase {
    fn expected_reward_snbt(&self) -> CommandResult<String> {
        // Canonical item components are structured values, not a reparsed item
        // command. The shared component encoder preserves literal strings and
        // native number types. Codecs may still normalize forms; a mismatch must retain
        // the live stack rather than weakening full equality to a subset match.
        let components = self
            .reward
            .stack_components()
            .map_err(|error| CommandError::new("purchase", "reward", error.to_string()))?;
        let mut expected = serde_json::json!({
            "id": components.base_item(),
            "count": self.reward.count_value(),
        });
        if !components.is_component_free() {
            expected["components"] =
                serde_json::Value::Object(components.components().iter().cloned().collect());
        }
        Ok(runtime::component_nbt(&expected)?.to_string())
    }

    fn build(&self, part: &str) -> CommandResult<Actions> {
        let error = |field, detail: String| CommandError::new("purchase", field, detail);
        let id = crate::function::resolve_export_location(&self.id)?;
        let lock = super::inventory_transfer::inventory_lock(&id)?;
        let scratch = identity::resource(&id, "purchase_scratch");
        let payment_score =
            ObjectiveName::logical(format!("sand purchase {} payment scratch", id.namespace()));
        crate::function::request_numeric_objectives([payment_score.clone()])?;
        if let Some((payment_slot, _, amount)) = &self.item_cost
            && (*payment_slot == self.slot || !(1..=99).contains(amount))
        {
            return Err(error(
                "item_cost",
                "payment must use a distinct slot and a quantity from 1 through 99".into(),
            ));
        }
        let function =
            |part: &str| identity::resource(&id, &format!("purchase_{part}")).to_string();
        self.reward
            .validate()
            .map_err(|e| error("reward", e.to_string()))?;
        if self.reward.id().to_string() == "minecraft:air" {
            return Err(error("reward", "air is not a deliverable reward".into()));
        }
        let mut schemas = std::collections::BTreeMap::new();
        for reference in self
            .cost
            .map(|(field, _)| field.field_reference())
            .into_iter()
            .chain(self.claim.map(|field| field.field_reference()))
        {
            let schema = inventory::iter::<crate::state::StateDescriptor>
                .into_iter()
                .find(|s| s.id == reference.component)
                .ok_or_else(|| error("state", "requires registered canonical State".into()))?;
            if !schema.fields.iter().any(|field| {
                ObjectiveName::logical(field.objective).as_str() == reference.objective
                    && field.field == reference.descriptor
            }) {
                return Err(error(
                    "state",
                    "field metadata does not match its registered State".into(),
                ));
            }
            schemas.insert(reference.component, schema);
        }
        let dirty = |key: &str| {
            matches!(
                schemas[key].scope,
                crate::state::StateScope::Entity | crate::state::StateScope::Living
            )
        };
        let holder = |key: &str| match schemas[key].scope {
            crate::state::StateScope::Global(holder) => holder,
            _ => "@s",
        };
        let slot = ItemLocation::PlayerInventory(self.slot);
        let rejected =
            || Cmd::ReturnRun(Box::new(Cmd::Function(function("rejected")))).into_commands();
        match part {
            "place" => {
                let mut write = slot
                    .set(self.reward.clone())
                    .map_err(|e| error("reward", e.to_string()))?;
                Ok(Cmd::ReturnRun(Box::new(write.0.remove(0))).into_commands())
            }
            "load" => Ok(format!(
                "scoreboard players set #busy {lock} 0\ndata remove storage {scratch} payment"
            )
            .into_commands()),
            "debit" => {
                let Some((payment_slot, _, _)) = &self.item_cost else {
                    return Ok(Cmd::Return(0).into_commands());
                };
                let payment = ItemLocation::PlayerInventory(*payment_slot);
                let modifier = runtime::count_modifier(
                    identity::resource(&id, "purchase_payment_count"),
                    &payment_score,
                    "#count",
                );
                let sand_commands::inventory::ItemCommandLocation::Entity { slot, .. } =
                    payment.item_target_slot().expect("numbered player slot")
                else {
                    unreachable!()
                };
                let modify = sand_commands::Inventory::of(sand_commands::Target::current_player())
                    .try_modify(slot, modifier.to_string())?;
                let mut clear = payment
                    .clear()
                    .map_err(|e| error("item_cost", e.to_string()))?;
                let mut actions = format!(
                    "execute unless score #count {payment_score} matches 0 run return run {modify}"
                )
                .into_commands();
                actions.extend([Cmd::ReturnRun(Box::new(clear.0.remove(0)))]);
                Ok(actions)
            }
            "payment_changed" => {
                // The placement listener has already run. This is evidence of
                // the current stack, never the expected reward: compare it with
                // the canonical full stack staged before native placement.
                let path = slot.nbt().as_str().to_owned();
                let mut actions = format!("data remove storage {scratch} payment.reward_actual\ndata modify storage {scratch} payment.reward_actual set from entity @s {path}\nexecute unless data storage {scratch} payment.reward_actual run return run function {}\nexecute unless data storage {scratch} payment.reward_expected run return run function {}\ndata remove storage {scratch} payment.reward_actual.Slot\nexecute store success score #reward_changed {payment_score} run data modify storage {scratch} payment.reward_actual set from storage {scratch} payment.reward_expected\nexecute if score #reward_changed {payment_score} matches 1 run return run function {}", function("rejected"), function("rejected"), function("rejected")).into_commands();
                actions.extend([
                    slot.clear().map_err(|e| error("slot", e.to_string()))?,
                    rejected(),
                ]);
                Ok(actions)
            }
            "success" | "rejected" => {
                let mut body = format!(
                    "data remove storage {scratch} payment\nscoreboard players set #busy {lock} 0"
                )
                .into_commands();
                let callback = if part == "success" {
                    &self.success
                } else {
                    &self.rejected
                };
                if let Some(callback) = callback {
                    body.extend([runtime::isolated(callback())]);
                }
                Ok(body)
            }
            "buy" => {
                let mut actions =
                    "execute unless entity @s[type=minecraft:player] run return 0".into_commands();
                // Native inventory listeners may synchronously invoke another
                // purchase. Serialize the mutation window across this pack,
                // including distinct definitions sharing a claim or currency.
                // Reentrant admission is a no-op; rejection callbacks would
                // themselves be able to reenter and recurse without bound.
                actions.extend([
                    format!("execute if score #busy {lock} matches 1 run return 0").into_commands(),
                    format!("scoreboard players set #busy {lock} 1").into_commands(),
                ]);
                for schema in schemas.values() {
                    let presence = ObjectiveName::logical(schema.presence_objective);
                    let owner = holder(schema.id);
                    actions.extend([format!(
                        "execute unless score {owner} {presence} matches {} run return run function {}",
                        schema.version,
                        function("rejected")
                    )
                    .into_commands()]);
                }
                if let Some((field, amount)) = self.cost {
                    let amount =
                        i32::try_from(amount).map_err(|_| error("cost", "must fit i32".into()))?;
                    let minimum = field.descriptor().bounds.map_or(0, |(min, _)| min.max(0));
                    let threshold = minimum
                        .checked_add(amount)
                        .ok_or_else(|| error("cost", "cannot fit above balance minimum".into()))?;
                    let enough = field
                        .bind_to(
                            holder(&field.field_reference().component),
                            dirty(&field.field_reference().component),
                        )
                        .matches(threshold..)
                        .map_err(|e| error("cost", e.to_string()))?;
                    actions.extend([enough.guard_actions(true, &[], rejected())]);
                }
                if let Some(flag) = self.claim {
                    actions.extend([flag
                        .bind_to(
                            holder(&flag.field_reference().component),
                            dirty(&flag.field_reference().component),
                        )
                        .is_enabled()
                        .guard_actions(false, &[], rejected())]);
                }
                actions.extend([slot
                    .is_empty()
                    .map_err(|e| error("slot", e.to_string()))?
                    .guard_actions(true, &[], rejected())]);
                if let Some((payment_slot, predicate, amount)) = &self.item_cost {
                    let payment = ItemLocation::PlayerInventory(*payment_slot);
                    actions.extend([payment
                        .matches(predicate.clone())
                        .map_err(|e| error("item_cost", e.to_string()))?
                        .guard_actions(true, &[], rejected())]);
                    let path = payment.nbt().as_str().to_owned();
                    actions.extend([format!("execute store result score #count {payment_score} run data get entity @s {path}.count\nexecute unless score #count {payment_score} matches {amount}..99 run return run function {}\nscoreboard players remove #count {payment_score} {amount}\ndata remove storage {scratch} payment\ndata modify storage {scratch} payment.before set from entity @s {path}\ndata modify storage {scratch} payment.reward_expected set value {}", function("rejected"), self.expected_reward_snbt()?).into_commands()]);
                }
                actions.extend([format!(
                    "execute unless function {} run return run function {}",
                    function("place"),
                    function("rejected")
                )
                .into_commands()]);
                if let Some((payment_slot, _, _)) = &self.item_cost {
                    let payment = ItemLocation::PlayerInventory(*payment_slot);
                    let path = payment.nbt().as_str().to_owned();
                    actions.extend([format!("execute unless data entity @s {path} run return run function {}\ndata modify storage {scratch} payment.after set from entity @s {path}\nexecute store success score #changed {payment_score} run data modify storage {scratch} payment.after set from storage {scratch} payment.before\nexecute if score #changed {payment_score} matches 1 run return run function {}", function("payment_changed"), function("payment_changed")).into_commands()]);
                    actions.extend([format!(
                        "execute unless function {} run return run function {}",
                        function("debit"),
                        function("payment_changed")
                    )
                    .into_commands()]);
                }
                if let Some((field, amount)) = self.cost {
                    actions.extend([field
                        .bind_to(
                            holder(&field.field_reference().component),
                            dirty(&field.field_reference().component),
                        )
                        .add(-(amount as i32))
                        .into_commands()]);
                }
                if let Some(flag) = self.claim {
                    actions.extend([flag
                        .bind_to(
                            holder(&flag.field_reference().component),
                            dirty(&flag.field_reference().component),
                        )
                        .enable()
                        .into_commands()]);
                }
                actions.extend([format!("function {}", function("success")).into_commands()]);
                Ok(actions)
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expected_reward_uses_complete_canonical_identity_count_and_components() {
        let reward = ItemStack::new(sand_components::ItemId::minecraft("diamond").unwrap())
            .count(3)
            .component(sand_components::ItemComponent::custom_data_marker("reward"));
        let purchase = Purchase::new(
            "proof:reward".parse().unwrap(),
            reward,
            InventoryIndex::new(0).unwrap(),
        );
        assert_eq!(
            purchase.expected_reward_snbt().unwrap(),
            "{components:{\"minecraft:custom_data\":{reward:1b}},count:3,id:\"minecraft:diamond\"}"
        );
    }

    #[test]
    fn rejects_payment_reward_aliases_and_invalid_whole_stack_quantities() {
        let _export = crate::function::ExportFunctionRegistryScope::enter_namespace("proof");
        let reward_slot = InventoryIndex::new(0).unwrap();
        let payment_slot = InventoryIndex::new(1).unwrap();
        let reward = ItemStack::new(sand_components::ItemId::minecraft("diamond").unwrap());
        for (slot, amount) in [(reward_slot, 3), (payment_slot, 0), (payment_slot, 100)] {
            let purchase = Purchase::new(
                "proof:payment".parse().unwrap(),
                reward.clone(),
                reward_slot,
            )
            .item_cost(
                slot,
                sand_components::ItemId::minecraft("emerald").unwrap(),
                amount,
            );
            let error = purchase.build("buy").unwrap_err().to_string();
            assert!(error.contains("distinct slot and a quantity"), "{error}");
        }
    }
}
