//! Owned native interaction hitboxes using canonical Lifetime cleanup.
use super::{Lifetime, Menu, identity, runtime};
use crate::{
    IntoCommands,
    entity::{EntityKind, PlayerKind},
    ir::Actions,
    registration::{DatapackRegistration, IntoDatapack},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use std::sync::Arc;

/// An invisible, stationary interaction hitbox owned by a Lifetime instance.
///
/// Start it at the desired object position using the lifetime's origin. It
/// records the latest use and attack between loaded ticks, prioritizing use when
/// both are pending. It is not a lossless event
/// queue. Callbacks execute as the recorded player at the object's position;
/// canonical player State and world-relative block operations can be composed
/// directly. Each record is consumed before callbacks. Lifetime cancellation,
/// expiration and owner loss remove the hitbox. No chunks are force-loaded.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable", module = "sand::systems",
summary = "A lifetime-owned native interaction hitbox.", context = "Register the definition before use. Lifetime supplies ownership and cleanup. Callbacks execute as the actor at the stationary object position.", minecraft = "Consumes latest native interaction/attack records before actor callbacks; multiple clicks between observations may coalesce. Loaded-tick observation does not force chunks.", use_when = ["Authoring clickable world objects"], avoid_when = ["Lossless input queues or cancelling native block use"],
example = "use sand::systems::Interactable;")]
pub struct Interactable<K: EntityKind = PlayerKind> {
    lifetime: Lifetime<K>,
    width: f32,
    height: f32,
    range: f32,
    use_action: Option<Arc<dyn Fn() -> Actions>>,
    attack_action: Option<Arc<dyn Fn() -> Actions>>,
    menu: Option<Menu>,
}
impl<K: EntityKind> Interactable<K> {
    /// Create a one-block hitbox with a six-block actor validation radius.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable::new", module = "sand::systems",
summary = "Create an owned interaction hitbox.", context = "Register the definition before use. Lifetime supplies ownership and cleanup. Callbacks execute as the actor at the stationary object position.", minecraft = "Consumes latest native interaction/attack records before actor callbacks; multiple clicks between observations may coalesce. Loaded-tick observation does not force chunks.", use_when = ["Authoring clickable world objects"], avoid_when = ["Lossless input queues or cancelling native block use"],
params(lifetime = "Canonical lifetime controlling capacity, origin, duration and cleanup."),
returns = "The configured definition or retained lifecycle action.",
example = "use sand::systems::Interactable;")]
    pub fn new(lifetime: Lifetime<K>) -> Self {
        Self {
            lifetime,
            width: 1.0,
            height: 1.0,
            range: 6.0,
            use_action: None,
            attack_action: None,
            menu: None,
        }
    }
    /// Set finite positive hitbox dimensions and maximum actor distance.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable::bounds", module = "sand::systems",
summary = "Set hitbox dimensions and actor validation radius.", context = "Register the definition before use. Lifetime supplies ownership and cleanup. Callbacks execute as the actor at the stationary object position.", minecraft = "Consumes latest native interaction/attack records before actor callbacks; multiple clicks between observations may coalesce. Loaded-tick observation does not force chunks.", use_when = ["Authoring clickable world objects"], avoid_when = ["Lossless input queues or cancelling native block use"],
params(width = "Positive finite width in blocks.", height = "Positive finite height in blocks.", range = "Positive finite distance from object to actor."),
returns = "The configured definition or retained lifecycle action.",
example = "use sand::systems::Interactable;")]
    pub fn bounds(mut self, width: f32, height: f32, range: f32) -> Self {
        self.width = width;
        self.height = height;
        self.range = range;
        self
    }
    /// Handle the latest observed right click as its player at the object.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable::on_use", module = "sand::systems",
summary = "Handle the latest observed use at the object.", context = "Register the definition before use. Lifetime supplies ownership and cleanup. Callbacks execute as the actor at the stationary object position.", minecraft = "Consumes latest native interaction/attack records before actor callbacks; multiple clicks between observations may coalesce. Loaded-tick observation does not force chunks.", use_when = ["Authoring clickable world objects"], avoid_when = ["Lossless input queues or cancelling native block use"],
params(action = "Gameplay actions executed as the player at the object position."),
returns = "The configured definition or retained lifecycle action.",
example = "use sand::systems::Interactable;")]
    pub fn on_use<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.use_action = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Register and open a menu bound to the exact clicked lifetime instance.
    /// Do not register the menu separately or combine this with `on_use`.
    /// Leaving the configured range, changing dimension, or removing the hitbox
    /// invalidates the session. Choices run as the player at the object.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable::opens_menu", module = "sand::systems",
