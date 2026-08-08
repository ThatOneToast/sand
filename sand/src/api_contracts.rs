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
    signature: "pub fn new(location: PredicateId, root: PredicateRoot) -> Predicate",
    summary: "Creates a reusable Minecraft predicate resource.",
    context: "The constructor binds a validated namespaced resource location to the typed condition emitted for that predicate.",
    minecraft: "Generates a predicate JSON resource whose condition is evaluated only when Minecraft references it.",
    use_when: ["Entity property checks", "Equipment predicates", "Location or weather checks"],
    avoid_when: ["Mutable runtime state", "Scoreboard arithmetic"],
    params: [
        "location" => "The typed namespaced identifier of the generated predicate.",
        "root" => "The root typed condition evaluated by the predicate."
    ],
    returns: Some("A predicate component ready for registration with #[component]."),
    example: "Predicate::new(PredicateId::custom(\"demo:is_ready\".parse()?), PredicateRoot::random_chance(0.25))"
}

register! {
    path: "sand::predicate::PredicateRoot",
    aliases: ["sand::prelude::PredicateRoot", "sand::component::PredicateRoot"],
    module: "sand::predicate",
    kind: Struct,
    signature: "pub struct PredicateRoot",
    summary: "Models the typed root condition tree of a standalone predicate.",
    context: "The opaque builder captures boolean composition and common vanilla condition families while retaining an explicit, fallible raw escape hatch for unsupported shapes.",
    minecraft: "Serializes to one vanilla loot-condition object in a predicate JSON resource.",
    use_when: ["Composing reusable boolean or world-state checks", "Converting an existing loot condition"],
    avoid_when: ["A typed condition exists and Raw would discard its validation"],
    params: [],
    returns: None,
    example: "PredicateRoot::inverted(PredicateRoot::random_chance(0.1))"
}

register! {
    path: "sand::predicate::PredicateRoot::entity_properties",
    aliases: ["sand::component::PredicateRoot::entity_properties", "sand::prelude::PredicateRoot::entity_properties"],
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
    aliases: ["sand::component::PredicateRoot::random_chance", "sand::prelude::PredicateRoot::random_chance"],
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
 path: "sand::predicate::BlockPredicate", aliases: ["sand::prelude::BlockPredicate"], module: "sand::predicate", kind: Struct,
 signature: "pub struct BlockPredicate", summary: "Matches a block by typed identity, tag, state, or block-entity data.", context: "Block conditions in location predicates need one composable description of the block at the tested position.", minecraft: "Serializes the vanilla block predicate nested under a location check.",
 use_when: ["Restricting a location by the block occupying it"], avoid_when: ["Testing or placing a block through commands"], params: [],
 returns: None, example: "BlockPredicate::new().blocks(vec![BlockId::minecraft(\"stone\")?])"
}

register! {
 path: "sand::predicate::DamagePredicate", aliases: ["sand::prelude::DamagePredicate"], module: "sand::predicate", kind: Struct,
 signature: "pub struct DamagePredicate", summary: "Matches the amount, source, and blocking state of a damage event.", context: "Damage-sensitive advancements and entity conditions combine dealt and taken amounts with a typed source model.", minecraft: "Serializes vanilla damage requirements for damage-related triggers.",
 use_when: ["Constraining a trigger by how damage occurred"], avoid_when: ["Applying damage or tracking mutable health"], params: [],
 returns: None, example: "DamagePredicate::new().taken(FloatRange::at_least(4.0))"
}

register! {
 path: "sand::predicate::DamageSourcePredicate", aliases: ["sand::prelude::DamageSourcePredicate"], module: "sand::predicate", kind: Struct,
 signature: "pub struct DamageSourcePredicate", summary: "Matches the typed cause and participating entities of a damage event.", context: "The source model separates damage-type tags, the responsible entity, and the immediate damaging entity.", minecraft: "Serializes vanilla damage-source properties nested in a damage predicate.",
 use_when: ["Distinguishing projectile, environmental, or entity-caused damage"], avoid_when: ["Issuing a damage command"], params: [],
 returns: None, example: "DamageSourcePredicate::new().requires_tag(tag)"
}

register! {
 path: "sand::predicate::DistancePredicate", aliases: ["sand::prelude::DistancePredicate"], module: "sand::predicate", kind: Struct,
 signature: "pub struct DistancePredicate", summary: "Constrains displacement along axes and by horizontal or absolute distance.", context: "Advancement and entity predicates use this model to compare a subject with a context reference point.", minecraft: "Serializes vanilla x, y, z, horizontal, and absolute distance ranges.",
 use_when: ["Restricting a trigger by relative distance"], avoid_when: ["Selecting entities around a command position"], params: [],
 returns: None, example: "DistancePredicate::horizontal_at_most(16.0)"
}

register! {
 path: "sand::predicate::EffectPredicate", aliases: [], module: "sand::predicate", kind: Struct,
 signature: "pub struct EffectPredicate", summary: "Constrains one active status effect's amplifier, duration, and display flags.", context: "Entity predicates map an EffectId to this value to describe the required live effect state.", minecraft: "Serializes one entry in the vanilla effects predicate object.",
 use_when: ["Matching the strength or duration of an active effect"], avoid_when: ["Applying or removing an effect"], params: [],
 returns: None, example: "EffectPredicate::new().amplifier(IntRange::at_least(1))"
}

register! {
 path: "sand::predicate::EntityEquipment", aliases: ["sand::prelude::EntityEquipment"], module: "sand::predicate", kind: Struct,
 signature: "pub struct EntityEquipment", summary: "Matches item predicates in the six vanilla entity equipment slots.", context: "Keeping each slot typed makes equipment conditions composable within EntityPredicate.", minecraft: "Serializes head, chest, legs, feet, mainhand, and offhand item requirements.",
 use_when: ["Checking worn armor or held items"], avoid_when: ["Addressing inventory slots for mutation"], params: [],
 returns: None, example: "EntityEquipment::new().head(ItemPredicate::id(item))"
}

register! {
 path: "sand::predicate::EntityFlags", aliases: ["sand::prelude::EntityFlags"], module: "sand::predicate", kind: Struct,
 signature: "pub struct EntityFlags", summary: "Matches boolean runtime flags exposed by vanilla entity predicates.", context: "Flags describe observable entity state such as fire, movement stance, swimming, or age.", minecraft: "Serializes the vanilla flags object nested in an entity predicate.",
 use_when: ["Restricting a condition by entity state flags"], avoid_when: ["Changing those flags or selecting unrelated entity properties"], params: [],
 returns: None, example: "EntityFlags::new().sneaking(true)"
}

register! {
 path: "sand::predicate::FloatRange", aliases: [], module: "sand::predicate", kind: Struct,
 signature: "pub struct FloatRange", summary: "Represents a bounded or one-sided floating-point predicate range.", context: "Damage and distance conditions share this range representation and validate finite ordered bounds.", minecraft: "Serializes min and max members while omitting unbounded sides.",
 use_when: ["Constraining damage amounts or distances"], avoid_when: ["An exact integer count is required"], params: [],
 returns: None, example: "FloatRange::between(1.5, 4.0)"
}

register! {
 path: "sand::predicate::IntRange", aliases: [], module: "sand::predicate", kind: Struct,
 signature: "pub struct IntRange", summary: "Represents an exact, bounded, or one-sided integer predicate range.", context: "Counts, durations, amplifiers, and game times share this validated integer range.", minecraft: "Serializes an exact integer or an object with min and max members.",
 use_when: ["Constraining discrete Minecraft values"], avoid_when: ["A floating-point damage or distance range is required"], params: [],
 returns: None, example: "IntRange::between(1, 5)"
}

register! {
 path: "sand::predicate::LocationPredicate", aliases: ["sand::prelude::LocationPredicate"], module: "sand::predicate", kind: Struct,
 signature: "pub struct LocationPredicate", summary: "Matches biome, dimension, block, smokey state, and world coordinates.", context: "World-sensitive entity and standalone predicates compose these location properties in one typed value.", minecraft: "Serializes the vanilla location predicate object.",
 use_when: ["Restricting a condition by world position or environment"], avoid_when: ["Moving an entity or changing the world"], params: [],
 returns: None, example: "LocationPredicate::new().dimension(DimensionId::minecraft(\"overworld\")?)"
}

