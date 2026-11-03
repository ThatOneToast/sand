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
/// This deliberately does not merge stacks, drop overflow, remove arbitrary
/// payment items or promise rollback for user callbacks. Attach referenced State
/// through its normal lifecycle before invoking the purchase.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase", module = "sand::systems", summary = "Validated player purchases and one-time item rewards.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or arbitrary item-payment removal"],
example = "use sand::systems::Purchase;")]
pub struct Purchase {
    id: ResourceLocation,
    reward: ItemStack,
    slot: InventoryIndex,
    cost: Option<(EntityScore<i32>, u32)>,
    claim: Option<EntityFlag>,
    success: Option<Arc<dyn Fn() -> Actions>>,
    rejected: Option<Arc<dyn Fn() -> Actions>>,
}
impl Purchase {
    /// Define a reward and its destination player inventory slot (0–35).
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::new", module = "sand::systems", summary = "Define a reward delivered to an empty player slot.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or arbitrary item-payment removal"],
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
            success: None,
            rejected: None,
        }
    }
    /// Charge canonical integer currency only after successful delivery.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::cost", module = "sand::systems", summary = "Set the canonical score currency cost.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or arbitrary item-payment removal"],
params(balance = "Registered player/entity integer State field.", amount = "Nonnegative currency amount fitting i32."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn cost(mut self, balance: EntityScore<i32>, amount: u32) -> Self {
        self.cost = Some((balance, amount));
        self
    }
    /// Require an unset claim flag and set it before success callbacks.
    /// Player/entity flags create personal claims; global flags create shared claims.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::claim_once", module = "sand::systems", summary = "Make the reward claimable once per canonical claim holder.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or arbitrary item-payment removal"],
params(claimed = "Canonical State flag; global flags produce shared claims."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn claim_once(mut self, claimed: EntityFlag) -> Self {
        self.claim = Some(claimed);
        self
    }
    /// Run as the purchasing player after item delivery and state commits.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::on_success", module = "sand::systems", summary = "Run after delivery and committed payment.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or arbitrary item-payment removal"],
params(action = "Player-context success actions."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn on_success<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.success = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Run as the player when admission or delivery fails, with no charge.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::on_rejected", module = "sand::systems", summary = "Run after admission or placement rejection.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or arbitrary item-payment removal"],
params(action = "Player-context failure actions."),
returns = "The configured purchase or retained player action.",
example = "use sand::systems::Purchase;")]
    pub fn on_rejected<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.rejected = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Attempt the registered purchase for the executing player.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Purchase::buy", module = "sand::systems", summary = "Attempt a purchase for the executing player.", context = "Register the definition and attach its canonical State before use. Empty-slot and balance checks precede placement; payment and claims precede success callbacks.", minecraft = "One native player-slot write with checked result, then canonical State mutations. Full destination rejects without charge. No arbitrary-command rollback.", use_when = ["Building shops and personal reward containers"], avoid_when = ["Stack merging or arbitrary item-payment removal"],
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
        for part in ["load", "buy", "place", "success", "rejected"] {
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
    fn build(&self, part: &str) -> CommandResult<Actions> {
        let error = |field, detail: String| CommandError::new("purchase", field, detail);
        let id = crate::function::resolve_export_location(&self.id)?;
        let lock =
            ObjectiveName::logical(format!("sand purchase {} transaction lock", id.namespace()));
        crate::function::request_numeric_objectives([lock.clone()])?;
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
            "load" => Ok(format!("scoreboard players set #busy {lock} 0").into_commands()),
            "success" | "rejected" => {
                let mut body = format!("scoreboard players set #busy {lock} 0").into_commands();
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
                actions.extend([format!(
                    "execute unless function {} run return run function {}",
                    function("place"),
                    function("rejected")
                )
                .into_commands()]);
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
