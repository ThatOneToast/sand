//! Addressing or discarding a copy must not install runtime machinery.
use sand::prelude::*;

#[test]
fn unused_live_copy_adds_no_resources() {
    let before = sand::advanced::try_export_pack_json("unused_copy", "26.2").unwrap();
    let inventory = EntityContext::<PlayerKind>::default().inventory();
    let _ = inventory
        .offhand()
        .replace_from(&inventory.mainhand())
        .unwrap();
    let _ = ItemLocation::block(BlockPos::here())
        .slot(0)
        .unwrap()
        .replace_from(&inventory.mainhand())
        .unwrap();
    assert_eq!(
        before,
        sand::advanced::try_export_pack_json("unused_copy", "26.2").unwrap()
    );
}
