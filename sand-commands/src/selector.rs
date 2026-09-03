//! Entity/player selector (`@a`, `@e`, `@s`, etc.) with a typed builder API.

use std::fmt;
use std::marker::PhantomData;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::validate;

// ── Entity type conversion ──────────────────────────────────────────────────────

/// Conversion accepted by entity-type filter/target methods (`entity_type`,
/// `not_type`, `summon`, ...).
///
/// Implemented for `&str`/`String` (the untyped escape hatch — no validation
/// beyond what the selector/command syntax itself enforces) and for Sand's
/// typed vanilla/custom entity-type identifiers: the generated vanilla
/// entity-type enum when the selected Minecraft profile provides one, and
/// `EntityTypeId` (validated custom/modded IDs).
/// Prefer the typed identifiers in normal code; the string forms remain for
/// compatibility and cases with no typed representation yet.
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::IntoEntityType",
    aliases = ["sand::cmd::IntoEntityType", "sand::prelude::cmd::IntoEntityType"],
    module = "sand::command",
    summary = "Conversion accepted by entity-type filter/target methods (`entity_type`, `not_type`, `summon`, ...).",
    context = "Conversion accepted by entity-type filter/target methods (`entity_type`, `not_type`, `summon`, ...). Implemented for `&str`/`String` (the untyped escape hatch — no validation beyond what the selector/command syntax itself enforces) and for Sand's typed vanilla/custom entity-type identifiers: the generated vanilla entity-type enum when the selected Minecraft profile provides one, and `EntityTypeId` (validated custom/modded IDs). Prefer the typed identifiers in normal code; the string forms remain for compatibility and cases with no typed representation yet.",
    minecraft = "Implemented for `&str`/`String` (the untyped escape hatch — no validation beyond what the selector/command syntax itself enforces) and for Sand's typed vanilla/custom entity-type identifiers: the generated vanilla entity-type enum when the selected Minecraft profile provides one, and `EntityTypeId` (validated custom/modded IDs). Prefer the typed identifiers in normal code; the string forms remain for compatibility and cases with no typed representation yet.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::IntoEntityType;",
)]
pub trait IntoEntityType {
    /// Convert to the entity type's resource location, e.g. `"minecraft:marker"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::IntoEntityType::into_entity_type",
        aliases = ["sand::cmd::IntoEntityType::into_entity_type", "sand::prelude::cmd::IntoEntityType::into_entity_type"],
        module = "sand::command",
        summary = "Convert to the entity type's resource location, e.g. `\"minecraft:marker\"`.",
        context = "Convert to the entity type's resource location, e.g. `\"minecraft:marker\"`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to convert to the entity type's resource location, e.g. `\"minecraft:marker\"`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::IntoEntityType>(into_entity_type_value: T)  {\n    let into_entity_type = into_entity_type_value.into_entity_type();\n}",
    )]
    fn into_entity_type(self) -> String;
}

impl IntoEntityType for String {
    fn into_entity_type(self) -> String {
        self
    }
}

impl IntoEntityType for &str {
    fn into_entity_type(self) -> String {
        self.to_string()
    }
}

impl IntoEntityType for &String {
    fn into_entity_type(self) -> String {
        self.clone()
    }
}

// ── Public types ──────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Selector",
    aliases = ["sand::cmd::Selector", "sand::prelude::Selector", "sand::prelude::cmd::Selector"],
    module = "sand::command",
    summary = "An entity/player selector for use in Minecraft commands.",
    context = "An entity/player selector for use in Minecraft commands. Selectors target entities in the world. Construct with a base selector (e.g., `all_players()`) then refine with builder methods to add filters (tags, distance, team, etc.).",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Selector;",
)]
/// An entity/player selector for use in Minecraft commands.
///
/// Selectors target entities in the world. Construct with a base selector (e.g., `all_players()`)
/// then refine with builder methods to add filters (tags, distance, team, etc.).
///
/// # Examples
/// ```
/// use sand_commands::selector::Selector;
///
/// // @a[tag=ready,limit=1]
/// let sel = Selector::all_players().tag("ready").limit(1);
/// assert_eq!(sel.to_string(), "@a[tag=ready,limit=1]");
///
/// // @s
/// assert_eq!(Selector::self_().to_string(), "@s");
/// ```
#[derive(Debug, Clone)]
#[must_use = "selectors do nothing until passed to a command"]
pub struct Selector {
    base: TargetBase,
    args: Vec<SelectorArg>,
}

impl PartialEq for Selector {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Selector {}

impl From<Selector> for String {
    fn from(s: Selector) -> Self {
        s.to_string()
    }
}

impl From<&Selector> for String {
    fn from(s: &Selector) -> Self {
        s.to_string()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::TargetBase",
    aliases = ["sand::cmd::TargetBase", "sand::prelude::cmd::TargetBase"],
    module = "sand::command",
    summary = "The base target variant of a selector.",
    context = "The base target variant of a selector. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::TargetBase;",
    variants(AllEntities = "Selects the all entities form of the target base Minecraft command value.", AllPlayers = "Selects the all players form of the target base Minecraft command value.", NearestPlayer = "Selects the nearest player form of the target base Minecraft command value.", Player = "Selects the player form of the target base Minecraft command value.", RandomPlayer = "Selects the random player form of the target base Minecraft command value.", Raw = "Explicit unchecked selector syntax for advanced/modded grammar.", Self_ = "Selects the self  form of the target base Minecraft command value."),
    variant_fields(Player = ["Selects the player form of the target base Minecraft command value."], Raw = ["Explicit unchecked selector syntax for advanced/modded grammar."]),
)]
/// The base target variant of a selector.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetBase {
    #[doc = "Selects the all players form of the target base Minecraft command value."]
    AllPlayers,
    #[doc = "Selects the all entities form of the target base Minecraft command value."]
    AllEntities,
    #[doc = "Selects the nearest player form of the target base Minecraft command value."]
    NearestPlayer,
    #[doc = "Selects the self  form of the target base Minecraft command value."]
    Self_,
    #[doc = "Selects the random player form of the target base Minecraft command value."]
    RandomPlayer,
    #[doc = "Selects the player form of the target base Minecraft command value."]
    Player(#[doc = "Selects the player form of the target base Minecraft command value."] String),
    /// Explicit unchecked selector syntax for advanced/modded grammar.
    Raw(#[doc = "Explicit unchecked selector syntax for advanced/modded grammar."] String),
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::One",
    aliases = ["sand::cmd::One", "sand::prelude::cmd::One"],
    module = "sand::command",
    summary = "Marker for selector wrappers that are statically known to select one target.",
    context = "Marker for selector wrappers that are statically known to select one target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::One;",
)]
/// Marker for selector wrappers that are statically known to select one target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum One {}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Many",
    aliases = ["sand::cmd::Many", "sand::prelude::cmd::Many"],
    module = "sand::command",
    summary = "Marker for selector wrappers that may select multiple targets.",
    context = "Marker for selector wrappers that may select multiple targets. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Many;",
)]
/// Marker for selector wrappers that may select multiple targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Many {}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::EntityTarget",
    aliases = ["sand::cmd::EntityTarget", "sand::prelude::cmd::EntityTarget"],
    module = "sand::command",
    summary = "Entity selector with statically modeled arity.",
    context = "Entity selector with statically modeled arity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::EntityTarget;",
)]
/// Entity selector with statically modeled arity.
#[derive(Debug, Clone)]
#[must_use = "targets do nothing until passed to a command"]
pub struct EntityTarget<A> {
    raw: Selector,
    _arity: PhantomData<A>,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::PlayerTarget",
    aliases = ["sand::cmd::PlayerTarget", "sand::prelude::cmd::PlayerTarget"],
    module = "sand::command",
    summary = "Player selector with statically modeled arity.",
    context = "Player selector with statically modeled arity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::PlayerTarget;",
)]
/// Player selector with statically modeled arity.
#[derive(Debug, Clone)]
#[must_use = "targets do nothing until passed to a command"]
pub struct PlayerTarget<A> {
    raw: Selector,
    _arity: PhantomData<A>,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::SingleEntity",
    aliases = ["sand::cmd::SingleEntity", "sand::prelude::SingleEntity", "sand::prelude::cmd::SingleEntity"],
    module = "sand::command",
    summary = "An entity target that resolves to at most one entity.",
    context = "An entity target that resolves to at most one entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::SingleEntity;",
)]
/// An entity target that resolves to at most one entity.
pub type SingleEntity = EntityTarget<One>;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::EntityTargets",
    aliases = ["sand::cmd::EntityTargets", "sand::prelude::EntityTargets", "sand::prelude::cmd::EntityTargets"],
    module = "sand::command",
    summary = "An entity target that may resolve to zero or more entities.",
    context = "An entity target that may resolve to zero or more entities. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::EntityTargets;",
)]
/// An entity target that may resolve to zero or more entities.
pub type EntityTargets = EntityTarget<Many>;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::SinglePlayer",
    aliases = ["sand::cmd::SinglePlayer", "sand::prelude::SinglePlayer", "sand::prelude::cmd::SinglePlayer"],
    module = "sand::command",
    summary = "A player target that resolves to at most one player.",
    context = "A player target that resolves to at most one player. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::SinglePlayer;",
)]
/// A player target that resolves to at most one player.
pub type SinglePlayer = PlayerTarget<One>;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::PlayerTargets",
    aliases = ["sand::cmd::PlayerTargets", "sand::prelude::PlayerTargets", "sand::prelude::cmd::PlayerTargets"],
    module = "sand::command",
    summary = "A player target that may resolve to zero or more players.",
    context = "A player target that may resolve to zero or more players. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::PlayerTargets;",
)]
/// A player target that may resolve to zero or more players.
pub type PlayerTargets = PlayerTarget<Many>;

