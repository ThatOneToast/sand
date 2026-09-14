//! Unified author-definition registration for datapack compilation.
//!
//! [`DatapackComponent`] describes one resource file. [`IntoDatapack`] is the
//! wider compiler boundary: one author-facing definition may contribute any
//! number of resources together with lifecycle work and function-tag
//! membership. The exporter expands the complete registration before it
//! validates identities or renders compiler-owned lifecycle and tag files.

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleContribution {
    phase: LifecyclePhase,
    command: String,
}

impl LifecycleContribution {
    /// Contribute a command to Sand's canonical load function.
    pub fn load(command: impl Into<String>) -> Self {
        Self {
            phase: LifecyclePhase::Load,
            command: command.into(),
        }
    }

    /// Contribute a command to Sand's canonical tick function.
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTagContribution {
    tag: ResourceLocation,
    function: FunctionId,
}

impl FunctionTagContribution {
    /// Associate `function` with `tag`; the exporter performs the canonical
    /// deterministic merge and emits the single resulting vanilla tag file.
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
pub struct DatapackRegistration {
    components: Vec<Box<dyn DatapackComponent>>,
    lifecycle: Vec<LifecycleContribution>,
    function_tags: Vec<FunctionTagContribution>,
}

impl DatapackRegistration {
    /// Create an empty registration contribution.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one ordinary datapack resource.
    pub fn component(mut self, component: impl DatapackComponent + 'static) -> Self {
        self.components.push(Box::new(component));
        self
    }

    /// Add an already boxed datapack resource.
    pub fn boxed_component(mut self, component: Box<dyn DatapackComponent>) -> Self {
        self.components.push(component);
        self
    }

    /// Add several ordinary datapack resources.
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
    pub fn lifecycle(mut self, contribution: LifecycleContribution) -> Self {
        self.lifecycle.push(contribution);
        self
    }

    /// Add one function-tag membership for canonical exporter merging.
    pub fn function_tag(mut self, contribution: FunctionTagContribution) -> Self {
        self.function_tags.push(contribution);
        self
    }

    /// Merge another component or higher-level registration-producing
    /// definition into this registration.
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
pub trait IntoDatapack {
    /// Expand this definition into an inert, export-scoped registration.
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
