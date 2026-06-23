use sand_core::cmd;
use sand_core::prelude::*;

#[test]
fn typed_custom_name_lore_model_rarity_and_give_command() {
    let item = CustomItem::new(ItemId::minecraft("diamond_sword").unwrap())
        .id("arcane:inferno_blade")
        .component(ItemComponent::custom_name(Text::new("Inferno Blade").red()))
        .component(ItemComponent::lore(vec![
            Text::new("A weapon of pure flame").dark_red(),
            Text::new("Forged below the world").gray(),
        ]))
        .component(ItemComponent::custom_model_data(1001))
        .component(ItemComponent::rarity(Rarity::Epic));

    assert_eq!(
        item.to_string(),
        "minecraft:diamond_sword[custom_data={\"arcane:inferno_blade\":1b},custom_model_data={floats:[1001.0f]},custom_name={color:\"red\",text:\"Inferno Blade\"},lore=[{color:\"dark_red\",text:\"A weapon of pure flame\"},{color:\"gray\",text:\"Forged below the world\"}],rarity=\"epic\"]"
    );
    assert_eq!(
        cmd::give(Selector::self_(), item).to_string(),
        "give @s minecraft:diamond_sword[custom_data={\"arcane:inferno_blade\":1b},custom_model_data={floats:[1001.0f]},custom_name={color:\"red\",text:\"Inferno Blade\"},lore=[{color:\"dark_red\",text:\"A weapon of pure flame\"},{color:\"gray\",text:\"Forged below the world\"}],rarity=\"epic\"]"
    );
}

#[test]
fn typed_enchantments_and_attribute_modifiers() {
    let item = CustomItem::new("minecraft:diamond_sword")
        .component(ItemComponent::enchantment(
            EnchantmentId::minecraft("sharpness").unwrap(),
            5,
        ))
        .component(ItemComponent::attribute_modifier(
            AttributeModifier::new(AttributeId::AttackDamage)
                .amount(10.0)
                .operation(AttributeOperation::AddValue)
                .slot(EquipmentSlotGroup::Mainhand),
        ));

    assert_eq!(
        item.to_string(),
        "minecraft:diamond_sword[enchantments={\"minecraft:sharpness\":5},attribute_modifiers=[{id:\"minecraft:attack_damage\",type:\"minecraft:attack_damage\",amount:10d,operation:\"add_value\",slot:\"mainhand\"}]]"
    );
}

#[test]
fn typed_food_consumable_equippable_and_tool_components() {
    let item = CustomItem::new("minecraft:apple")
        .component(ItemComponent::food(
            FoodProperties::new(8, 12.8).can_always_eat(true),
        ))
        .component(ItemComponent::consumable(
            ConsumableProperties::new(1.0)
                .animation(ConsumableAnimation::Eat)
                .sound("minecraft:entity.player.burp"),
        ))
        .component(ItemComponent::equippable(
            EquippableProperties::new(EquipmentSlot::Head).swappable(false),
        ))
        .component(ItemComponent::tool(
            ToolProperties::new()
                .rule(
                    ToolRule::new("#minecraft:mineable/pickaxe")
                        .speed(12.0)
                        .correct_for_drops(true),
                )
                .damage_per_block(2),
        ));

    assert_eq!(
        item.to_string(),
        "minecraft:apple[food={nutrition:8,saturation:12.8f,can_always_eat:true},consumable={consume_seconds:1f,animation:\"eat\",has_consume_particles:true,sound:\"minecraft:entity.player.burp\"},tool={rules:[{blocks:\"#minecraft:mineable/pickaxe\",speed:12f,correct_for_drops:true}],default_mining_speed:1f,damage_per_block:2},equippable={slot:\"head\",dispensable:true,swappable:false,damage_on_hurt:true}]"
    );
}

#[test]
fn typed_potion_and_suspicious_stew_effect_components() {
    let item = CustomItem::new("minecraft:suspicious_stew")
        .component(ItemComponent::potion_contents(
            PotionContents::new()
                .potion(PotionId::Swiftness)
                .custom_effect(
                    StatusEffectInstance::new(EffectId::Haste)
                        .duration(Ticks::seconds(5))
                        .amplifier(1),
                ),
        ))
        .component(ItemComponent::suspicious_stew_effect(
            SuspiciousStewEffect::seconds(EffectId::NightVision, 7),
        ));

    assert_eq!(
        item.to_string(),
        "minecraft:suspicious_stew[potion_contents={potion:\"minecraft:swiftness\",custom_effects:[{id:\"minecraft:haste\",duration:100,amplifier:1}]},suspicious_stew_effects=[{id:\"minecraft:night_vision\",duration:140}]]"
    );
}

#[test]
fn typed_stack_damage_unbreakable_custom_data_and_raw_escape_hatch() {
    let item = CustomItem::new("minecraft:bow")
        .component(ItemComponent::max_stack_size(1))
        .component(ItemComponent::max_damage(384))
        .component(ItemComponent::damage(12))
        .component(ItemComponent::unbreakable(false))
        .component(ItemComponent::custom_data(CustomData::raw(RawSnbt::new(
            "{charges:3,owner:\"tester\"}",
        ))))
        .component(ItemComponent::raw_component(RawComponent::new(
            "mymod:future_component",
            "{enabled:true}",
        )));

    assert_eq!(
        item.to_string(),
        "minecraft:bow[custom_data={charges:3,owner:\"tester\"},max_stack_size=1,max_damage=384,damage=12,unbreakable={show_in_tooltip:false},mymod:future_component={enabled:true}]"
    );
}

#[test]
fn custom_item_predicate_uses_typed_custom_data_marker() {
    let item = CustomItem::new("minecraft:stick").id("arcane:dash_wand");
    let pred = serde_json::to_value(item.item_predicate()).unwrap();

    assert_eq!(pred["items"], "minecraft:stick");
    assert_eq!(
        pred["components"]["minecraft:custom_data"]["arcane:dash_wand"],
        true
    );
}