impl<A> EntityTarget<A> {
    /// Access the underlying selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::selector",
        aliases = ["sand::cmd::EntityTarget::selector", "sand::cmd::EntityTargets::selector", "sand::cmd::SingleEntity::selector", "sand::command::EntityTargets::selector", "sand::command::SingleEntity::selector", "sand::prelude::EntityTargets::selector", "sand::prelude::SingleEntity::selector", "sand::prelude::cmd::EntityTarget::selector", "sand::prelude::cmd::EntityTargets::selector", "sand::prelude::cmd::SingleEntity::selector"],
        module = "sand::command",
        kind = "method",
        summary = "Access the underlying selector.",
        context = "Access the underlying selector. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `& Selector` value produced to acces the underlying selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: &sand::command::EntityTarget < A >)  {\n    let selector = entity_target_value.selector();\n}",
    )]
    pub fn selector(&self) -> &Selector {
        &self.raw
    }

    /// Convert this typed target into the underlying selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::into_selector",
        aliases = ["sand::cmd::EntityTarget::into_selector", "sand::cmd::EntityTargets::into_selector", "sand::cmd::SingleEntity::into_selector", "sand::command::EntityTargets::into_selector", "sand::command::SingleEntity::into_selector", "sand::prelude::EntityTargets::into_selector", "sand::prelude::SingleEntity::into_selector", "sand::prelude::cmd::EntityTarget::into_selector", "sand::prelude::cmd::EntityTargets::into_selector", "sand::prelude::cmd::SingleEntity::into_selector"],
        module = "sand::command",
        kind = "method",
        summary = "Convert this typed target into the underlying selector.",
        context = "Convert this typed target into the underlying selector. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `Selector` value produced to convert this typed target into the underlying selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >)  {\n    let into_selector = entity_target_value.into_selector();\n}",
    )]
    pub fn into_selector(self) -> Selector {
        self.raw
    }

    /// `tag=<tag>` — select only entities that have the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::tag",
        aliases = ["sand::cmd::EntityTarget::tag", "sand::cmd::EntityTargets::tag", "sand::cmd::SingleEntity::tag", "sand::command::EntityTargets::tag", "sand::command::SingleEntity::tag", "sand::prelude::EntityTargets::tag", "sand::prelude::SingleEntity::tag", "sand::prelude::cmd::EntityTarget::tag", "sand::prelude::cmd::EntityTargets::tag", "sand::prelude::cmd::SingleEntity::tag"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=<tag>` — select only entities that have the given tag.",
        context = "`tag=<tag>` — select only entities that have the given tag. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — select only entities that have the given tag form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `tag=<tag>` — select only entities that have the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, tag: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.tag(tag);\n}",
    )]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.tag(tag);
        self
    }

    /// `tag=!<tag>` — select only entities that do NOT have the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::not_tag",
        aliases = ["sand::cmd::EntityTarget::not_tag", "sand::cmd::EntityTargets::not_tag", "sand::cmd::SingleEntity::not_tag", "sand::command::EntityTargets::not_tag", "sand::command::SingleEntity::not_tag", "sand::prelude::EntityTargets::not_tag", "sand::prelude::SingleEntity::not_tag", "sand::prelude::cmd::EntityTarget::not_tag", "sand::prelude::cmd::EntityTargets::not_tag", "sand::prelude::cmd::SingleEntity::not_tag"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=!<tag>` — select only entities that do NOT have the given tag.",
        context = "`tag=!<tag>` — select only entities that do NOT have the given tag. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=!<tag>` — select only entities that do NOT have the given tag form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `tag=!<tag>` — select only entities that do NOT have the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, tag: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.not_tag(tag);\n}",
    )]
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.not_tag(tag);
        self
    }

    /// `type=<entity_type>` — select only entities of the given type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::entity_type",
        aliases = ["sand::cmd::EntityTarget::entity_type", "sand::cmd::EntityTargets::entity_type", "sand::cmd::SingleEntity::entity_type", "sand::command::EntityTargets::entity_type", "sand::command::SingleEntity::entity_type", "sand::prelude::EntityTargets::entity_type", "sand::prelude::SingleEntity::entity_type", "sand::prelude::cmd::EntityTarget::entity_type", "sand::prelude::cmd::EntityTargets::entity_type", "sand::prelude::cmd::SingleEntity::entity_type"],
        module = "sand::command",
        kind = "method",
        summary = "`type=<entity_type>` — select only entities of the given type.",
        context = "`type=<entity_type>` — select only entities of the given type. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(ty = "`ty` supplies the documented `type=<entity_type>` — select only entities of the given type form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `type=<entity_type>` — select only entities of the given type form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, ty: impl sand::command::IntoEntityType)  {\n    let updated_entity_target = entity_target_value.entity_type(ty);\n}",
    )]
    pub fn entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.raw = self.raw.entity_type(ty);
        self
    }

    /// `type=!<entity_type>` — select only entities NOT of the given type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::not_type",
        aliases = ["sand::cmd::EntityTarget::not_type", "sand::cmd::EntityTargets::not_type", "sand::cmd::SingleEntity::not_type", "sand::command::EntityTargets::not_type", "sand::command::SingleEntity::not_type", "sand::prelude::EntityTargets::not_type", "sand::prelude::SingleEntity::not_type", "sand::prelude::cmd::EntityTarget::not_type", "sand::prelude::cmd::EntityTargets::not_type", "sand::prelude::cmd::SingleEntity::not_type"],
        module = "sand::command",
        kind = "method",
        summary = "`type=!<entity_type>` — select only entities NOT of the given type.",
        context = "`type=!<entity_type>` — select only entities NOT of the given type. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(ty = "`ty` supplies the documented `type=!<entity_type>` — select only entities NOT of the given type form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `type=!<entity_type>` — select only entities NOT of the given type form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, ty: impl sand::command::IntoEntityType)  {\n    let updated_entity_target = entity_target_value.not_type(ty);\n}",
    )]
    pub fn not_type(mut self, ty: impl IntoEntityType) -> Self {
        self.raw = self.raw.not_type(ty);
        self
    }

    /// `type=!minecraft:player` — exclude players from the target set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::excluding_players",
        aliases = ["sand::cmd::EntityTarget::excluding_players", "sand::cmd::EntityTargets::excluding_players", "sand::cmd::SingleEntity::excluding_players", "sand::command::EntityTargets::excluding_players", "sand::command::SingleEntity::excluding_players", "sand::prelude::EntityTargets::excluding_players", "sand::prelude::SingleEntity::excluding_players", "sand::prelude::cmd::EntityTarget::excluding_players", "sand::prelude::cmd::EntityTargets::excluding_players", "sand::prelude::cmd::SingleEntity::excluding_players"],
        module = "sand::command",
        kind = "method",
        summary = "`type=!minecraft:player` — exclude players from the target set.",
        context = "`type=!minecraft:player` — exclude players from the target set. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `type=!minecraft:player` — exclude players from the target set form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >)  {\n    let updated_entity_target = entity_target_value.excluding_players();\n}",
    )]
    pub fn excluding_players(self) -> Self {
        self.not_type("minecraft:player")
    }

    /// Add one typed scoreboard filter without formatting a selector score map.
    ///
    /// ```
    /// use sand_commands::ObjectiveName;
    /// use sand_commands::selector::{EntityTargets, ScoreRange};
    ///
    /// let targets = EntityTargets::all()
    ///     .score(ObjectiveName::new("threat"), ScoreRange::at_least(5))
    ///     .unwrap();
    /// assert_eq!(targets.to_string(), "@e[scores={threat=5..}]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::score",
        aliases = ["sand::cmd::EntityTarget::score", "sand::cmd::EntityTargets::score", "sand::cmd::SingleEntity::score", "sand::command::EntityTargets::score", "sand::command::SingleEntity::score", "sand::prelude::EntityTargets::score", "sand::prelude::SingleEntity::score", "sand::prelude::cmd::EntityTarget::score", "sand::prelude::cmd::EntityTargets::score", "sand::prelude::cmd::SingleEntity::score"],
        module = "sand::command",
        kind = "method",
        summary = "Add one typed scoreboard filter without formatting a selector score map.",
        context = "Add one typed scoreboard filter without formatting a selector score map. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(objective = "`objective` provides the objective added when building one typed scoreboard filter without formatting a selector score map.", range = "`range` provides the accepted numeric range used to add one typed scoreboard filter without formatting a selector score map."),
        returns = "On success, the value produced to add one typed scoreboard filter without formatting a selector score map; otherwise, the documented validation or export diagnostic.",
        example = "use sand::command::ObjectiveName;\nuse {sand::command::EntityTargets, sand::command::ScoreRange};\nlet targets = EntityTargets::all()\n.score(ObjectiveName::new(\"threat\"), ScoreRange::at_least(5))\n.unwrap();\nassert_eq!(targets.to_string(), \"@e[scores={threat=5..}]\");",
    )]
    pub fn score(
        mut self,
        objective: crate::ObjectiveName,
        range: ScoreRange,
    ) -> CommandResult<Self> {
        self.raw = self.raw.score_typed(objective, range)?;
        Ok(self)
    }

    /// `distance=0.1..` — exclude the current executor when centered at `@s`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::excluding_self",
        aliases = ["sand::cmd::EntityTarget::excluding_self", "sand::cmd::EntityTargets::excluding_self", "sand::cmd::SingleEntity::excluding_self", "sand::command::EntityTargets::excluding_self", "sand::command::SingleEntity::excluding_self", "sand::prelude::EntityTargets::excluding_self", "sand::prelude::SingleEntity::excluding_self", "sand::prelude::cmd::EntityTarget::excluding_self", "sand::prelude::cmd::EntityTargets::excluding_self", "sand::prelude::cmd::SingleEntity::excluding_self"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=0.1..` — exclude the current executor when centered at `@s`.",
        context = "`distance=0.1..` — exclude the current executor when centered at `@s`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `distance=0.1..` — exclude the current executor when centered at `@s` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >)  {\n    let updated_entity_target = entity_target_value.excluding_self();\n}",
    )]
    pub fn excluding_self(mut self) -> Self {
        self.raw = self.raw.exclude_self_distance();
        self
    }

    /// `distance=..<max>` — select targets within `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::within_blocks",
        aliases = ["sand::cmd::EntityTarget::within_blocks", "sand::cmd::EntityTargets::within_blocks", "sand::cmd::SingleEntity::within_blocks", "sand::command::EntityTargets::within_blocks", "sand::command::SingleEntity::within_blocks", "sand::prelude::EntityTargets::within_blocks", "sand::prelude::SingleEntity::within_blocks", "sand::prelude::cmd::EntityTarget::within_blocks", "sand::prelude::cmd::EntityTargets::within_blocks", "sand::prelude::cmd::SingleEntity::within_blocks"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=..<max>` — select targets within `max` blocks.",
        context = "`distance=..<max>` — select targets within `max` blocks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(max = "`distance=..<max>` — select targets within `max` blocks."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `distance=..<max>` — select targets within `max` blocks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, max: f64)  {\n    let updated_entity_target = entity_target_value.within_blocks(max);\n}",
    )]
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.raw = self.raw.distance_max(max);
        self
    }

    /// `distance=<range>` — select only entities within a distance range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::distance",
        aliases = ["sand::cmd::EntityTarget::distance", "sand::cmd::EntityTargets::distance", "sand::cmd::SingleEntity::distance", "sand::command::EntityTargets::distance", "sand::command::SingleEntity::distance", "sand::prelude::EntityTargets::distance", "sand::prelude::SingleEntity::distance", "sand::prelude::cmd::EntityTarget::distance", "sand::prelude::cmd::EntityTargets::distance", "sand::prelude::cmd::SingleEntity::distance"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<range>` — select only entities within a distance range.",
        context = "`distance=<range>` — select only entities within a distance range. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` supplies the documented `distance=<range>` — select only entities within a distance range form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `distance=<range>` — select only entities within a distance range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, range: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.distance(range);\n}",
    )]
    pub fn distance(mut self, range: impl Into<String>) -> Self {
        self.raw = self.raw.distance(range);
        self
    }

    /// `distance=<min>..<max>` — select only entities between `min` and `max`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::distance_range",
        aliases = ["sand::cmd::EntityTarget::distance_range", "sand::cmd::EntityTargets::distance_range", "sand::cmd::SingleEntity::distance_range", "sand::command::EntityTargets::distance_range", "sand::command::SingleEntity::distance_range", "sand::prelude::EntityTargets::distance_range", "sand::prelude::SingleEntity::distance_range", "sand::prelude::cmd::EntityTarget::distance_range", "sand::prelude::cmd::EntityTargets::distance_range", "sand::prelude::cmd::SingleEntity::distance_range"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<min>..<max>` — select only entities between `min` and `max`.",
        context = "`distance=<min>..<max>` — select only entities between `min` and `max`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`distance=<min>..<max>` — select only entities between `min` and `max`.", max = "`distance=<min>..<max>` — select only entities between `min` and `max`."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `distance=<min>..<max>` — select only entities between `min` and `max` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, min: f64, max: f64)  {\n    let updated_entity_target = entity_target_value.distance_range(min, max);\n}",
    )]
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.raw = self.raw.distance_range(min, max);
        self
    }

    /// `distance=<min>..` — select only entities at least `min` blocks away.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::distance_min",
        aliases = ["sand::cmd::EntityTarget::distance_min", "sand::cmd::EntityTargets::distance_min", "sand::cmd::SingleEntity::distance_min", "sand::command::EntityTargets::distance_min", "sand::command::SingleEntity::distance_min", "sand::prelude::EntityTargets::distance_min", "sand::prelude::SingleEntity::distance_min", "sand::prelude::cmd::EntityTarget::distance_min", "sand::prelude::cmd::EntityTargets::distance_min", "sand::prelude::cmd::SingleEntity::distance_min"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<min>..` — select only entities at least `min` blocks away.",
        context = "`distance=<min>..` — select only entities at least `min` blocks away. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`distance=<min>..` — select only entities at least `min` blocks away."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `distance=<min>..` — select only entities at least `min` blocks away form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, min: f64)  {\n    let updated_entity_target = entity_target_value.distance_min(min);\n}",
    )]
    pub fn distance_min(mut self, min: f64) -> Self {
        self.raw = self.raw.distance_min(min);
        self
    }

    /// `distance=<range>` — select only entities within a typed distance
    /// range, using [`SelectorRange`] instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{EntityTargets, SelectorRange};
    ///
    /// let targets = EntityTargets::all().distance_typed(SelectorRange::at_most(16.0));
    /// assert_eq!(targets.to_string(), "@e[distance=..16]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::distance_typed",
        aliases = ["sand::cmd::EntityTarget::distance_typed", "sand::cmd::EntityTargets::distance_typed", "sand::cmd::SingleEntity::distance_typed", "sand::command::EntityTargets::distance_typed", "sand::command::SingleEntity::distance_typed", "sand::prelude::EntityTargets::distance_typed", "sand::prelude::SingleEntity::distance_typed", "sand::prelude::cmd::EntityTarget::distance_typed", "sand::prelude::cmd::EntityTargets::distance_typed", "sand::prelude::cmd::SingleEntity::distance_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string.",
        context = "`distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` provides the Minecraft target selection used to emit the documented `distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string form.",
        example = "use sand::command::{EntityTargets, SelectorRange};\nlet targets = EntityTargets::all().distance_typed(SelectorRange::at_most(16.0));\nassert_eq!(targets.to_string(), \"@e[distance=..16]\");",
    )]
    pub fn distance_typed(mut self, range: SelectorRange) -> Self {
        self.raw = self.raw.distance_typed(range);
        self
    }

    /// `tag=<tag>` — select only entities with the given tag, using a typed
    /// [`EntityTag`] instead of a raw string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::tag_typed",
        aliases = ["sand::cmd::EntityTarget::tag_typed", "sand::cmd::EntityTargets::tag_typed", "sand::cmd::SingleEntity::tag_typed", "sand::command::EntityTargets::tag_typed", "sand::command::SingleEntity::tag_typed", "sand::prelude::EntityTargets::tag_typed", "sand::prelude::SingleEntity::tag_typed", "sand::prelude::cmd::EntityTarget::tag_typed", "sand::prelude::cmd::EntityTargets::tag_typed", "sand::prelude::cmd::SingleEntity::tag_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string.",
        context = "`tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, tag: sand::command::EntityTag)  {\n    let updated_entity_target = entity_target_value.tag_typed(tag);\n}",
    )]
    pub fn tag_typed(mut self, tag: EntityTag) -> Self {
        self.raw = self.raw.tag_typed(tag);
        self
    }

    /// `team=<team>` — select only entities on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::team",
        aliases = ["sand::cmd::EntityTarget::team", "sand::cmd::EntityTargets::team", "sand::cmd::SingleEntity::team", "sand::command::EntityTargets::team", "sand::command::SingleEntity::team", "sand::prelude::EntityTargets::team", "sand::prelude::SingleEntity::team", "sand::prelude::cmd::EntityTarget::team", "sand::prelude::cmd::EntityTargets::team", "sand::prelude::cmd::SingleEntity::team"],
        module = "sand::command",
        kind = "method",
        summary = "`team=<team>` — select only entities on the given team.",
        context = "`team=<team>` — select only entities on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=<team>` — select only entities on the given team form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `team=<team>` — select only entities on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, team: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.team(team);\n}",
    )]
    pub fn team(mut self, team: impl Into<String>) -> Self {
        self.raw = self.raw.team(team);
        self
    }

    /// `team=!<team>` — select only entities NOT on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::not_team",
        aliases = ["sand::cmd::EntityTarget::not_team", "sand::cmd::EntityTargets::not_team", "sand::cmd::SingleEntity::not_team", "sand::command::EntityTargets::not_team", "sand::command::SingleEntity::not_team", "sand::prelude::EntityTargets::not_team", "sand::prelude::SingleEntity::not_team", "sand::prelude::cmd::EntityTarget::not_team", "sand::prelude::cmd::EntityTargets::not_team", "sand::prelude::cmd::SingleEntity::not_team"],
        module = "sand::command",
        kind = "method",
        summary = "`team=!<team>` — select only entities NOT on the given team.",
        context = "`team=!<team>` — select only entities NOT on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=!<team>` — select only entities NOT on the given team form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `team=!<team>` — select only entities NOT on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, team: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.not_team(team);\n}",
    )]
    pub fn not_team(mut self, team: impl Into<String>) -> Self {
        self.raw = self.raw.not_team(team);
        self
    }

    /// `team=<team>` — select only entities on the given team, using a typed
    /// [`TeamName`] instead of a raw string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::team_typed",
        aliases = ["sand::cmd::EntityTarget::team_typed", "sand::cmd::EntityTargets::team_typed", "sand::cmd::SingleEntity::team_typed", "sand::command::EntityTargets::team_typed", "sand::command::SingleEntity::team_typed", "sand::prelude::EntityTargets::team_typed", "sand::prelude::SingleEntity::team_typed", "sand::prelude::cmd::EntityTarget::team_typed", "sand::prelude::cmd::EntityTargets::team_typed", "sand::prelude::cmd::SingleEntity::team_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string.",
        context = "`team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, team: sand::command::TeamName)  {\n    let updated_entity_target = entity_target_value.team_typed(team);\n}",
    )]
    pub fn team_typed(mut self, team: TeamName) -> Self {
        self.raw = self.raw.team_typed(team);
        self
    }

    /// `name=<name>` — select only entities with the exact display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::name",
        aliases = ["sand::cmd::EntityTarget::name", "sand::cmd::EntityTargets::name", "sand::cmd::SingleEntity::name", "sand::command::EntityTargets::name", "sand::command::SingleEntity::name", "sand::prelude::EntityTargets::name", "sand::prelude::SingleEntity::name", "sand::prelude::cmd::EntityTarget::name", "sand::prelude::cmd::EntityTargets::name", "sand::prelude::cmd::SingleEntity::name"],
        module = "sand::command",
        kind = "method",
        summary = "`name=<name>` — select only entities with the exact display name.",
        context = "`name=<name>` — select only entities with the exact display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` supplies the documented `name=<name>` — select only entities with the exact display name form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `name=<name>` — select only entities with the exact display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, name: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.name(name);\n}",
    )]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.raw = self.raw.name(name);
        self
    }

    /// `name=!<name>` — select only entities WITHOUT the given display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::not_name",
        aliases = ["sand::cmd::EntityTarget::not_name", "sand::cmd::EntityTargets::not_name", "sand::cmd::SingleEntity::not_name", "sand::command::EntityTargets::not_name", "sand::command::SingleEntity::not_name", "sand::prelude::EntityTargets::not_name", "sand::prelude::SingleEntity::not_name", "sand::prelude::cmd::EntityTarget::not_name", "sand::prelude::cmd::EntityTargets::not_name", "sand::prelude::cmd::SingleEntity::not_name"],
        module = "sand::command",
        kind = "method",
        summary = "`name=!<name>` — select only entities WITHOUT the given display name.",
        context = "`name=!<name>` — select only entities WITHOUT the given display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` supplies the documented `name=!<name>` — select only entities WITHOUT the given display name form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `name=!<name>` — select only entities WITHOUT the given display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, name: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.not_name(name);\n}",
    )]
    pub fn not_name(mut self, name: impl Into<String>) -> Self {
        self.raw = self.raw.not_name(name);
        self
    }

    /// `scores={<objective>=<range>,...}` — select only entities with
    /// matching scoreboard scores, built from typed [`SelectorScores`]
    /// entries instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{EntityTargets, ScoreRange, SelectorScores};
    ///
    /// let targets = EntityTargets::all().scores_typed(
    ///     SelectorScores::new()
    ///         .with("threat", ScoreRange::at_least(5))
    ///         .with("kills", ScoreRange::exact(0)),
    /// );
    /// assert_eq!(targets.to_string(), "@e[scores={threat=5..,kills=0}]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::scores_typed",
        aliases = ["sand::cmd::EntityTarget::scores_typed", "sand::cmd::EntityTargets::scores_typed", "sand::cmd::SingleEntity::scores_typed", "sand::command::EntityTargets::scores_typed", "sand::command::SingleEntity::scores_typed", "sand::prelude::EntityTargets::scores_typed", "sand::prelude::SingleEntity::scores_typed", "sand::prelude::cmd::EntityTarget::scores_typed", "sand::prelude::cmd::EntityTargets::scores_typed", "sand::prelude::cmd::SingleEntity::scores_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string.",
        context = "`scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(scores = "`scores` provides the Minecraft target selection used to emit the documented `scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string form.",
        example = "use sand::command::{EntityTargets, ScoreRange, SelectorScores};\nlet targets = EntityTargets::all().scores_typed(\nSelectorScores::new()\n.with(\"threat\", ScoreRange::at_least(5))\n.with(\"kills\", ScoreRange::exact(0)),\n);\nassert_eq!(targets.to_string(), \"@e[scores={threat=5..,kills=0}]\");",
    )]
    pub fn scores_typed(mut self, scores: SelectorScores) -> Self {
        self.raw = self.raw.scores_typed(scores);
        self
    }

    /// `predicate=<id>` — select only entities matching a loot table
    /// predicate, using a typed [`PredicateId`] instead of a raw string.
    ///
    /// ```
    /// use sand_commands::selector::{EntityTargets, PredicateId};
    ///
    /// let targets = EntityTargets::all().predicate_id(PredicateId::new("my_pack:is_burning"));
    /// assert_eq!(targets.to_string(), "@e[predicate=my_pack:is_burning]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::predicate_id",
        aliases = ["sand::cmd::EntityTarget::predicate_id", "sand::cmd::EntityTargets::predicate_id", "sand::cmd::SingleEntity::predicate_id", "sand::command::EntityTargets::predicate_id", "sand::command::SingleEntity::predicate_id", "sand::prelude::EntityTargets::predicate_id", "sand::prelude::SingleEntity::predicate_id", "sand::prelude::cmd::EntityTarget::predicate_id", "sand::prelude::cmd::EntityTargets::predicate_id", "sand::prelude::cmd::SingleEntity::predicate_id"],
        module = "sand::command",
        kind = "method",
        summary = "`predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string.",
        context = "`predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to emit the documented `predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string form.",
        example = "use {sand::command::EntityTargets, sand::predicate::PredicateId};\nlet targets = EntityTargets::all().predicate_id(PredicateId::new(\"my_pack:is_burning\"));\nassert_eq!(targets.to_string(), \"@e[predicate=my_pack:is_burning]\");",
    )]
    pub fn predicate_id(mut self, id: PredicateId) -> Self {
        self.raw = self.raw.predicate_id(id);
        self
    }

    /// `dx/dy/dz` — set a bounding box volume filter.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::volume",
        aliases = ["sand::cmd::EntityTarget::volume", "sand::cmd::EntityTargets::volume", "sand::cmd::SingleEntity::volume", "sand::command::EntityTargets::volume", "sand::command::SingleEntity::volume", "sand::prelude::EntityTargets::volume", "sand::prelude::SingleEntity::volume", "sand::prelude::cmd::EntityTarget::volume", "sand::prelude::cmd::EntityTargets::volume", "sand::prelude::cmd::SingleEntity::volume"],
        module = "sand::command",
        kind = "method",
        summary = "`dx/dy/dz` — set a bounding box volume filter.",
        context = "`dx/dy/dz` — set a bounding box volume filter. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(dx = "`dx` provides the x-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form.", dy = "`dy` provides the y-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form.", dz = "`dz` provides the z-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `dx/dy/dz` — set a bounding box volume filter form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, dx: f64, dy: f64, dz: f64)  {\n    let updated_entity_target = entity_target_value.volume(dx, dy, dz);\n}",
    )]
    pub fn volume(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.raw = self.raw.volume(dx, dy, dz);
        self
    }

    /// `x/y/z` — set the origin point for distance and volume checks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::at_pos",
        aliases = ["sand::cmd::EntityTarget::at_pos", "sand::cmd::EntityTargets::at_pos", "sand::cmd::SingleEntity::at_pos", "sand::command::EntityTargets::at_pos", "sand::command::SingleEntity::at_pos", "sand::prelude::EntityTargets::at_pos", "sand::prelude::SingleEntity::at_pos", "sand::prelude::cmd::EntityTarget::at_pos", "sand::prelude::cmd::EntityTargets::at_pos", "sand::prelude::cmd::SingleEntity::at_pos"],
        module = "sand::command",
        kind = "method",
        summary = "`x/y/z` — set the origin point for distance and volume checks.",
        context = "`x/y/z` — set the origin point for distance and volume checks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form.", y = "`y` provides the y-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form.", z = "`z` provides the z-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form."),
        returns = "The `EntityTarget` value with the documented change applied to emit the documented `x/y/z` — set the origin point for distance and volume checks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, x: f64, y: f64, z: f64)  {\n    let updated_entity_target = entity_target_value.at_pos(x, y, z);\n}",
    )]
    pub fn at_pos(mut self, x: f64, y: f64, z: f64) -> Self {
        self.raw = self.raw.at_pos(x, y, z);
        self
    }

    /// Explicit raw escape hatch for `scores=...` syntax.
    ///
    /// This opts out of Sand's typed score model: the fragment is passed
    /// through verbatim (e.g. `"kills=1..10,deaths=0"`) and only checked for
    /// shape at [`Selector::try_build`] time. Prefer
    /// [`EntityTarget::scores_typed`] in normal code; use this only for score
    /// syntax Sand cannot model yet. Delegates to [`Selector::scores_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::scores_raw",
        aliases = ["sand::cmd::EntityTarget::scores_raw", "sand::cmd::EntityTargets::scores_raw", "sand::cmd::SingleEntity::scores_raw", "sand::command::EntityTargets::scores_raw", "sand::command::SingleEntity::scores_raw", "sand::prelude::EntityTargets::scores_raw", "sand::prelude::SingleEntity::scores_raw", "sand::prelude::cmd::EntityTarget::scores_raw", "sand::prelude::cmd::EntityTargets::scores_raw", "sand::prelude::cmd::SingleEntity::scores_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `scores=...` syntax.",
        context = "Explicit raw escape hatch for `scores=...` syntax. This opts out of Sand's typed score model: the fragment is passed through verbatim (e.g. `\"kills=1..10,deaths=0\"`) and only checked for shape at [`Selector::try_build`] time. Prefer [`EntityTarget::scores_typed`] in normal code; use this only for score syntax Sand cannot model yet. Delegates to [`Selector::scores_raw`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(scores = "`scores` sets the scores for explicit raw escape hatch for `scores=...` syntax."),
        returns = "The `EntityTarget` value with the documented change applied to use explicit raw escape hatch for `scores=...` syntax.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, scores: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.scores_raw(scores);\n}",
    )]
    pub fn scores_raw(mut self, scores: impl Into<String>) -> Self {
        self.raw = self.raw.scores_raw(scores);
        self
    }

    /// Explicit raw escape hatch for `nbt=...` syntax.
    ///
    /// This crate has no typed SNBT representation yet, so this remains the
    /// normal path for NBT filters — the compound is passed through verbatim
    /// and only balance-checked at [`Selector::try_build`] time. Delegates to
    /// [`Selector::nbt_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::nbt_raw",
        aliases = ["sand::cmd::EntityTarget::nbt_raw", "sand::cmd::EntityTargets::nbt_raw", "sand::cmd::SingleEntity::nbt_raw", "sand::command::EntityTargets::nbt_raw", "sand::command::SingleEntity::nbt_raw", "sand::prelude::EntityTargets::nbt_raw", "sand::prelude::SingleEntity::nbt_raw", "sand::prelude::cmd::EntityTarget::nbt_raw", "sand::prelude::cmd::EntityTargets::nbt_raw", "sand::prelude::cmd::SingleEntity::nbt_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`].",
        context = "Explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`].",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(nbt = "`nbt` provides the NBT payload used to use explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`]."),
        returns = "The `EntityTarget` value with the documented change applied to use explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`].",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, nbt: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.nbt_raw(nbt);\n}",
    )]
    pub fn nbt_raw(mut self, nbt: impl Into<String>) -> Self {
        self.raw = self.raw.nbt_raw(nbt);
        self
    }

    /// Explicit raw escape hatch for `predicate=...` syntax.
    ///
    /// This opts out of the typed [`PredicateId`] wrapper: the string is
    /// passed through verbatim and only resource-location-shape checked at
    /// [`Selector::try_build`] time. Prefer
    /// [`EntityTarget::predicate_id`] in normal code. Delegates to
    /// [`Selector::predicate_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::predicate_raw",
        aliases = ["sand::cmd::EntityTarget::predicate_raw", "sand::cmd::EntityTargets::predicate_raw", "sand::cmd::SingleEntity::predicate_raw", "sand::command::EntityTargets::predicate_raw", "sand::command::SingleEntity::predicate_raw", "sand::prelude::EntityTargets::predicate_raw", "sand::prelude::SingleEntity::predicate_raw", "sand::prelude::cmd::EntityTarget::predicate_raw", "sand::prelude::cmd::EntityTargets::predicate_raw", "sand::prelude::cmd::SingleEntity::predicate_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `predicate=...` syntax.",
        context = "Explicit raw escape hatch for `predicate=...` syntax. This opts out of the typed [`PredicateId`] wrapper: the string is passed through verbatim and only resource-location-shape checked at [`Selector::try_build`] time. Prefer [`EntityTarget::predicate_id`] in normal code. Delegates to [`Selector::predicate_raw`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(predicate = "`predicate` provides the predicate that must match used to use explicit raw escape hatch for `predicate=...` syntax."),
        returns = "The `EntityTarget` value with the documented change applied to use explicit raw escape hatch for `predicate=...` syntax.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(entity_target_value: sand::command::EntityTarget < A >, predicate: impl Into < String >)  {\n    let updated_entity_target = entity_target_value.predicate_raw(predicate);\n}",
    )]
    pub fn predicate_raw(mut self, predicate: impl Into<String>) -> Self {
        self.raw = self.raw.predicate_raw(predicate);
        self
    }
}

impl<A> Validate for EntityTarget<A> {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.raw.validate(profile)
    }
}

