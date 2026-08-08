//! Generated/forwarded contracts for facade APIs whose definitions live in
//! implementation or proc-macro crates.
//!
//! Stable procedural macros cannot attach an attribute to a `pub use`, so the
//! facade owns these canonical identities and aliases in one auditable table.
//! Signatures are written once here and checked by focused facade tests.

use sand_api_contract::{ApiKind, ApiRegistration, StaticApiParameter};

macro_rules! register {
    (
        path: $path:literal,
        aliases: [$($alias:literal),* $(,)?],
        module: $module:literal,
        kind: $kind:ident,
        signature: $signature:literal,
        summary: $summary:literal,
        context: $context:literal,
        minecraft: $minecraft:literal,
        use_when: [$($use_when:literal),+ $(,)?],
        avoid_when: [$($avoid_when:literal),+ $(,)?],
        params: [$($param:literal => $param_doc:literal),* $(,)?],
        returns: $returns:expr,
        example: $example:literal $(,
        availability: [$($availability:literal),* $(,)?])?
    ) => {
        sand_api_contract::inventory::submit! {
            ApiRegistration {
                canonical_path: $path,
                aliases: &[$($alias),*],
                canonical_module: $module,
                kind: ApiKind::$kind,
                signature: $signature,
                summary: $summary,
                context: $context,
                minecraft: $minecraft,
                use_when: &[$($use_when),+],
                avoid_when: &[$($avoid_when),+],
                parameters: &[$(StaticApiParameter { name: $param, description: $param_doc }),*],
                returns: $returns,
                example: $example,
                availability: &[$($($availability),*)?],
            }
        }
    };
}

register! {
    path: "sand::prelude",
    aliases: [],
    module: "sand",
    kind: Module,
    signature: "pub mod prelude",
    summary: "Collects Sand's ordinary datapack-authoring vocabulary in one import.",
    context: "The prelude aliases canonical APIs from focused topic modules and is the stable starting point for author code.",
    minecraft: "Importing the prelude has no runtime effect; the imported builders and macros generate commands and datapack resources when used.",
    use_when: ["Starting a Sand datapack module", "Following Sand examples and book code"],
    avoid_when: ["A narrow import is needed to prevent a Rust name collision"],
    params: [],
    returns: None,
    example: "use sand::prelude::*;"
}

register! {
    path: "sand::predicate::Predicate",
    aliases: ["sand::prelude::Predicate", "sand::component::Predicate"],
    module: "sand::predicate",
    kind: Struct,
    signature: "pub struct Predicate",
    summary: "Represents one reusable namespaced Minecraft predicate resource.",
    context: "A predicate gives a typed condition tree a stable resource identity so commands, loot tables, advancements, and other resources can share it.",
    minecraft: "Exports data/<namespace>/predicate/<path>.json containing the root loot-condition object.",
    use_when: ["A condition is referenced from more than one place", "A command uses execute if predicate"],
    avoid_when: ["The condition is only a transient scoreboard calculation"],
    params: [],
    returns: None,
    example: "Predicate::new(id, PredicateRoot::random_chance(0.25))"
}