register! {
 path: "sand::predicate::WeatherPredicate", aliases: ["sand::component::WeatherPredicate", "sand::prelude::WeatherPredicate"], module: "sand::predicate", kind: Struct,
 signature: "pub struct WeatherPredicate", summary: "Matches the world's raining and thundering states.", context: "Standalone weather roots use this reusable value to keep both vanilla weather flags explicit.", minecraft: "Serializes raining and thundering fields in a weather-check condition.",
 use_when: ["Gating a predicate on current weather"], avoid_when: ["Changing weather or tracking a forecast"], params: [],
 returns: None, example: "WeatherPredicate::new().raining(true)"
}

register! {
 path: "sand::predicate::IntRange::at_least", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn at_least(min: i64) -> IntRange", summary: "Creates a range matching values at or above the bound.", context: "IntRange provides a shared typed bound for Minecraft predicate fields.", minecraft: "Serializes only the min member of the range object.",
 use_when: ["Expressing a one-sided predicate bound"], avoid_when: ["Both a lower and upper bound are required"], params: ["min" => "The inclusive lower bound."],
 returns: Some("A one-sided IntRange."), example: "IntRange::at_least(5)"
}

register! {
 path: "sand::predicate::IntRange::at_most", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn at_most(max: i64) -> IntRange", summary: "Creates a range matching values at or below the bound.", context: "IntRange provides a shared typed bound for Minecraft predicate fields.", minecraft: "Serializes only the max member of the range object.",
 use_when: ["Expressing a one-sided predicate bound"], avoid_when: ["Both a lower and upper bound are required"], params: ["max" => "The inclusive upper bound."],
 returns: Some("A one-sided IntRange."), example: "IntRange::at_most(5)"
}

register! {
 path: "sand::predicate::IntRange::between", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn between(min: i64, max: i64) -> IntRange", summary: "Creates a range matching values between two inclusive bounds.", context: "IntRange validates bound order before predicate export.", minecraft: "Serializes min and max members in the range object.",
 use_when: ["Expressing a closed predicate interval"], avoid_when: ["Only one side of the interval is bounded"], params: ["min" => "The inclusive lower bound.", "max" => "The inclusive upper bound."],
 returns: Some("A bounded IntRange."), example: "IntRange::between(2, 8)"
}

register! {
 path: "sand::predicate::IntRange::exact", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn exact(n: i64) -> IntRange", summary: "Matches one exact integer value.", context: "Exact integer ranges are compact and avoid repeating equal minimum and maximum bounds.", minecraft: "Serializes directly as one integer.",
 use_when: ["Matching one discrete count, duration, or level"], avoid_when: ["More than one value should be accepted"], params: ["n" => "The only accepted value."],
 returns: Some("An exact IntRange."), example: "IntRange::exact(5)"
}

register! {
 path: "sand::predicate::FloatRange::at_least", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn at_least(min: f64) -> FloatRange", summary: "Creates a range matching values at or above the bound.", context: "FloatRange provides a shared typed bound for Minecraft predicate fields.", minecraft: "Serializes only the min member of the range object.",
 use_when: ["Expressing a one-sided predicate bound"], avoid_when: ["Both a lower and upper bound are required"], params: ["min" => "The inclusive lower bound."],
 returns: Some("A one-sided FloatRange."), example: "FloatRange::at_least(5.0)"
}

register! {
 path: "sand::predicate::FloatRange::at_most", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn at_most(max: f64) -> FloatRange", summary: "Creates a range matching values at or below the bound.", context: "FloatRange provides a shared typed bound for Minecraft predicate fields.", minecraft: "Serializes only the max member of the range object.",
 use_when: ["Expressing a one-sided predicate bound"], avoid_when: ["Both a lower and upper bound are required"], params: ["max" => "The inclusive upper bound."],
 returns: Some("A one-sided FloatRange."), example: "FloatRange::at_most(5.0)"
}

register! {
 path: "sand::predicate::FloatRange::between", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn between(min: f64, max: f64) -> FloatRange", summary: "Creates a range matching values between two inclusive bounds.", context: "FloatRange validates bound order before predicate export.", minecraft: "Serializes min and max members in the range object.",
 use_when: ["Expressing a closed predicate interval"], avoid_when: ["Only one side of the interval is bounded"], params: ["min" => "The inclusive lower bound.", "max" => "The inclusive upper bound."],
 returns: Some("A bounded FloatRange."), example: "FloatRange::between(2.0, 8.0)"
}

register! {
 path: "sand::predicate::BlockPredicate::new", aliases: ["sand::prelude::BlockPredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> BlockPredicate", summary: "Creates an unconstrained BlockPredicate.", context: "Builder methods add only the BlockPredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty BlockPredicate builder."), example: "BlockPredicate::new()"
}

register! {
 path: "sand::predicate::DamagePredicate::new", aliases: ["sand::prelude::DamagePredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> DamagePredicate", summary: "Creates an unconstrained DamagePredicate.", context: "Builder methods add only the DamagePredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty DamagePredicate builder."), example: "DamagePredicate::new()"
}

register! {
 path: "sand::predicate::DamageSourcePredicate::new", aliases: ["sand::prelude::DamageSourcePredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> DamageSourcePredicate", summary: "Creates an unconstrained DamageSourcePredicate.", context: "Builder methods add only the DamageSourcePredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty DamageSourcePredicate builder."), example: "DamageSourcePredicate::new()"
}

register! {
 path: "sand::predicate::DistancePredicate::new", aliases: ["sand::prelude::DistancePredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> DistancePredicate", summary: "Creates an unconstrained DistancePredicate.", context: "Builder methods add only the DistancePredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty DistancePredicate builder."), example: "DistancePredicate::new()"
}

register! {
 path: "sand::predicate::EffectPredicate::new", aliases: [], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> EffectPredicate", summary: "Creates an unconstrained EffectPredicate.", context: "Builder methods add only the EffectPredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty EffectPredicate builder."), example: "EffectPredicate::new()"
}

register! {
 path: "sand::predicate::EntityEquipment::new", aliases: ["sand::prelude::EntityEquipment::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> EntityEquipment", summary: "Creates an unconstrained EntityEquipment.", context: "Builder methods add only the EntityEquipment requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty EntityEquipment builder."), example: "EntityEquipment::new()"
}

register! {
 path: "sand::predicate::EntityFlags::new", aliases: ["sand::prelude::EntityFlags::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> EntityFlags", summary: "Creates an unconstrained EntityFlags.", context: "Builder methods add only the EntityFlags requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty EntityFlags builder."), example: "EntityFlags::new()"
}

register! {
 path: "sand::predicate::EntityPredicate::new", aliases: ["sand::component::EntityPredicate::new", "sand::prelude::EntityPredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> EntityPredicate", summary: "Creates an unconstrained EntityPredicate.", context: "Builder methods add only the EntityPredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty EntityPredicate builder."), example: "EntityPredicate::new()"
}

register! {
 path: "sand::predicate::ItemPredicate::new", aliases: ["sand::component::ItemPredicate::new", "sand::prelude::ItemPredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> ItemPredicate", summary: "Creates an unconstrained ItemPredicate.", context: "Builder methods add only the ItemPredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty ItemPredicate builder."), example: "ItemPredicate::new()"
}

register! {
 path: "sand::predicate::LocationPredicate::new", aliases: ["sand::prelude::LocationPredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> LocationPredicate", summary: "Creates an unconstrained LocationPredicate.", context: "Builder methods add only the LocationPredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty LocationPredicate builder."), example: "LocationPredicate::new()"
}

register! {
 path: "sand::predicate::WeatherPredicate::new", aliases: ["sand::component::WeatherPredicate::new", "sand::prelude::WeatherPredicate::new"], module: "sand::predicate", kind: Method,
 signature: "pub fn new() -> WeatherPredicate", summary: "Creates an unconstrained WeatherPredicate.", context: "Builder methods add only the WeatherPredicate requirements relevant to the surrounding condition.", minecraft: "Serializes an empty predicate object until constraints are added.",
 use_when: ["Building a typed predicate incrementally"], avoid_when: ["No constraints will be added"], params: [],
 returns: Some("An empty WeatherPredicate builder."), example: "WeatherPredicate::new()"
}