impl<A> RenderCommand for EntityTarget<A> {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

impl EntityTargets {
    /// `@e` — all entities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::all",
        aliases = ["sand::cmd::EntityTarget::all", "sand::cmd::EntityTargets::all", "sand::cmd::SingleEntity::all", "sand::command::EntityTargets::all", "sand::command::SingleEntity::all", "sand::prelude::EntityTargets::all", "sand::prelude::SingleEntity::all", "sand::prelude::cmd::EntityTarget::all", "sand::prelude::cmd::EntityTargets::all", "sand::prelude::cmd::SingleEntity::all"],
        module = "sand::command",
        kind = "method",
        summary = "`@e` — all entities.",
        context = "`@e` — all entities. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "An `EntityTarget` that emits the documented `@e` — all entities form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let entity_target = sand::command::EntityTargets::all();\n}",
    )]
    pub fn all() -> Self {
        Self::from_selector(Selector::all_entities())
    }

    /// `@e[distance=..<radius>]` — all entities within a radius of the executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::nearby",
        aliases = ["sand::cmd::EntityTarget::nearby", "sand::cmd::EntityTargets::nearby", "sand::cmd::SingleEntity::nearby", "sand::command::EntityTargets::nearby", "sand::command::SingleEntity::nearby", "sand::prelude::EntityTargets::nearby", "sand::prelude::SingleEntity::nearby", "sand::prelude::cmd::EntityTarget::nearby", "sand::prelude::cmd::EntityTargets::nearby", "sand::prelude::cmd::SingleEntity::nearby"],
        module = "sand::command",
        kind = "method",
        summary = "`@e[distance=..<radius>]` — all entities within a radius of the executor.",
        context = "`@e[distance=..<radius>]` — all entities within a radius of the executor. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(radius = "`radius` supplies the documented `@e[distance=..<radius>]` — all entities within a radius of the executor form."),
        returns = "An `EntityTarget` that emits the documented `@e[distance=..<radius>]` — all entities within a radius of the executor form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(radius: f64)  {\n    let entity_target = sand::command::EntityTargets::nearby(radius);\n}",
    )]
    pub fn nearby(radius: f64) -> Self {
        Self::all().within_blocks(radius)
    }

    /// Add `limit=1` and convert to a single-entity target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::limit",
        aliases = ["sand::cmd::EntityTarget::limit", "sand::cmd::EntityTargets::limit", "sand::cmd::SingleEntity::limit", "sand::command::EntityTargets::limit", "sand::command::SingleEntity::limit", "sand::prelude::EntityTargets::limit", "sand::prelude::SingleEntity::limit", "sand::prelude::cmd::EntityTarget::limit", "sand::prelude::cmd::EntityTargets::limit", "sand::prelude::cmd::SingleEntity::limit"],
        module = "sand::command",
        kind = "method",
        summary = "Add `limit=1` and convert to a single-entity target.",
        context = "Add `limit=1` and convert to a single-entity target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`n` provides the n added when building `limit=1` and convert to a single-entity target."),
        returns = "On success, the value produced to add `limit=1` and convert to a single-entity target; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_target_value: sand::command::EntityTargets, n: i32)  {\n    let limit = entity_target_value.limit(n);\n}",
    )]
    pub fn limit(mut self, n: i32) -> CommandResult<SingleEntity> {
        if n != 1 {
            return Err(CommandError::new(
                "EntityTargets::limit",
                "limit",
                format!("single-entity narrowing requires `limit=1`, got `{n}`"),
            ));
        }
        self.raw = self.raw.limit(n);
        Ok(SingleEntity::from_selector(self.raw))
    }

    /// Pick the nearest matching entity as a single target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::nearest",
        aliases = ["sand::cmd::EntityTarget::nearest", "sand::cmd::EntityTargets::nearest", "sand::cmd::SingleEntity::nearest", "sand::command::EntityTargets::nearest", "sand::command::SingleEntity::nearest", "sand::prelude::EntityTargets::nearest", "sand::prelude::SingleEntity::nearest", "sand::prelude::cmd::EntityTarget::nearest", "sand::prelude::cmd::EntityTargets::nearest", "sand::prelude::cmd::SingleEntity::nearest"],
        module = "sand::command",
        kind = "method",
        summary = "Pick the nearest matching entity as a single target.",
        context = "Pick the nearest matching entity as a single target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `SingleEntity` value produced to pick the nearest matching entity as a single target.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_target_value: sand::command::EntityTargets)  {\n    let nearest = entity_target_value.nearest();\n}",
    )]
    pub fn nearest(mut self) -> SingleEntity {
        self.raw = self.raw.sort(SortOrder::Nearest).limit(1);
        SingleEntity::from_selector(self.raw)
    }
}

impl SingleEntity {
    /// `@s` — the current executor as a single entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::self_",
        aliases = ["sand::cmd::EntityTarget::self_", "sand::cmd::EntityTargets::self_", "sand::cmd::SingleEntity::self_", "sand::command::EntityTargets::self_", "sand::command::SingleEntity::self_", "sand::prelude::EntityTargets::self_", "sand::prelude::SingleEntity::self_", "sand::prelude::cmd::EntityTarget::self_", "sand::prelude::cmd::EntityTargets::self_", "sand::prelude::cmd::SingleEntity::self_"],
        module = "sand::command",
        kind = "method",
        summary = "`@s` — the current executor as a single entity.",
        context = "`@s` — the current executor as a single entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "An `EntityTarget` that emits the documented `@s` — the current executor as a single entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let entity_target = sand::command::SingleEntity::self_();\n}",
    )]
    pub fn self_() -> Self {
        Self::from_selector(Selector::self_())
    }

    /// Explicit unchecked single-entity selector syntax.
    ///
    /// This opts out of Sand's cardinality proof. Use only when advanced or
    /// modded syntax guarantees zero or one result.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTarget::raw",
        aliases = ["sand::cmd::EntityTarget::raw", "sand::cmd::EntityTargets::raw", "sand::cmd::SingleEntity::raw", "sand::command::EntityTargets::raw", "sand::command::SingleEntity::raw", "sand::prelude::EntityTargets::raw", "sand::prelude::SingleEntity::raw", "sand::prelude::cmd::EntityTarget::raw", "sand::prelude::cmd::EntityTargets::raw", "sand::prelude::cmd::SingleEntity::raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit unchecked single-entity selector syntax.",
        context = "Explicit unchecked single-entity selector syntax. This opts out of Sand's cardinality proof. Use only when advanced or modded syntax guarantees zero or one result.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to use explicit unchecked single-entity selector syntax."),
        returns = "An `EntityTarget` configured for explicit unchecked single-entity selector syntax.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: impl Into < String >)  {\n    let entity_target = sand::command::SingleEntity::raw(selector);\n}",
    )]
    pub fn raw(selector: impl Into<String>) -> Self {
        Self::from_selector(Selector::raw(selector))
    }
}

