use sand::prelude::*;

#[function("unknown", context = player)]
fn unknown(player: EntityContext<PlayerKind>) {
    player.call("external:missing".parse::<FunctionId>().unwrap());
}

#[test]
fn player_call_rejects_unregistered_callees_at_export() {
    let error = sand::advanced::try_export_pack_json("handles", "26.2")
        .unwrap_err()
        .to_string();
    assert!(error.contains("handles:unknown"), "{error}");
    assert!(error.contains("external:missing"), "{error}");
    assert!(error.contains("not a registered #[function]"), "{error}");
}
