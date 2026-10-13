//! Execution-scoped runtime world and block gameplay.
//!
//! These handles describe work in the current Minecraft execution context.
//! They do not load chunks, scan regions, or configure a generated world.

use crate::{
    IntoCommands,
    condition::Condition,
    ir::{Actions, Cmd, ExecuteOp},
};
use sand_commands::{BlockPos, BlockState, Nbt, SetBlockMode};
use sand_components::registry::DimensionId;

/// Runtime operations in the current execution dimension and position.
///
/// Unlike [`crate::build::World`], this is not world-generation configuration.
/// A handle describes the context where its actions eventually run; it does
/// not capture a persistent world identity or position at Rust construction.
/// Use [`Self::in_dimension`] to scope a whole gameplay body to a dimension.
#[derive(Debug, Clone, Copy)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::WorldContext",
    aliases = ["sand::prelude::WorldContext"],
    module = "sand::world",
    summary = "Describes runtime block gameplay in the current execution context.",
    context = "Shares canonical block positions, states, inventories, and NBT instead of creating a separate world state model.",
    minecraft = "Uses the command execution dimension and position; construction creates no scans, chunk loads, or lifecycle work.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],


    example = "use sand::prelude::*;",
)]
pub struct WorldContext {
    _private: (),
}

impl WorldContext {
    /// Uses the execution dimension and position of the generated function.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::WorldContext::current",
    aliases = ["sand::prelude::WorldContext::current"],
    module = "sand::world",
    kind = "method",
    summary = "Accesses the current runtime world context.",
    context = "The resulting handle is execution-scoped; it is not a snapshot or a persistent world reference.",
    minecraft = "Leaves the caller dimension, position, rotation, and executor unchanged.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],

