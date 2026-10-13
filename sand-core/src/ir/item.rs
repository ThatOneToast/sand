//! Owned inventory writes using canonical item locations and stack values.
use super::{Actions, Cmd};
use crate::item::ItemLocation;
use sand_commands::inventory::{ItemReplaceCommand, ItemReplaceSource};
use sand_commands::{CommandError, CommandProfile, CommandResult, RenderCommand};
use sand_components::{ItemStack, ResourceLocation};

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct ItemMutation {
    destination: ItemLocation,
    source: ItemMutationSource,
}

#[derive(Debug, Clone)]
enum ItemMutationSource {
    Stack(Box<ItemStack>),
    Clear,
    Location(ItemLocation),
}

impl ItemMutation {
    pub(crate) fn set(destination: ItemLocation, stack: ItemStack) -> Actions {
        Self::actions(destination, ItemMutationSource::Stack(Box::new(stack)))
    }

    pub(crate) fn clear(destination: ItemLocation) -> Actions {
        Self::actions(destination, ItemMutationSource::Clear)
    }

    pub(crate) fn copy(destination: ItemLocation, source: ItemLocation) -> Actions {
        Self::actions(destination, ItemMutationSource::Location(source))
    }

    fn actions(destination: ItemLocation, source: ItemMutationSource) -> Actions {
        Actions(vec![Cmd::ItemMutation(Box::new(Self {
            destination,
            source,
        }))])
    }

    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let operation = match self.source {
            ItemMutationSource::Stack(_) => "inventory set",
            ItemMutationSource::Clear => "inventory clear",
            ItemMutationSource::Location(_) => "inventory copy",
        };
        let error = |field, detail: String| {
            CommandError::new(operation, field, format!("{owner}: {detail}"))
        };
        let destination = self
            .destination
            .item_target_slot()
            .map_err(|e| error("destination", e.to_string()))?;
        let source = match &self.source {
            ItemMutationSource::Stack(stack) => {
                stack
                    .validate()
                    .map_err(|e| error("stack", e.to_string()))?;
                ItemReplaceSource::Stack {
                    item: stack
                        .custom_item()
                        .try_to_string()
                        .map_err(|e| error("stack", e.to_string()))?,
                    count: Some(stack.count_value()),
                }
            }
            ItemMutationSource::Clear => ItemReplaceSource::Clear,
            ItemMutationSource::Location(source) => ItemReplaceSource::Location(
                source
                    .item_target_slot()
                    .map_err(|e| error("source", e.to_string()))?,
            ),
        };
        let line = ItemReplaceCommand {
            destination,
            source,
        }
        .render(&CommandProfile::unprofiled())
        .map_err(|e| error("command", e.to_string()))?;
        Ok(crate::IntoCommands::into_commands(line).0)
    }
}
