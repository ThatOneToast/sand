use sand_core::prelude::*;

#[derive(Debug)]
struct PlayerMagic;

static MAGIC: StorageSchema<PlayerMagic> = StorageSchema::new("arcane:players", "player.magic");
static MANA: StorageField<PlayerMagic, i32> = MAGIC.field("mana");
static SCHOOL: StorageField<PlayerMagic, String> = MAGIC.field("school");
static SPELLS: StorageField<PlayerMagic, Vec<String>> = MAGIC.field("spells");

#[test]
fn snbt_values_format_primitives_lists_and_compounds() {
    assert_eq!(SnbtValue::from(1_i8).to_string(), "1b");
    assert_eq!(SnbtValue::from(2_i16).to_string(), "2s");
    assert_eq!(SnbtValue::from(3_i32).to_string(), "3");
    assert_eq!(SnbtValue::from(4_i64).to_string(), "4L");
    assert_eq!(SnbtValue::from(1.25_f32).to_string(), "1.25f");
    assert_eq!(SnbtValue::from(2.5_f64).to_string(), "2.5d");
    assert_eq!(SnbtValue::from(true).to_string(), "1b");
    assert_eq!(
        SnbtValue::from(r#"say "hi""#).to_string(),
        r#""say \"hi\"""#
    );

    let compound = SnbtCompound::new()
        .field("mana", 100)
        .field("school", "pyromancy")
        .field("arcane:rank", 2_i8)
        .field("spells", SnbtValue::from(vec!["dash", "shield"]));

    assert_eq!(
        compound.to_string(),
        r#"{mana:100,school:"pyromancy","arcane:rank":2b,spells:["dash","shield"]}"#
    );
}

#[test]
fn typed_path_construction_quotes_needed_segments() {
    let path = NbtPath::root("player")
        .field("magic")
        .index(0)
        .field("arcane:mana");

    assert_eq!(path.as_str(), r#"player.magic[0]."arcane:mana""#);
}

#[test]
fn storage_schema_and_fields_emit_commands() {
    assert_eq!(MAGIC.storage(), "arcane:players");
    assert_eq!(MAGIC.root_path(), "player.magic");
    assert_eq!(
        MAGIC.set(SnbtCompound::new().field("mana", 100)),
        "data modify storage arcane:players player.magic set value {mana:100}"
    );
    assert_eq!(
        MANA.set(100),
        "data modify storage arcane:players player.magic.mana set value 100"
    );
    assert_eq!(
        SCHOOL.set("pyromancy"),
        r#"data modify storage arcane:players player.magic.school set value "pyromancy""#
    );
    assert_eq!(
        MANA.get(),
        "data get storage arcane:players player.magic.mana"
    );
    assert_eq!(
        MANA.remove(),
        "data remove storage arcane:players player.magic.mana"
    );
    assert!(matches!(MANA.exists(), Condition::StorageExists { .. }));
}

#[test]
fn storage_field_copy_append_merge_and_raw_escape_hatch() {
    let backup: StorageField<PlayerMagic, i32> = MAGIC.field("backup_mana");

    assert_eq!(
        MANA.copy_from(backup),
        "data modify storage arcane:players player.magic.mana set from storage arcane:players player.magic.backup_mana"
    );
    assert_eq!(
        SPELLS.append("dash"),
        r#"data modify storage arcane:players player.magic.spells append value "dash""#
    );
    assert_eq!(
        MAGIC.merge(SnbtCompound::new().field("school", "pyromancy")),
        r#"data modify storage arcane:players player.magic merge value {school:"pyromancy"}"#
    );
    assert_eq!(
        SCHOOL.set_raw_snbt(RawSnbt::new("\"raw_school\"")),
        r#"data modify storage arcane:players player.magic.school set value "raw_school""#
    );
}
