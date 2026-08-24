use sand::prelude::*;

#[test]
fn documented_attribute_id_alias_is_available_from_the_prelude() {
    let modifier = AttributeModifier::new(AttributeId::MovementSpeed)
        .amount(0.02)
        .operation(AttributeOperation::AddValue)
        .slot(EquipmentSlotGroup::Feet);

    let _: AttributeType = AttributeId::MovementSpeed;
    let _ = modifier;
}
