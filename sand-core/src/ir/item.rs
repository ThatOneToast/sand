//! Owned inventory writes using canonical item locations and stack values.
use super::{Actions, Cmd};
use crate::item::ItemLocation;
use sand_commands::inventory::{ItemReplaceCommand, ItemReplaceSource};
use sand_commands::{CommandError, CommandProfile, CommandResult, RenderCommand};
use sand_components::{ItemStack, ResourceLocation};

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct ItemMutation {
    operation: ItemMutationOperation,
}

#[derive(Debug, Clone)]
enum ItemMutationOperation {
    Replace {
        destination: ItemLocation,
        source: ItemMutationSource,
    },
    Remove {
        target: sand_commands::Selector,
        predicate: Box<sand_components::ItemPredicate>,
        amount: u32,
    },
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
            operation: ItemMutationOperation::Replace {
                destination,
                source,
            },
        }))])
    }

    pub(crate) fn remove(
        target: sand_commands::Selector,
        predicate: sand_components::ItemPredicate,
        amount: u32,
    ) -> Actions {
        Actions(vec![Cmd::ItemMutation(Box::new(Self {
            operation: ItemMutationOperation::Remove {
                target,
                predicate: Box::new(predicate),
                amount,
            },
        }))])
    }

    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        let (destination, source) = match &self.operation {
            ItemMutationOperation::Remove {
                target,
                predicate,
                amount,
            } => {
                let error = |error: CommandError| {
                    CommandError::new("inventory remove", "operation", format!("{owner}: {error}"))
                };
                sand_commands::ScoreHolder::entity(target.clone())
                    .validate_single(&CommandProfile::unprofiled())
                    .map_err(error)?;
                let argument = lower_item_predicate(predicate, owner)?;
                let line = sand_commands::Inventory::of(target.clone())
                    .try_clear_item_count(argument, *amount)
                    .map_err(error)?;
                return Ok(crate::IntoCommands::into_commands(line).0);
            }
            ItemMutationOperation::Replace {
                destination,
                source,
            } => (destination, source),
        };
        let operation = match source {
            ItemMutationSource::Stack(_) => "inventory set",
            ItemMutationSource::Clear => "inventory clear",
            ItemMutationSource::Location(_) => "inventory copy",
        };
        let error = |field, detail: String| {
            CommandError::new(operation, field, format!("{owner}: {detail}"))
        };
        let destination = destination
            .item_target_slot()
            .map_err(|e| error("destination", e.to_string()))?;
        let source = match source {
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

/// Owned slot predicate retained through condition composition and export.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct ItemMatch {
    pub(crate) location: ItemLocation,
    pub(crate) predicate: sand_components::ItemPredicate,
}

impl ItemMatch {
    pub(crate) fn register(&self) -> String {
        let path = crate::function::register_dyn_fn_dedup(
            "sand/conditions/items",
            Actions(vec![Cmd::ItemMatch(Box::new(self.clone()))]),
        );
        format!("{}:{path}", crate::function::SAND_LOCAL_NS)
    }

    pub(super) fn lower(&self, owner: &ResourceLocation) -> CommandResult<Vec<Cmd>> {
        use sand_commands::inventory::ItemCommandLocation;
        use sand_commands::{ConditionIr, ExecuteOp};
        let item = lower_item_predicate(&self.predicate, owner)?;
        let target = self.location.item_target_slot().map_err(|error| {
            CommandError::new("inventory match", "location", format!("{owner}: {error}"))
        })?;
        let condition = match target {
            ItemCommandLocation::Entity { target, slot } => {
                ConditionIr::ItemsEntity { target, slot, item }
            }
            ItemCommandLocation::Block { position, slot } => ConditionIr::ItemsBlock {
                position,
                slot,
                item,
            },
        };
        Ok(vec![
            Cmd::Execute {
                operations: vec![ExecuteOp::If(condition)],
                run: Box::new(Cmd::Return(1)),
            },
            Cmd::Return(0),
        ])
    }
}

pub(crate) fn lower_item_predicate(
    predicate: &sand_components::ItemPredicate,
    owner: &ResourceLocation,
) -> CommandResult<String> {
    let error =
        |detail| CommandError::new("inventory predicate", "item", format!("{owner}: {detail}"));
    let lowered =
        sand_components::predicates::lower_item_predicate_for_command(predicate).map_err(error)?;
    if lowered.requires_components {
        crate::function::require_generated_feature(
            owner,
            sand_version::ComponentFeature::ItemComponents,
        );
    }
    let mut items = lowered.items.unwrap_or_default();
    items.sort_by_key(ToString::to_string);
    items.dedup();
    let base = match items.as_slice() {
        [] => "*".to_owned(),
        [item] => item.to_string(),
        _ => {
            let identity = items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            let path = crate::function::generated_resource_path("sand/items", &identity);
            let location = ResourceLocation::new(owner.namespace(), &path)
                .map_err(|e| error(e.to_string()))?;
            let mut tag =
                sand_components::TypedTag::<sand_components::ItemId>::new(location.clone().into())
                    .replace(true);
            for item in items {
                tag = tag.entry(item);
            }
            crate::function::register_generated_component(tag);
            format!("#{location}")
        }
    };
    Ok(if lowered.constraints.is_empty() {
        base
    } else {
        format!("{base}[{}]", lowered.constraints.join(","))
    })
}
