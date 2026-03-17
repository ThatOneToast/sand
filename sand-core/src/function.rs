/// Descriptor for a datapack function registered via `#[sand_macros::function]`.
///
/// All descriptors submitted with [`inventory::submit!`] are collected at
/// program startup and iterable via [`inventory::iter::<FunctionDescriptor>`].
///
/// # Fields
/// - `path` — the resource location *path* component (e.g. `"hello_world"`,
///   `"utils/tick"`). The namespace is applied by the caller at build time.
/// - `make` — a zero-argument factory function that returns the list of
///   command strings for this function. Using a factory enables both static
///   string literals and dynamic [`crate::Command`] builder values.
pub struct FunctionDescriptor {
    pub path: &'static str,
    pub make: fn() -> Vec<String>,
}

inventory::collect!(FunctionDescriptor);

/// Registry entry for a `#[component]`-annotated function.
///
/// The `make` fn pointer is a zero-argument function that constructs the
/// component and boxes it as a trait object. Registered at link time via
/// `inventory::submit!` — no user wiring needed.
pub struct ComponentFactory {
    /// Factory function that returns a boxed datapack component.
    pub make: fn() -> Box<dyn crate::DatapackComponent>,
}
inventory::collect!(ComponentFactory);

/// Registers a function as an entry in a Minecraft function tag.
///
/// Produced by `#[component(Tick)]`, `#[component(Load)]`, and
/// `#[component(Tag = "ns:name")]`. During `sand build` all descriptors for
/// the same `tag` are merged into a single tag JSON file:
///
/// | Variant | `tag` value | Output file |
/// |---|---|---|
/// | `Tick` | `"minecraft:tick"` | `data/minecraft/tags/function/tick.json` |
/// | `Load` | `"minecraft:load"` | `data/minecraft/tags/function/load.json` |
/// | `Tag = "ns:name"` | `"ns:name"` | `data/ns/tags/function/name.json` |
pub struct FunctionTagDescriptor {
    /// Full tag resource location, e.g. `"minecraft:tick"`.
    pub tag: &'static str,
    /// Function path component (namespace applied at build time), e.g. `"my_tick"`.
    pub function_path: &'static str,
}
inventory::collect!(FunctionTagDescriptor);

/// How a Sand event is dispatched at runtime.
///
/// Most events use an advancement trigger (`Advancement`). Death events with no
/// entity/killing_blow filter use `DeathTick`, which detects all deaths (including
/// fall damage, drowning, `/kill`, etc.) via a `deathCount` scoreboard tick loop.
pub enum EventDispatch {
    /// Classic advancement-backed event.
    ///
    /// Fires when the player completes the advancement criteria. If `revoke` is
    /// `true`, `advancement revoke @s only <id>` is prepended to the function so
    /// the event can fire again next time the trigger is met.
    Advancement {
        make_trigger: fn() -> crate::AdvancementTrigger,
        revoke: bool,
    },
    /// Tick-loop death detection via the `deathCount` scoreboard criterion.
    ///
    /// Fires for **all** player deaths (mobs, fall, fire, void, `/kill`, …).
    /// Sand generates a single `__sand_death_check` tick function that runs
    /// `execute as @a[scores={__sand_dc=1..}]` and dispatches to each handler.
    DeathTick,
}

/// Descriptor for a function registered via `#[sand_macros::event]`.
///
/// Collected via [`inventory::iter::<EventDescriptor>`] at export time.
///
/// # Fields
/// - `path` — function resource location path (no namespace), e.g. `"on_join"`
/// - `id_override` — optional full advancement ID override (advancement dispatch only)
/// - `make` — factory that returns the Vec<String> of mcfunction commands
/// - `dispatch` — whether to use an advancement trigger or the DeathTick tick loop
pub struct EventDescriptor {
    pub path: &'static str,
    pub id_override: Option<&'static str>,
    pub make: fn() -> Vec<String>,
    pub dispatch: EventDispatch,
}
inventory::collect!(EventDescriptor);

/// Which inventory slot to watch for [`ArmorEventDescriptor`] events.
///
/// Slot IDs match Minecraft's NBT slot bytes:
/// `Head=103b`, `Chest=102b`, `Legs=101b`, `Feet=100b`, `Offhand=-106b`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    /// Helmet slot. NBT: `Slot:103b`
    Head,
    /// Chestplate slot. NBT: `Slot:102b`
    Chest,
    /// Leggings slot. NBT: `Slot:101b`
    Legs,
    /// Boots slot. NBT: `Slot:100b`
    Feet,
    /// Offhand slot. NBT: `Slot:-106b`
    Offhand,
}

impl ArmorSlot {
    /// Get the NBT slot byte for this armor slot.
    pub fn slot_byte(self) -> i8 {
        match self {
            ArmorSlot::Head => 103,
            ArmorSlot::Chest => 102,
            ArmorSlot::Legs => 101,
            ArmorSlot::Feet => 100,
            ArmorSlot::Offhand => -106,
        }
    }

    /// Get the tag name segment for this armor slot (used in entity tag names).
    pub fn tag_name_segment(self) -> &'static str {
        match self {
            ArmorSlot::Head => "head",
            ArmorSlot::Chest => "chest",
            ArmorSlot::Legs => "legs",
            ArmorSlot::Feet => "feet",
            ArmorSlot::Offhand => "offhand",
        }
    }

    /// Slot name for `execute if items entity @s <slot>`.
    pub fn slot_name(self) -> &'static str {
        match self {
            ArmorSlot::Head => "armor.head",
            ArmorSlot::Chest => "armor.chest",
            ArmorSlot::Legs => "armor.legs",
            ArmorSlot::Feet => "armor.feet",
            ArmorSlot::Offhand => "weapon.offhand",
        }
    }
}

/// Whether the event fires on equip or unequip.
#[derive(Clone, Copy)]
pub enum ArmorEventKind {
    /// Fires on the tick the item appears in the watched slot.
    Equip,
    /// Fires on the tick the item is removed from the watched slot.
    Unequip,
}

/// Descriptor for `#[sand_macros::armor_event]` annotated functions.
///
/// At export time, all descriptors are combined into a single
/// `__sand_armor_check` mcfunction registered to `minecraft:tick`.
pub struct ArmorEventDescriptor {
    /// Function path (no namespace), e.g. `"on_boots_equip"`.
    pub path: &'static str,
    /// Factory that returns the mcfunction commands.
    pub make: fn() -> Vec<String>,
    /// Which slot to watch.
    pub slot: ArmorSlot,
    /// Equip or Unequip.
    pub kind: ArmorEventKind,
    /// Item ID filter, e.g. `"minecraft:leather_boots"`. `None` = any item.
    pub item_id: Option<&'static str>,
    /// SNBT for `minecraft:custom_data` matching, e.g. `"{mana_boots:true}"`.
    /// Generates: `components:{"minecraft:custom_data":<snbt>}` in the NBT selector.
    pub custom_data_snbt: Option<&'static str>,
}
inventory::collect!(ArmorEventDescriptor);
