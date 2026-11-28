mod support;

use support::*;

const MENU: &str = r#"
#![pack(namespace = "scout", minecraft = "26.2")]
menu Supplies {
    title: "Camp supplies",
    ticks: 200,
    page_size: 1,
    choices: [
        { label: "Bread", action: bread },
        { label: "Compass", action: compass },
    ],
}
fn bread() { give_item(Item::Bread); }
fn compass() { give_item(Item::Compass); }
fn show() { open_menu(Supplies); }
fn close() { close_menu(Supplies); }
"#;

#[test]
fn native_menu_resources_keep_page_order_labels_and_callback_targets() {
    let analysis = analyze_one(MENU);
    let pack = compile(&analysis);
    let show = function(&pack, "scout/function/show");
    assert!(show.contains("function scout:__sand_system/lifetime/supplies/menu_open"));
    let close = function(&pack, "scout/function/close");
    assert!(close.contains("function scout:__sand_system/lifetime/supplies/menu_close"));
    for (index, label, navigation) in [(0, "Bread", "Next"), (1, "Compass", "Previous")] {
        let page = function(
            &pack,
            &format!("scout/function/__sand_system/lifetime/supplies/menu_show_{index}"),
        );
        assert!(page.starts_with("$dialog show @s "));
        assert!(page.contains("minecraft:multi_action"));
        assert!(page.contains(label));
        assert!(page.contains(navigation));
        assert!(page.contains("/trigger "));
    }
    let document = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&document, &Default::default()).unwrap();
    let recompiled = sand::advanced::compiler::Compiler::compile(&decoded).unwrap();
    assert_eq!(pack.resources, recompiled.resources);
}

#[test]
fn menus_validate_bounds_and_parameterless_actions() {
    for (from, to, expected) in [
        ("ticks: 200", "ticks: 0", "expected a value from 1"),
        (
            "page_size: 1",
            "page_size: 65",
            "expected a value from 1 to 64",
        ),
        (
            "fn bread()",
            "fn bread(amount: number)",
            "expected a parameterless function",
        ),
    ] {
        let analysis = analyze_one(&MENU.replace(from, to));
        assert!(
            errors(&analysis)
                .iter()
                .any(|error| error.contains(expected)),
            "{:?}",
            errors(&analysis)
        );
    }
    let empty = analyze_one(
        r#"#![pack(namespace = "scout")] menu Supplies { title: "Camp", ticks: 10, choices: [] }"#,
    );
    assert!(
        errors(&empty)
            .iter()
            .any(|error| error.contains("between 1 and 256 choices"))
    );
}

const CONVERSATION: &str = r#"
#![pack(namespace = "scout", minecraft = "26.2")]
player state Trade { available: bool = true, confirmed: bool = true, purchases: number }
menu Supplies {
    title: "Camp supplies",
    ticks: 200,
    start: "confirm",
    while_open: Trade.available,
    choices: [{ label: "Open confirmation", next: "confirm" }],
    views: [{
        name: "confirm",
        title: "Confirm purchase",
        choices: [
            { label: "Buy", action: buy, when: Trade.confirmed },
            { label: "Back", next: "main" },
        ],
    }],
}
fn buy() { Trade.purchases += 1; }
fn show() { open_menu(Supplies); }
"#;

#[test]
fn native_menu_views_export_with_checked_player_state_and_deterministic_transitions() {
    let analysis = analyze_one(CONVERSATION);
    let pack = compile(&analysis);
    let resources = &pack.resources;
    let base = "scout/function/__sand_system/lifetime/supplies";
    assert!(function(&pack, &format!("{base}/menu_show_0")).contains("Open confirmation"));
    let confirmation = function(&pack, &format!("{base}/menu_show_1"));
    assert!(confirmation.contains("Confirm purchase"));
    assert!(confirmation.contains("Back"));
    assert!(resources.iter().any(|(path, content)| {
        path.starts_with("data/scout/function/")
            && path.ends_with(".mcfunction")
            && std::str::from_utf8(content)
                .unwrap()
                .contains("function scout:buy")
    }));
    let document = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&document, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn conversations_require_boolean_guards_and_known_views() {
    for (from, to, expected) in [
        (
            "start: \"confirm\"",
            "start: \"missing\"",
            "unknown starting menu view",
        ),
        (
            "next: \"confirm\"",
            "next: \"missing\"",
            "unknown destination menu view",
        ),
        (
            "name: \"confirm\"",
            "name: \"main\"",
            "unique menu view name",
        ),
        (
            "when: Trade.confirmed",
            "when: Trade.purchases",
            "expected `bool`",
        ),
        (
            "next: \"main\"",
            "when: Trade.confirmed",
            "needs an action or destination",
        ),
    ] {
        let analysis = analyze_one(&CONVERSATION.replace(from, to));
        let messages: Vec<_> = analysis
            .compile()
            .expect_err("invalid menu compiled")
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect();
        assert!(
            messages.iter().any(|error| error.contains(expected)),
            "expected {expected:?}, got {messages:?}"
        );
    }
}
