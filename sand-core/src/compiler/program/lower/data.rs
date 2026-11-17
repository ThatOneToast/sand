//! Owned storage lowering. Only integer indices enter macro arguments.
use super::*;

impl Lower {
    pub(super) fn is_data_field(&self, field: &FieldReference) -> bool {
        self.data_states
            .get(&field.state.to_string())
            .is_some_and(|state| {
                state
                    .data_fields
                    .iter()
                    .any(|candidate| candidate.name == field.field)
            })
    }

    pub(super) fn copy_data_field(
        &self,
        field: &FieldReference,
        storage: &str,
        path: &str,
        write: bool,
    ) -> Vec<String> {
        crate::state::owned::with_data_field(
            &self.data_states[&field.state.to_string()],
            &field.field,
            |descriptor| {
                crate::entity::state::state_data_copy_commands(descriptor, storage, path, write)
            },
        )
    }

    fn data_selection(
        &self,
        reference: &DataReference,
        storage: &str,
        root: &str,
        commands: &mut Vec<String>,
        indices: &mut usize,
    ) -> String {
        commands.push(format!("data remove storage {storage} {root}"));
        commands.extend(self.copy_data_field(&reference.field, storage, root, false));
        let mut path = root.to_owned();
        for projection in &reference.path {
            match projection {
                DataProjection::Member { name } => {
                    path.push('.');
                    path.push_str(name);
                }
                DataProjection::Each => path.push_str("[]"),
                DataProjection::Index {
                    index: ScoreOperand::Value(index),
                } => path.push_str(&format!("[{index}]")),
                DataProjection::Index {
                    index: ScoreOperand::Field(index),
                } => {
                    let argument = format!("i{}", *indices);
                    *indices += 1;
                    let holder = &self.holders[&index.state.to_string()];
                    let objective = objective(index);
                    commands.push(format!("execute store result storage {storage} args.{argument} int 1 run scoreboard players get {holder} {objective}"));
                    commands.push(format!("execute unless score {holder} {objective} matches 0.. run data modify storage {storage} args.valid set value 0b"));
                    path.push_str(&format!("[$({argument})]"));
                }
            }
        }
        path
    }

    pub(super) fn data_operation(
        &mut self,
        action: &Action,
        owner: &str,
        path: &str,
        module: &str,
        ptr: &str,
    ) -> Result<Vec<Cmd>, Vec<Diagnostic>> {
        let storage = format!("{}:__program_data", self.namespace);
        let mut commands = vec![format!(
            "data modify storage {storage} args set value {{valid:1b}}"
        )];
        let mut indices = 0;
        let mut body = Vec::new();
        let mut result = Vec::new();
        let mut written_score = None;
        match action {
            Action::DataModify {
                target,
                operator,
                source,
            } => {
                // Copy both sides before changing either, including a projected
                // source of the same root as the target.
                let source = match source {
                    DataOperand::Reference { value } => {
                        self.data_selection(value, &storage, "input", &mut commands, &mut indices)
                    }
                    DataOperand::Value { value } => {
                        commands.push(format!(
                            "data modify storage {storage} input set value {value}"
                        ));
                        "input".into()
                    }
                    DataOperand::Score { value } => {
                        match value {
                            ScoreOperand::Value(value) => commands.push(format!("data modify storage {storage} input set value {value}")),
                            ScoreOperand::Field(field) => commands.push(format!("execute store result storage {storage} input int 1 run scoreboard players get {} {}", self.holders[&field.state.to_string()], objective(field))),
                        }
                        "input".into()
                    }
                };
                let target_path =
                    self.data_selection(target, &storage, "output", &mut commands, &mut indices);
                let operator = match operator {
                    sand_commands::DataModifyOperation::Set => "set".into(),
                    sand_commands::DataModifyOperation::Append => "append".into(),
                    sand_commands::DataModifyOperation::Prepend => "prepend".into(),
                    sand_commands::DataModifyOperation::Insert(index) => format!("insert {index}"),
                    sand_commands::DataModifyOperation::Merge => "merge".into(),
                };
                body.push(format!("data modify storage {storage} {target_path} {operator} from storage {storage} {source}"));
                body.extend(self.copy_data_field(&target.field, &storage, "output", true));
            }
            Action::DataRemove { target } => {
                if target.path.is_empty() {
                    return Ok(crate::state::owned::with_data_field(
                        &self.data_states[&target.field.state.to_string()],
                        &target.field.field,
                        crate::entity::state::state_data_remove_commands,
                    )
                    .into_iter()
                    .map(Cmd::Raw)
                    .collect());
                }
                let target_path =
                    self.data_selection(target, &storage, "output", &mut commands, &mut indices);
                body.push(format!("data remove storage {storage} {target_path}"));
                body.extend(self.copy_data_field(&target.field, &storage, "output", true));
            }
            Action::DataRead { score, source } => {
                written_score = Some(score);
                let source =
                    self.data_selection(source, &storage, "input", &mut commands, &mut indices);
                let holder = &self.holders[&score.state.to_string()];
                let objective = objective(score);
                commands.push(format!("scoreboard players set {holder} {objective} 0"));
                body.push(format!("execute store result score {holder} {objective} run data get storage {storage} {source}"));
            }
            Action::DataEqual { score, left, right } => {
                written_score = Some(score);
                let left = self.data_selection(left, &storage, "left", &mut commands, &mut indices);
                let right =
                    self.data_selection(right, &storage, "right", &mut commands, &mut indices);
                let holder = &self.holders[&score.state.to_string()];
                let objective = objective(score);
                commands.push(format!("scoreboard players set {holder} {objective} 0"));
                body.push(format!("data remove storage {storage} compare"));
                body.push(format!(
                    "data modify storage {storage} compare set from storage {storage} {left}"
                ));
                body.push(format!("execute store success storage {storage} unequal byte 1 run data modify storage {storage} compare set from storage {storage} {right}"));
                body.push(format!("execute if data storage {storage} {left} if data storage {storage} {right} if data storage {storage} {{unequal:0b}} run scoreboard players set {holder} {objective} 1"));
            }
            _ => unreachable!("only storage operations use storage lowering"),
        }
        let name = helper(
            &self.namespace,
            &self.helper_keys[owner],
            path,
            "data_value",
        );
        let body = body
            .into_iter()
            .map(|line| {
                if line.contains("$(") {
                    format!("${line}")
                } else {
                    line
                }
            })
            .collect();
        self.function(&name, body, module, ptr)?;
        let call = if indices == 0 {
            format!("function {name}")
        } else {
            format!("function {name} with storage {storage} args")
        };
        commands.push(if indices == 0 {
            call
        } else {
            format!("execute if data storage {storage} args{{valid:1b}} run {call}")
        });
        // No callback or user code can run between staging and the final copy.
        // Keep the scratch bounded and free arbitrary previous authored values.
        commands.push(format!("data remove storage {storage} input"));
        commands.push(format!("data remove storage {storage} output"));
        commands.push(format!("data remove storage {storage} left"));
        commands.push(format!("data remove storage {storage} right"));
        commands.push(format!("data remove storage {storage} compare"));
        commands.push(format!("data remove storage {storage} args"));
        result.extend(commands.into_iter().map(Cmd::Raw));
        if let Some(score) = written_score
            && let Some(bounds) = self
                .bounds
                .get(&format!("{}.{}", score.state, score.field))
                .copied()
        {
            result.extend(crate::ir::score_bounds(
                &self.holders[&score.state.to_string()],
                &objective(score),
                bounds,
            ));
        }
        Ok(result)
    }
}