register! {
    path: "sand::predicate::Predicate::new",
    aliases: ["sand::prelude::Predicate::new", "sand::component::Predicate::new"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn new(location: ResourceLocation, root: PredicateRoot) -> Predicate",
    summary: "Creates a reusable Minecraft predicate resource.",
    context: "The constructor binds a validated namespaced resource location to the typed condition emitted for that predicate.",
    minecraft: "Generates a predicate JSON resource whose condition is evaluated only when Minecraft references it.",
    use_when: ["Entity property checks", "Equipment predicates", "Location or weather checks"],
    avoid_when: ["Mutable runtime state", "Scoreboard arithmetic"],
    params: [
        "location" => "The validated namespaced identifier of the generated predicate.",
        "root" => "The root typed condition evaluated by the predicate."
    ],
    returns: Some("A predicate component ready for registration with #[component]."),
    example: "Predicate::new(ResourceLocation::new(\"demo\", \"is_ready\")?, PredicateRoot::random_chance(0.25))"
}

register! {
    path: "sand::predicate::PredicateRoot",
    aliases: ["sand::prelude::PredicateRoot", "sand::component::PredicateRoot"],
    module: "sand::predicate",
    kind: Enum,
    signature: "pub enum PredicateRoot",
    summary: "Models the typed root condition tree of a standalone predicate.",
    context: "The enum captures boolean composition and common vanilla condition families while retaining an explicit raw escape hatch for unsupported shapes.",
    minecraft: "Serializes to one vanilla loot-condition object in a predicate JSON resource.",
    use_when: ["Composing reusable boolean or world-state checks", "Converting an existing loot condition"],
    avoid_when: ["A typed condition exists and Raw would discard its validation"],
    params: [],
    returns: None,
    example: "PredicateRoot::inverted(PredicateRoot::random_chance(0.1))"
}

register! {
    path: "sand::predicate::PredicateRoot::entity_properties",
    aliases: ["sand::prelude::PredicateRoot::entity_properties"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn entity_properties(target: EntityPredicateTarget, predicate: EntityPredicate) -> PredicateRoot",
    summary: "Checks typed properties of an entity in the current predicate context.",
    context: "Loot contexts name entities by roles such as this, killer, or killer_player; this constructor keeps that role separate from the property model.",
    minecraft: "Emits minecraft:entity_properties with entity and predicate fields.",
    use_when: ["Testing entity type, equipment, flags, or location", "Selecting a loot-context entity role"],
    avoid_when: ["The entity must be found through a command selector outside a loot context"],
    params: [
        "target" => "The loot-context entity role to inspect.",
        "predicate" => "The typed properties required of that entity."
    ],
    returns: Some("An entity-properties predicate root."),
    example: "PredicateRoot::entity_properties(EntityPredicateTarget::This, EntityPredicate::new())"
}

register! {
    path: "sand::predicate::PredicateRoot::random_chance",
    aliases: ["sand::prelude::PredicateRoot::random_chance"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn random_chance(chance: f64) -> PredicateRoot",
    summary: "Creates a predicate that succeeds with a fixed probability.",
    context: "Random chance is useful for nondeterministic gates that do not depend on entity or world state.",
    minecraft: "Emits minecraft:random_chance; validation requires a finite value from 0.0 through 1.0.",
    use_when: ["Applying a probability gate to loot or a command branch"],
    avoid_when: ["The outcome must be deterministic or stateful"],
    params: ["chance" => "Inclusive probability from 0.0 to 1.0."],
    returns: Some("A random-chance predicate root."),
    example: "PredicateRoot::random_chance(0.25)"
}

register! {
    path: "sand::predicate::EntityPredicate",
    aliases: ["sand::prelude::EntityPredicate", "sand::component::EntityPredicate"],
    module: "sand::predicate",
    kind: Struct,
    signature: "pub struct EntityPredicate",
    summary: "Describes typed properties required of a Minecraft entity.",
    context: "Entity predicates combine identity, flags, equipment, effects, location, distance, and nested relationships for vanilla condition evaluation.",
    minecraft: "Serializes the entity predicate object nested inside advancement, loot, or standalone predicate conditions.",
    use_when: ["Checking entity equipment or flags", "Restricting an event or loot condition by entity properties"],
    avoid_when: ["Selecting live command targets without a predicate context"],
    params: [],
    returns: None,
    example: "EntityPredicate::new().equipment(EntityEquipment::new())"
}

register! {
    path: "sand::predicate::ItemPredicate",
    aliases: ["sand::prelude::ItemPredicate", "sand::component::ItemPredicate"],
    module: "sand::predicate",
    kind: Struct,
    signature: "pub struct ItemPredicate",
    summary: "Describes typed properties required of a Minecraft item stack.",
    context: "Item predicates express the item identity, count, components, enchantments, and related constraints consumed by vanilla condition formats.",
    minecraft: "Serializes an item predicate nested in loot, advancement, equipment, or execute-if-items conditions.",
    use_when: ["Matching equipment or inventory contents", "Constraining an item-sensitive trigger"],
    avoid_when: ["Constructing a new item stack rather than matching one"],
    params: [],
    returns: None,
    example: "ItemPredicate::new().item(ItemId::minecraft(\"diamond\")?)"
}

register! {
    path: "sand::predicate::LootCondition",
    aliases: ["sand::prelude::LootCondition", "sand::component::LootCondition"],
    module: "sand::predicate",
    kind: Enum,
    signature: "pub enum LootCondition",
    summary: "Models vanilla conditions shared by loot tables and predicate resources.",
    context: "Loot conditions are the common condition vocabulary behind loot pools, functions, and many standalone predicate shapes.",
    minecraft: "Serializes a vanilla loot-condition object with its condition identifier and typed fields.",
    use_when: ["Gating loot pools or loot functions", "Converting compatible conditions into PredicateRoot"],
    avoid_when: ["A command execute condition or scoreboard comparison is the actual domain"],
    params: [],
    returns: None,
    example: "LootCondition::RandomChance { chance: 0.5 }"
}
