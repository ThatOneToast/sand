use super::*;
use sand::build::{FlatGenerator, FlatLayer};
use sand::predicate::{LocationPredicate, Predicate, PredicateId, PredicateRoot};
use sand::registry::{BlockId, DimensionId, DimensionTypeId};

#[datapack_component]
pub fn frontier_menu() -> Dialog {
    Dialog::multi_action_local("journal")
        .title(Text::new("Expedition Journal").gold())
        .body(DialogBody::text("Explore the Nether to earn your charter. Every encounter and dimension crossing is recorded independently for each explorer."))
        .button(DialogButton::new("Read objectives").action(DialogAction::callback(journal)))
        .button(DialogButton::new("Inscribe held equipment").action(DialogAction::callback(inscribe_equipment)))
        .button(DialogButton::new("Close").action(DialogAction::close()))
}
#[datapack_component]
pub fn journal_access() -> DialogTag {
    DialogTag::quick_actions().dialog(sand::resource_ref::DialogId::custom(id("journal")))
}
#[datapack_component]
pub fn expedition_supplies() -> TypedTag<ItemId> {
    TypedTag::new(TagId::custom(id("supplies")))
        .entry(item("bread"))
        .entry(item("iron_ingot"))
        .entry(item("torch"))
}
#[datapack_component]
pub fn in_frontier() -> Predicate {
    Predicate::new(
        PredicateId::custom(id("in_frontier")),
        PredicateRoot::location(
            LocationPredicate::new().dimension(DimensionId::minecraft("the_nether").unwrap()),
        ),
    )
}
#[datapack_component]
pub fn chartered_item() -> ItemModifier {
    ItemModifier::new(id("chartered")).function(LootFunction::set_name(
        Text::new("Chartered Expedition Equipment").gold(),
    ))
}
#[datapack_component]
pub fn pathfinder() -> Enchantment {
    Enchantment::new(id("pathfinder"))
        .description(Text::new("Pathfinder"))
        .supported_items(item("iron_sword"))
        .max_level(3)
        .slot(EquipmentSlotGroup::Mainhand)
        .damage_effect(
            EnchantmentValueOperation::Add,
            LevelBasedValue::Linear {
                base: 1.0,
                per_level_above_first: 0.5,
            },
        )
}
#[datapack_component]
pub fn sanctuary_type() -> DimensionType {
    DimensionType::overworld_like(id("sanctuary"))
        .fixed_time(6000)
        .has_raids(false)
}
#[datapack_component]
pub fn sanctuary() -> Dimension {
    Dimension::flat_generator(
        id("sanctuary"),
        DimensionTypeId::custom(id("sanctuary")),
        FlatGenerator::new(vec![
            FlatLayer::new(BlockId::minecraft("bedrock").unwrap(), 1),
            FlatLayer::new(BlockId::minecraft("dirt").unwrap(), 3),
            FlatLayer::new(BlockId::minecraft("grass_block").unwrap(), 1),
        ])
        .with_structures(false),
    )
}

/// Representative worldgen structure-set resource reusing vanilla's authored ruins.
#[datapack_component]
pub fn expedition_ruins() -> StructureSet {
    StructureSet::random_spread(
        id("expedition_ruins"),
        sand::registry::StructureId::minecraft("trail_ruins").unwrap(),
        48,
        16,
        982451,
    )
}