    returns = "An execution-scoped WorldContext.",
    example = "use sand::prelude::*; let world = WorldContext::current();",
)]
    pub const fn current() -> Self {
        Self { _private: () }
    }

    /// Selects players in the dimension where the actions execute.
    ///
    /// Returns the canonical Target, so filters, cardinality narrowing, and
    /// [`crate::entity::TargetExecution::each`] retain their existing behavior.
    /// Without an explicit distance filter, a zero minimum distance pins
    /// selection to the execution dimension without a maximum radius. Add
    /// `within_blocks` to bound nearby work.
    /// Construction generates no scan. Iteration runs explicitly and binds
    /// each selected entity's executor and position. This is not a persistent
    /// dimension reference; use the target inside the intended dimension scope.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::world::WorldContext::players",
        aliases = ["sand::prelude::WorldContext::players"],
        module = "sand::world",
        kind = "method",
        summary = "Selects players in the current execution dimension.",
        context = "Returns the canonical Target with its existing filters, cardinality, and typed iteration; no parallel world collection is introduced.",
        minecraft = "A supplied distance filter constrains selection to the execution dimension; otherwise Sand adds distance=0.. without a maximum radius. Iteration establishes each selected entity's position and executor.",
        use_when = ["Selecting players for dimension-scoped gameplay"],
        avoid_when = ["Capturing a persistent dimension identity", "Creating implicit tick scans"],
        returns = "A canonical typed target constrained to the current dimension.",
        example = "use sand::prelude::*; let nearby = WorldContext::current().players().within_blocks(32.0);",
    )]
    pub fn players(&self) -> sand_commands::selector::Target<sand_commands::selector::PlayersOnly> {
        sand_commands::selector::Target::players().in_current_dimension()
    }

    /// Selects loaded entities in the dimension where the actions execute.
    ///
    /// Returns the canonical Target, so filters, cardinality narrowing, and
    /// [`crate::entity::TargetExecution::each`] retain their existing behavior.
    /// Without an explicit distance filter, a zero minimum distance pins
    /// selection to the execution dimension without a maximum radius. Add
    /// `within_blocks` to bound nearby work.
    /// Construction generates no scan. Iteration runs explicitly and binds
    /// each selected entity's executor and position. This is not a persistent
    /// dimension reference; use the target inside the intended dimension scope.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::world::WorldContext::entities",
        aliases = ["sand::prelude::WorldContext::entities"],
        module = "sand::world",
        kind = "method",
        summary = "Selects loaded entities in the current execution dimension.",
        context = "Returns the canonical Target with its existing filters, cardinality, and typed iteration; no parallel world collection is introduced.",
        minecraft = "A supplied distance filter constrains selection to the execution dimension; otherwise Sand adds distance=0.. without a maximum radius. Iteration establishes each selected entity's position and executor.",
        use_when = ["Selecting loaded entities for dimension-scoped gameplay"],
        avoid_when = ["Capturing a persistent dimension identity", "Creating implicit tick scans"],
        returns = "A canonical typed target constrained to the current dimension.",
        example = "use sand::prelude::*; let nearby = WorldContext::current().entities().within_blocks(32.0);",
    )]
    pub fn entities(&self) -> sand_commands::selector::Target<sand_commands::selector::AnyTarget> {
        sand_commands::selector::Target::entities().in_current_dimension()
    }

    /// Addresses a block using canonical absolute or relative block coordinates.
    /// Relative coordinates are resolved when the generated operation executes.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::WorldContext::block",
    aliases = ["sand::prelude::WorldContext::block"],
    module = "sand::world",
    kind = "method",
    summary = "Addresses one runtime block at a typed position.",
    context = "The handle retains BlockPos for checks, mutations, container addressing, and block data; it does not prove a block is loaded or has inventory.",
    minecraft = "Coordinates are resolved in the current execution dimension and position at runtime.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],
    params(position = "Canonical absolute or relative block coordinates."),
    returns = "A BlockContext for composing checks and actions.",
    example = "use sand::prelude::*; let block = WorldContext::current().block(BlockPos::here());",
)]
    pub fn block(&self, position: BlockPos) -> BlockContext {
        BlockContext { position }
    }

    /// Runs an owned action body in an explicit dimension.
    ///
    /// Minecraft scales the execution origin when crossing dimensions with
    /// different coordinate scales. Absolute block positions remain absolute;
    /// relative positions use that resulting origin. The caller's executor is
    /// retained. A nested entity query may establish its own position/dimension.
    /// The body does not load chunks, and the surrounding caller is unchanged.
    /// Straight-line function-macro actions stay in the argument-bearing caller.
    /// A macro body that returns early is rejected at export; use an explicit
    /// macro function with [`crate::cmd::FunctionMacroArgs::call_with`] instead.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::WorldContext::in_dimension",
    aliases = ["sand::prelude::WorldContext::in_dimension"],
    module = "sand::world",
    kind = "method",
    summary = "Scopes a gameplay action body to a typed dimension.",
    context = "Keeps the entire owned body together so block checks, inventory operations, and data accesses share the selected execution context.",
    minecraft = "Lowers through execute in and an owned helper, preserving vanilla coordinate scaling and caller executor semantics.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],
    params(dimension = "The canonical dimension identifier.", body = "Builds actions in the selected runtime world context."),
    returns = "Owned Actions to execute in the requested dimension.",
    example = "use sand::prelude::*; let dimension: sand::registry::DimensionId = \"minecraft:the_nether\".parse().unwrap(); let actions = WorldContext::current().in_dimension(dimension, |world| world.block(BlockPos::here()).destroy());",
)]
    pub fn in_dimension<R: IntoCommands>(
        &self,
        dimension: impl Into<DimensionId>,
        body: impl FnOnce(Self) -> R,
    ) -> Actions {
        let body = body(Self::current()).into_commands();
        if body.0.is_empty() {
            return Actions::default();
        }
        let operation = ExecuteOp::In(dimension.into().to_string());
        // Macro substitution must happen in the argument-bearing function.
        // Each straight-line command inherits the same caller origin, so each
        // dimension modifier applies vanilla coordinate scaling exactly once.
        if body.has_macro_lines() && !body.may_return_from_frame() {
            return Actions(
                body.0
                    .into_iter()
                    .map(|command| Cmd::Execute {
                        operations: vec![operation.clone()],
                        run: Box::new(command),
                    })
                    .collect(),
            );
        }
        Actions(vec![Cmd::Execute {
            operations: vec![operation],
            run: Box::new(Cmd::AnonymousFunction {
                prefix: "sand/world/dimension".into(),
                body,
            }),
        }])
    }
}