summary = "Open a menu bound to the exact clicked object instance.", context = "Registers the menu automatically. Mutually exclusive with on_use. Navigation preserves the binding; choices retain the player executor at the object position.", minecraft = "Reuses the lifetime instance score and validates the loaded hitbox in the player's dimension and interaction range before dispatch.", use_when = ["Authoring object-bound shops and conversations"], avoid_when = ["Menus that remain usable after leaving the object"], params(menu = "Unique menu definition registered by this interactable."), returns = "The configured interactable.", example = "use sand::systems::{Interactable, Menu};")]
    pub fn opens_menu(mut self, menu: Menu) -> Self {
        self.menu = Some(menu);
        self
    }
    /// Handle the latest observed attack as its player at the object.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable::on_attack", module = "sand::systems",
summary = "Handle the latest observed attack at the object.", context = "Register the definition before use. Lifetime supplies ownership and cleanup. Callbacks execute as the actor at the stationary object position.", minecraft = "Consumes latest native interaction/attack records before actor callbacks; multiple clicks between observations may coalesce. Loaded-tick observation does not force chunks.", use_when = ["Authoring clickable world objects"], avoid_when = ["Lossless input queues or cancelling native block use"],
params(action = "Gameplay actions executed as the player at the object position."),
returns = "The configured definition or retained lifecycle action.",
example = "use sand::systems::Interactable;")]
    pub fn on_attack<R: IntoCommands>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.attack_action = Some(Arc::new(move || action().into_commands()));
        self
    }
    /// Start an independently owned hitbox at the configured lifetime origin.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable::start", module = "sand::systems",
summary = "Start a lifetime-owned hitbox.", context = "Register the definition before use. Lifetime supplies ownership and cleanup. Callbacks execute as the actor at the stationary object position.", minecraft = "Consumes latest native interaction/attack records before actor callbacks; multiple clicks between observations may coalesce. Loaded-tick observation does not force chunks.", use_when = ["Authoring clickable world objects"], avoid_when = ["Lossless input queues or cancelling native block use"],
returns = "The configured definition or retained lifecycle action.",
example = "use sand::systems::Interactable;")]
    pub fn start(&self) -> Actions {
        self.lifetime.start()
    }
    /// Cancel this owner's instances and remove their loaded hitboxes.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Interactable::cancel", module = "sand::systems",