impl<A> PlayerTarget<A> {
    /// Access the underlying selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::selector",
        aliases = ["sand::cmd::PlayerTarget::selector", "sand::cmd::PlayerTargets::selector", "sand::cmd::SinglePlayer::selector", "sand::command::PlayerTargets::selector", "sand::command::SinglePlayer::selector", "sand::prelude::PlayerTargets::selector", "sand::prelude::SinglePlayer::selector", "sand::prelude::cmd::PlayerTarget::selector", "sand::prelude::cmd::PlayerTargets::selector", "sand::prelude::cmd::SinglePlayer::selector"],
        module = "sand::command",
        kind = "method",
        summary = "Access the underlying selector.",
        context = "Access the underlying selector. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `& Selector` value produced to acces the underlying selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: &sand::command::PlayerTarget < A >)  {\n    let selector = player_target_value.selector();\n}",
    )]
    pub fn selector(&self) -> &Selector {
        &self.raw
    }

    /// Convert this typed target into the underlying selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::into_selector",
        aliases = ["sand::cmd::PlayerTarget::into_selector", "sand::cmd::PlayerTargets::into_selector", "sand::cmd::SinglePlayer::into_selector", "sand::command::PlayerTargets::into_selector", "sand::command::SinglePlayer::into_selector", "sand::prelude::PlayerTargets::into_selector", "sand::prelude::SinglePlayer::into_selector", "sand::prelude::cmd::PlayerTarget::into_selector", "sand::prelude::cmd::PlayerTargets::into_selector", "sand::prelude::cmd::SinglePlayer::into_selector"],
        module = "sand::command",
        kind = "method",
        summary = "Convert this typed target into the underlying selector.",
        context = "Convert this typed target into the underlying selector. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `Selector` value produced to convert this typed target into the underlying selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >)  {\n    let into_selector = player_target_value.into_selector();\n}",
    )]
    pub fn into_selector(self) -> Selector {
        self.raw
    }

    /// `tag=<tag>` — select only players that have the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::tag",
        aliases = ["sand::cmd::PlayerTarget::tag", "sand::cmd::PlayerTargets::tag", "sand::cmd::SinglePlayer::tag", "sand::command::PlayerTargets::tag", "sand::command::SinglePlayer::tag", "sand::prelude::PlayerTargets::tag", "sand::prelude::SinglePlayer::tag", "sand::prelude::cmd::PlayerTarget::tag", "sand::prelude::cmd::PlayerTargets::tag", "sand::prelude::cmd::SinglePlayer::tag"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=<tag>` — select only players that have the given tag.",
        context = "`tag=<tag>` — select only players that have the given tag. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — select only players that have the given tag form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `tag=<tag>` — select only players that have the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, tag: impl Into < String >)  {\n    let updated_player_target = player_target_value.tag(tag);\n}",
    )]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.tag(tag);
        self
    }

    /// `tag=!<tag>` — select only players that do NOT have the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::not_tag",
        aliases = ["sand::cmd::PlayerTarget::not_tag", "sand::cmd::PlayerTargets::not_tag", "sand::cmd::SinglePlayer::not_tag", "sand::command::PlayerTargets::not_tag", "sand::command::SinglePlayer::not_tag", "sand::prelude::PlayerTargets::not_tag", "sand::prelude::SinglePlayer::not_tag", "sand::prelude::cmd::PlayerTarget::not_tag", "sand::prelude::cmd::PlayerTargets::not_tag", "sand::prelude::cmd::SinglePlayer::not_tag"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=!<tag>` — select only players that do NOT have the given tag.",
        context = "`tag=!<tag>` — select only players that do NOT have the given tag. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=!<tag>` — select only players that do NOT have the given tag form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `tag=!<tag>` — select only players that do NOT have the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, tag: impl Into < String >)  {\n    let updated_player_target = player_target_value.not_tag(tag);\n}",
    )]
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.raw = self.raw.not_tag(tag);
        self
    }

    /// `distance=..<max>` — select players within `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::within_blocks",
        aliases = ["sand::cmd::PlayerTarget::within_blocks", "sand::cmd::PlayerTargets::within_blocks", "sand::cmd::SinglePlayer::within_blocks", "sand::command::PlayerTargets::within_blocks", "sand::command::SinglePlayer::within_blocks", "sand::prelude::PlayerTargets::within_blocks", "sand::prelude::SinglePlayer::within_blocks", "sand::prelude::cmd::PlayerTarget::within_blocks", "sand::prelude::cmd::PlayerTargets::within_blocks", "sand::prelude::cmd::SinglePlayer::within_blocks"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=..<max>` — select players within `max` blocks.",
        context = "`distance=..<max>` — select players within `max` blocks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(max = "`distance=..<max>` — select players within `max` blocks."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `distance=..<max>` — select players within `max` blocks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, max: f64)  {\n    let updated_player_target = player_target_value.within_blocks(max);\n}",
    )]
    pub fn within_blocks(mut self, max: f64) -> Self {
        self.raw = self.raw.distance_max(max);
        self
    }

    /// `distance=<min>..<max>` — select only players between `min` and `max`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::distance_range",
        aliases = ["sand::cmd::PlayerTarget::distance_range", "sand::cmd::PlayerTargets::distance_range", "sand::cmd::SinglePlayer::distance_range", "sand::command::PlayerTargets::distance_range", "sand::command::SinglePlayer::distance_range", "sand::prelude::PlayerTargets::distance_range", "sand::prelude::SinglePlayer::distance_range", "sand::prelude::cmd::PlayerTarget::distance_range", "sand::prelude::cmd::PlayerTargets::distance_range", "sand::prelude::cmd::SinglePlayer::distance_range"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<min>..<max>` — select only players between `min` and `max`.",
        context = "`distance=<min>..<max>` — select only players between `min` and `max`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`distance=<min>..<max>` — select only players between `min` and `max`.", max = "`distance=<min>..<max>` — select only players between `min` and `max`."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `distance=<min>..<max>` — select only players between `min` and `max` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, min: f64, max: f64)  {\n    let updated_player_target = player_target_value.distance_range(min, max);\n}",
    )]
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.raw = self.raw.distance_range(min, max);
        self
    }

    /// `distance=<min>..` — select only players at least `min` blocks away.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::distance_min",
        aliases = ["sand::cmd::PlayerTarget::distance_min", "sand::cmd::PlayerTargets::distance_min", "sand::cmd::SinglePlayer::distance_min", "sand::command::PlayerTargets::distance_min", "sand::command::SinglePlayer::distance_min", "sand::prelude::PlayerTargets::distance_min", "sand::prelude::SinglePlayer::distance_min", "sand::prelude::cmd::PlayerTarget::distance_min", "sand::prelude::cmd::PlayerTargets::distance_min", "sand::prelude::cmd::SinglePlayer::distance_min"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<min>..` — select only players at least `min` blocks away.",
        context = "`distance=<min>..` — select only players at least `min` blocks away. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`distance=<min>..` — select only players at least `min` blocks away."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `distance=<min>..` — select only players at least `min` blocks away form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, min: f64)  {\n    let updated_player_target = player_target_value.distance_min(min);\n}",
    )]
    pub fn distance_min(mut self, min: f64) -> Self {
        self.raw = self.raw.distance_min(min);
        self
    }

    /// `distance=<range>` — select only players within a distance range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::distance",
        aliases = ["sand::cmd::PlayerTarget::distance", "sand::cmd::PlayerTargets::distance", "sand::cmd::SinglePlayer::distance", "sand::command::PlayerTargets::distance", "sand::command::SinglePlayer::distance", "sand::prelude::PlayerTargets::distance", "sand::prelude::SinglePlayer::distance", "sand::prelude::cmd::PlayerTarget::distance", "sand::prelude::cmd::PlayerTargets::distance", "sand::prelude::cmd::SinglePlayer::distance"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<range>` — select only players within a distance range.",
        context = "`distance=<range>` — select only players within a distance range. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` supplies the documented `distance=<range>` — select only players within a distance range form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `distance=<range>` — select only players within a distance range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, range: impl Into < String >)  {\n    let updated_player_target = player_target_value.distance(range);\n}",
    )]
    pub fn distance(mut self, range: impl Into<String>) -> Self {
        self.raw = self.raw.distance(range);
        self
    }

    /// `distance=<range>` — select only players within a typed distance
    /// range, using [`SelectorRange`] instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{PlayerTargets, SelectorRange};
    ///
    /// let targets = PlayerTargets::all().distance_typed(SelectorRange::between(0.5, 10.0));
    /// assert_eq!(targets.to_string(), "@a[distance=0.5..10]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::distance_typed",
        aliases = ["sand::cmd::PlayerTarget::distance_typed", "sand::cmd::PlayerTargets::distance_typed", "sand::cmd::SinglePlayer::distance_typed", "sand::command::PlayerTargets::distance_typed", "sand::command::SinglePlayer::distance_typed", "sand::prelude::PlayerTargets::distance_typed", "sand::prelude::SinglePlayer::distance_typed", "sand::prelude::cmd::PlayerTarget::distance_typed", "sand::prelude::cmd::PlayerTargets::distance_typed", "sand::prelude::cmd::SinglePlayer::distance_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<range>` — select only players within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string.",
        context = "`distance=<range>` — select only players within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` provides the Minecraft target selection used to emit the documented `distance=<range>` — select only players within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `distance=<range>` — select only players within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string form.",
        example = "use sand::command::{PlayerTargets, SelectorRange};\nlet targets = PlayerTargets::all().distance_typed(SelectorRange::between(0.5, 10.0));\nassert_eq!(targets.to_string(), \"@a[distance=0.5..10]\");",
    )]
    pub fn distance_typed(mut self, range: SelectorRange) -> Self {
        self.raw = self.raw.distance_typed(range);
        self
    }

    /// `distance=0.1..` — exclude the current executor when centered at `@s`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::excluding_self",
        aliases = ["sand::cmd::PlayerTarget::excluding_self", "sand::cmd::PlayerTargets::excluding_self", "sand::cmd::SinglePlayer::excluding_self", "sand::command::PlayerTargets::excluding_self", "sand::command::SinglePlayer::excluding_self", "sand::prelude::PlayerTargets::excluding_self", "sand::prelude::SinglePlayer::excluding_self", "sand::prelude::cmd::PlayerTarget::excluding_self", "sand::prelude::cmd::PlayerTargets::excluding_self", "sand::prelude::cmd::SinglePlayer::excluding_self"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=0.1..` — exclude the current executor when centered at `@s`.",
        context = "`distance=0.1..` — exclude the current executor when centered at `@s`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `distance=0.1..` — exclude the current executor when centered at `@s` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >)  {\n    let updated_player_target = player_target_value.excluding_self();\n}",
    )]
    pub fn excluding_self(mut self) -> Self {
        self.raw = self.raw.exclude_self_distance();
        self
    }

    /// `tag=<tag>` — select only players with the given tag, using a typed
    /// [`EntityTag`] instead of a raw string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::tag_typed",
        aliases = ["sand::cmd::PlayerTarget::tag_typed", "sand::cmd::PlayerTargets::tag_typed", "sand::cmd::SinglePlayer::tag_typed", "sand::command::PlayerTargets::tag_typed", "sand::command::SinglePlayer::tag_typed", "sand::prelude::PlayerTargets::tag_typed", "sand::prelude::SinglePlayer::tag_typed", "sand::prelude::cmd::PlayerTarget::tag_typed", "sand::prelude::cmd::PlayerTargets::tag_typed", "sand::prelude::cmd::SinglePlayer::tag_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=<tag>` — select only players with the given tag, using a typed [`EntityTag`] instead of a raw string.",
        context = "`tag=<tag>` — select only players with the given tag, using a typed [`EntityTag`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — select only players with the given tag, using a typed [`EntityTag`] instead of a raw string form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `tag=<tag>` — select only players with the given tag, using a typed [`EntityTag`] instead of a raw string form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, tag: sand::command::EntityTag)  {\n    let updated_player_target = player_target_value.tag_typed(tag);\n}",
    )]
    pub fn tag_typed(mut self, tag: EntityTag) -> Self {
        self.raw = self.raw.tag_typed(tag);
        self
    }

    /// `team=<team>` — select only players on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::team",
        aliases = ["sand::cmd::PlayerTarget::team", "sand::cmd::PlayerTargets::team", "sand::cmd::SinglePlayer::team", "sand::command::PlayerTargets::team", "sand::command::SinglePlayer::team", "sand::prelude::PlayerTargets::team", "sand::prelude::SinglePlayer::team", "sand::prelude::cmd::PlayerTarget::team", "sand::prelude::cmd::PlayerTargets::team", "sand::prelude::cmd::SinglePlayer::team"],
        module = "sand::command",
        kind = "method",
        summary = "`team=<team>` — select only players on the given team.",
        context = "`team=<team>` — select only players on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=<team>` — select only players on the given team form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `team=<team>` — select only players on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, team: impl Into < String >)  {\n    let updated_player_target = player_target_value.team(team);\n}",
    )]
    pub fn team(mut self, team: impl Into<String>) -> Self {
        self.raw = self.raw.team(team);
        self
    }

    /// `team=!<team>` — select only players NOT on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::not_team",
        aliases = ["sand::cmd::PlayerTarget::not_team", "sand::cmd::PlayerTargets::not_team", "sand::cmd::SinglePlayer::not_team", "sand::command::PlayerTargets::not_team", "sand::command::SinglePlayer::not_team", "sand::prelude::PlayerTargets::not_team", "sand::prelude::SinglePlayer::not_team", "sand::prelude::cmd::PlayerTarget::not_team", "sand::prelude::cmd::PlayerTargets::not_team", "sand::prelude::cmd::SinglePlayer::not_team"],
        module = "sand::command",
        kind = "method",
        summary = "`team=!<team>` — select only players NOT on the given team.",
        context = "`team=!<team>` — select only players NOT on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=!<team>` — select only players NOT on the given team form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `team=!<team>` — select only players NOT on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, team: impl Into < String >)  {\n    let updated_player_target = player_target_value.not_team(team);\n}",
    )]
    pub fn not_team(mut self, team: impl Into<String>) -> Self {
        self.raw = self.raw.not_team(team);
        self
    }

    /// `team=<team>` — select only players on the given team, using a typed
    /// [`TeamName`] instead of a raw string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::team_typed",
        aliases = ["sand::cmd::PlayerTarget::team_typed", "sand::cmd::PlayerTargets::team_typed", "sand::cmd::SinglePlayer::team_typed", "sand::command::PlayerTargets::team_typed", "sand::command::SinglePlayer::team_typed", "sand::prelude::PlayerTargets::team_typed", "sand::prelude::SinglePlayer::team_typed", "sand::prelude::cmd::PlayerTarget::team_typed", "sand::prelude::cmd::PlayerTargets::team_typed", "sand::prelude::cmd::SinglePlayer::team_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`team=<team>` — select only players on the given team, using a typed [`TeamName`] instead of a raw string.",
        context = "`team=<team>` — select only players on the given team, using a typed [`TeamName`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=<team>` — select only players on the given team, using a typed [`TeamName`] instead of a raw string form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `team=<team>` — select only players on the given team, using a typed [`TeamName`] instead of a raw string form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, team: sand::command::TeamName)  {\n    let updated_player_target = player_target_value.team_typed(team);\n}",
    )]
    pub fn team_typed(mut self, team: TeamName) -> Self {
        self.raw = self.raw.team_typed(team);
        self
    }

    /// `name=<name>` — select only players with the exact display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::name",
        aliases = ["sand::cmd::PlayerTarget::name", "sand::cmd::PlayerTargets::name", "sand::cmd::SinglePlayer::name", "sand::command::PlayerTargets::name", "sand::command::SinglePlayer::name", "sand::prelude::PlayerTargets::name", "sand::prelude::SinglePlayer::name", "sand::prelude::cmd::PlayerTarget::name", "sand::prelude::cmd::PlayerTargets::name", "sand::prelude::cmd::SinglePlayer::name"],
        module = "sand::command",
        kind = "method",
        summary = "`name=<name>` — select only players with the exact display name.",
        context = "`name=<name>` — select only players with the exact display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` supplies the documented `name=<name>` — select only players with the exact display name form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `name=<name>` — select only players with the exact display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, name: impl Into < String >)  {\n    let updated_player_target = player_target_value.name(name);\n}",
    )]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.raw = self.raw.name(name);
        self
    }

    /// `name=!<name>` — select only players WITHOUT the given display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::not_name",
        aliases = ["sand::cmd::PlayerTarget::not_name", "sand::cmd::PlayerTargets::not_name", "sand::cmd::SinglePlayer::not_name", "sand::command::PlayerTargets::not_name", "sand::command::SinglePlayer::not_name", "sand::prelude::PlayerTargets::not_name", "sand::prelude::SinglePlayer::not_name", "sand::prelude::cmd::PlayerTarget::not_name", "sand::prelude::cmd::PlayerTargets::not_name", "sand::prelude::cmd::SinglePlayer::not_name"],
        module = "sand::command",
        kind = "method",
        summary = "`name=!<name>` — select only players WITHOUT the given display name.",
        context = "`name=!<name>` — select only players WITHOUT the given display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` supplies the documented `name=!<name>` — select only players WITHOUT the given display name form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `name=!<name>` — select only players WITHOUT the given display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, name: impl Into < String >)  {\n    let updated_player_target = player_target_value.not_name(name);\n}",
    )]
    pub fn not_name(mut self, name: impl Into<String>) -> Self {
        self.raw = self.raw.not_name(name);
        self
    }

    /// Add one typed scoreboard filter without formatting a selector score map.
    ///
    /// ```
    /// use sand_commands::ObjectiveName;
    /// use sand_commands::selector::{PlayerTargets, ScoreRange};
    ///
    /// let targets = PlayerTargets::all()
    ///     .score(ObjectiveName::new("kills"), ScoreRange::at_least(1))
    ///     .unwrap();
    /// assert_eq!(targets.to_string(), "@a[scores={kills=1..}]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::score",
        aliases = ["sand::cmd::PlayerTarget::score", "sand::cmd::PlayerTargets::score", "sand::cmd::SinglePlayer::score", "sand::command::PlayerTargets::score", "sand::command::SinglePlayer::score", "sand::prelude::PlayerTargets::score", "sand::prelude::SinglePlayer::score", "sand::prelude::cmd::PlayerTarget::score", "sand::prelude::cmd::PlayerTargets::score", "sand::prelude::cmd::SinglePlayer::score"],
        module = "sand::command",
        kind = "method",
        summary = "Add one typed scoreboard filter without formatting a selector score map.",
        context = "Add one typed scoreboard filter without formatting a selector score map. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(objective = "`objective` provides the objective added when building one typed scoreboard filter without formatting a selector score map.", range = "`range` provides the accepted numeric range used to add one typed scoreboard filter without formatting a selector score map."),
        returns = "On success, the value produced to add one typed scoreboard filter without formatting a selector score map; otherwise, the documented validation or export diagnostic.",
        example = "use sand::command::ObjectiveName;\nuse {sand::command::PlayerTargets, sand::command::ScoreRange};\nlet targets = PlayerTargets::all()\n.score(ObjectiveName::new(\"kills\"), ScoreRange::at_least(1))\n.unwrap();\nassert_eq!(targets.to_string(), \"@a[scores={kills=1..}]\");",
    )]
    pub fn score(
        mut self,
        objective: crate::ObjectiveName,
        range: ScoreRange,
    ) -> CommandResult<Self> {
        self.raw = self.raw.score_typed(objective, range)?;
        Ok(self)
    }

    /// `scores={<objective>=<range>,...}` — select only players with matching
    /// scoreboard scores, built from typed [`SelectorScores`] entries instead
    /// of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{PlayerTargets, ScoreRange, SelectorScores};
    ///
    /// let targets = PlayerTargets::all().scores_typed(
    ///     SelectorScores::new()
    ///         .with("kills", ScoreRange::between(1, 10))
    ///         .with("deaths", ScoreRange::exact(0)),
    /// );
    /// assert_eq!(targets.to_string(), "@a[scores={kills=1..10,deaths=0}]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::scores_typed",
        aliases = ["sand::cmd::PlayerTarget::scores_typed", "sand::cmd::PlayerTargets::scores_typed", "sand::cmd::SinglePlayer::scores_typed", "sand::command::PlayerTargets::scores_typed", "sand::command::SinglePlayer::scores_typed", "sand::prelude::PlayerTargets::scores_typed", "sand::prelude::SinglePlayer::scores_typed", "sand::prelude::cmd::PlayerTarget::scores_typed", "sand::prelude::cmd::PlayerTargets::scores_typed", "sand::prelude::cmd::SinglePlayer::scores_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`scores={<objective>=<range>,...}` — select only players with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string.",
        context = "`scores={<objective>=<range>,...}` — select only players with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(scores = "`scores` provides the Minecraft target selection used to emit the documented `scores={<objective>=<range>,...}` — select only players with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `scores={<objective>=<range>,...}` — select only players with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string form.",
        example = "use sand::command::{PlayerTargets, ScoreRange, SelectorScores};\nlet targets = PlayerTargets::all().scores_typed(\nSelectorScores::new()\n.with(\"kills\", ScoreRange::between(1, 10))\n.with(\"deaths\", ScoreRange::exact(0)),\n);\nassert_eq!(targets.to_string(), \"@a[scores={kills=1..10,deaths=0}]\");",
    )]
    pub fn scores_typed(mut self, scores: SelectorScores) -> Self {
        self.raw = self.raw.scores_typed(scores);
        self
    }

    /// `predicate=<id>` — select only players matching a loot table
    /// predicate, using a typed [`PredicateId`] instead of a raw string.
    ///
    /// ```
    /// use sand_commands::selector::{PlayerTargets, PredicateId};
    ///
    /// let targets = PlayerTargets::all().predicate_id(PredicateId::new("my_pack:is_sneaking"));
    /// assert_eq!(targets.to_string(), "@a[predicate=my_pack:is_sneaking]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::predicate_id",
        aliases = ["sand::cmd::PlayerTarget::predicate_id", "sand::cmd::PlayerTargets::predicate_id", "sand::cmd::SinglePlayer::predicate_id", "sand::command::PlayerTargets::predicate_id", "sand::command::SinglePlayer::predicate_id", "sand::prelude::PlayerTargets::predicate_id", "sand::prelude::SinglePlayer::predicate_id", "sand::prelude::cmd::PlayerTarget::predicate_id", "sand::prelude::cmd::PlayerTargets::predicate_id", "sand::prelude::cmd::SinglePlayer::predicate_id"],
        module = "sand::command",
        kind = "method",
        summary = "`predicate=<id>` — select only players matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string.",
        context = "`predicate=<id>` — select only players matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to emit the documented `predicate=<id>` — select only players matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `predicate=<id>` — select only players matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string form.",
        example = "use {sand::command::PlayerTargets, sand::predicate::PredicateId};\nlet targets = PlayerTargets::all().predicate_id(PredicateId::new(\"my_pack:is_sneaking\"));\nassert_eq!(targets.to_string(), \"@a[predicate=my_pack:is_sneaking]\");",
    )]
    pub fn predicate_id(mut self, id: PredicateId) -> Self {
        self.raw = self.raw.predicate_id(id);
        self
    }

    /// `level=<range>` — select only players within the given XP level range.
    ///
    /// Raw/compatibility: `range` is a hand-formatted string, validated at
    /// [`Selector::try_build`] time. Prefer [`PlayerTarget::level_typed`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::level",
        aliases = ["sand::cmd::PlayerTarget::level", "sand::cmd::PlayerTargets::level", "sand::cmd::SinglePlayer::level", "sand::command::PlayerTargets::level", "sand::command::SinglePlayer::level", "sand::prelude::PlayerTargets::level", "sand::prelude::SinglePlayer::level", "sand::prelude::cmd::PlayerTarget::level", "sand::prelude::cmd::PlayerTargets::level", "sand::prelude::cmd::SinglePlayer::level"],
        module = "sand::command",
        kind = "method",
        summary = "`level=<range>` — select only players within the given XP level range.",
        context = "`level=<range>` — select only players within the given XP level range. Raw/compatibility: `range` is a hand-formatted string, validated at [`Selector::try_build`] time. Prefer [`PlayerTarget::level_typed`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "Raw/compatibility: `range` is a hand-formatted string, validated at [`Selector::try_build`] time. Prefer [`PlayerTarget::level_typed`]."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `level=<range>` — select only players within the given XP level range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, range: impl Into < String >)  {\n    let updated_player_target = player_target_value.level(range);\n}",
    )]
    pub fn level(mut self, range: impl Into<String>) -> Self {
        self.raw = self.raw.level(range);
        self
    }

    /// `level=<range>` — select only players within a typed XP level range,
    /// using [`SelectorRange`] instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{PlayerTargets, SelectorRange};
    ///
    /// let targets = PlayerTargets::all().level_typed(SelectorRange::between(10.0, 30.0));
    /// assert_eq!(targets.to_string(), "@a[level=10..30]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::level_typed",
        aliases = ["sand::cmd::PlayerTarget::level_typed", "sand::cmd::PlayerTargets::level_typed", "sand::cmd::SinglePlayer::level_typed", "sand::command::PlayerTargets::level_typed", "sand::command::SinglePlayer::level_typed", "sand::prelude::PlayerTargets::level_typed", "sand::prelude::SinglePlayer::level_typed", "sand::prelude::cmd::PlayerTarget::level_typed", "sand::prelude::cmd::PlayerTargets::level_typed", "sand::prelude::cmd::SinglePlayer::level_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string.",
        context = "`level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` provides the Minecraft target selection used to emit the documented `level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string form.",
        example = "use sand::command::{PlayerTargets, SelectorRange};\nlet targets = PlayerTargets::all().level_typed(SelectorRange::between(10.0, 30.0));\nassert_eq!(targets.to_string(), \"@a[level=10..30]\");",
    )]
    pub fn level_typed(mut self, range: SelectorRange) -> Self {
        self.raw = self.raw.level_typed(range);
        self
    }

    /// `gamemode=<mode>` — select only players in the given gamemode.
    ///
    /// Raw/compatibility: `mode` is a string, validated against the vanilla
    /// gamemode set at [`Selector::try_build`] time rather than at the type
    /// level. Prefer [`PlayerTarget::gamemode_typed`] in normal code.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::gamemode",
        aliases = ["sand::cmd::PlayerTarget::gamemode", "sand::cmd::PlayerTargets::gamemode", "sand::cmd::SinglePlayer::gamemode", "sand::command::PlayerTargets::gamemode", "sand::command::SinglePlayer::gamemode", "sand::prelude::PlayerTargets::gamemode", "sand::prelude::SinglePlayer::gamemode", "sand::prelude::cmd::PlayerTarget::gamemode", "sand::prelude::cmd::PlayerTargets::gamemode", "sand::prelude::cmd::SinglePlayer::gamemode"],
        module = "sand::command",
        kind = "method",
        summary = "`gamemode=<mode>` — select only players in the given gamemode.",
        context = "`gamemode=<mode>` — select only players in the given gamemode. Raw/compatibility: `mode` is a string, validated against the vanilla gamemode set at [`Selector::try_build`] time rather than at the type level. Prefer [`PlayerTarget::gamemode_typed`] in normal code.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(mode = "Raw/compatibility: `mode` is a string, validated against the vanilla gamemode set at [`Selector::try_build`] time rather than at the type level. Prefer [`PlayerTarget::gamemode_typed`] in normal code."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `gamemode=<mode>` — select only players in the given gamemode form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, mode: impl Into < String >)  {\n    let updated_player_target = player_target_value.gamemode(mode);\n}",
    )]
    pub fn gamemode(mut self, mode: impl Into<String>) -> Self {
        self.raw = self.raw.gamemode(mode);
        self
    }

    /// `gamemode=<mode>` — select only players in the given gamemode, using
    /// the canonical typed [`GameMode`] enum instead of a validated string.
    ///
    /// ```
    /// use sand_commands::selector::{GameMode, PlayerTargets};
    ///
    /// let targets = PlayerTargets::all().gamemode_typed(GameMode::Adventure);
    /// assert_eq!(targets.to_string(), "@a[gamemode=adventure]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::gamemode_typed",
        aliases = ["sand::cmd::PlayerTarget::gamemode_typed", "sand::cmd::PlayerTargets::gamemode_typed", "sand::cmd::SinglePlayer::gamemode_typed", "sand::command::PlayerTargets::gamemode_typed", "sand::command::SinglePlayer::gamemode_typed", "sand::prelude::PlayerTargets::gamemode_typed", "sand::prelude::SinglePlayer::gamemode_typed", "sand::prelude::cmd::PlayerTarget::gamemode_typed", "sand::prelude::cmd::PlayerTargets::gamemode_typed", "sand::prelude::cmd::SinglePlayer::gamemode_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string.",
        context = "`gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(mode = "`mode` supplies the documented `gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string form.",
        example = "use {sand::command::GameMode, sand::command::PlayerTargets};\nlet targets = PlayerTargets::all().gamemode_typed(GameMode::Adventure);\nassert_eq!(targets.to_string(), \"@a[gamemode=adventure]\");",
    )]
    pub fn gamemode_typed(mut self, mode: GameMode) -> Self {
        self.raw = self.raw.gamemode_typed(mode);
        self
    }

    /// `gamemode=!<mode>` — exclude players in the given gamemode.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::not_gamemode_typed",
        aliases = ["sand::cmd::PlayerTarget::not_gamemode_typed", "sand::cmd::PlayerTargets::not_gamemode_typed", "sand::cmd::SinglePlayer::not_gamemode_typed", "sand::command::PlayerTargets::not_gamemode_typed", "sand::command::SinglePlayer::not_gamemode_typed", "sand::prelude::PlayerTargets::not_gamemode_typed", "sand::prelude::SinglePlayer::not_gamemode_typed", "sand::prelude::cmd::PlayerTarget::not_gamemode_typed", "sand::prelude::cmd::PlayerTargets::not_gamemode_typed", "sand::prelude::cmd::SinglePlayer::not_gamemode_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`gamemode=!<mode>` — exclude players in the given gamemode.",
        context = "`gamemode=!<mode>` — exclude players in the given gamemode. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(mode = "`mode` supplies the documented `gamemode=!<mode>` — exclude players in the given gamemode form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `gamemode=!<mode>` — exclude players in the given gamemode form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, mode: sand::command::GameMode)  {\n    let updated_player_target = player_target_value.not_gamemode_typed(mode);\n}",
    )]
    pub fn not_gamemode_typed(mut self, mode: GameMode) -> Self {
        self.raw = self.raw.not_gamemode_typed(mode);
        self
    }

    /// `dx/dy/dz` — set a bounding box volume filter.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::volume",
        aliases = ["sand::cmd::PlayerTarget::volume", "sand::cmd::PlayerTargets::volume", "sand::cmd::SinglePlayer::volume", "sand::command::PlayerTargets::volume", "sand::command::SinglePlayer::volume", "sand::prelude::PlayerTargets::volume", "sand::prelude::SinglePlayer::volume", "sand::prelude::cmd::PlayerTarget::volume", "sand::prelude::cmd::PlayerTargets::volume", "sand::prelude::cmd::SinglePlayer::volume"],
        module = "sand::command",
        kind = "method",
        summary = "`dx/dy/dz` — set a bounding box volume filter.",
        context = "`dx/dy/dz` — set a bounding box volume filter. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(dx = "`dx` provides the x-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form.", dy = "`dy` provides the y-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form.", dz = "`dz` provides the z-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `dx/dy/dz` — set a bounding box volume filter form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, dx: f64, dy: f64, dz: f64)  {\n    let updated_player_target = player_target_value.volume(dx, dy, dz);\n}",
    )]
    pub fn volume(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.raw = self.raw.volume(dx, dy, dz);
        self
    }

    /// `x/y/z` — set the origin point for distance and volume checks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::at_pos",
        aliases = ["sand::cmd::PlayerTarget::at_pos", "sand::cmd::PlayerTargets::at_pos", "sand::cmd::SinglePlayer::at_pos", "sand::command::PlayerTargets::at_pos", "sand::command::SinglePlayer::at_pos", "sand::prelude::PlayerTargets::at_pos", "sand::prelude::SinglePlayer::at_pos", "sand::prelude::cmd::PlayerTarget::at_pos", "sand::prelude::cmd::PlayerTargets::at_pos", "sand::prelude::cmd::SinglePlayer::at_pos"],
        module = "sand::command",
        kind = "method",
        summary = "`x/y/z` — set the origin point for distance and volume checks.",
        context = "`x/y/z` — set the origin point for distance and volume checks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form.", y = "`y` provides the y-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form.", z = "`z` provides the z-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form."),
        returns = "The `PlayerTarget` value with the documented change applied to emit the documented `x/y/z` — set the origin point for distance and volume checks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, x: f64, y: f64, z: f64)  {\n    let updated_player_target = player_target_value.at_pos(x, y, z);\n}",
    )]
    pub fn at_pos(mut self, x: f64, y: f64, z: f64) -> Self {
        self.raw = self.raw.at_pos(x, y, z);
        self
    }

    /// Explicit raw escape hatch for `scores=...` syntax.
    ///
    /// This opts out of Sand's typed score model: the fragment is passed
    /// through verbatim (e.g. `"kills=1..10,deaths=0"`) and only checked for
    /// shape at [`Selector::try_build`] time. Prefer
    /// [`PlayerTarget::scores_typed`] in normal code; use this only for score
    /// syntax Sand cannot model yet. Delegates to [`Selector::scores_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::scores_raw",
        aliases = ["sand::cmd::PlayerTarget::scores_raw", "sand::cmd::PlayerTargets::scores_raw", "sand::cmd::SinglePlayer::scores_raw", "sand::command::PlayerTargets::scores_raw", "sand::command::SinglePlayer::scores_raw", "sand::prelude::PlayerTargets::scores_raw", "sand::prelude::SinglePlayer::scores_raw", "sand::prelude::cmd::PlayerTarget::scores_raw", "sand::prelude::cmd::PlayerTargets::scores_raw", "sand::prelude::cmd::SinglePlayer::scores_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `scores=...` syntax.",
        context = "Explicit raw escape hatch for `scores=...` syntax. This opts out of Sand's typed score model: the fragment is passed through verbatim (e.g. `\"kills=1..10,deaths=0\"`) and only checked for shape at [`Selector::try_build`] time. Prefer [`PlayerTarget::scores_typed`] in normal code; use this only for score syntax Sand cannot model yet. Delegates to [`Selector::scores_raw`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(scores = "`scores` sets the scores for explicit raw escape hatch for `scores=...` syntax."),
        returns = "The `PlayerTarget` value with the documented change applied to use explicit raw escape hatch for `scores=...` syntax.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, scores: impl Into < String >)  {\n    let updated_player_target = player_target_value.scores_raw(scores);\n}",
    )]
    pub fn scores_raw(mut self, scores: impl Into<String>) -> Self {
        self.raw = self.raw.scores_raw(scores);
        self
    }

    /// Explicit raw escape hatch for `nbt=...` syntax.
    ///
    /// This crate has no typed SNBT representation yet, so this remains the
    /// normal path for NBT filters — the compound is passed through verbatim
    /// and only balance-checked at [`Selector::try_build`] time. Delegates to
    /// [`Selector::nbt_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::nbt_raw",
        aliases = ["sand::cmd::PlayerTarget::nbt_raw", "sand::cmd::PlayerTargets::nbt_raw", "sand::cmd::SinglePlayer::nbt_raw", "sand::command::PlayerTargets::nbt_raw", "sand::command::SinglePlayer::nbt_raw", "sand::prelude::PlayerTargets::nbt_raw", "sand::prelude::SinglePlayer::nbt_raw", "sand::prelude::cmd::PlayerTarget::nbt_raw", "sand::prelude::cmd::PlayerTargets::nbt_raw", "sand::prelude::cmd::SinglePlayer::nbt_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`].",
        context = "Explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`].",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(nbt = "`nbt` provides the NBT payload used to use explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`]."),
        returns = "The `PlayerTarget` value with the documented change applied to use explicit raw escape hatch for `nbt=...` syntax. This crate has no typed SNBT representation yet, so this remains the normal path for NBT filters — the compound is passed through verbatim and only balance-checked at [`Selector::try_build`] time. Delegates to [`Selector::nbt_raw`].",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, nbt: impl Into < String >)  {\n    let updated_player_target = player_target_value.nbt_raw(nbt);\n}",
    )]
    pub fn nbt_raw(mut self, nbt: impl Into<String>) -> Self {
        self.raw = self.raw.nbt_raw(nbt);
        self
    }

    /// Explicit raw escape hatch for `predicate=...` syntax.
    ///
    /// This opts out of the typed [`PredicateId`] wrapper: the string is
    /// passed through verbatim and only resource-location-shape checked at
    /// [`Selector::try_build`] time. Prefer [`PlayerTarget::predicate_id`] in
    /// normal code. Delegates to [`Selector::predicate_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::predicate_raw",
        aliases = ["sand::cmd::PlayerTarget::predicate_raw", "sand::cmd::PlayerTargets::predicate_raw", "sand::cmd::SinglePlayer::predicate_raw", "sand::command::PlayerTargets::predicate_raw", "sand::command::SinglePlayer::predicate_raw", "sand::prelude::PlayerTargets::predicate_raw", "sand::prelude::SinglePlayer::predicate_raw", "sand::prelude::cmd::PlayerTarget::predicate_raw", "sand::prelude::cmd::PlayerTargets::predicate_raw", "sand::prelude::cmd::SinglePlayer::predicate_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `predicate=...` syntax.",
        context = "Explicit raw escape hatch for `predicate=...` syntax. This opts out of the typed [`PredicateId`] wrapper: the string is passed through verbatim and only resource-location-shape checked at [`Selector::try_build`] time. Prefer [`PlayerTarget::predicate_id`] in normal code. Delegates to [`Selector::predicate_raw`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(predicate = "`predicate` provides the predicate that must match used to use explicit raw escape hatch for `predicate=...` syntax."),
        returns = "The `PlayerTarget` value with the documented change applied to use explicit raw escape hatch for `predicate=...` syntax.",
        example = "use sand::prelude::*;\n\nfn demonstrate<A: 'static>(player_target_value: sand::command::PlayerTarget < A >, predicate: impl Into < String >)  {\n    let updated_player_target = player_target_value.predicate_raw(predicate);\n}",
    )]
    pub fn predicate_raw(mut self, predicate: impl Into<String>) -> Self {
        self.raw = self.raw.predicate_raw(predicate);
        self
    }
}

