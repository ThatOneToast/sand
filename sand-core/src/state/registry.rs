//! Immutable lifecycle descriptors emitted by `#[derive(State)]`.
//!
//! Descriptors are collected at link time and rebuilt into deterministic,
//! export-scoped load/init/tick work for every export.

use std::collections::BTreeMap;

use super::score::objective_name;

/// One automatically exported typed-state lifecycle declaration.
///
/// Compiler descriptor emitted by `#[derive(State)]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateLifecycle {
    objective: &'static str,
    criterion: &'static str,
    default: Option<i32>,
    auto_tick: bool,
    scope: StateScope,
}

/// Runtime scope for a derived state schema's automatic lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateScope {
    /// State owned by each online player.
    Player,
    /// Singleton state stored on a deterministic fake score holder.
    Global(&'static str),
}

impl StateLifecycle {
    /// Declare a dummy scoreboard-backed state objective.
    pub const fn score(objective: &'static str) -> Self {
        Self {
            objective,
            criterion: "dummy",
            default: None,
            auto_tick: false,
            scope: StateScope::Player,
        }
    }

    /// Override the vanilla scoreboard criterion.
    pub const fn criterion(mut self, criterion: &'static str) -> Self {
        self.criterion = criterion;
        self
    }

    /// Initialize owners that do not yet have a score without overwriting
    /// existing progress.
    pub const fn default(mut self, value: i32) -> Self {
        self.default = Some(value);
        self
    }

    /// Opt this state into countdown ticking.
    pub const fn auto_tick(mut self) -> Self {
        self.auto_tick = true;
        self
    }

    /// Bind this lifecycle to singleton global state.
    #[doc(hidden)]
    pub const fn global(mut self, holder: &'static str) -> Self {
        self.scope = StateScope::Global(holder);
        self
    }
}

/// Link-time descriptor for an automatically managed state declaration.
pub struct StateDescriptor {
    pub lifecycle: StateLifecycle,
}

impl StateDescriptor {
    pub const fn new(lifecycle: StateLifecycle) -> Self {
        Self { lifecycle }
    }
}

inventory::collect!(StateDescriptor);

/// Deterministic automatic lifecycle output built afresh for each export.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct AutomaticLifecycle {
    pub load_commands: Vec<String>,
    pub player_init_commands: Vec<String>,
    pub player_tick_commands: Vec<String>,
    pub global_init_commands: Vec<String>,
    pub global_tick_commands: Vec<String>,
}

/// Resolve link-time state declarations.
///
/// Identical declarations deduplicate. Any criterion, default, auto-tick, or
/// scope disagreement for the same resolved objective is reported before
/// output is written.
pub(crate) fn automatic_lifecycle() -> Result<AutomaticLifecycle, String> {
    automatic_lifecycle_from(
        inventory::iter::<StateDescriptor>().map(|descriptor| descriptor.lifecycle.clone()),
    )
}

fn automatic_lifecycle_from(
    declarations: impl IntoIterator<Item = StateLifecycle>,
) -> Result<AutomaticLifecycle, String> {
    let mut states: BTreeMap<String, (&'static str, Option<i32>, bool, StateScope)> =
        BTreeMap::new();
    let mut declarations: Vec<_> = declarations.into_iter().collect();
    declarations.sort_by_key(|declaration| {
        (
            objective_name(declaration.objective),
            declaration.criterion,
            declaration.default,
            declaration.auto_tick,
            declaration.scope,
        )
    });

    for declaration in declarations {
        let objective = objective_name(declaration.objective);
        let definition = (
            declaration.criterion,
            declaration.default,
            declaration.auto_tick,
            declaration.scope,
        );
        match states.get(&objective) {
            Some(existing) if existing == &definition => {}
            Some(existing) => {
                return Err(format!(
                    "conflicting automatic state `{objective}`: first declaration has criterion `{}`, default {:?}, auto_tick {}, scope {:?}; conflicting declaration has criterion `{}`, default {:?}, auto_tick {}, scope {:?}",
                    existing.0,
                    existing.1,
                    existing.2,
                    existing.3,
                    definition.0,
                    definition.1,
                    definition.2,
                    definition.3
                ));
            }
            None => {
                states.insert(objective, definition);
            }
        }
    }

    let mut output = AutomaticLifecycle::default();
    for (objective, (criterion, default, auto_tick, scope)) in states {
        output
            .load_commands
            .push(format!("scoreboard objectives add {objective} {criterion}"));
        let holder = match scope {
            StateScope::Player => "@s",
            StateScope::Global(holder) => holder,
        };
        if let Some(default) = default {
            let command = format!(
                "execute unless score {holder} {objective} matches -2147483648.. run scoreboard players set {holder} {objective} {default}"
            );
            match scope {
                StateScope::Player => output.player_init_commands.push(command),
                StateScope::Global(_) => output.global_init_commands.push(command),
            }
        }
        if auto_tick {
            let command = format!(
                "execute if score {holder} {objective} matches 1.. run scoreboard players remove {holder} {objective} 1"
            );
            match scope {
                StateScope::Player => output.player_tick_commands.push(command),
                StateScope::Global(_) => output.global_tick_commands.push(command),
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{StateLifecycle, automatic_lifecycle_from};

    #[test]
    fn automatic_declarations_sort_dedupe_and_generate_player_safe_commands() {
        let output = automatic_lifecycle_from([
            StateLifecycle::score("z_timer").default(0).auto_tick(),
            StateLifecycle::score("alpha").default(100),
            StateLifecycle::score("z_timer").default(0).auto_tick(),
        ])
        .unwrap();

        assert_eq!(
            output.load_commands,
            vec![
                "scoreboard objectives add alpha dummy",
                "scoreboard objectives add z_timer dummy",
            ]
        );
        assert_eq!(
            output.player_init_commands,
            vec![
                "execute unless score @s alpha matches -2147483648.. run scoreboard players set @s alpha 100",
                "execute unless score @s z_timer matches -2147483648.. run scoreboard players set @s z_timer 0",
            ]
        );
        assert_eq!(
            output.player_tick_commands,
            vec![
                "execute if score @s z_timer matches 1.. run scoreboard players remove @s z_timer 1"
            ]
        );
    }

    #[test]
    fn automatic_global_state_never_scans_entities() {
        let output = automatic_lifecycle_from([StateLifecycle::score("world_phase")
            .default(1)
            .auto_tick()
            .global("#world")])
        .unwrap();

        assert!(output.player_init_commands.is_empty());
        assert!(output.player_tick_commands.is_empty());
        assert_eq!(
            output.global_init_commands,
            vec![
                "execute unless score #world world_phase matches -2147483648.. run scoreboard players set #world world_phase 1"
            ]
        );
        assert_eq!(
            output.global_tick_commands,
            vec![
                "execute if score #world world_phase matches 1.. run scoreboard players remove #world world_phase 1"
            ]
        );
    }

    #[test]
    fn automatic_conflicts_report_all_lifecycle_options() {
        let err = automatic_lifecycle_from([
            StateLifecycle::score("mana").default(100),
            StateLifecycle::score("mana").default(0),
        ])
        .unwrap_err();
        assert!(err.contains("conflicting automatic state `mana`"));
        assert!(err.contains("default Some(100)"));
        assert!(err.contains("default Some(0)"));
    }
}
