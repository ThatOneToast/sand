//! Owned settings from the existing version-generated gamerule builders.
use sand_commands::{CommandError, CommandResult};

/// One world gamerule assignment, retaining its generated value constraints.
///
/// Pass a version-generated command builder with `.value(...)` directly to
/// [`crate::build::World::gamerule`]. Sand generates conversions for supported
/// boolean and integer setters from the same command schema as those builders.
/// A query without a value is diagnosed by [`crate::build::SandBuild::validate`].
/// Use [`Self::raw`] only for unsupported server rules or future syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::build::GameRuleSetting",
    module = "sand::build",
    summary = "An owned gamerule assignment from a version-generated typed command builder.",
    context = "World configuration consumes the existing generated gamerule builders through Into<GameRuleSetting>, retaining boolean/integer types and schema bounds without another rule-name catalog.",
    minecraft = "Lowers to one gamerule setter in the world initialization function. Missing values and invalid numeric ranges fail build validation.",
    use_when = ["Configuring a world with a generated gamerule builder"],
    avoid_when = ["Querying a gamerule without assigning a value"],
    example = "use sand::build::{GameRuleSetting, World}; fn configured(setting: GameRuleSetting) -> World { World::new().gamerule(setting) }",
)]
pub struct GameRuleSetting {
    name: String,
    value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Boolean(Option<bool>),
    Integer {
        value: Option<i32>,
        minimum: i32,
        maximum: i32,
    },
    Raw(String),
}

impl GameRuleSetting {
    /// Explicit escape hatch for unsupported rules or future setter syntax.
    ///
    /// Build validation requires nonempty, single-token names and values.
    /// The caller owns rule existence, value semantics, and version support.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::build::GameRuleSetting::raw",
        module = "sand::build",
        kind = "method",
        summary = "Creates an explicit raw gamerule assignment for unsupported server rules.",
        context = "Prefer generated gamerule builders for supported vanilla settings. Raw assignments retain token validation while leaving rule existence and value semantics to the author.",
        minecraft = "Produces gamerule <name> <value> after rejecting empty tokens, whitespace, and control characters.",
        use_when = ["Interoperating with a server rule absent from the generated schema"],
        avoid_when = ["A generated gamerule builder can represent the assignment"],
        params(name = "The unsupported rule's single-token name.", value = "Its single-token value."),
        returns = "An owned raw setting validated when the world build is validated.",
        example = "use sand::build::GameRuleSetting; let setting = GameRuleSetting::raw(\"custom:round_limit\", \"5\");",
    )]
    pub fn raw(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Value::Raw(value.into()),
        }
    }

    pub(crate) fn boolean(name: &str, value: Option<bool>) -> Self {
        Self {
            name: name.into(),
            value: Value::Boolean(value),
        }
    }

    pub(crate) fn integer(name: &str, value: Option<i32>, minimum: i32, maximum: i32) -> Self {
        Self {
            name: name.into(),
            value: Value::Integer {
                value,
                minimum,
                maximum,
            },
        }
    }

    pub(crate) fn key(&self) -> &str {
        // Vanilla exposes both default-namespace spellings in newer schemas.
        self.name.strip_prefix("minecraft:").unwrap_or(&self.name)
    }

    pub(crate) fn command(&self) -> CommandResult<String> {
        sand_commands::validate::no_whitespace_or_control(&self.name, "gamerule", "name")?;
        let missing = || {
            CommandError::new(
                "gamerule",
                "value",
                format!(
                    "{} is a query; world configuration requires an assignment with .value(...)",
                    self.name
                ),
            )
        };
        let value = match &self.value {
            Value::Boolean(value) => value.ok_or_else(missing)?.to_string(),
            Value::Integer {
                value,
                minimum,
                maximum,
            } => {
                let value = value.ok_or_else(missing)?;
                if value < *minimum || value > *maximum {
                    return Err(CommandError::new(
                        "gamerule",
                        "value",
                        format!("{} requires {minimum}..={maximum}, got {value}", self.name),
                    ));
                }
                value.to_string()
            }
            Value::Raw(value) => {
                sand_commands::validate::no_whitespace_or_control(value, "gamerule", "value")?;
                value.clone()
            }
        };
        Ok(format!("gamerule {} {value}", self.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{SandBuild, World, lower_world};

    #[test]
    fn generated_settings_retain_types_bounds_and_assignment_validation() {
        let build = SandBuild::new().world(
            World::new()
                .gamerule(crate::cmd::gamerule_keep_inventory().value(true))
                .gamerule(crate::cmd::gamerule_random_tick_speed().value(0)),
        );
        build.validate().unwrap();
        let records = lower_world("rules", &build);
        let init = records
            .iter()
            .find(|r| r.path == "__sand_world_init")
            .unwrap();
        assert_eq!(
            init.content,
            "gamerule keep_inventory true\ngamerule random_tick_speed 0\n"
        );
        for setting in [
            crate::cmd::gamerule_keep_inventory().into(),
            crate::cmd::gamerule_random_tick_speed().value(-1).into(),
            GameRuleSetting::raw("", "true"),
            GameRuleSetting::raw("keep_inventory", ""),
            GameRuleSetting::raw("keep_inventory\nsay", "true"),
            GameRuleSetting::raw("keep_inventory", "true\nsay injected"),
        ] {
            assert!(
                SandBuild::new()
                    .world(World::new().gamerule(setting))
                    .validate()
                    .is_err()
            );
        }
    }

    #[test]
    fn default_namespace_aliases_replace_the_same_setting() {
        let world = World::new()
            .gamerule(crate::cmd::gamerule_keep_inventory().value(true))
            .gamerule(crate::cmd::gamerule_minecraft_keep_inventory().value(false));
        assert_eq!(world.gamerules().len(), 1);
        assert_eq!(
            world.gamerules()["keep_inventory"].command().unwrap(),
            "gamerule minecraft:keep_inventory false"
        );
    }
}