impl<A> Validate for PlayerTarget<A> {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.raw.validate(profile)
    }
}

impl<A> RenderCommand for PlayerTarget<A> {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

impl PlayerTargets {
    /// `@a` — all players.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::all",
        aliases = ["sand::cmd::PlayerTarget::all", "sand::cmd::PlayerTargets::all", "sand::cmd::SinglePlayer::all", "sand::command::PlayerTargets::all", "sand::command::SinglePlayer::all", "sand::prelude::PlayerTargets::all", "sand::prelude::SinglePlayer::all", "sand::prelude::cmd::PlayerTarget::all", "sand::prelude::cmd::PlayerTargets::all", "sand::prelude::cmd::SinglePlayer::all"],
        module = "sand::command",
        kind = "method",
        summary = "`@a` — all players.",
        context = "`@a` — all players. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `PlayerTarget` that emits the documented `@a` — all players form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let player_target = sand::command::PlayerTargets::all();\n}",
    )]
    pub fn all() -> Self {
        Self::from_selector(Selector::all_players())
    }

    /// Add `limit=1` and convert to a single-player target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::limit",
        aliases = ["sand::cmd::PlayerTarget::limit", "sand::cmd::PlayerTargets::limit", "sand::cmd::SinglePlayer::limit", "sand::command::PlayerTargets::limit", "sand::command::SinglePlayer::limit", "sand::prelude::PlayerTargets::limit", "sand::prelude::SinglePlayer::limit", "sand::prelude::cmd::PlayerTarget::limit", "sand::prelude::cmd::PlayerTargets::limit", "sand::prelude::cmd::SinglePlayer::limit"],
        module = "sand::command",
        kind = "method",
        summary = "Add `limit=1` and convert to a single-player target.",
        context = "Add `limit=1` and convert to a single-player target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`n` provides the n added when building `limit=1` and convert to a single-player target."),
        returns = "On success, the value produced to add `limit=1` and convert to a single-player target; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(player_target_value: sand::command::PlayerTargets, n: i32)  {\n    let limit = player_target_value.limit(n);\n}",
    )]
    pub fn limit(mut self, n: i32) -> CommandResult<SinglePlayer> {
        if n != 1 {
            return Err(CommandError::new(
                "PlayerTargets::limit",
                "limit",
                format!("single-player narrowing requires `limit=1`, got `{n}`"),
            ));
        }
        self.raw = self.raw.limit(n);
        Ok(SinglePlayer::from_selector(self.raw))
    }

    /// Pick the nearest matching player as a single target.
    pub fn nearest(mut self) -> SinglePlayer {
        self.raw = self.raw.sort(SortOrder::Nearest).limit(1);
        SinglePlayer::from_selector(self.raw)
    }
}

impl SinglePlayer {
    /// `@s` — the current executor as a single player.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::self_",
        aliases = ["sand::cmd::PlayerTarget::self_", "sand::cmd::PlayerTargets::self_", "sand::cmd::SinglePlayer::self_", "sand::command::PlayerTargets::self_", "sand::command::SinglePlayer::self_", "sand::prelude::PlayerTargets::self_", "sand::prelude::SinglePlayer::self_", "sand::prelude::cmd::PlayerTarget::self_", "sand::prelude::cmd::PlayerTargets::self_", "sand::prelude::cmd::SinglePlayer::self_"],
        module = "sand::command",
        kind = "method",
        summary = "`@s` — the current executor as a single player.",
        context = "`@s` — the current executor as a single player. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `PlayerTarget` that emits the documented `@s` — the current executor as a single player form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let player_target = sand::command::SinglePlayer::self_();\n}",
    )]
    pub fn self_() -> Self {
        Self::from_selector(Selector::self_())
    }

    /// `@p` — the nearest player.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::nearest",
        aliases = ["sand::cmd::PlayerTarget::nearest", "sand::cmd::PlayerTargets::nearest", "sand::cmd::SinglePlayer::nearest", "sand::command::PlayerTargets::nearest", "sand::command::SinglePlayer::nearest", "sand::prelude::PlayerTargets::nearest", "sand::prelude::SinglePlayer::nearest", "sand::prelude::cmd::PlayerTarget::nearest", "sand::prelude::cmd::PlayerTargets::nearest", "sand::prelude::cmd::SinglePlayer::nearest"],
        module = "sand::command",
        kind = "method",
        summary = "`@p` — the nearest player.",
        context = "`@p` — the nearest player. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `PlayerTarget` that emits the documented `@p` — the nearest player form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let player_target = sand::command::SinglePlayer::nearest();\n}",
    )]
    pub fn nearest() -> Self {
        Self::from_selector(Selector::nearest_player())
    }

    /// Explicit unchecked single-player selector syntax.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::PlayerTarget::raw",
        aliases = ["sand::cmd::PlayerTarget::raw", "sand::cmd::PlayerTargets::raw", "sand::cmd::SinglePlayer::raw", "sand::command::PlayerTargets::raw", "sand::command::SinglePlayer::raw", "sand::prelude::PlayerTargets::raw", "sand::prelude::SinglePlayer::raw", "sand::prelude::cmd::PlayerTarget::raw", "sand::prelude::cmd::PlayerTargets::raw", "sand::prelude::cmd::SinglePlayer::raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit unchecked single-player selector syntax.",
        context = "Explicit unchecked single-player selector syntax. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to use explicit unchecked single-player selector syntax."),
        returns = "A `PlayerTarget` configured for explicit unchecked single-player selector syntax.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: impl Into < String >)  {\n    let player_target = sand::command::SinglePlayer::raw(selector);\n}",
    )]
    pub fn raw(selector: impl Into<String>) -> Self {
        Self::from_selector(Selector::raw(selector))
    }
}

impl SingleEntity {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl EntityTargets {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl SinglePlayer {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl PlayerTargets {
    fn from_selector(raw: Selector) -> Self {
        Self {
            raw,
            _arity: PhantomData,
        }
    }
}

impl TryFrom<Selector> for SingleEntity {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate_single("SingleEntity")?;
        Ok(Self::from_selector(raw))
    }
}

impl TryFrom<Selector> for EntityTargets {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate(&CommandProfile::unprofiled())?;
        Ok(Self::from_selector(raw))
    }
}

impl TryFrom<Selector> for SinglePlayer {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate_player("SinglePlayer")?;
        raw.validate_single("SinglePlayer")?;
        Ok(Self::from_selector(raw))
    }
}

impl TryFrom<Selector> for PlayerTargets {
    type Error = CommandError;
    fn try_from(raw: Selector) -> CommandResult<Self> {
        raw.validate_player("PlayerTargets")?;
        Ok(Self::from_selector(raw))
    }
}

impl From<SinglePlayer> for SingleEntity {
    fn from(player: SinglePlayer) -> Self {
        SingleEntity::from_selector(player.raw)
    }
}

impl From<PlayerTargets> for EntityTargets {
    fn from(players: PlayerTargets) -> Self {
        EntityTargets::from_selector(players.raw)
    }
}

impl<A> fmt::Display for EntityTarget<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<A> fmt::Display for PlayerTarget<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::SortOrder",
    aliases = ["sand::cmd::SortOrder", "sand::prelude::cmd::SortOrder"],
    module = "sand::command",
    summary = "Sort order for entity selection in `@a`/`@e` selectors.",
    context = "Sort order for entity selection in `@a`/`@e` selectors. Determines the order entities are iterated when using commands like `execute as`.",
    minecraft = "Determines the order entities are iterated when using commands like `execute as`.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::SortOrder;",
    variants(Arbitrary = "No specific order (performance optimized).", Furthest = "Sort by distance from executor (furthest first).", Nearest = "Sort by distance from executor (nearest first).", Random = "Randomize the order."),
)]
/// Sort order for entity selection in `@a`/`@e` selectors.
///
/// Determines the order entities are iterated when using commands like `execute as`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort by distance from executor (nearest first).
    Nearest,
    /// Sort by distance from executor (furthest first).
    Furthest,
    /// Randomize the order.
    Random,
    /// No specific order (performance optimized).
    Arbitrary,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Nearest => write!(f, "nearest"),
            SortOrder::Furthest => write!(f, "furthest"),
            SortOrder::Random => write!(f, "random"),
            SortOrder::Arbitrary => write!(f, "arbitrary"),
        }
    }
}

/// A single selector argument key=value pair.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SelectorArg {
    Tag(String),
    NotTag(String),
    Team(String),
    NotTeam(String),
    Name(String),
    NotName(String),
    Type(String),
    NotType(String),
    Limit(i32),
    Sort(SortOrder),
    Distance(String),
    Level(String),
    XRotation(String),
    YRotation(String),
    Gamemode(String),
    Scores(String),
    Nbt(String),
    Predicate(String),
    X(f64),
    Y(f64),
    Z(f64),
    Dx(f64),
    Dy(f64),
    Dz(f64),
}

impl fmt::Display for SelectorArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tag(v) => write!(f, "tag={v}"),
            Self::NotTag(v) => write!(f, "tag=!{v}"),
            Self::Team(v) => write!(f, "team={v}"),
            Self::NotTeam(v) => write!(f, "team=!{v}"),
            Self::Name(v) => write!(f, "name={v}"),
            Self::NotName(v) => write!(f, "name=!{v}"),
            Self::Type(v) => write!(f, "type={v}"),
            Self::NotType(v) => write!(f, "type=!{v}"),
            Self::Limit(v) => write!(f, "limit={v}"),
            Self::Sort(v) => write!(f, "sort={v}"),
            Self::Distance(v) => write!(f, "distance={v}"),
            Self::Level(v) => write!(f, "level={v}"),
            Self::XRotation(v) => write!(f, "x_rotation={v}"),
            Self::YRotation(v) => write!(f, "y_rotation={v}"),
            Self::Gamemode(v) => write!(f, "gamemode={v}"),
            Self::Scores(v) => write!(f, "scores={{{v}}}"),
            Self::Nbt(v) => write!(f, "nbt={v}"),
            Self::Predicate(v) => write!(f, "predicate={v}"),
            Self::X(v) => write!(f, "x={v}"),
            Self::Y(v) => write!(f, "y={v}"),
            Self::Z(v) => write!(f, "z={v}"),
            Self::Dx(v) => write!(f, "dx={v}"),
            Self::Dy(v) => write!(f, "dy={v}"),
            Self::Dz(v) => write!(f, "dz={v}"),
        }
    }
}

// ── Constructor methods ───────────────────────────────────────────────────────

impl Selector {
    /// `@a` — all players currently connected to the server.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::all_players",
        aliases = ["sand::cmd::Selector::all_players", "sand::prelude::Selector::all_players", "sand::prelude::cmd::Selector::all_players"],
        module = "sand::command",
        kind = "method",
        summary = "`@a` — all players currently connected to the server.",
        context = "`@a` — all players currently connected to the server. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Selector` that emits the documented `@a` — all players currently connected to the server form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let selector = sand::command::Selector::all_players();\n}",
    )]
    pub fn all_players() -> Self {
        Self {
            base: TargetBase::AllPlayers,
            args: vec![],
        }
    }

    /// `@e` — all entities in the world.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::all_entities",
        aliases = ["sand::cmd::Selector::all_entities", "sand::prelude::Selector::all_entities", "sand::prelude::cmd::Selector::all_entities"],
        module = "sand::command",
        kind = "method",
        summary = "`@e` — all entities in the world.",
        context = "`@e` — all entities in the world. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Selector` that emits the documented `@e` — all entities in the world form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let selector = sand::command::Selector::all_entities();\n}",
    )]
    pub fn all_entities() -> Self {
        Self {
            base: TargetBase::AllEntities,
            args: vec![],
        }
    }

    /// `@p` — the nearest player to the command executor.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::nearest_player",
        aliases = ["sand::cmd::Selector::nearest_player", "sand::prelude::Selector::nearest_player", "sand::prelude::cmd::Selector::nearest_player"],
        module = "sand::command",
        kind = "method",
        summary = "`@p` — the nearest player to the command executor.",
        context = "`@p` — the nearest player to the command executor. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Selector` that emits the documented `@p` — the nearest player to the command executor form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let selector = sand::command::Selector::nearest_player();\n}",
    )]
    pub fn nearest_player() -> Self {
        Self {
            base: TargetBase::NearestPlayer,
            args: vec![],
        }
    }

    /// `@s` — the entity currently executing the command.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::self_",
        aliases = ["sand::cmd::Selector::self_", "sand::prelude::Selector::self_", "sand::prelude::cmd::Selector::self_"],
        module = "sand::command",
        kind = "method",
        summary = "`@s` — the entity currently executing the command.",
        context = "`@s` — the entity currently executing the command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Selector` that emits the documented `@s` — the entity currently executing the command form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let selector = sand::command::Selector::self_();\n}",
    )]
    pub fn self_() -> Self {
        Self {
            base: TargetBase::Self_,
            args: vec![],
        }
    }

    /// `@r` — a random player from the current players.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::random_player",
        aliases = ["sand::cmd::Selector::random_player", "sand::prelude::Selector::random_player", "sand::prelude::cmd::Selector::random_player"],
        module = "sand::command",
        kind = "method",
        summary = "`@r` — a random player from the current players.",
        context = "`@r` — a random player from the current players. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Selector` that emits the documented `@r` — a random player from the current players form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let selector = sand::command::Selector::random_player();\n}",
    )]
    pub fn random_player() -> Self {
        Self {
            base: TargetBase::RandomPlayer,
            args: vec![],
        }
    }

    /// A specific player by exact name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::player",
        aliases = ["sand::cmd::Selector::player", "sand::prelude::Selector::player", "sand::prelude::cmd::Selector::player"],
        module = "sand::command",
        kind = "method",
        summary = "A specific player by exact name.",
        context = "A specific player by exact name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` sets the author-visible text for a specific player by exact name."),
        returns = "A `Selector` configured for a specific player by exact name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let selector = sand::command::Selector::player(name);\n}",
    )]
    pub fn player(name: impl Into<String>) -> Self {
        Self {
            base: TargetBase::Player(name.into()),
            args: vec![],
        }
    }

    /// Wrap advanced selector syntax without typed validation.
    ///
    /// Prefer the typed builder methods for normal selectors. Raw selectors
    /// are preserved verbatim and should be limited to syntax Sand cannot yet
    /// model.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::raw",
        aliases = ["sand::cmd::Selector::raw", "sand::prelude::Selector::raw", "sand::prelude::cmd::Selector::raw"],
        module = "sand::command",
        kind = "method",
        summary = "Wrap advanced selector syntax without typed validation.",
        context = "Wrap advanced selector syntax without typed validation. Prefer the typed builder methods for normal selectors. Raw selectors are preserved verbatim and should be limited to syntax Sand cannot yet model.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Prefer the typed builder methods for normal selectors. Raw selectors are preserved verbatim and should be limited to syntax Sand cannot yet model."],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to wrap advanced selector syntax without typed validation."),
        returns = "A `Selector` wrapping advanced selector syntax without typed validation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: impl Into < String >)  {\n    let selector = sand::command::Selector::raw(selector);\n}",
    )]
    pub fn raw(selector: impl Into<String>) -> Self {
        Self {
            base: TargetBase::Raw(selector.into()),
            args: vec![],
        }
    }
}

// ── Builder methods ───────────────────────────────────────────────────────────

