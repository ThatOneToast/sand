//! Unified author-definition registration for datapack compilation.
//!
//! [`DatapackComponent`] describes one resource file. [`IntoDatapack`] is the
//! wider compiler boundary: one author-facing definition may contribute any
//! number of resources together with lifecycle work and function-tag
//! membership. The exporter expands the complete registration before it
//! validates identities or renders compiler-owned lifecycle and tag files.

use sand_macros::api;

use sand_components::{DatapackComponent, FunctionId, ResourceLocation};

/// The compiler lifecycle phase targeted by a registration contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LifecyclePhase {
    /// Run once through Sand's canonical `minecraft:load` aggregation.
    Load,
    /// Run every tick through Sand's canonical `minecraft:tick` aggregation.
    Tick,
}

/// One command contributed to Sand's compiler-owned lifecycle functions.
///
/// Each command runs in the server context; use a typed `execute` builder
/// when entity context is needed. Render typed commands with `to_string()`.
/// Within each phase, contributions run in stable factory-owner order and
/// retain insertion order within a registration, including merged bundles.
#[derive(Clone, Debug, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::registration::LifecycleContribution",
    module = "sand::registration",
    summary = "One command added to the canonical load or tick lifecycle.",
    context = "Construct lifecycle contributions when a bundle needs setup or recurring work.",
    minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
    use_when = ["Authoring a custom multi-resource component"],
    avoid_when = ["An ordinary single resource builder already suffices"],
    example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
)]
pub struct LifecycleContribution {
    phase: LifecyclePhase,
    command: String,
}

impl LifecycleContribution {
    /// Contribute a command to Sand's canonical load function.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::LifecycleContribution::load",
        module = "sand::registration",
        kind = "method",
        params(command = "One rendered command to execute at load."),
        returns = "The completed contribution or registration value.",
        summary = "Contributes one command at datapack load.",
        context = "Commands retain insertion order within the registration and run in the server context.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn load(command: impl Into<String>) -> Self {
        Self {
            phase: LifecyclePhase::Load,
            command: command.into(),
        }
    }

    /// Contribute a command to Sand's canonical tick function.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::LifecycleContribution::tick",
        module = "sand::registration",
        kind = "method",
        params(command = "One rendered command to execute each tick."),
        returns = "The completed contribution or registration value.",
        summary = "Contributes one command on every datapack tick.",
        context = "Commands retain insertion order within the registration and run in the server context.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn tick(command: impl Into<String>) -> Self {
        Self {
            phase: LifecyclePhase::Tick,
            command: command.into(),
        }
    }

    pub(crate) fn phase(&self) -> LifecyclePhase {
        self.phase
    }

    pub(crate) fn command(&self) -> &str {
        &self.command
    }
}

/// One typed function membership in a datapack function tag.
///
/// Local function references resolve to the current export namespace before
/// aggregation. Memberships merge with macro-declared tags, sort by resolved
/// identifier, and deduplicate without displacing compiler setup entries.
#[derive(Clone, Debug, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::registration::FunctionTagContribution",
    module = "sand::registration",
    summary = "One typed function membership in a function tag.",
    context = "Use memberships instead of generating a competing tag resource.",
    minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
    use_when = ["Authoring a custom multi-resource component"],
    avoid_when = ["An ordinary single resource builder already suffices"],
    example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
)]
pub struct FunctionTagContribution {
    tag: ResourceLocation,
    function: FunctionId,
}

impl FunctionTagContribution {
    /// Associate `function` with `tag`; the exporter performs the canonical
    /// deterministic merge and emits the single resulting vanilla tag file.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::FunctionTagContribution::new",
        module = "sand::registration",
        kind = "method",
        params(tag = "The tag resource identifier.", function = "The typed function to include."),
        returns = "The completed contribution or registration value.",
        summary = "Associates a typed function with a function tag.",
        context = "The exporter merges memberships deterministically and removes duplicate entries.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn new(tag: ResourceLocation, function: FunctionId) -> Self {
        Self { tag, function }
    }

    pub(crate) fn into_parts(self) -> (ResourceLocation, FunctionId) {
        (self.tag, self.function)
    }
}

