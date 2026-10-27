//! Furnace-family gameplay composed from canonical locations, values and branches.
use super::BlockContext;
use crate::{
    IntoCommands,
    condition::Condition,
    entity::StatCurve,
    ir::{Actions, Cmd},
    item::ItemLocation,
};
use sand_commands::{NbtPath, nbt::NbtRefLowering};

const PROFILE: &str = "26.2";

impl BlockContext {
    /// Runs one grouped body when this address is a furnace, blast furnace or smoker.
    ///
    /// The closure runs while building the pack; its returned actions run only after
    /// the runtime family check. Entry is checked once, even if the body replaces
    /// the machine. Returns end the body, not the surrounding authored function.
    /// An empty body emits nothing. Inline macro lines require an explicit macro
    /// function call with arguments, just like other grouped branches.
    ///
    /// This is entry-only validity: later replacement or execution-context changes
    /// can invalidate or rebind its locations and reads. Unloaded or wrong-type
    /// blocks skip the body. No chunk loading, observation, or transfer is implicit.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::world::BlockContext::when_furnace",
        aliases = ["sand::prelude::BlockContext::when_furnace"], module = "sand::world", kind = "method",
        summary = "Runs a grouped gameplay body after one furnace-family entry check.",
        context = "The capability supplies canonical ItemLocations and StatCurves; it is not a persistent block reference or a transaction.",
        minecraft = "Supports verified 26.2 only. Uses the generic if/else dispatcher without decision scratch; preserves caller context and isolates body returns.",
        use_when = ["Inspecting or interacting with one explicitly addressed furnace-family machine"],
        avoid_when = ["Assuming a runtime type check remains valid after replacement or context rebinding"],
        params(body = "Build-time closure returning the owned gameplay actions for a matching machine."),
        returns = "Owned entry-guarded Actions; empty bodies produce no resources.",
        example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).when_furnace(|furnace| furnace.output().clear().unwrap());",
    )]
    pub fn when_furnace<R: IntoCommands>(&self, body: impl FnOnce(Furnace) -> R) -> Actions {
        let body = body(Furnace {
            block: self.clone(),
        })
        .into_commands();
        if body.0.is_empty() {
            return Actions::default();
        }
        let family = Condition::any([
            self.matches(sand_commands::BlockState::of("minecraft:furnace")),
            self.matches(sand_commands::BlockState::of("minecraft:blast_furnace")),
            self.matches(sand_commands::BlockState::of("minecraft:smoker")),
        ]);
        let actions = crate::execute_when::if_(family)
            .then_all([body])
            .else_all(std::iter::empty::<Actions>());
        Actions(
            actions
                .0
                .into_iter()
                .map(|run| Cmd::WithProfileRequirement {
                    version: PROFILE,
                    capability: "furnace gameplay",
                    run: Box::new(run),
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restricted_and_raw_equivalent_nbt_have_distinct_helper_identities() {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let furnace = Furnace { block: crate::world::WorldContext::current()
            .block(sand_commands::BlockPos::here()) };
        let reference = furnace.block.data().typed_path::<i16>(NbtPath::root("lit_time_remaining"));
        let raw = StatCurve::native_nbt(reference.__location().clone(), reference.path_value().clone(), None);
        let restricted = furnace.burn_remaining();
        let call = |value: StatCurve| crate::ir::test_support::emitted(
            crate::execute_when::when(value.gt(0)).then_one("say positive"));
        let first = call(raw.clone());
        let second = call(restricted.clone());
        assert_ne!(first, second, "profile validation must not share an unprofiled helper identity");
        assert_eq!(first, call(raw));
        assert_eq!(second, call(restricted));
        let helpers = crate::ir::test_support::drain_emitted();
        assert_eq!(helpers.iter().filter(|(_, body)| body.iter().any(|line|
            line.contains("append from block ~ ~ ~ lit_time_remaining"))).count(), 2);
    }
}

/// Entry-scoped access shared by furnaces, blast furnaces and smokers.
///
/// Constructed only by [`BlockContext::when_furnace`]. Locations and numeric reads
/// retain the block address, not a captured dimension or durable type proof.
/// Relative/local positions resolve where the operation ultimately executes.
/// Use them in the intended scope; replacing the block or entering another
/// entity/dimension context can invalidate or rebind them.
///
/// Reads expose vanilla's command-visible signed-short tick values, including
/// negative values after native narrowing. Missing reads fail through the normal
/// numeric backend; they are not zero/idle. Slots use `/item` semantics, including
/// explicit replacement of output and items that a GUI would refuse to insert.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::world::Furnace", module = "sand::world",
    summary = "Composes slot access and native processing reads for the furnace family.",
    context = "One capability shares ItemLocation, Nbt and StatCurve; obtain it inside BlockContext::when_furnace.",
    minecraft = "Verified for 26.2 furnace, blast furnace and smoker; entry validity is not a persistent runtime type guarantee.",
    use_when = ["Combining machine processing state with State, Conditions and item actions"],
    avoid_when = ["Loading chunks, observing machines automatically, or transferring items transactionally"],
    example = "use sand::world::Furnace;",
)]
pub struct Furnace {
    block: BlockContext,
}

impl Furnace {
    /// Addresses the recipe input through the canonical ItemLocation.
    /// This retains an address, not a type proof. Explicit writes follow vanilla
    /// item commands and do not enforce GUI insertion or recipe rules.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::world::Furnace::input", module = "sand::world", kind = "method",
        summary = "Addresses the furnace-family recipe input.",
        context = "Reuses ItemLocation for matching, reading, setting and clearing; no new slot taxonomy.",
        minecraft = "Uses container.0 on the addressed block. Invalid runtime targets fail normally; item commands do not enforce GUI slot permissions.",
        use_when = ["Interacting with the machine's recipe input"],
        avoid_when = ["Assuming an atomic item transfer or persistent container validity"],
        returns = "The existing live ItemLocation for container slot 0.",
        example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).when_furnace(|f| f.input().clear().unwrap());",
    )]
    pub fn input(&self) -> ItemLocation {
        ItemLocation::block(self.block.position.clone())
            .slot(0)
            .expect("verified furnace slot")
    }

    /// Addresses the fuel stack through the canonical ItemLocation.
    /// This retains an address, not a type proof. Explicit writes follow vanilla
    /// item commands and do not enforce GUI insertion or recipe rules.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::world::Furnace::fuel", module = "sand::world", kind = "method",
        summary = "Addresses the furnace-family fuel stack.",
        context = "Reuses ItemLocation for matching, reading, setting and clearing; no new slot taxonomy.",
        minecraft = "Uses container.1 on the addressed block. Invalid runtime targets fail normally; item commands do not enforce GUI slot permissions.",
        use_when = ["Interacting with the machine's fuel stack"],
        avoid_when = ["Assuming an atomic item transfer or persistent container validity"],
        returns = "The existing live ItemLocation for container slot 1.",
        example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).when_furnace(|f| f.fuel().clear().unwrap());",
    )]
    pub fn fuel(&self) -> ItemLocation {
        ItemLocation::block(self.block.position.clone())
            .slot(1)
            .expect("verified furnace slot")
    }

    /// Addresses the recipe output through the canonical ItemLocation.
    /// This retains an address, not a type proof. Explicit writes follow vanilla
    /// item commands and do not enforce GUI insertion or recipe rules.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::world::Furnace::output", module = "sand::world", kind = "method",
        summary = "Addresses the furnace-family recipe output.",
        context = "Reuses ItemLocation for matching, reading, setting and clearing; no new slot taxonomy.",
        minecraft = "Uses container.2 on the addressed block. Invalid runtime targets fail normally; item commands do not enforce GUI slot permissions.",
        use_when = ["Interacting with the machine's recipe output"],
        avoid_when = ["Assuming an atomic item transfer or persistent container validity"],
        returns = "The existing live ItemLocation for container slot 2.",
        example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).when_furnace(|f| f.output().clear().unwrap());",
    )]
    pub fn output(&self) -> ItemLocation {
        ItemLocation::block(self.block.position.clone())
            .slot(2)
            .expect("verified furnace slot")
    }

    /// Reads the current cooking progress in ticks as a canonical runtime expression.
    /// Progress can reset or cool down; it is not lifetime production.
    /// Vanilla serializes its int as a signed short; out-of-range values narrow
    /// rather than clamp. Missing/failed reads do not become zero and abort the
    /// numeric assignment before commit. This is a live read at eventual context,
    /// not a snapshot or an automatically observed State dependency.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::world::Furnace::cooking_elapsed", module = "sand::world", kind = "method",
        summary = "Reads the furnace-family current cooking progress in serialized ticks.",
        context = "Returns StatCurve for canonical State assignment, arithmetic and Conditions. Progress can reset or cool down; it is not lifetime production.",
        minecraft = "Reads 26.2 cooking_time_spent through canonical block NBT; signed-short serialization and numeric failure rules are preserved, including cached or escaped reads.",
        use_when = ["Combining native machine processing with State and Conditions"],
        avoid_when = ["Treating unavailable data as idle, or assuming a handle captures a dimension"],
        returns = "A StatCurve reading the command-visible signed-short tick value.",
        example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).when_furnace(|f| when(f.cooking_elapsed().gt(0)).then_all([f.output().clear().unwrap()]));",
    )]
    pub fn cooking_elapsed(&self) -> StatCurve {
        self.read("cooking_time_spent", "furnace cooking_elapsed")
    }

    /// Reads the stored cooking total in ticks as a canonical runtime expression.
    /// The total can be zero or the native 200-tick no-recipe fallback; it does not prove a recipe is available.
    /// Vanilla serializes its int as a signed short; out-of-range values narrow
    /// rather than clamp. Missing/failed reads do not become zero and abort the
    /// numeric assignment before commit. This is a live read at eventual context,
    /// not a snapshot or an automatically observed State dependency.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::world::Furnace::cooking_total", module = "sand::world", kind = "method",
        summary = "Reads the furnace-family stored cooking total in serialized ticks.",
        context = "Returns StatCurve for canonical State assignment, arithmetic and Conditions. The total can be zero or the native 200-tick no-recipe fallback; it does not prove a recipe is available.",
        minecraft = "Reads 26.2 cooking_total_time through canonical block NBT; signed-short serialization and numeric failure rules are preserved, including cached or escaped reads.",
        use_when = ["Combining native machine processing with State and Conditions"],
        avoid_when = ["Treating unavailable data as idle, or assuming a handle captures a dimension"],
        returns = "A StatCurve reading the command-visible signed-short tick value.",
        example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).when_furnace(|f| when(f.cooking_total().gt(0)).then_all([f.output().clear().unwrap()]));",
    )]
    pub fn cooking_total(&self) -> StatCurve {
        self.read("cooking_total_time", "furnace cooking_total")
    }

    /// Reads the remaining burn time in ticks as a canonical runtime expression.
    /// Burning does not imply productive cooking. Smoker and blast-furnace fuel scaling is already reflected in this value.
    /// Vanilla serializes its int as a signed short; out-of-range values narrow
    /// rather than clamp. Missing/failed reads do not become zero and abort the
    /// numeric assignment before commit. This is a live read at eventual context,
    /// not a snapshot or an automatically observed State dependency.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::world::Furnace::burn_remaining", module = "sand::world", kind = "method",
        summary = "Reads the furnace-family remaining burn time in serialized ticks.",
        context = "Returns StatCurve for canonical State assignment, arithmetic and Conditions. Burning does not imply productive cooking. Smoker and blast-furnace fuel scaling is already reflected in this value.",
        minecraft = "Reads 26.2 lit_time_remaining through canonical block NBT; signed-short serialization and numeric failure rules are preserved, including cached or escaped reads.",
        use_when = ["Combining native machine processing with State and Conditions"],
        avoid_when = ["Treating unavailable data as idle, or assuming a handle captures a dimension"],
        returns = "A StatCurve reading the command-visible signed-short tick value.",
        example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).when_furnace(|f| when(f.burn_remaining().gt(0)).then_all([f.output().clear().unwrap()]));",
    )]
    pub fn burn_remaining(&self) -> StatCurve {
        self.read("lit_time_remaining", "furnace burn_remaining")
    }

    fn read(&self, key: &str, capability: &'static str) -> StatCurve {
        let reference = self.block.data().typed_path::<i16>(NbtPath::root(key));
        StatCurve::native_nbt(
            reference.__location().clone(),
            reference.path_value().clone(),
            Some((PROFILE, capability)),
        )
    }
}