impl Selector {
    /// `tag=<tag>` — select only entities that have the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::tag",
        aliases = ["sand::cmd::Selector::tag", "sand::prelude::Selector::tag", "sand::prelude::cmd::Selector::tag"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=<tag>` — select only entities that have the given tag.",
        context = "`tag=<tag>` — select only entities that have the given tag. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — select only entities that have the given tag form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `tag=<tag>` — select only entities that have the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, tag: impl Into < String >)  {\n    let updated_selector = selector_value.tag(tag);\n}",
    )]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Tag(tag.into()));
        self
    }

    /// `tag=!<tag>` — select only entities that do NOT have the given tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::not_tag",
        aliases = ["sand::cmd::Selector::not_tag", "sand::prelude::Selector::not_tag", "sand::prelude::cmd::Selector::not_tag"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=!<tag>` — select only entities that do NOT have the given tag.",
        context = "`tag=!<tag>` — select only entities that do NOT have the given tag. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=!<tag>` — select only entities that do NOT have the given tag form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `tag=!<tag>` — select only entities that do NOT have the given tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, tag: impl Into < String >)  {\n    let updated_selector = selector_value.not_tag(tag);\n}",
    )]
    pub fn not_tag(mut self, tag: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTag(tag.into()));
        self
    }

    /// `team=<team>` — select only entities on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::team",
        aliases = ["sand::cmd::Selector::team", "sand::prelude::Selector::team", "sand::prelude::cmd::Selector::team"],
        module = "sand::command",
        kind = "method",
        summary = "`team=<team>` — select only entities on the given team.",
        context = "`team=<team>` — select only entities on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=<team>` — select only entities on the given team form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `team=<team>` — select only entities on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, team: impl Into < String >)  {\n    let updated_selector = selector_value.team(team);\n}",
    )]
    pub fn team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Team(team.into()));
        self
    }

    /// `team=!<team>` — select only entities NOT on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::not_team",
        aliases = ["sand::cmd::Selector::not_team", "sand::prelude::Selector::not_team", "sand::prelude::cmd::Selector::not_team"],
        module = "sand::command",
        kind = "method",
        summary = "`team=!<team>` — select only entities NOT on the given team.",
        context = "`team=!<team>` — select only entities NOT on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=!<team>` — select only entities NOT on the given team form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `team=!<team>` — select only entities NOT on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, team: impl Into < String >)  {\n    let updated_selector = selector_value.not_team(team);\n}",
    )]
    pub fn not_team(mut self, team: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotTeam(team.into()));
        self
    }

    /// `name=<name>` — select only entities with the exact display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::name",
        aliases = ["sand::cmd::Selector::name", "sand::prelude::Selector::name", "sand::prelude::cmd::Selector::name"],
        module = "sand::command",
        kind = "method",
        summary = "`name=<name>` — select only entities with the exact display name.",
        context = "`name=<name>` — select only entities with the exact display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` supplies the documented `name=<name>` — select only entities with the exact display name form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `name=<name>` — select only entities with the exact display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, name: impl Into < String >)  {\n    let updated_selector = selector_value.name(name);\n}",
    )]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Name(name.into()));
        self
    }

    /// `name=!<name>` — select only entities WITHOUT the given display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::not_name",
        aliases = ["sand::cmd::Selector::not_name", "sand::prelude::Selector::not_name", "sand::prelude::cmd::Selector::not_name"],
        module = "sand::command",
        kind = "method",
        summary = "`name=!<name>` — select only entities WITHOUT the given display name.",
        context = "`name=!<name>` — select only entities WITHOUT the given display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` supplies the documented `name=!<name>` — select only entities WITHOUT the given display name form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `name=!<name>` — select only entities WITHOUT the given display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, name: impl Into < String >)  {\n    let updated_selector = selector_value.not_name(name);\n}",
    )]
    pub fn not_name(mut self, name: impl Into<String>) -> Self {
        self.args.push(SelectorArg::NotName(name.into()));
        self
    }

    /// `type=<entity_type>` — select only entities of the given type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::entity_type",
        aliases = ["sand::cmd::Selector::entity_type", "sand::prelude::Selector::entity_type", "sand::prelude::cmd::Selector::entity_type"],
        module = "sand::command",
        kind = "method",
        summary = "`type=<entity_type>` — select only entities of the given type.",
        context = "`type=<entity_type>` — select only entities of the given type. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(ty = "`ty` supplies the documented `type=<entity_type>` — select only entities of the given type form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `type=<entity_type>` — select only entities of the given type form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, ty: impl sand::command::IntoEntityType)  {\n    let updated_selector = selector_value.entity_type(ty);\n}",
    )]
    pub fn entity_type(mut self, ty: impl IntoEntityType) -> Self {
        self.args.push(SelectorArg::Type(ty.into_entity_type()));
        self
    }

    /// `type=!<entity_type>` — select only entities NOT of the given type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::not_type",
        aliases = ["sand::cmd::Selector::not_type", "sand::prelude::Selector::not_type", "sand::prelude::cmd::Selector::not_type"],
        module = "sand::command",
        kind = "method",
        summary = "`type=!<entity_type>` — select only entities NOT of the given type.",
        context = "`type=!<entity_type>` — select only entities NOT of the given type. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(ty = "`ty` supplies the documented `type=!<entity_type>` — select only entities NOT of the given type form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `type=!<entity_type>` — select only entities NOT of the given type form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, ty: impl sand::command::IntoEntityType)  {\n    let updated_selector = selector_value.not_type(ty);\n}",
    )]
    pub fn not_type(mut self, ty: impl IntoEntityType) -> Self {
        self.args.push(SelectorArg::NotType(ty.into_entity_type()));
        self
    }

    /// `limit=<n>` — select at most `n` entities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::limit",
        aliases = ["sand::cmd::Selector::limit", "sand::prelude::Selector::limit", "sand::prelude::cmd::Selector::limit"],
        module = "sand::command",
        kind = "method",
        summary = "`limit=<n>` — select at most `n` entities.",
        context = "`limit=<n>` — select at most `n` entities. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`limit=<n>` — select at most `n` entities."),
        returns = "The `Selector` value with the documented change applied to emit the documented `limit=<n>` — select at most `n` entities form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, n: i32)  {\n    let updated_selector = selector_value.limit(n);\n}",
    )]
    pub fn limit(mut self, n: i32) -> Self {
        self.args.push(SelectorArg::Limit(n));
        self
    }

    /// `sort=<order>` — set the sort order before applying limit.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::sort",
        aliases = ["sand::cmd::Selector::sort", "sand::prelude::Selector::sort", "sand::prelude::cmd::Selector::sort"],
        module = "sand::command",
        kind = "method",
        summary = "`sort=<order>` — set the sort order before applying limit.",
        context = "`sort=<order>` — set the sort order before applying limit. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(order = "`order` supplies the documented `sort=<order>` — set the sort order before applying limit form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `sort=<order>` — set the sort order before applying limit form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, order: sand::command::SortOrder)  {\n    let updated_selector = selector_value.sort(order);\n}",
    )]
    pub fn sort(mut self, order: SortOrder) -> Self {
        self.args.push(SelectorArg::Sort(order));
        self
    }

    /// `distance=<range>` — select only entities within a distance range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::distance",
        aliases = ["sand::cmd::Selector::distance", "sand::prelude::Selector::distance", "sand::prelude::cmd::Selector::distance"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<range>` — select only entities within a distance range.",
        context = "`distance=<range>` — select only entities within a distance range. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` supplies the documented `distance=<range>` — select only entities within a distance range form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `distance=<range>` — select only entities within a distance range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, range: impl Into < String >)  {\n    let updated_selector = selector_value.distance(range);\n}",
    )]
    pub fn distance(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Distance(range.into()));
        self
    }

    /// `distance=..<max>` — select only entities at most `max` blocks away.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::distance_max",
        aliases = ["sand::cmd::Selector::distance_max", "sand::prelude::Selector::distance_max", "sand::prelude::cmd::Selector::distance_max"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=..<max>` — select only entities at most `max` blocks away.",
        context = "`distance=..<max>` — select only entities at most `max` blocks away. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(max = "`distance=..<max>` — select only entities at most `max` blocks away."),
        returns = "The `Selector` value with the documented change applied to emit the documented `distance=..<max>` — select only entities at most `max` blocks away form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, max: f64)  {\n    let updated_selector = selector_value.distance_max(max);\n}",
    )]
    pub fn distance_max(mut self, max: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("..{max}")));
        self
    }

    /// `distance=<min>..` — select only entities at least `min` blocks away.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::distance_min",
        aliases = ["sand::cmd::Selector::distance_min", "sand::prelude::Selector::distance_min", "sand::prelude::cmd::Selector::distance_min"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<min>..` — select only entities at least `min` blocks away.",
        context = "`distance=<min>..` — select only entities at least `min` blocks away. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`distance=<min>..` — select only entities at least `min` blocks away."),
        returns = "The `Selector` value with the documented change applied to emit the documented `distance=<min>..` — select only entities at least `min` blocks away form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, min: f64)  {\n    let updated_selector = selector_value.distance_min(min);\n}",
    )]
    pub fn distance_min(mut self, min: f64) -> Self {
        self.args.push(SelectorArg::Distance(format!("{min}..")));
        self
    }

    /// `distance=<min>..<max>` — select only entities between `min` and `max` blocks away.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::distance_range",
        aliases = ["sand::cmd::Selector::distance_range", "sand::prelude::Selector::distance_range", "sand::prelude::cmd::Selector::distance_range"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<min>..<max>` — select only entities between `min` and `max` blocks away.",
        context = "`distance=<min>..<max>` — select only entities between `min` and `max` blocks away. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`distance=<min>..<max>` — select only entities between `min` and `max` blocks away.", max = "`distance=<min>..<max>` — select only entities between `min` and `max` blocks away."),
        returns = "The `Selector` value with the documented change applied to emit the documented `distance=<min>..<max>` — select only entities between `min` and `max` blocks away form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, min: f64, max: f64)  {\n    let updated_selector = selector_value.distance_range(min, max);\n}",
    )]
    pub fn distance_range(mut self, min: f64, max: f64) -> Self {
        self.args
            .push(SelectorArg::Distance(format!("{min}..{max}")));
        self
    }

    /// `type=!minecraft:player` — exclude all players from the selection.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::not_player",
        aliases = ["sand::cmd::Selector::not_player", "sand::prelude::Selector::not_player", "sand::prelude::cmd::Selector::not_player"],
        module = "sand::command",
        kind = "method",
        summary = "`type=!minecraft:player` — exclude all players from the selection.",
        context = "`type=!minecraft:player` — exclude all players from the selection. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `Selector` value with the documented change applied to emit the documented `type=!minecraft:player` — exclude all players from the selection form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector)  {\n    let updated_selector = selector_value.not_player();\n}",
    )]
    pub fn not_player(mut self) -> Self {
        self.args
            .push(SelectorArg::NotType("minecraft:player".into()));
        self
    }

    /// `level=<range>` — select only players within the given XP level range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::level",
        aliases = ["sand::cmd::Selector::level", "sand::prelude::Selector::level", "sand::prelude::cmd::Selector::level"],
        module = "sand::command",
        kind = "method",
        summary = "`level=<range>` — select only players within the given XP level range.",
        context = "`level=<range>` — select only players within the given XP level range. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` supplies the documented `level=<range>` — select only players within the given XP level range form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `level=<range>` — select only players within the given XP level range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, range: impl Into < String >)  {\n    let updated_selector = selector_value.level(range);\n}",
    )]
    pub fn level(mut self, range: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Level(range.into()));
        self
    }

    /// `gamemode=<mode>` — select only players in the given gamemode.
    ///
    /// Raw/compatibility: `mode` is a string, validated against the vanilla
    /// gamemode set at [`Selector::try_build`] time rather than at the type
    /// level. Prefer [`Selector::gamemode_typed`] in normal code — see
    /// [#173](https://github.com/ThatOneToast/sand/issues/173).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::gamemode",
        aliases = ["sand::cmd::Selector::gamemode", "sand::prelude::Selector::gamemode", "sand::prelude::cmd::Selector::gamemode"],
        module = "sand::command",
        kind = "method",
        summary = "`gamemode=<mode>` — select only players in the given gamemode.",
        context = "`gamemode=<mode>` — select only players in the given gamemode. Raw/compatibility: `mode` is a string, validated against the vanilla gamemode set at [`Selector::try_build`] time rather than at the type level. Prefer [`Selector::gamemode_typed`] in normal code — see [#173](https://github.com/ThatOneToast/sand/issues/173).",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(mode = "Raw/compatibility: `mode` is a string, validated against the vanilla gamemode set at [`Selector::try_build`] time rather than at the type level. Prefer [`Selector::gamemode_typed`] in normal code — see [#173](https://github.com/ThatOneToast/sand/issues/173)."),
        returns = "The `Selector` value with the documented change applied to emit the documented `gamemode=<mode>` — select only players in the given gamemode form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, mode: impl Into < String >)  {\n    let updated_selector = selector_value.gamemode(mode);\n}",
    )]
    pub fn gamemode(mut self, mode: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Gamemode(mode.into()));
        self
    }

    /// `gamemode=<mode>` — select only players in the given gamemode, using
    /// the canonical typed [`GameMode`] enum instead of a validated string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::gamemode_typed",
        aliases = ["sand::cmd::Selector::gamemode_typed", "sand::prelude::Selector::gamemode_typed", "sand::prelude::cmd::Selector::gamemode_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string.",
        context = "`gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(mode = "`mode` supplies the documented `gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `gamemode=<mode>` — select only players in the given gamemode, using the canonical typed [`GameMode`] enum instead of a validated string form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, mode: sand::command::GameMode)  {\n    let updated_selector = selector_value.gamemode_typed(mode);\n}",
    )]
    pub fn gamemode_typed(mut self, mode: GameMode) -> Self {
        self.args.push(SelectorArg::Gamemode(mode.to_string()));
        self
    }

    /// `gamemode=!<mode>` — exclude players in the given gamemode.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::not_gamemode_typed",
        aliases = ["sand::cmd::Selector::not_gamemode_typed", "sand::prelude::Selector::not_gamemode_typed", "sand::prelude::cmd::Selector::not_gamemode_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`gamemode=!<mode>` — exclude players in the given gamemode.",
        context = "`gamemode=!<mode>` — exclude players in the given gamemode. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(mode = "`mode` supplies the documented `gamemode=!<mode>` — exclude players in the given gamemode form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `gamemode=!<mode>` — exclude players in the given gamemode form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, mode: sand::command::GameMode)  {\n    let updated_selector = selector_value.not_gamemode_typed(mode);\n}",
    )]
    pub fn not_gamemode_typed(mut self, mode: GameMode) -> Self {
        self.args.push(SelectorArg::Gamemode(format!("!{mode}")));
        self
    }

    /// `scores=<objective>=<range>` — select only entities with matching scoreboard score.
    ///
    /// Raw/compatibility: `scores` is a single pre-formatted fragment (e.g.
    /// `"kills=1..10,deaths=0"`), validated at [`Selector::try_build`] time
    /// rather than at the type level. Prefer [`Selector::scores_typed`] in
    /// normal code — see [#200](https://github.com/ThatOneToast/sand/issues/200).
    /// Equivalent to [`Selector::scores_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::scores",
        aliases = ["sand::cmd::Selector::scores", "sand::prelude::Selector::scores", "sand::prelude::cmd::Selector::scores"],
        module = "sand::command",
        kind = "method",
        summary = "`scores=<objective>=<range>` — select only entities with matching scoreboard score.",
        context = "`scores=<objective>=<range>` — select only entities with matching scoreboard score. Raw/compatibility: `scores` is a single pre-formatted fragment (e.g. `\"kills=1..10,deaths=0\"`), validated at [`Selector::try_build`] time rather than at the type level. Prefer [`Selector::scores_typed`] in normal code — see [#200](https://github.com/ThatOneToast/sand/issues/200). Equivalent to [`Selector::scores_raw`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(scores = "Raw/compatibility: `scores` is a single pre-formatted fragment (e.g. `\"kills=1..10,deaths=0\"`), validated at [`Selector::try_build`] time rather than at the type level. Prefer [`Selector::scores_typed`] in normal code — see [#200](https://github.com/ThatOneToast/sand/issues/200). Equivalent to [`Selector::scores_raw`]."),
        returns = "The `Selector` value with the documented change applied to emit the documented `scores=<objective>=<range>` — select only entities with matching scoreboard score form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, scores: impl Into < String >)  {\n    let updated_selector = selector_value.scores(scores);\n}",
    )]
    pub fn scores(mut self, scores: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Scores(scores.into()));
        self
    }

    /// Explicit raw escape hatch for `scores=...` syntax, e.g. hand-formatted
    /// fragments this crate has no typed representation for yet. Equivalent
    /// to [`Selector::scores`] — use whichever name best documents intent at
    /// the call site.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::scores_raw",
        aliases = ["sand::cmd::Selector::scores_raw", "sand::prelude::Selector::scores_raw", "sand::prelude::cmd::Selector::scores_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `scores=...` syntax, e.g. hand-formatted fragments this crate has no typed representation for yet. Equivalent to [`Selector::scores`] — use whichever name best documents intent at the call site.",
        context = "Explicit raw escape hatch for `scores=...` syntax, e.g. hand-formatted fragments this crate has no typed representation for yet. Equivalent to [`Selector::scores`] — use whichever name best documents intent at the call site. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(scores = "`scores` sets the scores for explicit raw escape hatch for `scores=...` syntax, e.g. hand-formatted fragments this crate has no typed representation for yet. Equivalent to [`Selector::scores`] — use whichever name best documents intent at the call site."),
        returns = "The `Selector` value with the documented change applied to use explicit raw escape hatch for `scores=...` syntax, e.g. hand-formatted fragments this crate has no typed representation for yet. Equivalent to [`Selector::scores`] — use whichever name best documents intent at the call site.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, scores: impl Into < String >)  {\n    let updated_selector = selector_value.scores_raw(scores);\n}",
    )]
    pub fn scores_raw(self, scores: impl Into<String>) -> Self {
        self.scores(scores)
    }

    /// `scores={<objective>=<range>,...}` — select only entities with
    /// matching scoreboard scores, built from typed [`SelectorScores`]
    /// entries instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, SelectorScores, ScoreRange};
    ///
    /// let sel = Selector::all_players().scores_typed(
    ///     SelectorScores::new()
    ///         .with("kills", ScoreRange::between(1, 10))
    ///         .with("deaths", ScoreRange::exact(0)),
    /// );
    /// assert_eq!(sel.to_string(), "@a[scores={kills=1..10,deaths=0}]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::scores_typed",
        aliases = ["sand::cmd::Selector::scores_typed", "sand::prelude::Selector::scores_typed", "sand::prelude::cmd::Selector::scores_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string.",
        context = "`scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(scores = "`scores` provides the Minecraft target selection used to emit the documented `scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `scores={<objective>=<range>,...}` — select only entities with matching scoreboard scores, built from typed [`SelectorScores`] entries instead of a hand-formatted string form.",
        example = "use sand::command::{Selector, SelectorScores, ScoreRange};\nlet sel = Selector::all_players().scores_typed(\nSelectorScores::new()\n.with(\"kills\", ScoreRange::between(1, 10))\n.with(\"deaths\", ScoreRange::exact(0)),\n);\nassert_eq!(sel.to_string(), \"@a[scores={kills=1..10,deaths=0}]\");",
    )]
    pub fn scores_typed(mut self, scores: SelectorScores) -> Self {
        self.args.push(SelectorArg::Scores(scores.to_string()));
        self
    }

    /// Add one typed scoreboard filter to the selector's score map.
    ///
    /// Repeated calls merge into one `scores={...}` argument. Reusing an
    /// objective is rejected so higher-level typed state queries cannot emit
    /// ambiguous filters.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::score_typed",
        aliases = ["sand::cmd::Selector::score_typed", "sand::prelude::Selector::score_typed", "sand::prelude::cmd::Selector::score_typed"],
        module = "sand::command",
        kind = "method",
        summary = "Add one typed scoreboard filter to the selector's score map.",
        context = "Add one typed scoreboard filter to the selector's score map. Repeated calls merge into one `scores={...}` argument. Reusing an objective is rejected so higher-level typed state queries cannot emit ambiguous filters.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(objective = "`objective` provides the objective added when building one typed scoreboard filter to the selector's score map.", range = "`range` provides the accepted numeric range used to add one typed scoreboard filter to the selector's score map."),
        returns = "On success, the value produced to add one typed scoreboard filter to the selector's score map; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, objective: sand::command::ObjectiveName, range: sand::command::ScoreRange)  {\n    let score_typed = selector_value.score_typed(objective, range);\n}",
    )]
    pub fn score_typed(
        mut self,
        objective: crate::ObjectiveName,
        range: ScoreRange,
    ) -> CommandResult<Self> {
        objective.validate(&CommandProfile::unprofiled())?;
        let objective = objective.as_str();
        if let Some(SelectorArg::Scores(scores)) = self
            .args
            .iter_mut()
            .find(|argument| matches!(argument, SelectorArg::Scores(_)))
        {
            if scores.split(',').any(|entry| {
                entry
                    .split_once('=')
                    .is_some_and(|(existing, _)| existing == objective)
            }) {
                return Err(CommandError::new(
                    "Selector",
                    "scores",
                    format!("duplicate typed score predicate for objective `{objective}`"),
                )
                .with_code("SAND-SELECTOR-SCORE-DUPLICATE"));
            }
            scores.push(',');
            scores.push_str(objective);
            scores.push('=');
            scores.push_str(&range.to_string());
        } else {
            self.args
                .push(SelectorArg::Scores(format!("{objective}={range}")));
        }
        Ok(self)
    }

    /// `nbt=<nbt>` — select only entities matching the given NBT compound.
    ///
    /// Raw escape hatch: no typed SNBT representation exists yet in this
    /// crate, so this remains the normal path for NBT filters. Equivalent to
    /// [`Selector::nbt_raw`], kept for readability at call sites that prefer
    /// the shorter name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::nbt",
        aliases = ["sand::cmd::Selector::nbt", "sand::prelude::Selector::nbt", "sand::prelude::cmd::Selector::nbt"],
        module = "sand::command",
        kind = "method",
        summary = "`nbt=<nbt>` — select only entities matching the given NBT compound.",
        context = "`nbt=<nbt>` — select only entities matching the given NBT compound. Raw escape hatch: no typed SNBT representation exists yet in this crate, so this remains the normal path for NBT filters. Equivalent to [`Selector::nbt_raw`], kept for readability at call sites that prefer the shorter name.",
        minecraft = "Raw escape hatch: no typed SNBT representation exists yet in this crate, so this remains the normal path for NBT filters. Equivalent to [`Selector::nbt_raw`], kept for readability at call sites that prefer the shorter name.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(nbt = "`nbt` provides the NBT payload used to emit the documented `nbt=<nbt>` — select only entities matching the given NBT compound form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `nbt=<nbt>` — select only entities matching the given NBT compound form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, nbt: impl Into < String >)  {\n    let updated_selector = selector_value.nbt(nbt);\n}",
    )]
    pub fn nbt(mut self, nbt: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Nbt(nbt.into()));
        self
    }

    /// Explicit raw escape hatch for `nbt=...` syntax. Equivalent to
    /// [`Selector::nbt`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::nbt_raw",
        aliases = ["sand::cmd::Selector::nbt_raw", "sand::prelude::Selector::nbt_raw", "sand::prelude::cmd::Selector::nbt_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `nbt=...` syntax. Equivalent to [`Selector::nbt`].",
        context = "Explicit raw escape hatch for `nbt=...` syntax. Equivalent to [`Selector::nbt`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(nbt = "`nbt` provides the NBT payload used to use explicit raw escape hatch for `nbt=...` syntax. Equivalent to [`Selector::nbt`]."),
        returns = "The `Selector` value with the documented change applied to use explicit raw escape hatch for `nbt=...` syntax. Equivalent to [`Selector::nbt`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, nbt: impl Into < String >)  {\n    let updated_selector = selector_value.nbt_raw(nbt);\n}",
    )]
    pub fn nbt_raw(self, nbt: impl Into<String>) -> Self {
        self.nbt(nbt)
    }

    /// `predicate=<predicate>` — select only entities matching a loot table predicate.
    ///
    /// Raw/compatibility: `predicate` is a string, validated for
    /// resource-location shape at [`Selector::try_build`] time. Prefer
    /// [`Selector::predicate_id`] in normal code — see
    /// [#200](https://github.com/ThatOneToast/sand/issues/200). Equivalent to
    /// [`Selector::predicate_raw`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::predicate",
        aliases = ["sand::cmd::Selector::predicate", "sand::prelude::Selector::predicate", "sand::prelude::cmd::Selector::predicate"],
        module = "sand::command",
        kind = "method",
        summary = "`predicate=<predicate>` — select only entities matching a loot table predicate.",
        context = "`predicate=<predicate>` — select only entities matching a loot table predicate. Raw/compatibility: `predicate` is a string, validated for resource-location shape at [`Selector::try_build`] time. Prefer [`Selector::predicate_id`] in normal code — see [#200](https://github.com/ThatOneToast/sand/issues/200). Equivalent to [`Selector::predicate_raw`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(predicate = "Raw/compatibility: `predicate` is a string, validated for resource-location shape at [`Selector::try_build`] time. Prefer [`Selector::predicate_id`] in normal code — see [#200](https://github.com/ThatOneToast/sand/issues/200). Equivalent to [`Selector::predicate_raw`]."),
        returns = "The `Selector` value with the documented change applied to emit the documented `predicate=<predicate>` — select only entities matching a loot table predicate form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, predicate: impl Into < String >)  {\n    let updated_selector = selector_value.predicate(predicate);\n}",
    )]
    pub fn predicate(mut self, predicate: impl Into<String>) -> Self {
        self.args.push(SelectorArg::Predicate(predicate.into()));
        self
    }

    /// Explicit raw escape hatch for `predicate=...` syntax. Equivalent to
    /// [`Selector::predicate`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::predicate_raw",
        aliases = ["sand::cmd::Selector::predicate_raw", "sand::prelude::Selector::predicate_raw", "sand::prelude::cmd::Selector::predicate_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit raw escape hatch for `predicate=...` syntax. Equivalent to [`Selector::predicate`].",
        context = "Explicit raw escape hatch for `predicate=...` syntax. Equivalent to [`Selector::predicate`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(predicate = "`predicate` provides the predicate that must match used to use explicit raw escape hatch for `predicate=...` syntax. Equivalent to [`Selector::predicate`]."),
        returns = "The `Selector` value with the documented change applied to use explicit raw escape hatch for `predicate=...` syntax. Equivalent to [`Selector::predicate`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, predicate: impl Into < String >)  {\n    let updated_selector = selector_value.predicate_raw(predicate);\n}",
    )]
    pub fn predicate_raw(self, predicate: impl Into<String>) -> Self {
        self.predicate(predicate)
    }

    /// `predicate=<id>` — select only entities matching a loot table
    /// predicate, using a typed [`PredicateId`] instead of a raw string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, PredicateId};
    ///
    /// let sel = Selector::all_players().predicate_id(PredicateId::new("my_pack:is_sneaking"));
    /// assert_eq!(sel.to_string(), "@a[predicate=my_pack:is_sneaking]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::predicate_id",
        aliases = ["sand::cmd::Selector::predicate_id", "sand::prelude::Selector::predicate_id", "sand::prelude::cmd::Selector::predicate_id"],
        module = "sand::command",
        kind = "method",
        summary = "`predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string.",
        context = "`predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to emit the documented `predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `predicate=<id>` — select only entities matching a loot table predicate, using a typed [`PredicateId`] instead of a raw string form.",
        example = "use {sand::command::Selector, sand::predicate::PredicateId};\nlet sel = Selector::all_players().predicate_id(PredicateId::new(\"my_pack:is_sneaking\"));\nassert_eq!(sel.to_string(), \"@a[predicate=my_pack:is_sneaking]\");",
    )]
    pub fn predicate_id(mut self, id: PredicateId) -> Self {
        self.args.push(SelectorArg::Predicate(id.to_string()));
        self
    }

    /// `distance=<range>` — select only entities within a typed distance
    /// range, using [`SelectorRange`] instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, SelectorRange};
    ///
    /// let sel = Selector::all_entities().distance_typed(SelectorRange::at_most(16.0));
    /// assert_eq!(sel.to_string(), "@e[distance=..16]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::distance_typed",
        aliases = ["sand::cmd::Selector::distance_typed", "sand::prelude::Selector::distance_typed", "sand::prelude::cmd::Selector::distance_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string.",
        context = "`distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` provides the Minecraft target selection used to emit the documented `distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `distance=<range>` — select only entities within a typed distance range, using [`SelectorRange`] instead of a hand-formatted string form.",
        example = "use sand::command::{Selector, SelectorRange};\nlet sel = Selector::all_entities().distance_typed(SelectorRange::at_most(16.0));\nassert_eq!(sel.to_string(), \"@e[distance=..16]\");",
    )]
    pub fn distance_typed(mut self, range: SelectorRange) -> Self {
        self.args.push(SelectorArg::Distance(range.to_string()));
        self
    }

    /// `level=<range>` — select only players within a typed XP level range,
    /// using [`SelectorRange`] instead of a hand-formatted string.
    ///
    /// ```
    /// use sand_commands::selector::{Selector, SelectorRange};
    ///
    /// let sel = Selector::all_players().level_typed(SelectorRange::between(10.0, 30.0));
    /// assert_eq!(sel.to_string(), "@a[level=10..30]");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::level_typed",
        aliases = ["sand::cmd::Selector::level_typed", "sand::prelude::Selector::level_typed", "sand::prelude::cmd::Selector::level_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string.",
        context = "`level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(range = "`range` provides the Minecraft target selection used to emit the documented `level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `level=<range>` — select only players within a typed XP level range, using [`SelectorRange`] instead of a hand-formatted string form.",
        example = "use sand::command::{Selector, SelectorRange};\nlet sel = Selector::all_players().level_typed(SelectorRange::between(10.0, 30.0));\nassert_eq!(sel.to_string(), \"@a[level=10..30]\");",
    )]
    pub fn level_typed(mut self, range: SelectorRange) -> Self {
        self.args.push(SelectorArg::Level(range.to_string()));
        self
    }

    /// `tag=<tag>` — select only entities with the given tag, using a typed
    /// [`EntityTag`] instead of a raw string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::tag_typed",
        aliases = ["sand::cmd::Selector::tag_typed", "sand::prelude::Selector::tag_typed", "sand::prelude::cmd::Selector::tag_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string.",
        context = "`tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` supplies the documented `tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `tag=<tag>` — select only entities with the given tag, using a typed [`EntityTag`] instead of a raw string form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, tag: sand::command::EntityTag)  {\n    let updated_selector = selector_value.tag_typed(tag);\n}",
    )]
    pub fn tag_typed(mut self, tag: EntityTag) -> Self {
        self.args.push(SelectorArg::Tag(tag.into_inner()));
        self
    }

    /// `team=<team>` — select only entities on the given team, using a typed
    /// [`TeamName`] instead of a raw string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::team_typed",
        aliases = ["sand::cmd::Selector::team_typed", "sand::prelude::Selector::team_typed", "sand::prelude::cmd::Selector::team_typed"],
        module = "sand::command",
        kind = "method",
        summary = "`team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string.",
        context = "`team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the documented `team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `team=<team>` — select only entities on the given team, using a typed [`TeamName`] instead of a raw string form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, team: sand::command::TeamName)  {\n    let updated_selector = selector_value.team_typed(team);\n}",
    )]
    pub fn team_typed(mut self, team: TeamName) -> Self {
        self.args.push(SelectorArg::Team(team.into_inner()));
        self
    }

    /// `dx/dy/dz` — set a bounding box volume filter.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::volume",
        aliases = ["sand::cmd::Selector::volume", "sand::prelude::Selector::volume", "sand::prelude::cmd::Selector::volume"],
        module = "sand::command",
        kind = "method",
        summary = "`dx/dy/dz` — set a bounding box volume filter.",
        context = "`dx/dy/dz` — set a bounding box volume filter. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(dx = "`dx` provides the x-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form.", dy = "`dy` provides the y-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form.", dz = "`dz` provides the z-axis offset or spread used to emit the documented `dx/dy/dz` — set a bounding box volume filter form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `dx/dy/dz` — set a bounding box volume filter form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, dx: f64, dy: f64, dz: f64)  {\n    let updated_selector = selector_value.volume(dx, dy, dz);\n}",
    )]
    pub fn volume(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.args.push(SelectorArg::Dx(dx));
        self.args.push(SelectorArg::Dy(dy));
        self.args.push(SelectorArg::Dz(dz));
        self
    }

    /// `x/y/z` — set the origin point for distance and volume checks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Selector::at_pos",
        aliases = ["sand::cmd::Selector::at_pos", "sand::prelude::Selector::at_pos", "sand::prelude::cmd::Selector::at_pos"],
        module = "sand::command",
        kind = "method",
        summary = "`x/y/z` — set the origin point for distance and volume checks.",
        context = "`x/y/z` — set the origin point for distance and volume checks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form.", y = "`y` provides the y-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form.", z = "`z` provides the z-coordinate used to emit the documented `x/y/z` — set the origin point for distance and volume checks form."),
        returns = "The `Selector` value with the documented change applied to emit the documented `x/y/z` — set the origin point for distance and volume checks form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector_value: sand::command::Selector, x: f64, y: f64, z: f64)  {\n    let updated_selector = selector_value.at_pos(x, y, z);\n}",
    )]
    pub fn at_pos(mut self, x: f64, y: f64, z: f64) -> Self {
        self.args.push(SelectorArg::X(x));
        self.args.push(SelectorArg::Y(y));
        self.args.push(SelectorArg::Z(z));
        self
    }

    fn exclude_self_distance(mut self) -> Self {
        for arg in &mut self.args {
            if let SelectorArg::Distance(range) = arg
                && let Some(max) = range.strip_prefix("..")
            {
                *range = format!("0.1..{max}");
                return self;
            }
        }
        self.args.push(SelectorArg::Distance("0.1..".to_string()));
        self
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = match &self.base {
            TargetBase::AllPlayers => "@a",
            TargetBase::AllEntities => "@e",
            TargetBase::NearestPlayer => "@p",
            TargetBase::Self_ => "@s",
            TargetBase::RandomPlayer => "@r",
            TargetBase::Player(n) => return write!(f, "{n}"),
            TargetBase::Raw(raw) => return write!(f, "{raw}"),
        };
        if self.args.is_empty() {
            write!(f, "{base}")
        } else {
            let args = self
                .args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(",");
            write!(f, "{base}[{args}]")
        }
    }
}