/// The complete contribution made by one registered author-facing feature.
///
/// Registrations are inert values. They never write files or mutate compiler
/// state themselves; the exporter collects all registrations for one export,
/// validates their combined identities, and centrally renders lifecycle and
/// tag resources.
#[derive(Default)]
#[api(
    registry = sand_api_contract,
    path = "sand::registration::DatapackRegistration",
    module = "sand::registration",
    summary = "An inert bundle of resources and lifecycle or tag contributions.",
    context = "Registration is scoped to one export and does not write files or mutate exporter state.",
    minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
    use_when = ["Authoring a custom multi-resource component"],
    avoid_when = ["An ordinary single resource builder already suffices"],
    example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
)]
pub struct DatapackRegistration {
    components: Vec<Box<dyn DatapackComponent>>,
    lifecycle: Vec<LifecycleContribution>,
    function_tags: Vec<FunctionTagContribution>,
}

impl DatapackRegistration {
    /// Create an empty registration contribution.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackRegistration::new",
        module = "sand::registration",
        kind = "method",
        returns = "The completed contribution or registration value.",
        summary = "Creates an empty registration.",
        context = "Add resources or lifecycle and function-tag contributions with the builder methods.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one ordinary datapack resource.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackRegistration::component",
        module = "sand::registration",
        kind = "method",
        params(component = "The resource builder to register."),
        returns = "The completed contribution or registration value.",
        summary = "Adds one ordinary resource to a registration.",
        context = "The resource is validated and checked for output collisions during export.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn component(mut self, component: impl DatapackComponent + 'static) -> Self {
        self.components.push(Box::new(component));
        self
    }

    /// Add an already boxed datapack resource.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackRegistration::boxed_component",
        module = "sand::registration",
        kind = "method",
        params(component = "The boxed resource to register."),
        returns = "The completed contribution or registration value.",
        summary = "Adds one boxed resource to a registration.",
        context = "Use this when a heterogeneous resource collection already stores trait objects.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn boxed_component(mut self, component: Box<dyn DatapackComponent>) -> Self {
        self.components.push(component);
        self
    }

    /// Add several ordinary datapack resources.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackRegistration::components",
        module = "sand::registration",
        kind = "method",
        params(components = "Resource builders to register."),
        returns = "The completed contribution or registration value.",
        summary = "Adds a sequence of ordinary resources.",
        context = "Each resource is independently validated and checked for output collisions during export.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn components<T>(mut self, components: impl IntoIterator<Item = T>) -> Self
    where
        T: DatapackComponent + 'static,
    {
        self.components.extend(
            components
                .into_iter()
                .map(|component| Box::new(component) as Box<dyn DatapackComponent>),
        );
        self
    }

    /// Add one compiler-owned lifecycle command.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackRegistration::lifecycle",
        module = "sand::registration",
        kind = "method",
        params(contribution = "The load or tick command to append."),
        returns = "The completed contribution or registration value.",
        summary = "Appends a lifecycle command to a registration.",
        context = "Load and tick work flows through canonical aggregation while preserving insertion order within each phase.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn lifecycle(mut self, contribution: LifecycleContribution) -> Self {
        self.lifecycle.push(contribution);
        self
    }

    /// Add one function-tag membership for canonical exporter merging.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackRegistration::function_tag",
        module = "sand::registration",
        kind = "method",
        params(contribution = "The function membership to merge."),
        returns = "The completed contribution or registration value.",
        summary = "Adds typed function-tag membership.",
        context = "Membership merges with macro-declared and compiler-generated entries in one tag resource.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn function_tag(mut self, contribution: FunctionTagContribution) -> Self {
        self.function_tags.push(contribution);
        self
    }

    /// Merge another component or higher-level registration-producing
    /// definition into this registration.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::DatapackRegistration::merge",
        module = "sand::registration",
        kind = "method",
        params(definition = "The component or bundle to append."),
        returns = "The completed contribution or registration value.",
        summary = "Appends another definition through IntoDatapack.",
        context = "The appended definition contributes its resources, lifecycle work, and tag memberships in order.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    pub fn merge(mut self, definition: impl IntoDatapack) -> Self {
        let (components, lifecycle, function_tags) = definition.into_datapack().into_parts();
        self.components.extend(components);
        self.lifecycle.extend(lifecycle);
        self.function_tags.extend(function_tags);
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<Box<dyn DatapackComponent>>,
        Vec<LifecycleContribution>,
        Vec<FunctionTagContribution>,
    ) {
        (self.components, self.lifecycle, self.function_tags)
    }
}

