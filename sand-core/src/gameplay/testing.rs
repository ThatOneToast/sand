use crate::{
    IntoCommands, McFunction,
    condition::Condition,
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack},
    state::Ticks,
};
use sand_commands::{CommandError, ObjectiveName};
use sand_components::{ComponentContent, DatapackComponent, ResourceLocation};

#[derive(Clone, Default)]
struct Phase {
    delay: u32,
    actions: Actions,
    assertions: Vec<(Condition, String)>,
}

/// A deterministic, server-executed gameplay scenario. Setup, delayed actions,
/// assertions and cleanup are compiled with the pack's normal typed APIs.
/// Run scenarios in an isolated vanilla test world, optionally with real test
/// clients; this API does not simulate players or command behavior in Rust.
///
/// Actions execute in server context, so entity work must select its actors.
/// Assertions run after their phase's actions. Failures accumulate and cleanup
/// still runs. Completion emits `SAND_TEST_PASS <id>` or `SAND_TEST_FAIL <id>`.
/// User callbacks have isolated return frames; the same test cannot overlap itself.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract,
path = "sand::systems::GameplayTest", module = "sand::systems",
summary = "A deterministic, server-executed gameplay scenario.",
context = "A deterministic, server-executed gameplay scenario. Setup, delayed actions, assertions and cleanup are compiled with the pack's normal typed APIs. Run scenarios in an isolated vanilla test world, optionally with real test clients; this API does not simulate players or command behavior in Rust.  Actions execute in server context, so entity work must select its actors. Assertions run after their phase's actions. Failures accumulate and cleanup still runs. Completion emits `SAND_TEST_PASS <id>` or `SAND_TEST_FAIL <id>`. User callbacks have isolated return frames; the same test cannot overlap itself.",
minecraft = "Exports isolated action frames, typed assertions and scheduled phases; cleanup runs before a machine-readable pass or fail marker.",
use_when = ["Validating gameplay against a real Minecraft server"],
avoid_when = ["Replacing vanilla runtime behavior with a host-side simulation"],
example = "use sand::systems::GameplayTest;",
)]
pub struct GameplayTest {
    id: ResourceLocation,
    phases: Vec<Phase>,
    cleanup: Actions,
}
impl GameplayTest {
    /// Create an inert scenario with a unique resource identity.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::GameplayTest::new", module = "sand::systems",
    kind = "method",
    summary = "Create an inert scenario with a unique resource identity.",
    context = "Creates an inert scenario identified by a unique resource location. Register it as a component and call run in an isolated vanilla test world.",
    minecraft = "Exports isolated action frames, typed assertions and scheduled phases; cleanup runs before a machine-readable pass or fail marker.",
    use_when = ["Validating gameplay against a real Minecraft server"],
    avoid_when = ["Replacing vanilla runtime behavior with a host-side simulation"],
    params(id = "Unique resource identity for the registered definition."),
    returns = "The GameplayTest definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{GameplayTest, Lifetime};\n\nfn demonstrate() {\n    let _result = GameplayTest::new(\"demo:scenario\".parse().unwrap());\n}",
    )]
    pub fn new(id: ResourceLocation) -> Self {
        Self {
            id,
            phases: vec![Phase::default()],
            cleanup: Actions::default(),
        }
    }
    /// Append setup actions to the initial server-context phase.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::GameplayTest::setup", module = "sand::systems",
    kind = "method",
    summary = "Append setup actions to the initial server-context phase.",
    context = "Appends server-context actions to the initial phase. Select entity actors explicitly; setup runs in its own return frame before initial assertions.",
    minecraft = "Exports isolated action frames, typed assertions and scheduled phases; cleanup runs before a machine-readable pass or fail marker.",
    use_when = ["Validating gameplay against a real Minecraft server"],
    avoid_when = ["Replacing vanilla runtime behavior with a host-side simulation"],
    params(actions = "Typed actions executed in server context for this scenario phase."),
    returns = "The GameplayTest definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{GameplayTest, Lifetime};\n\nfn demonstrate(value: GameplayTest) {\n    let _result = value.setup(cmd::say(\"Arrange scenario\"));\n}",
    )]
    pub fn setup(mut self, actions: impl IntoCommands) -> Self {
        self.phases[0].actions.extend([actions]);
        self
    }
    /// Add actions after a positive number of Minecraft ticks from the previous
    /// phase. Delays use vanilla scheduling, not host sleeps or simulated time.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::GameplayTest::after", module = "sand::systems",
    kind = "method",
    summary = "Add actions after a positive number of Minecraft ticks from the previous phase.",
    context = "Appends a phase delayed from the previous phase by a positive tick duration. Actions run in server context before assertions attached to this phase.",
    minecraft = "Exports isolated action frames, typed assertions and scheduled phases; cleanup runs before a machine-readable pass or fail marker.",
    use_when = ["Validating gameplay against a real Minecraft server"],
    avoid_when = ["Replacing vanilla runtime behavior with a host-side simulation"],
    params(delay = "Positive number of Minecraft ticks after the previous phase.", actions = "Typed actions executed in server context for this scenario phase."),
    returns = "The GameplayTest definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{GameplayTest, Lifetime};\n\nfn demonstrate(value: GameplayTest) {\n    let _result = value.after(Ticks::new(20), cmd::say(\"Delayed phase\"));\n}",
    )]
    pub fn after(mut self, delay: Ticks, actions: impl IntoCommands) -> Self {
        self.phases.push(Phase {
            delay: delay.get(),
            actions: actions.into_commands(),
            assertions: vec![],
        });
        self
    }
    /// Assert a typed runtime condition after the most recently added phase.
    /// Labels identify failures in server output and must occupy one line.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::GameplayTest::expect", module = "sand::systems",
    kind = "method",
    summary = "Assert a typed runtime condition after the most recently added phase.",
    context = "Adds a condition to the most recently declared phase. False conditions accumulate failures under the supplied single-line label and do not skip cleanup.",
    minecraft = "Exports isolated action frames, typed assertions and scheduled phases; cleanup runs before a machine-readable pass or fail marker.",
    use_when = ["Validating gameplay against a real Minecraft server"],
    avoid_when = ["Replacing vanilla runtime behavior with a host-side simulation"],
    params(condition = "Typed runtime condition evaluated in the execution context described above.", label = "Single-line diagnostic label printed when the assertion fails."),
    returns = "The GameplayTest definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{GameplayTest, Lifetime};\n\nfn demonstrate(value: GameplayTest, condition: Condition) {\n    let _result = value.expect(condition, \"owner has expected State\");\n}",
    )]
    pub fn expect(mut self, condition: Condition, label: impl Into<String>) -> Self {
        self.phases
            .last_mut()
            .unwrap()
            .assertions
            .push((condition, label.into()));
        self
    }
    /// Append cleanup that runs after all assertions, whether they pass or fail.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::GameplayTest::cleanup", module = "sand::systems",
    kind = "method",
    summary = "Append cleanup that runs after all assertions, whether they pass or fail.",
    context = "Appends actions that run after the final phase regardless of assertion results. Cleanup runs before the machine-readable completion marker.",
    minecraft = "Exports isolated action frames, typed assertions and scheduled phases; cleanup runs before a machine-readable pass or fail marker.",
    use_when = ["Validating gameplay against a real Minecraft server"],
    avoid_when = ["Replacing vanilla runtime behavior with a host-side simulation"],
    params(actions = "Typed actions executed in server context for this scenario phase."),
    returns = "The GameplayTest definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{GameplayTest, Lifetime};\n\nfn demonstrate(value: GameplayTest) {\n    let _result = value.cleanup(cmd::say(\"Cleanup scenario\"));\n}",
    )]
    pub fn cleanup(mut self, actions: impl IntoCommands) -> Self {
        self.cleanup.extend([actions]);
        self
    }
    /// Start the registered scenario on the next server tick. A running copy
    /// rejects another start; independently identified scenarios may coexist.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::GameplayTest::run", module = "sand::systems",
    kind = "method",
    summary = "Start the registered scenario on the next server tick.",
    context = "Schedules the registered test from server context. A running scenario rejects overlapping starts; completion emits SAND_TEST_PASS or SAND_TEST_FAIL with its identity.",
    minecraft = "Exports isolated action frames, typed assertions and scheduled phases; cleanup runs before a machine-readable pass or fail marker.",
    use_when = ["Validating gameplay against a real Minecraft server"],
    avoid_when = ["Replacing vanilla runtime behavior with a host-side simulation"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{GameplayTest, Lifetime};\n\nfn demonstrate(value: GameplayTest) {\n    let _result = value.run();\n}",
    )]
    pub fn run(&self) -> Actions {
        Cmd::Function(resource(&self.id, "run").to_string()).into_commands()
    }
}
fn resource(id: &ResourceLocation, part: &str) -> ResourceLocation {
    ResourceLocation::new(id.namespace(), format!("__sand_test/{}/{part}", id.path()))
        .expect("validated test identity")
}
impl IntoDatapack for GameplayTest {
    fn into_datapack(self) -> DatapackRegistration {
        let mut parts = vec!["run".to_string(), "cleanup".to_string()];
        for index in 0..self.phases.len() {
            parts.extend([format!("phase_{index}"), format!("actions_{index}")]);
        }
        let mut registration = DatapackRegistration::new();
        for part in parts {
            registration = registration.component(TestFunction {
                location: resource(&self.id, &part),
                part,
                definition: self.clone(),
            });
        }
        registration
    }
}
struct TestFunction {
    location: ResourceLocation,
    part: String,
    definition: GameplayTest,
}
impl TestFunction {
    fn build(&self) -> Result<McFunction, CommandError> {
        let d = &self.definition;
        let id = crate::function::resolve_export_location(&d.id)?;
        let objective = ObjectiveName::logical(format!("sand test {id}"));
        crate::function::request_numeric_objectives([objective.clone()])?;
        for (index, phase) in d.phases.iter().enumerate() {
            if index > 0 && (phase.delay == 0 || phase.delay > i32::MAX as u32) {
                return Err(CommandError::new(
                    "gameplay_test",
                    "delay",
                    "must be in 1..=2147483647",
                ));
            }
            if phase
                .assertions
                .iter()
                .any(|(_, label)| label.contains(['\n', '\r']) || label.is_empty())
            {
                return Err(CommandError::new(
                    "gameplay_test",
                    "label",
                    "must be a nonempty single line",
                ));
            }
        }
        let call = |part: &str| Cmd::Function(resource(&id, part).to_string());
        let actions = if self.part == "run" {
            vec![
                format!("execute if score #active {objective} matches 1 run return 0"),
                format!("scoreboard players set #active {objective} 1"),
                format!("scoreboard players set #failed {objective} 0"),
                format!("schedule function {} 1t replace", resource(&id, "phase_0")),
            ]
            .into_commands()
        } else if self.part == "cleanup" {
            d.cleanup.clone()
        } else if let Some(index) = self.part.strip_prefix("actions_") {
            d.phases[index.parse::<usize>().unwrap()].actions.clone()
        } else {
            let index = self
                .part
                .strip_prefix("phase_")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let phase = &d.phases[index];
            let mut actions = call(&format!("actions_{index}")).into_commands();
            for (condition, label) in &phase.assertions {
                let failure = Cmd::AnonymousFunction {
                    prefix: "sand/test/assertion".into(),
                    body: vec![
                        format!("scoreboard players add #failed {objective} 1"),
                        format!("say SAND_TEST_ASSERT {id}: {label}"),
                    ]
                    .into_commands(),
                };
                actions.extend([condition.guard_actions(true, &[], failure.into_commands())]);
            }
            if let Some(next) = d.phases.get(index + 1) {
                actions.extend([format!(
                    "schedule function {} {}t replace",
                    resource(&id, &format!("phase_{}", index + 1)),
                    next.delay
                )]);
            } else {
                actions.extend([call("cleanup")]);
                actions.extend([vec![format!("execute if score #failed {objective} matches 0 run say SAND_TEST_PASS {id}"),format!("execute unless score #failed {objective} matches 0 run say SAND_TEST_FAIL {id}"),format!("scoreboard players set #active {objective} 0")].into_commands()]);
            }
            actions
        };
        Ok(McFunction::new(self.location.clone()).commands(actions))
    }
}
impl DatapackComponent for TestFunction {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }
    fn component_dir(&self) -> &'static str {
        "function"
    }
    fn file_extension(&self) -> &'static str {
        "mcfunction"
    }
    fn to_json(&self) -> serde_json::Value {
        match self.try_content().expect("test must validate") {
            ComponentContent::Text(s) => s.into(),
            ComponentContent::Json(v) => v,
        }
    }
    fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
        self.build()
            .map_err(|error| sand_components::SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "gameplay_test".into(),
                field: error.field.clone(),
                message: error.to_string(),
            })?
            .try_content()
    }
}