impl Selector {
    pub(crate) fn is_statically_single(&self) -> bool {
        !matches!(self.base, TargetBase::Raw(_))
            && (matches!(
                self.base,
                TargetBase::NearestPlayer
                    | TargetBase::Self_
                    | TargetBase::RandomPlayer
                    | TargetBase::Player(_)
            ) || self
                .args
                .iter()
                .any(|arg| matches!(arg, SelectorArg::Limit(1))))
    }

    fn validate_single(&self, helper: &'static str) -> CommandResult<()> {
        self.validate(&CommandProfile::unprofiled())?;
        if self.is_statically_single() {
            Ok(())
        } else {
            Err(CommandError::new(
                helper,
                "selector",
                "target may match multiple entities; add `limit=1` or use a many-target type",
            ))
        }
    }

    fn validate_player(&self, helper: &'static str) -> CommandResult<()> {
        self.validate(&CommandProfile::unprofiled())?;
        if matches!(
            self.base,
            TargetBase::AllPlayers
                | TargetBase::NearestPlayer
                | TargetBase::Self_
                | TargetBase::RandomPlayer
                | TargetBase::Player(_)
        ) {
            Ok(())
        } else {
            Err(CommandError::new(
                helper,
                "selector",
                "selector is not statically player-targeting",
            ))
        }
    }
}

impl Validate for Selector {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        if let TargetBase::Raw(_) = self.base {
            if !self.args.is_empty() {
                return Err(CommandError::new(
                    "Selector",
                    "arguments",
                    "raw selectors cannot be combined with typed arguments",
                ));
            }
            return Ok(());
        }
        if let TargetBase::Player(ref name) = self.base {
            if !self.args.is_empty() {
                return Err(CommandError::new(
                    "Selector",
                    "arguments",
                    "literal player names cannot be combined with selector arguments",
                ));
            }
            if name.is_empty()
                || name.len() > 16
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return Err(CommandError::new(
                    "Selector",
                    "player_name",
                    format!("must be 1..=16 ASCII letters, digits, or `_`, got `{name}`"),
                ));
            }
        }

        let mut singleton_keys = std::collections::BTreeSet::new();
        let mut positive_type = false;
        for arg in &self.args {
            let (key, value): (&str, Option<&str>) = match arg {
                SelectorArg::Tag(v) | SelectorArg::NotTag(v) => {
                    validate_optional_token(v, "tag")?;
                    ("tag*", None)
                }
                SelectorArg::Team(v) | SelectorArg::NotTeam(v) => {
                    validate_optional_token(v, "team")?;
                    ("team*", None)
                }
                SelectorArg::Name(v) | SelectorArg::NotName(v) => ("name*", Some(v)),
                SelectorArg::Type(v) => {
                    if positive_type {
                        return Err(CommandError::new(
                            "Selector",
                            "type",
                            "duplicate positive `type` arguments are contradictory",
                        ));
                    }
                    positive_type = true;
                    validate::resource_location_shape(
                        v.strip_prefix('#').unwrap_or(v),
                        "Selector",
                        "type",
                    )?;
                    ("type+", None)
                }
                SelectorArg::NotType(v) => {
                    validate::resource_location_shape(
                        v.strip_prefix('#').unwrap_or(v),
                        "Selector",
                        "type",
                    )?;
                    ("type-", None)
                }
                SelectorArg::Limit(v) => {
                    if !matches!(self.base, TargetBase::AllPlayers | TargetBase::AllEntities) {
                        return Err(CommandError::new(
                            "Selector",
                            "limit",
                            "`limit` is only applicable to `@a` and `@e` selector bases",
                        ));
                    }
                    if *v <= 0 {
                        return Err(CommandError::new(
                            "Selector",
                            "limit",
                            format!("selector limits must be greater than zero, got `{v}`"),
                        ));
                    }
                    ("limit", None)
                }
                SelectorArg::Sort(_) => {
                    if !matches!(self.base, TargetBase::AllPlayers | TargetBase::AllEntities) {
                        return Err(CommandError::new(
                            "Selector",
                            "sort",
                            "`sort` is only applicable to `@a` and `@e` selector bases",
                        ));
                    }
                    ("sort", None)
                }
                SelectorArg::Distance(v) => {
                    validate_range(v, "distance", true)?;
                    ("distance", None)
                }
                SelectorArg::Level(v) => {
                    validate_range(v, "level", false)?;
                    ("level", None)
                }
                SelectorArg::XRotation(v) => {
                    validate_range(v, "x_rotation", true)?;
                    ("x_rotation", None)
                }
                SelectorArg::YRotation(v) => {
                    validate_range(v, "y_rotation", true)?;
                    ("y_rotation", None)
                }
                SelectorArg::Gamemode(v) => {
                    if !matches!(
                        v.strip_prefix('!').unwrap_or(v),
                        "survival" | "creative" | "adventure" | "spectator"
                    ) {
                        return Err(CommandError::new(
                            "Selector",
                            "gamemode",
                            format!("unknown vanilla gamemode `{v}`"),
                        ));
                    }
                    ("gamemode", None)
                }
                SelectorArg::Scores(v) => {
                    validate_scores(v)?;
                    ("scores", None)
                }
                SelectorArg::Nbt(v) => {
                    validate_snbt_compound(v)?;
                    ("nbt", None)
                }
                SelectorArg::Predicate(v) => {
                    validate::resource_location_shape(
                        v.strip_prefix('!').unwrap_or(v),
                        "Selector",
                        "predicate",
                    )?;
                    ("predicate", None)
                }
                SelectorArg::X(v) => {
                    validate::finite(*v, "Selector", "x")?;
                    ("x", None)
                }
                SelectorArg::Y(v) => {
                    validate::finite(*v, "Selector", "y")?;
                    ("y", None)
                }
                SelectorArg::Z(v) => {
                    validate::finite(*v, "Selector", "z")?;
                    ("z", None)
                }
                SelectorArg::Dx(v) => {
                    validate::finite(*v, "Selector", "dx")?;
                    ("dx", None)
                }
                SelectorArg::Dy(v) => {
                    validate::finite(*v, "Selector", "dy")?;
                    ("dy", None)
                }
                SelectorArg::Dz(v) => {
                    validate::finite(*v, "Selector", "dz")?;
                    ("dz", None)
                }
            };
            if let Some(v) = value {
                validate::no_whitespace_or_control(v, "Selector", key)?;
            }
            if !key.ends_with('*')
                && !key.ends_with('-')
                && !key.ends_with('+')
                && !singleton_keys.insert(key)
            {
                return Err(CommandError::new(
                    "Selector",
                    "arguments",
                    format!("duplicate `{key}` argument"),
                ));
            }
        }
        Ok(())
    }
}

impl RenderCommand for Selector {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

fn validate_range(value: &str, field: &'static str, allow_float: bool) -> CommandResult<()> {
    validate::non_empty(value, "Selector", field)?;
    let parse = |part: &str| -> CommandResult<Option<f64>> {
        if part.is_empty() {
            return Ok(None);
        }
        let n = part.parse::<f64>().map_err(|_| {
            CommandError::new("Selector", field, format!("invalid range bound `{part}`"))
        })?;
        validate::finite(n, "Selector", field)?;
        if !allow_float && n.fract() != 0.0 {
            return Err(CommandError::new(
                "Selector",
                field,
                "range requires integer bounds",
            ));
        }
        Ok(Some(n))
    };
    let (min, max) = if let Some((a, b)) = value.split_once("..") {
        if b.contains("..") {
            return Err(CommandError::new(
                "Selector",
                field,
                "range contains more than one `..`",
            ));
        }
        (parse(a)?, parse(b)?)
    } else {
        let exact = parse(value)?;
        (exact, exact)
    };
    if min.is_none() && max.is_none() {
        return Err(CommandError::new(
            "Selector",
            field,
            "range must contain at least one bound",
        ));
    }
    if let (Some(a), Some(b)) = (min, max)
        && a > b
    {
        return Err(CommandError::new(
            "Selector",
            field,
            format!("range lower bound `{a}` exceeds upper bound `{b}`"),
        ));
    }
    if matches!(field, "distance" | "level")
        && (min.is_some_and(|v| v < 0.0) || max.is_some_and(|v| v < 0.0))
    {
        return Err(CommandError::new(
            "Selector",
            field,
            format!("{field} cannot be negative"),
        ));
    }
    Ok(())
}

fn validate_optional_token(value: &str, field: &'static str) -> CommandResult<()> {
    if value.chars().any(|c| c.is_whitespace() || c.is_control()) {
        Err(CommandError::new(
            "Selector",
            field,
            format!("must not contain whitespace or control characters, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_snbt_compound(value: &str) -> CommandResult<()> {
    validate::non_empty(value, "Selector", "nbt")?;
    if !(value.starts_with('{') && value.ends_with('}')) {
        return Err(CommandError::new(
            "Selector",
            "nbt",
            "typed NBT filters must be an SNBT compound wrapped in `{...}`",
        ));
    }
    if value.contains(['\0', '\n', '\r']) {
        return Err(CommandError::new(
            "Selector",
            "nbt",
            "SNBT selector fragments must remain on one command line",
        ));
    }
    let mut delimiters = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '{' | '[' => delimiters.push(character),
            '}' if delimiters.pop() == Some('{') => {}
            ']' if delimiters.pop() == Some('[') => {}
            '}' | ']' => {
                return Err(CommandError::new(
                    "Selector",
                    "nbt",
                    "SNBT selector fragment has an unmatched closing delimiter",
                ));
            }
            _ => {}
        }
    }
    if quote.is_some() || !delimiters.is_empty() {
        return Err(CommandError::new(
            "Selector",
            "nbt",
            "SNBT selector fragment has unbalanced quotes or delimiters",
        ));
    }
    Ok(())
}

fn validate_scores(value: &str) -> CommandResult<()> {
    validate::non_empty(value, "Selector", "scores")?;
    let mut objectives = std::collections::BTreeSet::new();
    for entry in value.split(',') {
        let Some((objective, range)) = entry.split_once('=') else {
            return Err(CommandError::new(
                "Selector",
                "scores",
                format!("expected `objective=range`, got `{entry}`"),
            ));
        };
        validate::no_whitespace_or_control(objective, "Selector", "scores.objective")?;
        if objective.len() > 16 {
            return Err(CommandError::new(
                "Selector",
                "scores.objective",
                format!("objective `{objective}` exceeds 16 characters"),
            ));
        }
        if !objectives.insert(objective) {
            return Err(CommandError::new(
                "Selector",
                "scores",
                format!("duplicate objective `{objective}`"),
            ));
        }
        validate_range(range, "scores.range", false)?;
    }
    Ok(())
}

// ── SelectorRange ─────────────────────────────────────────────────────────────

/// A typed numeric range for selector arguments such as `distance` and
/// `level` (see [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Renders to vanilla's `min..max` range syntax. At least one bound must be
/// present; use [`SelectorRange::at_least`]/[`SelectorRange::at_most`] for
/// open-ended ranges. Impossible ranges (`min > max`) and non-finite bounds
/// are not rejected at construction — they are diagnosed uniformly with all
/// other selector arguments at [`Selector::try_build`] time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectorRange {
    min: Option<f64>,
    max: Option<f64>,
}

impl SelectorRange {
    /// An exact value: `n..n`, rendered as `n`.
    pub fn exact(n: f64) -> Self {
        Self {
            min: Some(n),
            max: Some(n),
        }
    }

    /// `n..` — at least `n`.
    pub fn at_least(n: f64) -> Self {
        Self {
            min: Some(n),
            max: None,
        }
    }

    /// `..n` — at most `n`.
    pub fn at_most(n: f64) -> Self {
        Self {
            min: None,
            max: Some(n),
        }
    }

    /// `min..max` — an inclusive range.
    pub fn between(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl fmt::Display for SelectorRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_bound(v: f64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{v}")
        }
        match (self.min, self.max) {
            (Some(a), Some(b)) if a == b => fmt_bound(a, f),
            (Some(a), Some(b)) => {
                fmt_bound(a, f)?;
                write!(f, "..")?;
                fmt_bound(b, f)
            }
            (Some(a), None) => {
                fmt_bound(a, f)?;
                write!(f, "..")
            }
            (None, Some(b)) => {
                write!(f, "..")?;
                fmt_bound(b, f)
            }
            (None, None) => Ok(()),
        }
    }
}

// ── ScoreRange ───────────────────────────────────────────────────────────────

/// A typed integer range for `scores={...}` selector entries (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Deliberately distinct from [`SelectorRange`]: Minecraft scoreboard scores
/// are always 32-bit integers, so `scores={obj=1.5..3.2}` is not legal
/// vanilla syntax even though the same `min..max` grammar shape is used for
/// `distance`/`level` (which *are* floating-point). Using an `i32`-based
/// type here at the API boundary makes a fractional score range a compile
/// error instead of a malformed-selector diagnostic discovered at
/// `try_build` time.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ScoreRange",
    aliases = ["sand::cmd::ScoreRange", "sand::prelude::cmd::ScoreRange"],
    module = "sand::command",
    summary = "A typed integer range for `scores={...}` selector entries (see [#200](https://github.com/ThatOneToast/sand/issues/200)).",
    context = "A typed integer range for `scores={...}` selector entries (see [#200](https://github.com/ThatOneToast/sand/issues/200)). Deliberately distinct from [`SelectorRange`]: Minecraft scoreboard scores are always 32-bit integers, so `scores={obj=1.5..3.2}` is not legal vanilla syntax even though the same `min..max` grammar shape is used for `distance`/`level` (which *are* floating-point). Using an `i32`-based type here at the API boundary makes a fractional score range a compile error instead of a malformed-selector diagnostic discovered at `try_build` time.",
    minecraft = "Deliberately distinct from [`SelectorRange`]: Minecraft scoreboard scores are always 32-bit integers, so `scores={obj=1.5..3.2}` is not legal vanilla syntax even though the same `min..max` grammar shape is used for `distance`/`level` (which *are* floating-point). Using an `i32`-based type here at the API boundary makes a fractional score range a compile error instead of a malformed-selector diagnostic discovered at `try_build` time.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ScoreRange;",
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreRange {
    min: Option<i32>,
    max: Option<i32>,
}

impl ScoreRange {
    /// An exact value: `n..n`, rendered as `n`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::exact",
        aliases = ["sand::cmd::ScoreRange::exact", "sand::prelude::cmd::ScoreRange::exact"],
        module = "sand::command",
        kind = "method",
        summary = "An exact value: `n..n`, rendered as `n`.",
        context = "An exact value: `n..n`, rendered as `n`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "An exact value: `n..n`, rendered as `n`."),
        returns = "A `ScoreRange` configured for an exact value: `n..n`, rendered as `n`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let score_range = sand::command::ScoreRange::exact(n);\n}",
    )]
    pub fn exact(n: i32) -> Self {
        Self {
            min: Some(n),
            max: Some(n),
        }
    }

