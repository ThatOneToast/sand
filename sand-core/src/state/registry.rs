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
    display_name: Option<&'static str>,
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
            display_name: None,
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

    /// Set the objective's optional JSON text display name.
    pub const fn display_name(mut self, display_name: &'static str) -> Self {
        self.display_name = Some(display_name);
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

#[derive(Debug, PartialEq, Eq)]
struct ResolvedLifecycle {
    criterion: &'static str,
    display_name: Option<&'static str>,
    default: Option<i32>,
    auto_tick: bool,
    scope: StateScope,
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
    let mut states: BTreeMap<String, ResolvedLifecycle> = BTreeMap::new();
    let mut declarations: Vec<_> = declarations.into_iter().collect();
    declarations.sort_by_key(|declaration| {
        (
            objective_name(declaration.objective),
            declaration.criterion,
            declaration.display_name,
            declaration.default,
            declaration.auto_tick,
            declaration.scope,
        )
    });

    for declaration in declarations {
        let objective = objective_name(declaration.objective);
        let definition = ResolvedLifecycle {
            criterion: declaration.criterion,
            display_name: declaration.display_name,
            default: declaration.default,
            auto_tick: declaration.auto_tick,
            scope: declaration.scope,
        };
        match states.get(&objective) {
            Some(existing) if existing == &definition => {}
            Some(existing) => {
                return Err(format!(
                    "conflicting automatic state `{objective}`: first declaration has criterion `{}`, display name {:?}, default {:?}, auto_tick {}, scope {:?}; conflicting declaration has criterion `{}`, display name {:?}, default {:?}, auto_tick {}, scope {:?}",
                    existing.criterion,
                    existing.display_name,
                    existing.default,
                    existing.auto_tick,
                    existing.scope,
                    definition.criterion,
                    definition.display_name,
                    definition.default,
                    definition.auto_tick,
                    definition.scope
                ));
            }
            None => {
                states.insert(objective, definition);
            }
        }
    }

    let mut output = AutomaticLifecycle::default();
    for (objective, state) in states {
        let display_name = state
            .display_name
            .map(|name| {
                format!(
                    " {}",
                    serde_json::to_string(name).expect("string serializes")
                )
            })
            .unwrap_or_default();
        output.load_commands.push(format!(
            "scoreboard objectives add {objective} {}{display_name}",
            state.criterion
        ));
        let holder = match state.scope {
            StateScope::Player => "@s",
            StateScope::Global(holder) => holder,
        };
        if let Some(default) = state.default {
            let command = format!(
                "execute unless score {holder} {objective} matches -2147483648.. run scoreboard players set {holder} {objective} {default}"
            );
            match state.scope {
                StateScope::Player => output.player_init_commands.push(command),
                StateScope::Global(_) => output.global_init_commands.push(command),
            }
        }
        if state.auto_tick {
            let command = format!(
                "execute if score {holder} {objective} matches 1.. run scoreboard players remove {holder} {objective} 1"
            );
            match state.scope {
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
