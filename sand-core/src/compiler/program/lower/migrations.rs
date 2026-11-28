//! Run portable State transforms at the existing lifecycle boundaries.
use super::*;
use crate::state::{owned, registry::AutomaticLifecycle};

impl Lower<'_> {
    pub(super) fn register_state_migrations(
        &mut self,
        program: &Program,
        automatic: &mut AutomaticLifecycle,
    ) -> Result<(), Vec<Diagnostic>> {
        let mut entity_tick = Vec::new();
        for module in &program.modules {
            let owner = module.id.to_string();
            for (index, state) in module.states.iter().enumerate() {
                if state.migrations.is_empty() {
                    continue;
                }
                let ptr = format!("/states/{index}");
                let base = self.state_migration_function(state);
                let presence = state_objective(&state.id.to_string(), "presence");
                let holder = owned::holder(state);
                let mut dispatch = Vec::new();
                for (step, migration) in state.migrations.iter().enumerate() {
                    let from = step + 1;
                    let to = from + 1;
                    let helper = format!("{base}/{from}");
                    self.function(
                        &helper,
                        vec![
                            format!("function {migration}"),
                            format!("scoreboard players set {holder} {presence} {to}"),
                        ],
                        &owner,
                        &format!("{ptr}/migrations/{step}"),
                    )?;
                    dispatch.push(format!(
                        "execute if score {holder} {presence} matches {from} run function {helper}"
                    ));
                }
                self.function(&base, dispatch, &owner, &ptr)?;
                match state.scope.as_str() {
                    "global" => automatic
                        .global_init_commands
                        .push(format!("function {base}")),
                    "player" => {
                        let suppression = state_objective(&state.id.to_string(), "suppressed");
                        automatic.player_init_commands.extend(guard_data(state, vec![format!(
                            "execute unless score @s {suppression} matches 1.. run function {base}"
                        )]));
                    }
                    "entity" => {
                        let helper = format!("{base}/attach");
                        self.function(
                            &helper,
                            self.entity_attachment_commands(state),
                            &owner,
                            &ptr,
                        )?;
                        let prior = state.revision - 1;
                        for dimension in [
                            "minecraft:overworld",
                            "minecraft:the_nether",
                            "minecraft:the_end",
                        ]
                        .into_iter()
                        .map(str::to_owned)
                        .chain(self.dimensions.iter().map(ToString::to_string))
                        {
                            entity_tick.push(format!(
                                "execute in {dimension} as @e[scores={{{presence}=1..{prior}}}] at @s run function {helper}"
                            ));
                        }
                        automatic
                            .tick_origins
                            .extend([owner.clone(), state.id.to_string()]);
                    }
                    _ => unreachable!("validated State scope"),
                }
            }
        }
        automatic.entity_tick_commands.splice(0..0, entity_tick);
        Ok(())
    }

    fn state_migration_function(&self, state: &State) -> String {
        format!(
            "{}:__sand_program/state_migrate/{}",
            self.namespace,
            state_objective(&state.id.to_string(), "presence"),
        )
    }

    pub(super) fn entity_attachment_commands(&self, state: &State) -> Vec<String> {
        let id = state.id.to_string();
        let mut commands = Vec::new();
        for field in &state.fields {
            let objective = state_objective(&id, &field.name);
            commands.push(format!(
                "execute unless score @s {objective} matches -2147483648.. run scoreboard players set @s {objective} {}",
                field.default
            ));
        }
        for field in &state.data_fields {
            commands.extend(owned::with_data_field(
                state,
                &field.name,
                crate::entity::state::state_data_initialize_commands,
            ));
        }
        let presence = state_objective(&id, "presence");
        if !state.migrations.is_empty() {
            commands.push(format!("function {}", self.state_migration_function(state)));
        }
        commands.push(format!(
            "execute unless score @s {presence} matches 1.. run scoreboard players set @s {presence} {}",
            state.revision
        ));
        if state.migrations.is_empty() {
            commands
        } else {
            guard_data(state, commands)
        }
    }

    pub(super) fn entity_detachment_commands(state: &State) -> Vec<String> {
        let id = state.id.to_string();
        let mut commands = Vec::new();
        for field in &state.fields {
            for name in [&field.name, &format!("{}.dirty", field.name)] {
                commands.push(format!(
                    "scoreboard players reset @s {}",
                    state_objective(&id, name)
                ));
            }
        }
        for field in &state.data_fields {
            commands.extend(owned::with_data_field(
                state,
                &field.name,
                crate::entity::state::state_data_remove_commands,
            ));
        }
        for field in ["reconcile_dirty", "presence"] {
            commands.push(format!(
                "scoreboard players reset @s {}",
                state_objective(&id, field)
            ));
        }
        commands
    }
}

fn guard_data(state: &State, commands: Vec<String>) -> Vec<String> {
    if state.data_fields.is_empty() {
        return commands;
    }
    owned::with_descriptors(std::slice::from_ref(state), |descriptors| {
        crate::entity::keyed_migration::guard(descriptors[0].data_fields, commands)
    })
}