register! {
 path: "sand::predicate::BlockPredicate::raw", aliases: ["sand::prelude::BlockPredicate::raw"], module: "sand::predicate", kind: Method,
 signature: "pub fn raw(v: RawJson) -> BlockPredicate", summary: "Creates a BlockPredicate from an unsupported raw JSON shape.", context: "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.", minecraft: "Emits the supplied JSON value in place of the typed predicate object.",
 use_when: ["Minecraft supports a predicate field Sand does not yet model"], avoid_when: ["Typed builder methods cover the required fields"], params: ["v" => "The complete raw JSON predicate value."],
 returns: Some("A raw BlockPredicate."), example: "BlockPredicate::raw(RawJson::new(json!({{}})))"
}

register! {
 path: "sand::predicate::DamagePredicate::raw", aliases: ["sand::prelude::DamagePredicate::raw"], module: "sand::predicate", kind: Method,
 signature: "pub fn raw(v: RawJson) -> DamagePredicate", summary: "Creates a DamagePredicate from an unsupported raw JSON shape.", context: "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.", minecraft: "Emits the supplied JSON value in place of the typed predicate object.",
 use_when: ["Minecraft supports a predicate field Sand does not yet model"], avoid_when: ["Typed builder methods cover the required fields"], params: ["v" => "The complete raw JSON predicate value."],
 returns: Some("A raw DamagePredicate."), example: "DamagePredicate::raw(RawJson::new(json!({{}})))"
}

register! {
 path: "sand::predicate::EntityPredicate::raw", aliases: ["sand::component::EntityPredicate::raw", "sand::prelude::EntityPredicate::raw"], module: "sand::predicate", kind: Method,
 signature: "pub fn raw(v: RawJson) -> EntityPredicate", summary: "Creates a EntityPredicate from an unsupported raw JSON shape.", context: "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.", minecraft: "Emits the supplied JSON value in place of the typed predicate object.",
 use_when: ["Minecraft supports a predicate field Sand does not yet model"], avoid_when: ["Typed builder methods cover the required fields"], params: ["v" => "The complete raw JSON predicate value."],
 returns: Some("A raw EntityPredicate."), example: "EntityPredicate::raw(RawJson::new(json!({{}})))"
}

register! {
 path: "sand::predicate::ItemPredicate::raw", aliases: ["sand::component::ItemPredicate::raw", "sand::prelude::ItemPredicate::raw"], module: "sand::predicate", kind: Method,
 signature: "pub fn raw(v: RawJson) -> ItemPredicate", summary: "Creates a ItemPredicate from an unsupported raw JSON shape.", context: "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.", minecraft: "Emits the supplied JSON value in place of the typed predicate object.",
 use_when: ["Minecraft supports a predicate field Sand does not yet model"], avoid_when: ["Typed builder methods cover the required fields"], params: ["v" => "The complete raw JSON predicate value."],
 returns: Some("A raw ItemPredicate."), example: "ItemPredicate::raw(RawJson::new(json!({{}})))"
}

register! {
 path: "sand::predicate::LocationPredicate::raw", aliases: ["sand::prelude::LocationPredicate::raw"], module: "sand::predicate", kind: Method,
 signature: "pub fn raw(v: RawJson) -> LocationPredicate", summary: "Creates a LocationPredicate from an unsupported raw JSON shape.", context: "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.", minecraft: "Emits the supplied JSON value in place of the typed predicate object.",
 use_when: ["Minecraft supports a predicate field Sand does not yet model"], avoid_when: ["Typed builder methods cover the required fields"], params: ["v" => "The complete raw JSON predicate value."],
 returns: Some("A raw LocationPredicate."), example: "LocationPredicate::raw(RawJson::new(json!({{}})))"
}

register! {
 path: "sand::predicate::WeatherPredicate::raw", aliases: ["sand::component::WeatherPredicate::raw", "sand::prelude::WeatherPredicate::raw"], module: "sand::predicate", kind: Method,
 signature: "pub fn raw(v: RawJson) -> WeatherPredicate", summary: "Creates a WeatherPredicate from an unsupported raw JSON shape.", context: "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.", minecraft: "Emits the supplied JSON value in place of the typed predicate object.",
 use_when: ["Minecraft supports a predicate field Sand does not yet model"], avoid_when: ["Typed builder methods cover the required fields"], params: ["v" => "The complete raw JSON predicate value."],
 returns: Some("A raw WeatherPredicate."), example: "WeatherPredicate::raw(RawJson::new(json!({{}})))"
}

register! {
    path: "sand::predicate::EntityPredicateTarget",
    aliases: ["sand::component::EntityPredicateTarget", "sand::prelude::EntityPredicateTarget"],
    module: "sand::predicate",
    kind: Enum,
    signature: "pub enum EntityPredicateTarget",
    summary: "Names the loot-context entity inspected by an entity-properties condition.",
    context: "Vanilla predicate evaluation supplies contextual entity roles instead of command selectors.",
    minecraft: "Serializes to the entity role accepted by minecraft:entity_properties.",
    use_when: ["Choosing which contextual entity a nested EntityPredicate examines"],
    avoid_when: ["Addressing an entity selected directly by a command"],
    params: [],
    returns: None,
    example: "EntityPredicateTarget::This"
}

register! {
    path: "sand::predicate::EntityPredicateTarget::This",
    aliases: ["sand::component::EntityPredicateTarget::This", "sand::prelude::EntityPredicateTarget::This"],
    module: "sand::predicate",
    kind: Variant,
    signature: "This",
    summary: "Targets the entity at the center of the current predicate context.",
    context: "This variant selects one precise role supplied by Minecraft's loot and predicate context.",
    minecraft: "Serializes as this in an entity-properties condition.",
    use_when: ["Matching properties of this contextual entity role"],
    avoid_when: ["A different context role or a command selector is required"],
    params: [],
    returns: None,
    example: "EntityPredicateTarget::This"
}

register! {
    path: "sand::predicate::EntityPredicateTarget::Killer",
    aliases: ["sand::component::EntityPredicateTarget::Killer", "sand::prelude::EntityPredicateTarget::Killer"],
    module: "sand::predicate",
    kind: Variant,
    signature: "Killer",
    summary: "Targets the entity credited with killing the context entity.",
    context: "This variant selects one precise role supplied by Minecraft's loot and predicate context.",
    minecraft: "Serializes as killer in an entity-properties condition.",
    use_when: ["Matching properties of this contextual entity role"],
    avoid_when: ["A different context role or a command selector is required"],
    params: [],
    returns: None,
    example: "EntityPredicateTarget::Killer"
}

register! {
    path: "sand::predicate::EntityPredicateTarget::KillerPlayer",
    aliases: ["sand::component::EntityPredicateTarget::KillerPlayer", "sand::prelude::EntityPredicateTarget::KillerPlayer"],
    module: "sand::predicate",
    kind: Variant,
    signature: "KillerPlayer",
    summary: "Targets the player credited with killing the context entity.",
    context: "This variant selects one precise role supplied by Minecraft's loot and predicate context.",
    minecraft: "Serializes as killer_player in an entity-properties condition.",
    use_when: ["Matching properties of this contextual entity role"],
    avoid_when: ["A different context role or a command selector is required"],
    params: [],
    returns: None,
    example: "EntityPredicateTarget::KillerPlayer"
}

register! {
    path: "sand::predicate::EntityPredicateTarget::DirectKiller",
    aliases: ["sand::component::EntityPredicateTarget::DirectKiller", "sand::prelude::EntityPredicateTarget::DirectKiller"],
    module: "sand::predicate",
    kind: Variant,
    signature: "DirectKiller",
    summary: "Targets the immediate damaging entity, such as an arrow rather than its shooter.",
    context: "This variant selects one precise role supplied by Minecraft's loot and predicate context.",
    minecraft: "Serializes as direct_killer in an entity-properties condition.",
    use_when: ["Matching properties of this contextual entity role"],
    avoid_when: ["A different context role or a command selector is required"],
    params: [],
    returns: None,
    example: "EntityPredicateTarget::DirectKiller"
}