/// Convert one author-facing definition into its complete compiler
/// registration.
///
/// Ordinary resource builders implement this automatically through the
/// blanket [`DatapackComponent`] implementation. Higher-level Sand features
/// implement it to return a [`DatapackRegistration`] containing all resources
/// and compiler behavior they require.
#[api(
    registry = sand_api_contract,
    path = "sand::registration::IntoDatapack",
    module = "sand::registration",
    summary = "Converts a component definition into its complete registration.",
    context = "Ordinary DatapackComponent builders implement this automatically; custom bundles implement it explicitly.",
    minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
    use_when = ["Authoring a custom multi-resource component"],
    avoid_when = ["An ordinary single resource builder already suffices"],
    example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
)]
pub trait IntoDatapack {
    /// Expand this definition into an inert, export-scoped registration.
    #[api(
        registry = sand_api_contract,
        path = "sand::registration::IntoDatapack::into_datapack",
        module = "sand::registration",
        returns = "The completed contribution or registration value.",
        summary = "Expands a definition into an inert export-scoped registration.",
        context = "Return all resources and lifecycle or tag work needed by this definition; the exporter validates the combined result.",
        minecraft = "Contributes to validated resource files and canonical lifecycle or function-tag output.",
        use_when = ["Authoring a custom multi-resource component"],
        avoid_when = ["An ordinary single resource builder already suffices"],
        example = "use sand::registration::DatapackRegistration;\nlet registration = DatapackRegistration::new();"
    )]
    fn into_datapack(self) -> DatapackRegistration;
}

impl<T> IntoDatapack for T
where
    T: DatapackComponent + 'static,
{
    fn into_datapack(self) -> DatapackRegistration {
        DatapackRegistration::new().component(self)
    }
}

impl IntoDatapack for DatapackRegistration {
    fn into_datapack(self) -> DatapackRegistration {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestComponent(ResourceLocation);

    impl DatapackComponent for TestComponent {
        fn resource_location(&self) -> &ResourceLocation {
            &self.0
        }

        fn to_json(&self) -> serde_json::Value {
            json!({"registered": true})
        }

        fn component_dir(&self) -> &'static str {
            "test"
        }
    }

    #[test]
    fn ordinary_component_becomes_one_complete_registration() {
        let registration =
            TestComponent(ResourceLocation::new("test", "one").unwrap()).into_datapack();
        let (components, lifecycle, tags) = registration.into_parts();
        assert_eq!(components.len(), 1);
        assert!(lifecycle.is_empty());
        assert!(tags.is_empty());
    }

    #[test]
    fn registration_can_hold_multiple_resources_and_compiler_contributions() {
        let registration = DatapackRegistration::new()
            .components([
                TestComponent(ResourceLocation::new("test", "one").unwrap()),
                TestComponent(ResourceLocation::new("test", "two").unwrap()),
            ])
            .lifecycle(LifecycleContribution::load("say load"))
            .lifecycle(LifecycleContribution::tick("say tick"))
            .function_tag(FunctionTagContribution::new(
                ResourceLocation::new("minecraft", "load").unwrap(),
                FunctionId::custom(ResourceLocation::new("test", "one").unwrap()),
            ));

        let (components, lifecycle, tags) = registration.into_parts();
        assert_eq!(components.len(), 2);
        assert_eq!(lifecycle.len(), 2);
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn registrations_compose_through_the_same_conversion_boundary() {
        let registration = DatapackRegistration::new()
            .merge(TestComponent(
                ResourceLocation::new("test", "component").unwrap(),
            ))
            .merge(
                DatapackRegistration::new().lifecycle(LifecycleContribution::tick("say composed")),
            );

        let (components, lifecycle, tags) = registration.into_parts();
        assert_eq!(components.len(), 1);
        assert_eq!(lifecycle.len(), 1);
        assert!(tags.is_empty());
    }
}
