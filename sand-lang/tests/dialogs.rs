mod support;

use support::*;

#[test]
fn notice_dialogs_export_native_resources_and_player_commands() {
    let source = r#"#![pack(namespace = "notices", minecraft = "26.2")]
dialog Welcome {
    title: "{gold}Welcome",
    body: [Text::translate("item.minecraft.written_book"), "Read the guide"],
    button: "Continue",
}
dialog Plain { title: "Plain" }
fn open() { show_dialog(Welcome); }
fn close() { clear_dialog(); }
"#;
    let pack = compile(&analyze_one(source));
    let welcome: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/notices/dialog/welcome.json"]).unwrap();
    assert_eq!(welcome["type"], "minecraft:notice");
    assert_eq!(welcome["title"]["extra"][0]["text"], "Welcome");
    assert!(
        welcome["body"]
            .to_string()
            .contains("item.minecraft.written_book")
    );
    assert!(
        welcome["buttons"][0]["label"]
            .to_string()
            .contains("Continue")
    );
    let plain: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/notices/dialog/plain.json"]).unwrap();
    assert_eq!(plain["buttons"][0]["label"]["translate"], "gui.done");
    assert!(function(&pack, "notices/function/open").contains("dialog show @s notices:welcome"));
    assert!(function(&pack, "notices/function/close").contains("dialog clear @s"));
}

#[test]
fn dialogs_validate_context_and_static_text() {
    for code in [
        r#"dialog Notice { title: "{player}" }"#,
        r#"dialog Notice { title: "Notice", body: "expected a list" }"#,
        r#"dialog Notice { title: "Notice" } #[tick] fn f() { show_dialog(Notice); }"#,
        r#"#[tick] fn f() { clear_dialog(); }"#,
    ] {
        let source = format!(r#"#![pack(namespace = "notices")] {code}"#);
        assert!(analyze_one(&source).has_errors(), "{code}");
    }
}