register! {
    path: "sand::predicate::PredicateRoot::all_of",
    aliases: ["sand::component::PredicateRoot::all_of", "sand::prelude::PredicateRoot::all_of"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn all_of(terms: impl IntoIterator<Item = PredicateRoot>) -> PredicateRoot",
    summary: "Combines predicate roots so every nested condition must succeed.",
    context: "Boolean composition keeps a reusable predicate tree typed without dropping to raw JSON.",
    minecraft: "Emits minecraft:all_of with a non-empty terms array.",
    use_when: ["Combining several independently meaningful predicate conditions"],
    avoid_when: ["The condition list may be empty"],
    params: ["terms" => "The non-empty sequence of condition roots to combine."],
    returns: Some("A composed predicate root."),
    example: "PredicateRoot::all_of([PredicateRoot::random_chance(0.5)])"
}

register! {
    path: "sand::predicate::Predicate::all_of",
    aliases: ["sand::component::Predicate::all_of", "sand::prelude::Predicate::all_of"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn all_of(location: PredicateId, terms: impl IntoIterator<Item = PredicateRoot>) -> Predicate",
    summary: "Creates a named predicate requiring every nested condition to succeed.",
    context: "This convenience constructor binds boolean composition directly to a reusable predicate resource.",
    minecraft: "Exports minecraft:all_of as the root condition of the named predicate.",
    use_when: ["Defining a reusable boolean combination in one expression"],
    avoid_when: ["A single root condition is clearer"],
    params: ["location" => "The typed namespaced identifier of the predicate resource.", "terms" => "The non-empty sequence of condition roots to combine."],
    returns: Some("A predicate component ready for registration."),
    example: "Predicate::all_of(id, [PredicateRoot::random_chance(0.5)])"
}

register! {
    path: "sand::predicate::PredicateRoot::any_of",
    aliases: ["sand::component::PredicateRoot::any_of", "sand::prelude::PredicateRoot::any_of"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn any_of(terms: impl IntoIterator<Item = PredicateRoot>) -> PredicateRoot",
    summary: "Combines predicate roots so at least one nested condition must succeed.",
    context: "Boolean composition keeps a reusable predicate tree typed without dropping to raw JSON.",
    minecraft: "Emits minecraft:any_of with a non-empty terms array.",
    use_when: ["Combining several independently meaningful predicate conditions"],
    avoid_when: ["The condition list may be empty"],
    params: ["terms" => "The non-empty sequence of condition roots to combine."],
    returns: Some("A composed predicate root."),
    example: "PredicateRoot::any_of([PredicateRoot::random_chance(0.5)])"
}

register! {
    path: "sand::predicate::Predicate::any_of",
    aliases: ["sand::component::Predicate::any_of", "sand::prelude::Predicate::any_of"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn any_of(location: PredicateId, terms: impl IntoIterator<Item = PredicateRoot>) -> Predicate",
    summary: "Creates a named predicate requiring at least one nested condition to succeed.",
    context: "This convenience constructor binds boolean composition directly to a reusable predicate resource.",
    minecraft: "Exports minecraft:any_of as the root condition of the named predicate.",
    use_when: ["Defining a reusable boolean combination in one expression"],
    avoid_when: ["A single root condition is clearer"],
    params: ["location" => "The typed namespaced identifier of the predicate resource.", "terms" => "The non-empty sequence of condition roots to combine."],
    returns: Some("A predicate component ready for registration."),
    example: "Predicate::any_of(id, [PredicateRoot::random_chance(0.5)])"
}

register! {
    path: "sand::predicate::PredicateRoot::inverted",
    aliases: ["sand::component::PredicateRoot::inverted", "sand::prelude::PredicateRoot::inverted"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn inverted(term: PredicateRoot) -> PredicateRoot",
    summary: "Negates a nested predicate condition.",
    context: "Typed root constructors map Sand domain values to a precise vanilla loot-condition shape.",
    minecraft: "minecraft:inverted with the nested term field.",
    use_when: ["Building a standalone predicate from typed conditions"],
    avoid_when: ["The check belongs in mutable scoreboard or command state"],
    params: ["term" => "The condition whose result is negated."],
    returns: Some("A typed predicate root."),
    example: "PredicateRoot::inverted(PredicateRoot::random_chance(0.5))"
}

register! {
    path: "sand::predicate::PredicateRoot::location",
    aliases: ["sand::component::PredicateRoot::location", "sand::prelude::PredicateRoot::location"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn location(predicate: LocationPredicate) -> PredicateRoot",
    summary: "Checks typed properties of the current predicate location.",
    context: "Typed root constructors map Sand domain values to a precise vanilla loot-condition shape.",
    minecraft: "minecraft:location_check with a nested location predicate.",
    use_when: ["Building a standalone predicate from typed conditions"],
    avoid_when: ["The check belongs in mutable scoreboard or command state"],
    params: ["predicate" => "The biome, dimension, block, or coordinate requirements."],
    returns: Some("A typed predicate root."),
    example: "PredicateRoot::location(LocationPredicate::new())"
}

register! {
    path: "sand::predicate::PredicateRoot::weather",
    aliases: ["sand::component::PredicateRoot::weather", "sand::prelude::PredicateRoot::weather"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn weather(predicate: WeatherPredicate) -> PredicateRoot",
    summary: "Checks rain or thunder in the current world.",
    context: "Typed root constructors map Sand domain values to a precise vanilla loot-condition shape.",
    minecraft: "minecraft:weather_check with the requested weather flags.",
    use_when: ["Building a standalone predicate from typed conditions"],
    avoid_when: ["The check belongs in mutable scoreboard or command state"],
    params: ["predicate" => "The required raining and thundering state."],
    returns: Some("A typed predicate root."),
    example: "PredicateRoot::weather(WeatherPredicate::new().raining(true))"
}

register! {
    path: "sand::predicate::PredicateRoot::time",
    aliases: ["sand::component::PredicateRoot::time", "sand::prelude::PredicateRoot::time"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn time(range: IntRange) -> PredicateRoot",
    summary: "Checks the current game time against an integer range.",
    context: "Typed root constructors map Sand domain values to a precise vanilla loot-condition shape.",
    minecraft: "minecraft:time_check with the range in its value field.",
    use_when: ["Building a standalone predicate from typed conditions"],
    avoid_when: ["The check belongs in mutable scoreboard or command state"],
    params: ["range" => "The accepted game-time values."],
    returns: Some("A typed predicate root."),
    example: "PredicateRoot::time(IntRange::between(0, 12000))"
}

register! {
    path: "sand::predicate::PredicateRoot::reference",
    aliases: ["sand::component::PredicateRoot::reference", "sand::prelude::PredicateRoot::reference"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn reference(id: PredicateId) -> PredicateRoot",
    summary: "Delegates evaluation to another named predicate resource.",
    context: "Typed root constructors map Sand domain values to a precise vanilla loot-condition shape.",
    minecraft: "minecraft:reference with the referenced predicate name.",
    use_when: ["Building a standalone predicate from typed conditions"],
    avoid_when: ["The check belongs in mutable scoreboard or command state"],
    params: ["id" => "The typed identifier of the predicate resource to evaluate."],
    returns: Some("A typed predicate root."),
    example: "PredicateRoot::reference(other_id)"
}

register! { path: "sand::predicate::DistancePredicate::horizontal_at_most", aliases: ["sand::prelude::DistancePredicate::horizontal_at_most"], module: "sand::predicate", kind: Method, signature: "pub fn horizontal_at_most(max: f64) -> DistancePredicate", summary: "Caps horizontal displacement.", context: "Adds one typed DistancePredicate constraint without disturbing its other requirements.", minecraft: "Sets horizontal to an inclusive maximum.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["max" => "Greatest horizontal distance in blocks."], returns: Some("The updated DistancePredicate predicate."), example: "DistancePredicate::horizontal_at_most(16.0)" }

register! { path: "sand::predicate::DistancePredicate::absolute_at_most", aliases: ["sand::prelude::DistancePredicate::absolute_at_most"], module: "sand::predicate", kind: Method, signature: "pub fn absolute_at_most(max: f64) -> DistancePredicate", summary: "Caps three-dimensional displacement.", context: "Adds one typed DistancePredicate constraint without disturbing its other requirements.", minecraft: "Sets absolute to an inclusive maximum.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["max" => "Greatest absolute distance in blocks."], returns: Some("The updated DistancePredicate predicate."), example: "DistancePredicate::absolute_at_most(16.0)" }

register! { path: "sand::predicate::DistancePredicate::x", aliases: ["sand::prelude::DistancePredicate::x"], module: "sand::predicate", kind: Method, signature: "pub fn x(self, r: FloatRange) -> DistancePredicate", summary: "Constrains x-axis displacement.", context: "Adds one typed DistancePredicate constraint without disturbing its other requirements.", minecraft: "Writes the x range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted x-axis distance range."], returns: Some("The updated DistancePredicate predicate."), example: "DistancePredicate::new().x(FloatRange::at_most(8.0))" }

register! { path: "sand::predicate::DistancePredicate::y", aliases: ["sand::prelude::DistancePredicate::y"], module: "sand::predicate", kind: Method, signature: "pub fn y(self, r: FloatRange) -> DistancePredicate", summary: "Constrains y-axis displacement.", context: "Adds one typed DistancePredicate constraint without disturbing its other requirements.", minecraft: "Writes the y range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted y-axis distance range."], returns: Some("The updated DistancePredicate predicate."), example: "DistancePredicate::new().y(FloatRange::at_most(8.0))" }

register! { path: "sand::predicate::DistancePredicate::z", aliases: ["sand::prelude::DistancePredicate::z"], module: "sand::predicate", kind: Method, signature: "pub fn z(self, r: FloatRange) -> DistancePredicate", summary: "Constrains z-axis displacement.", context: "Adds one typed DistancePredicate constraint without disturbing its other requirements.", minecraft: "Writes the z range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted z-axis distance range."], returns: Some("The updated DistancePredicate predicate."), example: "DistancePredicate::new().z(FloatRange::at_most(8.0))" }

register! { path: "sand::predicate::DistancePredicate::horizontal", aliases: ["sand::prelude::DistancePredicate::horizontal"], module: "sand::predicate", kind: Method, signature: "pub fn horizontal(self, r: FloatRange) -> DistancePredicate", summary: "Constrains horizontal displacement.", context: "Adds one typed DistancePredicate constraint without disturbing its other requirements.", minecraft: "Writes the horizontal range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted horizontal distance range."], returns: Some("The updated DistancePredicate predicate."), example: "DistancePredicate::new().horizontal(FloatRange::at_most(8.0))" }

register! { path: "sand::predicate::DistancePredicate::absolute", aliases: ["sand::prelude::DistancePredicate::absolute"], module: "sand::predicate", kind: Method, signature: "pub fn absolute(self, r: FloatRange) -> DistancePredicate", summary: "Constrains three-dimensional displacement.", context: "Adds one typed DistancePredicate constraint without disturbing its other requirements.", minecraft: "Writes the absolute range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted three-dimensional distance range."], returns: Some("The updated DistancePredicate predicate."), example: "DistancePredicate::new().absolute(FloatRange::at_most(8.0))" }

register! { path: "sand::predicate::EffectPredicate::amplifier", aliases: [], module: "sand::predicate", kind: Method, signature: "pub fn amplifier(self, r: IntRange) -> EffectPredicate", summary: "Constrains the effect amplifier.", context: "Adds one typed EffectPredicate constraint without disturbing its other requirements.", minecraft: "Writes the amplifier requirement.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Required effect amplifier."], returns: Some("The updated EffectPredicate predicate."), example: "EffectPredicate::new().amplifier(IntRange::at_least(1))" }

register! { path: "sand::predicate::EffectPredicate::duration", aliases: [], module: "sand::predicate", kind: Method, signature: "pub fn duration(self, r: IntRange) -> EffectPredicate", summary: "Constrains the remaining duration.", context: "Adds one typed EffectPredicate constraint without disturbing its other requirements.", minecraft: "Writes the duration requirement.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Required remaining duration."], returns: Some("The updated EffectPredicate predicate."), example: "EffectPredicate::new().duration(IntRange::at_least(1))" }

register! { path: "sand::predicate::EffectPredicate::ambient", aliases: [], module: "sand::predicate", kind: Method, signature: "pub fn ambient(self, v: bool) -> EffectPredicate", summary: "Constrains the ambient state.", context: "Adds one typed EffectPredicate constraint without disturbing its other requirements.", minecraft: "Writes the ambient requirement.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Required ambient state."], returns: Some("The updated EffectPredicate predicate."), example: "EffectPredicate::new().ambient(true)" }

register! { path: "sand::predicate::EffectPredicate::visible", aliases: [], module: "sand::predicate", kind: Method, signature: "pub fn visible(self, v: bool) -> EffectPredicate", summary: "Constrains the particle visibility.", context: "Adds one typed EffectPredicate constraint without disturbing its other requirements.", minecraft: "Writes the visible requirement.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Required particle visibility."], returns: Some("The updated EffectPredicate predicate."), example: "EffectPredicate::new().visible(true)" }

register! { path: "sand::predicate::DamageSourcePredicate::requires_tag", aliases: ["sand::prelude::DamageSourcePredicate::requires_tag"], module: "sand::predicate", kind: Method, signature: "pub fn requires_tag(self, tag: TagId<DamageTypeId>) -> DamageSourcePredicate", summary: "Requires a damage-type tag.", context: "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.", minecraft: "Adds a tag predicate expected to be true.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["tag" => "Damage-type tag tested against the event."], returns: Some("The updated DamageSourcePredicate predicate."), example: "DamageSourcePredicate::new().requires_tag(tag)" }

register! { path: "sand::predicate::DamageSourcePredicate::excludes_tag", aliases: ["sand::prelude::DamageSourcePredicate::excludes_tag"], module: "sand::predicate", kind: Method, signature: "pub fn excludes_tag(self, tag: TagId<DamageTypeId>) -> DamageSourcePredicate", summary: "Excludes a damage-type tag.", context: "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.", minecraft: "Adds a tag predicate expected to be false.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["tag" => "Damage-type tag tested against the event."], returns: Some("The updated DamageSourcePredicate predicate."), example: "DamageSourcePredicate::new().excludes_tag(tag)" }

register! { path: "sand::predicate::DamageSourcePredicate::source_entity", aliases: ["sand::prelude::DamageSourcePredicate::source_entity"], module: "sand::predicate", kind: Method, signature: "pub fn source_entity(self, ep: EntityPredicate) -> DamageSourcePredicate", summary: "Constrains the responsible entity.", context: "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.", minecraft: "Nests the entity predicate in source_entity.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["ep" => "Required properties of the responsible entity."], returns: Some("The updated DamageSourcePredicate predicate."), example: "DamageSourcePredicate::new().source_entity(EntityPredicate::new())" }

register! { path: "sand::predicate::DamageSourcePredicate::direct_entity", aliases: ["sand::prelude::DamageSourcePredicate::direct_entity"], module: "sand::predicate", kind: Method, signature: "pub fn direct_entity(self, ep: EntityPredicate) -> DamageSourcePredicate", summary: "Constrains the immediate damaging entity.", context: "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.", minecraft: "Nests the entity predicate in direct_entity.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["ep" => "Required properties of the immediate damaging entity."], returns: Some("The updated DamageSourcePredicate predicate."), example: "DamageSourcePredicate::new().direct_entity(EntityPredicate::new())" }

register! { path: "sand::predicate::DamagePredicate::dealt", aliases: ["sand::prelude::DamagePredicate::dealt"], module: "sand::predicate", kind: Method, signature: "pub fn dealt(self, r: FloatRange) -> DamagePredicate", summary: "Constrains raw damage dealt.", context: "Adds one typed DamagePredicate constraint without disturbing its other requirements.", minecraft: "Writes the dealt range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted range for raw damage dealt."], returns: Some("The updated DamagePredicate predicate."), example: "DamagePredicate::new().dealt(FloatRange::at_least(2.0))" }

register! { path: "sand::predicate::DamagePredicate::taken", aliases: ["sand::prelude::DamagePredicate::taken"], module: "sand::predicate", kind: Method, signature: "pub fn taken(self, r: FloatRange) -> DamagePredicate", summary: "Constrains damage taken after mitigation.", context: "Adds one typed DamagePredicate constraint without disturbing its other requirements.", minecraft: "Writes the taken range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted range for damage taken after mitigation."], returns: Some("The updated DamagePredicate predicate."), example: "DamagePredicate::new().taken(FloatRange::at_least(2.0))" }

register! { path: "sand::predicate::DamagePredicate::blocked", aliases: ["sand::prelude::DamagePredicate::blocked"], module: "sand::predicate", kind: Method, signature: "pub fn blocked(self, v: bool) -> DamagePredicate", summary: "Requires a shield-blocking state.", context: "Adds one typed DamagePredicate constraint without disturbing its other requirements.", minecraft: "Writes the blocked boolean.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Whether the event must be blocked."], returns: Some("The updated DamagePredicate predicate."), example: "DamagePredicate::new().blocked(true)" }

register! { path: "sand::predicate::DamagePredicate::source_entity", aliases: ["sand::prelude::DamagePredicate::source_entity"], module: "sand::predicate", kind: Method, signature: "pub fn source_entity(self, ep: EntityPredicate) -> DamagePredicate", summary: "Constrains the responsible entity.", context: "Adds one typed DamagePredicate constraint without disturbing its other requirements.", minecraft: "Nests source_entity.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["ep" => "Required responsible-entity properties."], returns: Some("The updated DamagePredicate predicate."), example: "DamagePredicate::new().source_entity(EntityPredicate::new())" }

register! { path: "sand::predicate::DamagePredicate::type_", aliases: ["sand::prelude::DamagePredicate::type_"], module: "sand::predicate", kind: Method, signature: "pub fn type_(self, dsp: DamageSourcePredicate) -> DamagePredicate", summary: "Constrains the damage source.", context: "Adds one typed DamagePredicate constraint without disturbing its other requirements.", minecraft: "Nests the type predicate.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["dsp" => "Required damage-source properties."], returns: Some("The updated DamagePredicate predicate."), example: "DamagePredicate::new().type_(DamageSourcePredicate::new())" }

register! { path: "sand::predicate::LocationPredicate::biome", aliases: ["sand::prelude::LocationPredicate::biome"], module: "sand::predicate", kind: Method, signature: "pub fn biome(self, biome: BiomeId) -> LocationPredicate", summary: "Requires one biome.", context: "Adds one typed LocationPredicate constraint without disturbing its other requirements.", minecraft: "Writes the typed biome identifier.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["biome" => "Required biome identifier."], returns: Some("The updated LocationPredicate predicate."), example: "LocationPredicate::new().biome(id)" }

register! { path: "sand::predicate::LocationPredicate::dimension", aliases: ["sand::prelude::LocationPredicate::dimension"], module: "sand::predicate", kind: Method, signature: "pub fn dimension(self, dimension: DimensionId) -> LocationPredicate", summary: "Requires one dimension.", context: "Adds one typed LocationPredicate constraint without disturbing its other requirements.", minecraft: "Writes the typed dimension identifier.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["dimension" => "Required dimension identifier."], returns: Some("The updated LocationPredicate predicate."), example: "LocationPredicate::new().dimension(id)" }

register! { path: "sand::predicate::LocationPredicate::smokey", aliases: ["sand::prelude::LocationPredicate::smokey"], module: "sand::predicate", kind: Method, signature: "pub fn smokey(self, v: bool) -> LocationPredicate", summary: "Requires the bee-smokey state.", context: "Adds one typed LocationPredicate constraint without disturbing its other requirements.", minecraft: "Writes the smokey boolean.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Whether the position must be smokey."], returns: Some("The updated LocationPredicate predicate."), example: "LocationPredicate::new().smokey(true)" }

register! { path: "sand::predicate::LocationPredicate::block", aliases: ["sand::prelude::LocationPredicate::block"], module: "sand::predicate", kind: Method, signature: "pub fn block(self, bp: BlockPredicate) -> LocationPredicate", summary: "Constrains the block at the position.", context: "Adds one typed LocationPredicate constraint without disturbing its other requirements.", minecraft: "Nests a block predicate.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["bp" => "Required block properties."], returns: Some("The updated LocationPredicate predicate."), example: "LocationPredicate::new().block(BlockPredicate::new())" }

register! { path: "sand::predicate::LocationPredicate::x", aliases: ["sand::prelude::LocationPredicate::x"], module: "sand::predicate", kind: Method, signature: "pub fn x(self, r: FloatRange) -> LocationPredicate", summary: "Constrains the x-coordinate.", context: "Adds one typed LocationPredicate constraint without disturbing its other requirements.", minecraft: "Writes position.x.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted x-coordinate range."], returns: Some("The updated LocationPredicate predicate."), example: "LocationPredicate::new().x(FloatRange::between(0.0, 16.0))" }

register! { path: "sand::predicate::LocationPredicate::y", aliases: ["sand::prelude::LocationPredicate::y"], module: "sand::predicate", kind: Method, signature: "pub fn y(self, r: FloatRange) -> LocationPredicate", summary: "Constrains the y-coordinate.", context: "Adds one typed LocationPredicate constraint without disturbing its other requirements.", minecraft: "Writes position.y.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted y-coordinate range."], returns: Some("The updated LocationPredicate predicate."), example: "LocationPredicate::new().y(FloatRange::between(0.0, 16.0))" }

register! { path: "sand::predicate::LocationPredicate::z", aliases: ["sand::prelude::LocationPredicate::z"], module: "sand::predicate", kind: Method, signature: "pub fn z(self, r: FloatRange) -> LocationPredicate", summary: "Constrains the z-coordinate.", context: "Adds one typed LocationPredicate constraint without disturbing its other requirements.", minecraft: "Writes position.z.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted z-coordinate range."], returns: Some("The updated LocationPredicate predicate."), example: "LocationPredicate::new().z(FloatRange::between(0.0, 16.0))" }

register! { path: "sand::predicate::BlockPredicate::blocks", aliases: ["sand::prelude::BlockPredicate::blocks"], module: "sand::predicate", kind: Method, signature: "pub fn blocks(self, ids: Vec<BlockId>) -> BlockPredicate", summary: "Matches typed block identifiers.", context: "Adds one typed BlockPredicate constraint without disturbing its other requirements.", minecraft: "Writes the blocks array.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["ids" => "Accepted block identifiers."], returns: Some("The updated BlockPredicate predicate."), example: "BlockPredicate::new().blocks(vec![block])" }

register! { path: "sand::predicate::BlockPredicate::tag", aliases: ["sand::prelude::BlockPredicate::tag"], module: "sand::predicate", kind: Method, signature: "pub fn tag(self, tag: TagId<BlockId>) -> BlockPredicate", summary: "Matches a typed block tag.", context: "Adds one typed BlockPredicate constraint without disturbing its other requirements.", minecraft: "Writes the block tag.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["tag" => "Tag whose members are accepted."], returns: Some("The updated BlockPredicate predicate."), example: "BlockPredicate::new().tag(tag)" }

register! { path: "sand::predicate::BlockPredicate::nbt", aliases: ["sand::prelude::BlockPredicate::nbt"], module: "sand::predicate", kind: Method, signature: "pub fn nbt(self, nbt: RawSnbt) -> BlockPredicate", summary: "Matches block-entity data.", context: "Adds one typed BlockPredicate constraint without disturbing its other requirements.", minecraft: "Writes the SNBT fragment.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["nbt" => "Block-entity data that must match."], returns: Some("The updated BlockPredicate predicate."), example: "BlockPredicate::new().nbt(nbt)" }

register! { path: "sand::predicate::BlockPredicate::state", aliases: ["sand::prelude::BlockPredicate::state"], module: "sand::predicate", kind: Method, signature: "pub fn state(self, state: BTreeMap<String, String>) -> BlockPredicate", summary: "Matches block-state properties.", context: "Adds one typed BlockPredicate constraint without disturbing its other requirements.", minecraft: "Writes the state property map.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["state" => "Exact property names and values."], returns: Some("The updated BlockPredicate predicate."), example: "BlockPredicate::new().state(properties)" }

register! { path: "sand::predicate::ItemPredicate::id", aliases: ["sand::component::ItemPredicate::id","sand::prelude::ItemPredicate::id"], module: "sand::predicate", kind: Method, signature: "pub fn id(id: impl Into<ItemId>) -> ItemPredicate", summary: "Creates a predicate for one typed item.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Initializes the item identity.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["id" => "Item identifier to match."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::id(item)" }

register! { path: "sand::predicate::ItemPredicate::item", aliases: ["sand::component::ItemPredicate::item","sand::prelude::ItemPredicate::item"], module: "sand::predicate", kind: Method, signature: "pub fn item(self, id: impl Into<ItemId>) -> ItemPredicate", summary: "Requires one typed item identity.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Writes the item identifier.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["id" => "Item identifier to match."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().item(item)" }

register! { path: "sand::predicate::ItemPredicate::count_min", aliases: ["sand::component::ItemPredicate::count_min","sand::prelude::ItemPredicate::count_min"], module: "sand::predicate", kind: Method, signature: "pub fn count_min(self, min: i64) -> ItemPredicate", summary: "Sets the inclusive minimum stack count.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Writes count.min.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["min" => "Inclusive minimum stack count."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().count_min(1)" }

register! { path: "sand::predicate::ItemPredicate::count_max", aliases: ["sand::component::ItemPredicate::count_max","sand::prelude::ItemPredicate::count_max"], module: "sand::predicate", kind: Method, signature: "pub fn count_max(self, max: i64) -> ItemPredicate", summary: "Sets the inclusive maximum stack count.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Writes count.max.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["max" => "Inclusive maximum stack count."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().count_max(1)" }

register! { path: "sand::predicate::ItemPredicate::count_range", aliases: ["sand::component::ItemPredicate::count_range","sand::prelude::ItemPredicate::count_range"], module: "sand::predicate", kind: Method, signature: "pub fn count_range(self, min: i64, max: i64) -> ItemPredicate", summary: "Sets an inclusive stack-count interval.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Writes both count bounds.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["min" => "Inclusive minimum stack count.","max" => "Inclusive maximum stack count."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().count_range(1, 16)" }

register! { path: "sand::predicate::ItemPredicate::count", aliases: ["sand::component::ItemPredicate::count","sand::prelude::ItemPredicate::count"], module: "sand::predicate", kind: Method, signature: "pub fn count(self, r: IntRange) -> ItemPredicate", summary: "Sets a typed stack-count range.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Writes the count range.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["r" => "Accepted stack counts."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().count(IntRange::at_least(1))" }

register! { path: "sand::predicate::ItemPredicate::custom_data_key", aliases: ["sand::component::ItemPredicate::custom_data_key","sand::prelude::ItemPredicate::custom_data_key"], module: "sand::predicate", kind: Method, signature: "pub fn custom_data_key(self, key: impl Into<String>) -> ItemPredicate", summary: "Requires a key in item custom data.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Writes a custom_data presence predicate.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["key" => "Exact custom-data key required."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().custom_data_key(\"quest_item\")" }

register! { path: "sand::predicate::ItemPredicate::raw_components", aliases: ["sand::component::ItemPredicate::raw_components","sand::prelude::ItemPredicate::raw_components"], module: "sand::predicate", kind: Method, signature: "pub fn raw_components(self, v: RawJson) -> ItemPredicate", summary: "Supplies unsupported raw item component values.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Merges JSON into the item components section.", use_when: ["Minecraft supports item data Sand does not yet model"], avoid_when: ["A typed item method expresses the requirement"], params: ["v" => "JSON object containing component values."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().raw_components(raw)" }

register! { path: "sand::predicate::ItemPredicate::raw_predicates", aliases: ["sand::component::ItemPredicate::raw_predicates","sand::prelude::ItemPredicate::raw_predicates"], module: "sand::predicate", kind: Method, signature: "pub fn raw_predicates(self, v: RawJson) -> ItemPredicate", summary: "Supplies unsupported raw item component predicate tests.", context: "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.", minecraft: "Merges JSON into the item predicates section.", use_when: ["Minecraft supports item data Sand does not yet model"], avoid_when: ["A typed item method expresses the requirement"], params: ["v" => "JSON object containing component predicate tests."], returns: Some("The updated ItemPredicate predicate."), example: "ItemPredicate::new().raw_predicates(raw)" }

register! { path: "sand::predicate::EntityEquipment::head", aliases: ["sand::prelude::EntityEquipment::head"], module: "sand::predicate", kind: Method, signature: "pub fn head(self, p: ItemPredicate) -> EntityEquipment", summary: "Constrains the entity's head slot.", context: "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.", minecraft: "Writes an item predicate under head.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["p" => "Item requirements for the head slot."], returns: Some("The updated EntityEquipment predicate."), example: "EntityEquipment::new().head(ItemPredicate::new())" }

register! { path: "sand::predicate::EntityEquipment::chest", aliases: ["sand::prelude::EntityEquipment::chest"], module: "sand::predicate", kind: Method, signature: "pub fn chest(self, p: ItemPredicate) -> EntityEquipment", summary: "Constrains the entity's chest slot.", context: "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.", minecraft: "Writes an item predicate under chest.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["p" => "Item requirements for the chest slot."], returns: Some("The updated EntityEquipment predicate."), example: "EntityEquipment::new().chest(ItemPredicate::new())" }

register! { path: "sand::predicate::EntityEquipment::legs", aliases: ["sand::prelude::EntityEquipment::legs"], module: "sand::predicate", kind: Method, signature: "pub fn legs(self, p: ItemPredicate) -> EntityEquipment", summary: "Constrains the entity's legs slot.", context: "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.", minecraft: "Writes an item predicate under legs.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["p" => "Item requirements for the legs slot."], returns: Some("The updated EntityEquipment predicate."), example: "EntityEquipment::new().legs(ItemPredicate::new())" }

register! { path: "sand::predicate::EntityEquipment::feet", aliases: ["sand::prelude::EntityEquipment::feet"], module: "sand::predicate", kind: Method, signature: "pub fn feet(self, p: ItemPredicate) -> EntityEquipment", summary: "Constrains the entity's feet slot.", context: "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.", minecraft: "Writes an item predicate under feet.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["p" => "Item requirements for the feet slot."], returns: Some("The updated EntityEquipment predicate."), example: "EntityEquipment::new().feet(ItemPredicate::new())" }

register! { path: "sand::predicate::EntityEquipment::mainhand", aliases: ["sand::prelude::EntityEquipment::mainhand"], module: "sand::predicate", kind: Method, signature: "pub fn mainhand(self, p: ItemPredicate) -> EntityEquipment", summary: "Constrains the entity's mainhand slot.", context: "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.", minecraft: "Writes an item predicate under mainhand.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["p" => "Item requirements for the mainhand slot."], returns: Some("The updated EntityEquipment predicate."), example: "EntityEquipment::new().mainhand(ItemPredicate::new())" }

register! { path: "sand::predicate::EntityEquipment::offhand", aliases: ["sand::prelude::EntityEquipment::offhand"], module: "sand::predicate", kind: Method, signature: "pub fn offhand(self, p: ItemPredicate) -> EntityEquipment", summary: "Constrains the entity's offhand slot.", context: "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.", minecraft: "Writes an item predicate under offhand.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["p" => "Item requirements for the offhand slot."], returns: Some("The updated EntityEquipment predicate."), example: "EntityEquipment::new().offhand(ItemPredicate::new())" }

register! { path: "sand::predicate::EntityFlags::on_fire", aliases: ["sand::prelude::EntityFlags::on_fire"], module: "sand::predicate", kind: Method, signature: "pub fn on_fire(self, v: bool) -> EntityFlags", summary: "Requires a specific burning state.", context: "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.", minecraft: "Writes the on_fire flag.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Required burning state."], returns: Some("The updated EntityFlags predicate."), example: "EntityFlags::new().on_fire(true)" }

register! { path: "sand::predicate::EntityFlags::sneaking", aliases: ["sand::prelude::EntityFlags::sneaking"], module: "sand::predicate", kind: Method, signature: "pub fn sneaking(self, v: bool) -> EntityFlags", summary: "Requires a specific sneaking state.", context: "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.", minecraft: "Writes the sneaking flag.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Required sneaking state."], returns: Some("The updated EntityFlags predicate."), example: "EntityFlags::new().sneaking(true)" }

register! { path: "sand::predicate::EntityFlags::sprinting", aliases: ["sand::prelude::EntityFlags::sprinting"], module: "sand::predicate", kind: Method, signature: "pub fn sprinting(self, v: bool) -> EntityFlags", summary: "Requires a specific sprinting state.", context: "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.", minecraft: "Writes the sprinting flag.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Required sprinting state."], returns: Some("The updated EntityFlags predicate."), example: "EntityFlags::new().sprinting(true)" }

register! { path: "sand::predicate::EntityFlags::swimming", aliases: ["sand::prelude::EntityFlags::swimming"], module: "sand::predicate", kind: Method, signature: "pub fn swimming(self, v: bool) -> EntityFlags", summary: "Requires a specific swimming state.", context: "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.", minecraft: "Writes the swimming flag.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Required swimming state."], returns: Some("The updated EntityFlags predicate."), example: "EntityFlags::new().swimming(true)" }

register! { path: "sand::predicate::EntityFlags::baby", aliases: ["sand::prelude::EntityFlags::baby"], module: "sand::predicate", kind: Method, signature: "pub fn baby(self, v: bool) -> EntityFlags", summary: "Requires a specific baby-age state.", context: "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.", minecraft: "Writes the baby flag.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Required baby-age state."], returns: Some("The updated EntityFlags predicate."), example: "EntityFlags::new().baby(true)" }

register! { path: "sand::predicate::EntityPredicate::type_", aliases: ["sand::component::EntityPredicate::type_","sand::prelude::EntityPredicate::type_"], module: "sand::predicate", kind: Method, signature: "pub fn type_(entity_type: impl Into<EntityTypeId>) -> EntityPredicate", summary: "Creates a predicate for one entity type.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Initializes the type field.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["entity_type" => "Entity type identifier to match."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::type_(entity_type)" }

register! { path: "sand::predicate::EntityPredicate::with_type", aliases: ["sand::component::EntityPredicate::with_type","sand::prelude::EntityPredicate::with_type"], module: "sand::predicate", kind: Method, signature: "pub fn with_type(self, entity_type: impl Into<EntityTypeId>) -> EntityPredicate", summary: "Requires one entity type.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Writes one typed entity identifier.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["entity_type" => "Entity type identifier to match."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::new().with_type(entity_type)" }

register! { path: "sand::predicate::EntityPredicate::with_type_any", aliases: ["sand::component::EntityPredicate::with_type_any","sand::prelude::EntityPredicate::with_type_any"], module: "sand::predicate", kind: Method, signature: "pub fn with_type_any(self, types: Vec<EntityTypeId>) -> EntityPredicate", summary: "Accepts any entity type in a typed list.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Writes entity-type alternatives.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["types" => "Non-empty accepted entity identifiers."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::new().with_type_any(vec![zombie, skeleton])" }

register! { path: "sand::predicate::EntityPredicate::nbt", aliases: ["sand::component::EntityPredicate::nbt","sand::prelude::EntityPredicate::nbt"], module: "sand::predicate", kind: Method, signature: "pub fn nbt(self, nbt: RawSnbt) -> EntityPredicate", summary: "Constrains entity NBT data.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Writes the SNBT fragment.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["nbt" => "Entity data fragment that must match."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::new().nbt(nbt)" }

register! { path: "sand::predicate::EntityPredicate::location", aliases: ["sand::component::EntityPredicate::location","sand::prelude::EntityPredicate::location"], module: "sand::predicate", kind: Method, signature: "pub fn location(self, lp: LocationPredicate) -> EntityPredicate", summary: "Constrains the entity's location.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Nests a location predicate.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["lp" => "World-location requirements."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::new().location(LocationPredicate::new())" }

register! { path: "sand::predicate::EntityPredicate::flags", aliases: ["sand::component::EntityPredicate::flags","sand::prelude::EntityPredicate::flags"], module: "sand::predicate", kind: Method, signature: "pub fn flags(self, flags: EntityFlags) -> EntityPredicate", summary: "Constrains entity state flags.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Nests the flags object.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["flags" => "Required fire, movement, and age flags."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::new().flags(EntityFlags::new())" }

register! { path: "sand::predicate::EntityPredicate::equipment", aliases: ["sand::component::EntityPredicate::equipment","sand::prelude::EntityPredicate::equipment"], module: "sand::predicate", kind: Method, signature: "pub fn equipment(self, eq: EntityEquipment) -> EntityPredicate", summary: "Constrains worn or held items.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Nests slot-specific item predicates.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["eq" => "Required equipment by slot."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::new().equipment(EntityEquipment::new())" }

register! { path: "sand::predicate::EntityPredicate::effect", aliases: ["sand::component::EntityPredicate::effect","sand::prelude::EntityPredicate::effect"], module: "sand::predicate", kind: Method, signature: "pub fn effect(self, effect_id: impl Into<EffectId>, pred: EffectPredicate) -> EntityPredicate", summary: "Constrains one active status effect.", context: "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.", minecraft: "Adds a typed effect and requirements to effects.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["effect_id" => "Status effect that must be active.","pred" => "Required amplifier, duration, and flags."], returns: Some("The updated EntityPredicate predicate."), example: "EntityPredicate::new().effect(effect_id, EffectPredicate::new())" }

register! { path: "sand::predicate::WeatherPredicate::raining", aliases: ["sand::component::WeatherPredicate::raining","sand::prelude::WeatherPredicate::raining"], module: "sand::predicate", kind: Method, signature: "pub fn raining(self, v: bool) -> WeatherPredicate", summary: "Requires a specific rain state.", context: "Adds one domain-specific WeatherPredicate requirement without disturbing its other constraints.", minecraft: "Writes the raining boolean.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Whether rain must be active."], returns: Some("The updated WeatherPredicate predicate."), example: "WeatherPredicate::new().raining(true)" }

register! { path: "sand::predicate::WeatherPredicate::thundering", aliases: ["sand::component::WeatherPredicate::thundering","sand::prelude::WeatherPredicate::thundering"], module: "sand::predicate", kind: Method, signature: "pub fn thundering(self, v: bool) -> WeatherPredicate", summary: "Requires a specific thunder state.", context: "Adds one domain-specific WeatherPredicate requirement without disturbing its other constraints.", minecraft: "Writes the thundering boolean.", use_when: ["Composing this property into a larger predicate"], avoid_when: ["The property should remain unconstrained"], params: ["v" => "Whether thunder must be active."], returns: Some("The updated WeatherPredicate predicate."), example: "WeatherPredicate::new().thundering(true)" }

register! {
    path: "sand::predicate::PredicateRoot::raw",
    aliases: ["sand::component::PredicateRoot::raw", "sand::prelude::PredicateRoot::raw"],
    module: "sand::predicate",
    kind: Method,
    signature: "pub fn raw(json: RawJson) -> serde_json::Result<PredicateRoot>",
    summary: "Validates an unsupported predicate condition supplied as raw JSON.",
    context: "The fallible escape hatch preserves access to new or modded conditions while rejecting non-object roots.",
    minecraft: "The object is emitted verbatim as a vanilla or modded condition.",
    use_when: ["Using a condition shape Sand does not yet model"],
    avoid_when: ["A typed PredicateRoot constructor can express the condition"],
    params: ["json" => "A JSON object containing the complete predicate condition."],
    returns: Some("A validated raw predicate root, or a JSON shape error."),
    example: "PredicateRoot::raw(RawJson::new(json!({\"condition\": \"minecraft:survives_explosion\"})))?"
}
