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