summary = "Cancel owned hitboxes and perform cleanup.", context = "Register the definition before use. Lifetime supplies ownership and cleanup. Callbacks execute as the actor at the stationary object position.", minecraft = "Consumes latest native interaction/attack records before actor callbacks; multiple clicks between observations may coalesce. Loaded-tick observation does not force chunks.", use_when = ["Authoring clickable world objects"], avoid_when = ["Lossless input queues or cancelling native block use"],
returns = "The configured definition or retained lifecycle action.",
example = "use sand::systems::Interactable;")]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl<K: EntityKind> AsRef<Lifetime<K>> for Interactable<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        &self.lifetime
    }
}
impl<K: EntityKind> IntoDatapack for Interactable<K> {
    fn into_datapack(mut self) -> DatapackRegistration {
        runtime::hooks(
            &mut self.lifetime,
            "interaction_init",
            "interaction_cleanup",
            "interaction_step",
        );
        let mut registration = self.lifetime.clone().into_datapack();
        if let Some(mut menu) = self.menu.clone() {
            menu.object = Some((self.lifetime.id.clone(), self.range));
            registration = registration.merge(menu);
        }
        for part in [
            "init",
            "spawn",
            "configure",
            "cleanup",
            "remove",
            "destroy",
            "step",
            "scan",
            "observe",
            "use",
            "attack",
            "menu_capture",
            "menu",
            "use_body",
            "attack_body",
        ] {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.lifetime.id,
                &format!("interaction_{part}"),
                move || definition.build(part),
            ));
        }
        registration
    }
}
impl<K: EntityKind> Interactable<K> {
    fn build(&self, part: &str) -> CommandResult<Actions> {
        for (field, value) in [
            ("width", self.width),
            ("height", self.height),
            ("range", self.range),
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(CommandError::new(
                    "interactable",
                    field,
                    "must be finite and positive",
                ));
            }
        }
        if self.menu.is_some() && self.use_action.is_some() {
            return Err(CommandError::new(
                "interactable",
                "menu",
                "opens_menu and on_use are mutually exclusive",
            ));
        }
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let group = group_objective(&id)?;
        crate::function::request_numeric_objectives([group.clone()])?;
        let tag = format!("{}.interaction", identity::task_tag(&id)?);
        let function =
            |part: &str| identity::resource(&id, &format!("interaction_{part}")).to_string();
        let select =
            format!("@e[type=minecraft:interaction,tag={tag},scores={{{group}=$(instance)}}]");
        let lines = match part {
            "init" => vec![format!(
                "function {} with entity @s data",
                function("spawn")
            )],
            "spawn" => vec![format!(
                "$execute summon minecraft:interaction run function {} {{instance:$(instance)}}",
                function("configure")
            )],
            "configure" => vec![
                format!("tag @s add {tag}"),
                format!("$scoreboard players set @s {group} $(instance)"),
                format!(
                    "data merge entity @s {{width:{}f,height:{}f,response:true}}",
                    self.width, self.height
                ),
            ],
            "cleanup" => vec![format!(
                "function {} with entity @s data",
                function("remove")
            )],
            "remove" => vec![format!(
                "$execute as {select} run function {}",
                function("destroy")
            )],
            "destroy" => vec![
                format!("scoreboard players reset @s {group}"),
                "kill @s".into(),
            ],
            "step" => vec![format!("function {} with entity @s data", function("scan"))],
            "scan" => vec![format!(
                "$execute as {select} at @s run function {}",
                function("observe")
            )],
            "observe" if self.menu.is_some() => vec![
                format!(
                    "execute if data entity @s interaction run return run function {}",
                    function("menu_capture")
                ),
                format!(
                    "execute if data entity @s attack run function {} with entity @s attack",
                    function("attack")
                ),
            ],
            "menu_capture" => vec![
                format!("data modify storage {id} menu_event set from entity @s interaction"),
                format!(
                    "execute store result storage {id} menu_event.instance int 1 run scoreboard players get @s {group}"
                ),
                format!("data remove entity @s interaction"),
                format!("function {} with storage {id} menu_event", function("menu")),
            ],
            "menu" => {
                let mut lines = vec![format!("data remove storage {id} menu_event")];
                if let Some(menu) = &self.menu {
                    let menu_id = crate::function::resolve_export_location(&menu.id)?;
                    lines.push(format!("$execute as @a[nbt={{UUID:$(player)}},distance=..{},limit=1] run function {} {{instance:$(instance)}}", self.range, identity::resource(&menu_id, "menu_open_bound")));
                }
                lines
            }
            "observe" => vec![
                format!(
                    "execute if data entity @s interaction run return run function {} with entity @s interaction",
                    function("use")
                ),
                format!(
                    "execute if data entity @s attack run function {} with entity @s attack",
                    function("attack")
                ),
            ],
            "use" | "attack" => {
                let field = if part == "use" {
                    "interaction"
                } else {
                    "attack"
                };
                vec![
                    format!("data remove entity @s {field}"),
                    format!(
                        "$execute as @a[nbt={{UUID:$(player)}},distance=..{},limit=1] run function {}",
                        self.range,
                        function(&format!("{part}_body"))
                    ),
                ]
            }
            "use_body" => {
                return Ok(self
                    .use_action
                    .as_ref()
                    .map(|f| runtime::isolated(f()))
                    .unwrap_or_default());
            }
            "attack_body" => {
                return Ok(self
                    .attack_action
                    .as_ref()
                    .map(|f| runtime::isolated(f()))
                    .unwrap_or_default());
            }
            _ => unreachable!(),
        };
        Ok(lines.into_commands())
    }
}

pub(super) fn group_objective(
    id: &sand_components::ResourceLocation,
) -> CommandResult<ObjectiveName> {
    let id = crate::function::resolve_export_location(id)?;
    Ok(ObjectiveName::logical(format!(
        "sand interaction {id} group"
    )))
}
