//! Exercise the supported facade and the real compiler with both shared sound shapes.
use sand::component::{Instrument, JukeboxSong, SoundEventDefinition};
use sand::registration::DatapackRegistration;
use sand::text::TextComponent;
use sand_core::serde_json;

#[sand::datapack_component]
fn sounds() -> DatapackRegistration {
    DatapackRegistration::new()
        .component(
            Instrument::new("__sand_local:horn".parse().unwrap())
                .sound_event(SoundEventDefinition::reference(
                    sand::vanilla::SoundEvent::BlockNoteBlockHarp,
                ))
                .description(TextComponent::literal("Horn")),
        )
        .component(
            JukeboxSong::new("__sand_local:song".parse().unwrap())
                .sound_event(SoundEventDefinition::inline(
                    "sound_facade:track".parse().unwrap(),
                    Some(-1.0),
                ))
                .song_length(10.0)
                .description(TextComponent::literal("Song")),
        )
}

#[test]
fn facade_exports_reference_and_inline_resources_deterministically() {
    let export = |namespace| sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
    let first = export("sound_facade");
    let other = export("other_namespace");
    assert_eq!(first, export("sound_facade"));
    let other_pack: serde_json::Value = serde_json::from_str(&other).unwrap();
    let other_records = other_pack["components"].as_array().unwrap();
    for kind in ["instrument", "jukebox_song"] {
        let record = other_records
            .iter()
            .find(|record| record["dir"] == kind)
            .unwrap();
        assert_eq!(record["namespace"], "other_namespace");
        assert!(
            !other_records
                .iter()
                .any(|record| record["dir"] == kind && record["namespace"] == "sound_facade")
        );
    }
    let pack: serde_json::Value = serde_json::from_str(&first).unwrap();
    let records = pack["components"].as_array().unwrap();
    for (kind, path, expected) in [
        (
            "instrument",
            "horn",
            serde_json::json!("minecraft:block.note_block.harp"),
        ),
        (
            "jukebox_song",
            "song",
            serde_json::json!({"sound_id":"sound_facade:track", "range":-1.0}),
        ),
    ] {
        let record = records
            .iter()
            .find(|record| {
                record["namespace"] == "sound_facade"
                    && record["dir"] == kind
                    && record["path"] == path
            })
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_str(record["content"].as_str().unwrap()).unwrap();
        assert_eq!(value["sound_event"], expected);
    }
}