/// A block address interpreted in the current runtime execution context.
///
/// This carries the canonical [`BlockPos`], not a persistent block identity.
/// Relative coordinates and dimension are those of the eventual action body.
/// Construct it through [`WorldContext::block`]. No chunk is loaded implicitly.
#[derive(Debug, Clone)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::BlockContext",
    aliases = ["sand::prelude::BlockContext"],
    module = "sand::world",
    summary = "Composes runtime block checks, mutations, and existing container/data addressing.",
    context = "One canonical BlockPos is reused across the block operations; actual block type and loaded-chunk availability remain runtime facts.",
    minecraft = "Generated operations target the current execution dimension without assuming the Overworld.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],


    example = "use sand::prelude::*;",
)]
pub struct BlockContext {
    position: BlockPos,
}

impl BlockContext {
    /// Tests this block against the canonical block state and optional properties.
    ///
    /// The Condition retains typed validation through composition and cached
    /// exports. An unloaded or mismatching block makes this query false; its
    /// negation consequently also matches an unloaded block.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::BlockContext::matches",
    aliases = ["sand::prelude::BlockContext::matches"],
    module = "sand::world",
    kind = "method",
    summary = "Tests a runtime block against a canonical typed block state.",
    context = "The Condition owns its block position and state until validation and lowering, and composes with State, entity, and inventory conditions.",
    minecraft = "Evaluates one execute-if-block query; unavailable chunks or mismatches produce false.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],
    params(block = "The canonical BlockState or typed vanilla block to match."),
    returns = "A composable Condition for this block address.",
    example = "use sand::prelude::*; let condition = WorldContext::current().block(BlockPos::here()).matches(vanilla::Block::Chest);",
)]
    pub fn matches(&self, block: impl Into<BlockState>) -> Condition {
        Condition::block_match(self.position.clone(), block.into())
    }

    /// Replaces this block without requesting destruction drops.
    /// Container contents and block-entity changes follow vanilla setblock
    /// behavior. This does not imply transactional inventory transfer.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::BlockContext::replace",
    aliases = ["sand::prelude::BlockContext::replace"],
    module = "sand::world",
    kind = "method",
    summary = "Replaces a runtime block using the canonical BlockState.",
    context = "Keeps the typed position and block state in owned mutation IR for export-time validation.",
    minecraft = "Lowers through the typed SetBlock command in replace mode; unloaded targets fail according to vanilla rules.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],
    params(block = "The replacement block and optional state properties."),
    returns = "Owned block-mutation Actions.",
    example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).replace(vanilla::Block::Stone);",
)]
    pub fn replace(&self, block: impl Into<BlockState>) -> Actions {
        self.write(block.into(), SetBlockMode::Replace)
    }

    /// Destroys the block and leaves air, using vanilla destruction/drop behavior.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::BlockContext::destroy",
    aliases = ["sand::prelude::BlockContext::destroy"],
    module = "sand::world",
    kind = "method",
    summary = "Destroys a runtime block and replaces it with air.",
    context = "This action uses the same typed block-mutation lowering as replacement and retains the current execution context.",
    minecraft = "Emits the canonical SetBlock destroy operation; drops and block updates follow vanilla behavior.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],

    returns = "Owned block-destruction Actions.",
    example = "use sand::prelude::*; let actions = WorldContext::current().block(BlockPos::here()).destroy();",
)]
    pub fn destroy(&self) -> Actions {
        self.write(BlockState::of("minecraft:air"), SetBlockMode::Destroy)
    }

    fn write(&self, block: BlockState, mode: SetBlockMode) -> Actions {
        Actions(vec![Cmd::BlockMutation(Box::new(
            crate::ir::BlockMutation {
                position: self.position.clone(),
                block,
                mode,
            },
        ))])
    }

    /// Addresses container slots through Sand's existing inventory model.
    /// This does not prove that the current block is a container or has every
    /// addressed slot. Use a block Condition where needed; vanilla operations
    /// fail if the live block cannot provide the requested container slot.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::BlockContext::inventory",
    aliases = ["sand::prelude::BlockContext::inventory"],
    module = "sand::world",
    kind = "method",
    summary = "Addresses this block through the canonical BlockInventory.",
    context = "Reuses ItemLocation and validated container indices; no second slot or item model is created and container existence is not implied.",
    minecraft = "Item operations resolve the same block position and current dimension as the block handle.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],

    returns = "The existing BlockInventory addressing capability.",
    example = "use sand::prelude::*; let slot = WorldContext::current().block(BlockPos::here()).inventory().slot(0).unwrap();",
)]
    pub fn inventory(&self) -> crate::item::BlockInventory {
        crate::item::ItemLocation::block(self.position.clone())
    }

    /// Starts canonical NBT addressing for a block entity at this position.
    /// This advanced capability does not imply the block has block-entity data;
    /// reads and writes fail in vanilla when the target cannot provide it.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::world::BlockContext::data",
    aliases = ["sand::prelude::BlockContext::data"],
    module = "sand::world",
    kind = "method",
    summary = "Addresses block-entity data through the canonical Nbt root.",
    context = "Reuses existing typed NBT paths, values, and commands rather than inventing a world-specific data schema; block-entity existence remains a runtime condition.",
    minecraft = "Data operations target this position in the active execution dimension and fail if the block has no block entity.",
    use_when = ["Authoring block gameplay in an explicit execution context"],
    avoid_when = ["Configuring a generated world; use build::World", "Keeping a persistent reference to an unloaded block"],

    returns = "The existing Nbt root for this block address.",
    example = "use sand::prelude::*; let data = WorldContext::current().block(BlockPos::here()).data();",
)]
    pub fn data(&self) -> Nbt {
        Nbt::block(self.position.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::TargetExecution;

    #[test]
    fn world_targets_keep_canonical_filters_and_dimension_bounds() {
        let world = WorldContext::current();
        assert_eq!(world.players().to_string(), "@a[distance=0..]");
        assert_eq!(world.entities().to_string(), "@e[distance=0..]");
        assert_eq!(
            world.players().within_blocks(32.0).to_string(),
            "@a[distance=..32]"
        );
        assert_eq!(
            world.entities().nearest().to_string(),
            "@e[sort=nearest,limit=1,distance=0..]"
        );
        let lines = crate::ir::test_support::emitted(world.players().each(|_| {
            world
                .block(BlockPos::here())
                .replace(crate::generated::Block::Stone)
        }));
        assert!(lines[0].starts_with("execute as @a[distance=0..] at @s run function "));
    }

    #[test]
    fn dimension_macro_body_keeps_comment_nodes_as_comments() {
        let dimension: DimensionId = "minecraft:the_nether".parse().unwrap();
        let actions = WorldContext::current().in_dimension(dimension, |_| {
            Actions(vec![
                Cmd::Comment("context".into()),
                Cmd::Raw("$say $(value)".into()),
            ])
        });
        assert_eq!(actions.0[0].try_render().unwrap(), "# context");
        assert_eq!(
            actions.0[1].try_render().unwrap(),
            "$execute in minecraft:the_nether run say $(value)"
        );
    }

    #[test]
    fn dimension_macro_early_return_is_diagnosed_instead_of_leaking_to_caller() {
        let args = crate::cmd::FunctionMacroArgs::new(["value"]).unwrap();
        let dimension: DimensionId = "minecraft:the_nether".parse().unwrap();
        let actions = WorldContext::current()
            .in_dimension(dimension, |_| args.line("return $(value)").unwrap());
        let error = actions.0[0].try_render().unwrap_err().to_string();
        assert!(
            error.contains("macro lines cannot move into a helper"),
            "{error}"
        );
    }
}
