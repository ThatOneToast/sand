//! Typed runtime block queries and writes retained through export.
use super::{Actions, Cmd};
use crate::IntoCommands;
use sand_commands::{
    BlockPos, BlockState, CommandProfile, CommandResult, ConditionIr, ExecuteOp, RenderCommand,
    SetBlock, SetBlockMode, Validate,
};
use sand_components::ResourceLocation;

/// A block check owned by a composable Condition.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMatch {
    pub(crate) position: BlockPos,
    pub(crate) block: BlockState,
}

impl BlockMatch {
    pub(crate) fn register(&self) -> String {
        self.reference(true)
    }

    /// Compute the same owned identity without registering discarded Actions.
    pub(crate) fn reference(&self, register: bool) -> String {
        let body = Actions(vec![Cmd::BlockMatch(Box::new(self.clone()))]);
        let prefix = "sand/conditions/blocks";
        let path = if register {
            crate::function::register_dyn_fn_dedup(prefix, body)
        } else {
            crate::function::generated_function_path(prefix, &body)
        };
        format!("{}:{path}", crate::function::SAND_LOCAL_NS)
    }

    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let profile = CommandProfile::unprofiled();
        self.position
            .validate(&profile)
            .map_err(|e| e.with_context(format!("{owner}: block query position")))?;
        self.block
            .validate(&profile)
            .map_err(|e| e.with_context(format!("{owner}: block query state")))?;
        Ok(vec![
            Cmd::Execute {
                operations: vec![ExecuteOp::If(ConditionIr::Block {
                    position: self.position.clone(),
                    block: self.block.to_string(),
                })],
                run: Box::new(Cmd::Return(1)),
            },
            Cmd::Return(0),
        ])
    }
}

/// A block replacement/destruction owned by gameplay Actions.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMutation {
    pub(crate) position: BlockPos,
    pub(crate) block: BlockState,
    pub(crate) mode: SetBlockMode,
}

impl BlockMutation {
    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let line = SetBlock::new(self.position.clone(), self.block.clone())
            .mode(self.mode)
            .render(&CommandProfile::unprofiled())
            .map_err(|e| e.with_context(format!("{owner}: block mutation")))?;
        Ok(line.into_commands().0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_states_keep_sorted_properties_in_queries_and_writes() {
        let owner = "game:world".parse().unwrap();
        let position = BlockPos::here();
        let block = BlockState::of("minecraft:chest")
            .prop("waterlogged", "false")
            .prop("facing", "north");
        let query = BlockMatch {
            position: position.clone(),
            block: block.clone(),
        };
        let lines = query
            .lower(&owner)
            .unwrap()
            .iter()
            .map(Cmd::try_render)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            lines,
            [
                "execute if block ~ ~ ~ minecraft:chest[facing=north,waterlogged=false] run return 1",
                "return 0",
            ]
        );
        let mutation = BlockMutation {
            position,
            block,
            mode: SetBlockMode::Replace,
        };
        let lines = mutation
            .lower(&owner)
            .unwrap()
            .iter()
            .map(Cmd::try_render)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            lines,
            ["setblock ~ ~ ~ minecraft:chest[facing=north,waterlogged=false]"]
        );
    }

    #[test]
    fn block_queries_and_writes_reject_invalid_typed_inputs() {
        let owner = "game:world".parse().unwrap();
        for (position, block) in [
            (BlockPos::new(0.5, 64, 0), BlockState::of("minecraft:stone")),
            (BlockPos::here(), BlockState::of("invalid block")),
            (
                BlockPos::here(),
                BlockState::of("minecraft:stone").prop("bad key", "value"),
            ),
        ] {
            let query = BlockMatch {
                position: position.clone(),
                block: block.clone(),
            };
            assert!(query.lower(&owner).is_err());
            let mutation = BlockMutation {
                position,
                block,
                mode: SetBlockMode::Replace,
            };
            assert!(mutation.lower(&owner).is_err());
        }
    }
}