    /// `n..` — at least `n`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::at_least",
        aliases = ["sand::cmd::ScoreRange::at_least", "sand::prelude::cmd::ScoreRange::at_least"],
        module = "sand::command",
        kind = "method",
        summary = "`n..` — at least `n`.",
        context = "`n..` — at least `n`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`n..` — at least `n`."),
        returns = "A `ScoreRange` that emits the documented `n..` — at least `n` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let score_range = sand::command::ScoreRange::at_least(n);\n}",
    )]
    pub fn at_least(n: i32) -> Self {
        Self {
            min: Some(n),
            max: None,
        }
    }

    /// `..n` — at most `n`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::at_most",
        aliases = ["sand::cmd::ScoreRange::at_most", "sand::prelude::cmd::ScoreRange::at_most"],
        module = "sand::command",
        kind = "method",
        summary = "`..n` — at most `n`.",
        context = "`..n` — at most `n`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`..n` — at most `n`."),
        returns = "A `ScoreRange` that emits the documented `..n` — at most `n` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let score_range = sand::command::ScoreRange::at_most(n);\n}",
    )]
    pub fn at_most(n: i32) -> Self {
        Self {
            min: None,
            max: Some(n),
        }
    }

    /// `min..max` — an inclusive range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreRange::between",
        aliases = ["sand::cmd::ScoreRange::between", "sand::prelude::cmd::ScoreRange::between"],
        module = "sand::command",
        kind = "method",
        summary = "`min..max` — an inclusive range.",
        context = "`min..max` — an inclusive range. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(min = "`min` provides the inclusive lower bound used to emit the documented `min..max` — an inclusive range form.", max = "`max` provides the inclusive upper bound used to emit the documented `min..max` — an inclusive range form."),
        returns = "A `ScoreRange` that emits the documented `min..max` — an inclusive range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(min: i32, max: i32)  {\n    let score_range = sand::command::ScoreRange::between(min, max);\n}",
    )]
    pub fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl fmt::Display for ScoreRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (Some(a), Some(b)) if a == b => write!(f, "{a}"),
            (Some(a), Some(b)) => write!(f, "{a}..{b}"),
            (Some(a), None) => write!(f, "{a}.."),
            (None, Some(b)) => write!(f, "..{b}"),
            (None, None) => Ok(()),
        }
    }
}

// ── SelectorScores ───────────────────────────────────────────────────────────

/// A typed `scores={...}` selector filter map (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Entries are rendered in insertion order, so equivalent construction order
/// always produces an identical rendered selector. Values are [`ScoreRange`]
/// (integer), not [`SelectorRange`] (float) — scoreboard scores are always
/// integers in vanilla Minecraft.
///
/// ```
/// use sand_commands::selector::{SelectorScores, ScoreRange};
///
/// let scores = SelectorScores::new()
///     .with("kills", ScoreRange::between(1, 10))
///     .with("deaths", ScoreRange::exact(0));
/// assert_eq!(scores.to_string(), "kills=1..10,deaths=0");
/// ```
#[derive(Debug, Clone, Default)]
pub struct SelectorScores {
    entries: Vec<(String, ScoreRange)>,
}

impl SelectorScores {
    /// Create an empty score filter map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `objective=range` to this filter map.
    pub fn with(mut self, objective: impl Into<String>, range: ScoreRange) -> Self {
        self.entries.push((objective.into(), range));
        self
    }
}

impl fmt::Display for SelectorScores {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = self
            .entries
            .iter()
            .map(|(objective, range)| format!("{objective}={range}"))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{rendered}")
    }
}

// ── PredicateId ──────────────────────────────────────────────────────────────

/// A typed `predicate=<namespace:path>` identifier (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
///
/// Resource-location shape is validated at [`Selector::try_build`] time
/// (consistent with [`Selector::predicate`]/[`Selector::predicate_raw`]),
/// not at construction, so this stays const/static-friendly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateId {
    location: String,
    negated: bool,
}

impl PredicateId {
    /// Create a predicate ID from a `namespace:path` resource location.
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            negated: false,
        }
    }

    /// `predicate=!<id>` — negate this predicate filter.
    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }
}

impl fmt::Display for PredicateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negated {
            write!(f, "!{}", self.location)
        } else {
            write!(f, "{}", self.location)
        }
    }
}

// ── EntityTag / TeamName ─────────────────────────────────────────────────────

/// A typed selector `tag` value (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)). Whitespace/
/// control-character validity is checked at [`Selector::try_build`] time,
/// matching [`Selector::tag`]'s existing validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::EntityTag",
    aliases = ["sand::cmd::EntityTag", "sand::prelude::cmd::EntityTag"],
    module = "sand::command",
    summary = "A typed selector `tag` value (see [#200](https://github.com/ThatOneToast/sand/issues/200)). Whitespace/ control-character validity is checked at [`Selector::try_build`] time, matching [`Selector::tag`]'s existing validation.",
    context = "A typed selector `tag` value (see [#200](https://github.com/ThatOneToast/sand/issues/200)). Whitespace/ control-character validity is checked at [`Selector::try_build`] time, matching [`Selector::tag`]'s existing validation. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::EntityTag;",
)]
pub struct EntityTag(String);

impl EntityTag {
    /// Wrap a tag value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::EntityTag::new",
        aliases = ["sand::cmd::EntityTag::new", "sand::prelude::cmd::EntityTag::new"],
        module = "sand::command",
        kind = "method",
        summary = "Wrap a tag value.",
        context = "Wrap a tag value. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(tag = "`tag` provides the tag wrapped when creating a tag value."),
        returns = "An `EntityTag` wrapping a tag value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag: impl Into < String >)  {\n    let entity_tag = sand::command::EntityTag::new(tag);\n}",
    )]
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for EntityTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A typed selector `team` value (see
/// [#200](https://github.com/ThatOneToast/sand/issues/200)).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::TeamName",
    aliases = ["sand::cmd::TeamName", "sand::prelude::cmd::TeamName"],
    module = "sand::command",
    summary = "A typed selector `team` value (see [#200](https://github.com/ThatOneToast/sand/issues/200)).",
    context = "A typed selector `team` value (see [#200](https://github.com/ThatOneToast/sand/issues/200)). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::TeamName;",
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamName(String);

impl TeamName {
    /// Wrap a team name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TeamName::new",
        aliases = ["sand::cmd::TeamName::new", "sand::prelude::cmd::TeamName::new"],
        module = "sand::command",
        kind = "method",
        summary = "Wrap a team name.",
        context = "Wrap a team name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` provides the team wrapped when creating a team name."),
        returns = "A `TeamName` wrapping a team name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(team: impl Into < String >)  {\n    let team_name = sand::command::TeamName::new(team);\n}",
    )]
    pub fn new(team: impl Into<String>) -> Self {
        Self(team.into())
    }

    fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for TeamName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── GameMode ──────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::GameMode",
    aliases = ["sand::cmd::GameMode", "sand::prelude::GameMode", "sand::prelude::cmd::GameMode"],
    module = "sand::command",
    summary = "Minecraft player game mode.",
    context = "Minecraft player game mode. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::GameMode;",
    variants(Adventure = "`adventure` — survival-like with block-break restrictions.", Creative = "`creative` — infinite resources and flight.", Spectator = "`spectator` — observe-only mode.", Survival = "`survival` — normal gameplay."),
)]
/// Minecraft player game mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// `survival` — normal gameplay.
    Survival,
    /// `creative` — infinite resources and flight.
    Creative,
    /// `adventure` — survival-like with block-break restrictions.
    Adventure,
    /// `spectator` — observe-only mode.
    Spectator,
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameMode::Survival => write!(f, "survival"),
            GameMode::Creative => write!(f, "creative"),
            GameMode::Adventure => write!(f, "adventure"),
            GameMode::Spectator => write!(f, "spectator"),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_selectors() {
        assert_eq!(Selector::all_players().to_string(), "@a");
        assert_eq!(Selector::all_entities().to_string(), "@e");
        assert_eq!(Selector::self_().to_string(), "@s");
        assert_eq!(Selector::nearest_player().to_string(), "@p");
        assert_eq!(Selector::random_player().to_string(), "@r");
        assert_eq!(Selector::player("Steve").to_string(), "Steve");
    }

    #[test]
    fn with_args() {
        let s = Selector::all_players().tag("ready").limit(1);
        assert_eq!(s.to_string(), "@a[tag=ready,limit=1]");
    }

    #[test]
    fn multiple_args() {
        let s = Selector::all_entities()
            .entity_type("minecraft:zombie")
            .not_tag("killed")
            .limit(5);
        assert_eq!(
            s.to_string(),
            "@e[type=minecraft:zombie,tag=!killed,limit=5]"
        );
    }

    #[test]
    fn negation() {
        assert_eq!(
            Selector::all_players().not_team("red").to_string(),
            "@a[team=!red]"
        );
    }

    #[test]
    fn typed_entity_targets_render_stably() {
        let targets = EntityTargets::nearby(5.0)
            .excluding_players()
            .excluding_self();
        assert_eq!(
            targets.to_string(),
            "@e[distance=0.1..5,type=!minecraft:player]"
        );
    }

    #[test]
    fn many_entity_limit_converts_to_single() {
        let target = EntityTargets::all()
            .entity_type("minecraft:zombie")
            .nearest();
        assert_eq!(
            target.to_string(),
            "@e[type=minecraft:zombie,sort=nearest,limit=1]"
        );
    }

    // ── Selector argument golden tests ────────────────────────────────────────

    #[test]
    fn scores_arg() {
        // scores() wraps the argument in { } automatically
        let s = Selector::all_players().scores("kills=1..10,deaths=0");
        assert_eq!(s.to_string(), "@a[scores={kills=1..10,deaths=0}]");
    }

    #[test]
    fn nbt_arg() {
        let s = Selector::all_entities().nbt("{CustomName:\"Boss\"}");
        assert_eq!(s.to_string(), r#"@e[nbt={CustomName:"Boss"}]"#);
    }

    #[test]
    fn predicate_arg() {
        let s = Selector::all_players().predicate("my_pack:is_sneaking");
        assert_eq!(s.to_string(), "@a[predicate=my_pack:is_sneaking]");
    }

    #[test]
    fn gamemode_arg() {
        let s = Selector::all_players().gamemode("survival");
        assert_eq!(s.to_string(), "@a[gamemode=survival]");
    }

    #[test]
    fn level_range_arg() {
        let s = Selector::all_players().level("10..30");
        assert_eq!(s.to_string(), "@a[level=10..30]");
    }

    #[test]
    fn distance_range_arg() {
        let s = Selector::all_entities().distance_range(0.5, 10.0);
        assert_eq!(s.to_string(), "@e[distance=0.5..10]");
    }

    #[test]
    fn distance_max_arg() {
        let s = Selector::nearest_player().distance_max(16.0);
        assert_eq!(s.to_string(), "@p[distance=..16]");
    }

    #[test]
    fn sort_random_arg() {
        let s = Selector::all_entities()
            .entity_type("minecraft:cow")
            .sort(SortOrder::Random)
            .limit(1);
        assert_eq!(s.to_string(), "@e[type=minecraft:cow,sort=random,limit=1]");
    }

    #[test]
    fn volume_box_arg() {
        let s = Selector::all_entities().volume(3.0, 1.0, 3.0);
        assert_eq!(s.to_string(), "@e[dx=3,dy=1,dz=3]");
    }

    #[test]
    fn at_pos_shifts_origin() {
        let s = Selector::all_entities().at_pos(10.0, 64.0, -20.0);
        assert_eq!(s.to_string(), "@e[x=10,y=64,z=-20]");
    }

    #[test]
    fn not_player_type_arg() {
        let s = Selector::all_entities()
            .not_player()
            .limit(3)
            .sort(SortOrder::Nearest);
        assert_eq!(
            s.to_string(),
            "@e[type=!minecraft:player,limit=3,sort=nearest]"
        );
    }

    #[test]
    fn name_and_not_name() {
        let s = Selector::all_players().name("Steve");
        assert_eq!(s.to_string(), "@a[name=Steve]");

        let s = Selector::all_players().not_name("Notch");
        assert_eq!(s.to_string(), "@a[name=!Notch]");
    }

    #[test]
    fn validation_rejects_invalid_limits_ranges_and_names() {
        assert!(Selector::all_players().limit(0).try_build().is_err());
        assert!(
            Selector::all_entities()
                .distance_range(5.0, 1.0)
                .try_build()
                .is_err()
        );
        assert!(
            Selector::all_entities()
                .distance_max(f64::NAN)
                .try_build()
                .is_err()
        );
        assert!(Selector::player("").try_build().is_err());
        assert!(Selector::player("has space").try_build().is_err());
        assert!(
            Selector::all_entities()
                .distance_max(-1.0)
                .try_build()
                .is_err()
        );
        assert!(Selector::all_players().level("-1..").try_build().is_err());
        assert!(
            Selector::all_entities()
                .nbt("{broken:[1,2}")
                .try_build()
                .is_err()
        );
        assert!(
            Selector::all_entities()
                .gamemode("!creative")
                .try_build()
                .is_ok()
        );
        assert!(
            Selector::all_entities()
                .predicate("!pack:ready")
                .try_build()
                .is_ok()
        );
        assert!(
            Selector::all_entities()
                .entity_type("#pack:mobs")
                .try_build()
                .is_ok()
        );
        assert!(Selector::all_entities().tag("").try_build().is_ok());
        assert!(Selector::self_().limit(1).try_build().is_err());
    }

    #[test]
    fn gamemode_typed_matches_string_gamemode_for_valid_input() {
        assert_eq!(
            Selector::all_players()
                .gamemode_typed(GameMode::Survival)
                .try_build()
                .unwrap(),
            Selector::all_players()
                .gamemode("survival")
                .try_build()
                .unwrap()
        );
    }

    #[test]
    fn not_gamemode_typed_renders_negation() {
        let selector = Selector::all_players().not_gamemode_typed(GameMode::Creative);
        assert_eq!(selector.try_build().unwrap(), "@a[gamemode=!creative]");
    }

    #[test]
    fn narrowing_is_fallible_and_safe_widening_remains_infallible() {
        assert!(SingleEntity::try_from(Selector::all_entities()).is_err());
        assert!(SinglePlayer::try_from(Selector::all_entities().limit(1)).is_err());
        assert!(SingleEntity::try_from(Selector::all_entities().limit(1)).is_ok());
        assert!(EntityTargets::all().limit(2).is_err());
        let entity: SingleEntity = SinglePlayer::self_().into();
        assert_eq!(entity.to_string(), "@s");
    }

    #[test]
    fn raw_selector_escape_hatch_remains_verbatim() {
        assert_eq!(
            Selector::raw("@e[modded_filter={x:1}]")
                .try_build()
                .unwrap(),
            "@e[modded_filter={x:1}]"
        );
    }

    // ── #200: typed selector filter helpers ───────────────────────────────

    #[test]
    fn distance_typed_matches_string_variants() {
        assert_eq!(
            Selector::all_entities()
                .distance_typed(SelectorRange::at_most(16.0))
                .try_build()
                .unwrap(),
            Selector::all_entities()
                .distance_max(16.0)
                .try_build()
                .unwrap()
        );
        assert_eq!(
            Selector::all_entities()
                .distance_typed(SelectorRange::between(0.5, 10.0))
                .try_build()
                .unwrap(),
            "@e[distance=0.5..10]"
        );
        assert_eq!(
            Selector::all_entities()
                .distance_typed(SelectorRange::at_least(2.0))
                .try_build()
                .unwrap(),
            "@e[distance=2..]"
        );
    }

    #[test]
    fn level_typed_renders_same_as_string_level() {
        assert_eq!(
            Selector::all_players()
                .level_typed(SelectorRange::between(10.0, 30.0))
                .try_build()
                .unwrap(),
            Selector::all_players().level("10..30").try_build().unwrap()
        );
    }

    #[test]
    fn selector_range_impossible_range_is_a_diagnostic_not_a_panic() {
        let err = Selector::all_entities()
            .distance_typed(SelectorRange::between(10.0, 1.0))
            .try_build()
            .unwrap_err();
        assert!(err.to_string().contains("distance"), "{err}");
    }

    #[test]
    fn scores_typed_matches_string_scores() {
        let typed = Selector::all_players()
            .scores_typed(
                SelectorScores::new()
                    .with("kills", ScoreRange::between(1, 10))
                    .with("deaths", ScoreRange::exact(0)),
            )
            .try_build()
            .unwrap();
        let stringly = Selector::all_players()
            .scores("kills=1..10,deaths=0")
            .try_build()
            .unwrap();
        assert_eq!(typed, stringly);
        assert_eq!(typed, "@a[scores={kills=1..10,deaths=0}]");
    }

    #[test]
    fn scores_typed_duplicate_objective_is_a_diagnostic() {
        let err = Selector::all_players()
            .scores_typed(
                SelectorScores::new()
                    .with("kills", ScoreRange::exact(1))
                    .with("kills", ScoreRange::exact(2)),
            )
            .try_build()
            .unwrap_err();
        assert!(err.to_string().contains("duplicate"), "{err}");
    }

    #[test]
    fn score_range_is_integer_typed_not_float() {
        // #200 review finding: `scores={...}` values must be integers in
        // vanilla Minecraft — `ScoreRange` uses `i32` bounds so a fractional
        // score range is a compile error, not a malformed-selector
        // diagnostic discovered later at `try_build` time. This test just
        // pins the rendered (always-integer) output shape.
        assert_eq!(ScoreRange::exact(0).to_string(), "0");
        assert_eq!(ScoreRange::between(-5, 5).to_string(), "-5..5");
        assert_eq!(ScoreRange::at_least(3).to_string(), "3..");
        assert_eq!(ScoreRange::at_most(-1).to_string(), "..-1");
    }

    #[test]
    fn predicate_id_matches_string_predicate() {
        assert_eq!(
            Selector::all_players()
                .predicate_id(PredicateId::new("my_pack:is_sneaking"))
                .try_build()
                .unwrap(),
            Selector::all_players()
                .predicate("my_pack:is_sneaking")
                .try_build()
                .unwrap()
        );
        assert_eq!(
            Selector::all_players()
                .predicate_id(PredicateId::new("my_pack:is_sneaking").negated())
                .try_build()
                .unwrap(),
            "@a[predicate=!my_pack:is_sneaking]"
        );
    }

    #[test]
    fn tag_typed_and_team_typed_match_string_variants() {
        assert_eq!(
            Selector::all_players()
                .tag_typed(EntityTag::new("ready"))
                .try_build()
                .unwrap(),
            Selector::all_players().tag("ready").try_build().unwrap()
        );
        assert_eq!(
            Selector::all_players()
                .team_typed(TeamName::new("red"))
                .try_build()
                .unwrap(),
            Selector::all_players().team("red").try_build().unwrap()
        );
    }

    #[test]
    fn raw_aliases_render_identically_to_normal_methods() {
        assert_eq!(
            Selector::all_entities()
                .nbt_raw("{CustomName:\"Boss\"}")
                .to_string(),
            Selector::all_entities()
                .nbt("{CustomName:\"Boss\"}")
                .to_string()
        );
        assert_eq!(
            Selector::all_players().scores_raw("kills=1..").to_string(),
            Selector::all_players().scores("kills=1..").to_string()
        );
        assert_eq!(
            Selector::all_players()
                .predicate_raw("pack:ready")
                .to_string(),
            Selector::all_players().predicate("pack:ready").to_string()
        );
    }

    #[test]
    fn selector_construction_order_is_deterministic() {
        // #200/#173: rebuilding the same selector from scratch, in the same
        // call order, must always render identically — no run-to-run
        // variance. `Selector`'s args and `SelectorScores`'s entries are
        // both backed by `Vec` (insertion order), not a hasher-seeded map,
        // so this is not merely "the same closure returns the same string
        // twice": each `build()` call constructs fresh `Vec`s from scratch,
        // and a `HashMap`-backed regression (each instance gets an
        // independently randomized iteration order in std) would show up as
        // flaky inequality across iterations. Loop many times for
        // confidence instead of relying on two calls that could coincide by
        // chance.
        let build = || {
            Selector::all_entities()
                .entity_type("minecraft:zombie")
                .tag("elite")
                .distance_typed(SelectorRange::at_most(20.0))
                .scores_typed(
                    SelectorScores::new()
                        .with("threat", ScoreRange::at_least(5))
                        .with("armor", ScoreRange::between(0, 3))
                        .with("kills", ScoreRange::exact(0)),
                )
                .limit(3)
                .to_string()
        };
        let expected = "@e[type=minecraft:zombie,tag=elite,distance=..20,scores={threat=5..,armor=0..3,kills=0},limit=3]";
        let first = build();
        assert_eq!(first, expected);
        for _ in 0..64 {
            assert_eq!(build(), first);
        }

        // Construction order is caller-controlled and semantically
        // significant for `SelectorScores` (it is not canonicalized/sorted),
        // so two *different* insertion orders are expected to render
        // differently from each other — while each remains internally
        // deterministic across repeated builds.
        let reordered = || {
            Selector::all_entities()
                .entity_type("minecraft:zombie")
                .tag("elite")
                .distance_typed(SelectorRange::at_most(20.0))
                .scores_typed(
                    SelectorScores::new()
                        .with("kills", ScoreRange::exact(0))
                        .with("armor", ScoreRange::between(0, 3))
                        .with("threat", ScoreRange::at_least(5)),
                )
                .limit(3)
                .to_string()
        };
        let reordered_first = reordered();
        assert_ne!(reordered_first, first);
        for _ in 0..64 {
            assert_eq!(reordered(), reordered_first);
        }
    }
}
