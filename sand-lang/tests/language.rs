//! Programming-language features: locals, parameters, return values, early
//! returns, loops, attributes, lints, and scope-aware editor help.

mod support;

use sand_lang::FileId;
use sand_lang::ide;
use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not found"))
}

const SHOP: &str = r#"
player state Wallet {
    #[clamp(0..=9999)]
    coins: number,
}

/// The price of one extra heart.
fn heart_price(owned: number) -> number {
    return 50 + owned * 25;
}

#[on(use_item = Item::CarrotOnAStick)]
fn buy_heart() {
    let health = get_attribute(Attribute::MaxHealth);
    if health >= 40 {
        actionbar!("{gold}You already have the maximum health.");
        return;
    }
    let price = heart_price((health - 20) / 2);
    if Wallet.coins < price {
        chat!("{red}You need {price} coins.");
        return;
    }
    Wallet.coins -= price;
    set_attribute(Attribute::MaxHealth, health + 2);
}
"#;

#[test]
fn spending_coins_on_max_health_compiles_with_early_returns() {
    let analysis = analyze_one(&pack(SHOP));
    let pack = compile(&analysis);
    let buy = function(&pack, "demo/function/buy_heart");
    // Branches that can return propagate it with `execute if function`.
    assert!(
        buy.contains("execute if function demo:__sand_program/"),
        "{buy}"
    );
    assert!(buy.contains(" run return 1"), "{buy}");
    assert!(
        buy.contains("run attribute @s minecraft:max_health base get 1"),
        "{buy}"
    );
    assert!(buy.contains("function demo:heart_price"), "{buy}");
    assert!(buy.contains("with storage demo:sand.args"), "{buy}");
    let price = function(&pack, "demo/function/heart_price");
    assert!(price.ends_with("return 1"), "{price}");
    let macro_helper = pack
        .resources
        .iter()
        .find(|(path, _)| path.ends_with("/attribute.mcfunction"))
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .unwrap();
    assert_eq!(
        macro_helper,
        "$attribute @s minecraft:max_health base set $(value)"
    );
}

#[test]
fn text_interpolates_local_variables() {
    let analysis = analyze_one(&pack(SHOP));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let pack = compile(&analysis);
    let message = pack
        .resources
        .values()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .find(|text| text.contains("You need "))
        .unwrap();
    assert!(
        message.contains(r##""score":{"name":"#sand_demo"##),
        "{message}"
    );
}

#[test]
fn nested_expressions_use_temporaries_and_clamp_once() {
    let analysis = analyze_one(&pack(
        r#"
player state S {
    #[clamp(0..=100)]
    hp: number,
    a: number,
    b: number,
}
fn f() {
    S.hp = S.a * 3 + S.b * 2;
}
"#,
    ));
    assert!(warnings(&analysis).is_empty(), "{:?}", warnings(&analysis));
    let pack = compile(&analysis);
    let f = function(&pack, "demo/function/f");
    // One clamp, after the final copy into the bounded field.
    assert_eq!(f.matches("matches 101..").count(), 1, "{f}");
}

#[test]
fn while_loops_and_continue() {
    let analysis = analyze_one(&pack(
        r#"
player state S { v: number }
fn count() {
    let mut n = 0;
    while n < 5 {
        n += 1;
    }
}
#[tick]
fn tick() {
    for player in players() {
        if S.v > 10 {
            continue;
        }
        S.v += 1;
    }
}
"#,
    ));
    let pack = compile(&analysis);
    let count = function(&pack, "demo/function/count");
    assert!(
        count.contains("run function demo:__sand_program/"),
        "{count}"
    );
    let looped = pack
        .resources
        .iter()
        .find(|(path, _)| path.ends_with("/loop.mcfunction"))
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .unwrap();
    assert!(looped.contains("run return run function"), "{looped}");
}

#[test]
fn control_flow_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
fn value() -> number {
    let x = 1;
    if x > 0 { return 1; }
}
fn nothing() { return 5; }
fn loop_return() {
    for player in players() { return; }
}
fn stray() { continue; }
fn recurse(n: number) { recurse(n); }
fn after() {
    return;
    let y = 2;
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`value` must return an `number` on every path",
        "this function does not return a value",
        "`return` inside a `for` loop would only end this iteration",
        "`continue` only works inside a `for` loop",
        "`recurse` calls itself (directly or indirectly), but its variables live in shared slots",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    assert!(warnings(&analysis).contains(&"unreachable statement".to_string()));
}

#[test]
fn variables_follow_rust_rules() {
    let analysis = analyze_one(&pack(
        r#"
fn f(amount: number) {
    let total = amount * 2;
    total = 3;
    amount += 1;
    let mut spare = 1;
    let _ignored = 4;
    let unused = 5;
    missing = 1;
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "cannot assign twice to immutable variable `total`",
        "cannot assign twice to immutable variable `amount`",
        "cannot find variable `missing` in this scope",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    let warnings = warnings(&analysis);
    assert!(
        warnings.contains(&"unused variable `unused`".to_string()),
        "{warnings:?}"
    );
    assert!(
        warnings.contains(&"unused variable `spare`".to_string()),
        "{warnings:?}"
    );
    assert!(
        !warnings.iter().any(|w| w.contains("_ignored")),
        "{warnings:?}"
    );
}

#[test]
fn calls_check_arity_and_return_values() {
    let analysis = analyze_one(&pack(
        r#"
fn add(a: number, b: number) -> number { return a + b; }
fn act() {}
fn f() {
    let x = add(1);
    let y = act();
    let z = add(x, y);
}
"#,
    ));
    let errors = errors(&analysis);
    assert!(
        errors
            .iter()
            .any(|e| e == "`add` takes 2 arguments, but 1 was given"),
        "{errors:#?}"
    );
    assert!(
        errors.iter().any(|e| e == "`act` does not return a value"),
        "{errors:#?}"
    );
}

#[test]
fn locals_hover_rename_and_complete_in_scope() {
    let text = pack(
        r#"
fn price(owned: number) -> number {
    let base = 50;
    if owned > 0 {
        let bonus = 5;
        return base + bonus;
    }
    return base + owned;
}
"#,
    );
    let analysis = analyze_one(&text);
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let hover = ide::hover(&analysis, FileId(0), at(&text, "base + owned")).unwrap();
    assert!(
        hover.markdown.contains("let base: number"),
        "{}",
        hover.markdown
    );
    assert!(
        hover.markdown.contains("Local variable of `fn price`"),
        "{}",
        hover.markdown
    );
    let param = ide::hover(&analysis, FileId(0), at(&text, "owned > 0")).unwrap();
    assert!(param.markdown.contains("owned: number") && param.markdown.contains("Parameter"));
    let function = ide::hover(&analysis, FileId(0), at(&text, "price(") + 1).unwrap();
    assert!(
        function
            .markdown
            .contains("fn price(owned: number) -> number"),
        "{}",
        function.markdown
    );

    let edits = ide::rename(&analysis, FileId(0), at(&text, "base = 50"), "start").unwrap();
    assert_eq!(edits.len(), 3, "{edits:?}");

    let labels = |offset: usize| -> Vec<String> {
        ide::completions(&analysis, FileId(0), offset)
            .into_iter()
            .map(|c| c.label)
            .collect()
    };
    let inner = labels(at(&text, "return base + bonus"));
    assert!(inner.contains(&"bonus".to_string()) && inner.contains(&"owned".to_string()));
    let outer = labels(at(&text, "return base + owned"));
    assert!(
        !outer.contains(&"bonus".to_string()),
        "out of scope: {outer:?}"
    );
    assert!(outer.contains(&"base".to_string()));
}

#[test]
fn signature_help_tracks_the_active_argument() {
    let text = pack(
        "fn add(a: number, b: number) -> number { return a + b; }\nfn f() {\n    let x = add(1, );\n    give_item(Item::Dirt, count = );\n}\n",
    );
    let analysis = analyze_one(&text);
    let help = ide::signature_help(&analysis, FileId(0), at(&text, ", );") + 2).unwrap();
    assert_eq!(help.label, "add(a: number, b: number) -> number");
    assert_eq!(help.active, 1);
    let give = ide::signature_help(&analysis, FileId(0), at(&text, "count = )") + 8).unwrap();
    assert_eq!(give.label, "give_item(item: Item, count?: number)");
    assert_eq!(give.active, 1);
    assert!(give.documentation.unwrap().contains("Give items"));
}

#[test]
fn vanilla_values_are_typed_checked_and_suggested() {
    let text = pack(
        r#"
#[on(use_item = Item::CarrotOnAStick)]
fn f() {
    give_item(Item::Diamond, 2);
    add_effect(Effect::Speed, seconds = 5);
    spawn_particle(Particle::Heart);
    set_attribute(Attribute::MaxHealth, 24.0);
    give_item(Effect::Speed);
    give_item(Item::Diamon);
    give_item("minecraft:diamond");
    give_item("minecraft:diamon");
    give_item("mymod:ruby");
}
"#,
    );
    let analysis = analyze_one(&text);
    let errors_ = errors(&analysis);
    assert_eq!(
        errors_,
        vec![
            "expected an `Item`, found an `Effect`",
            "no item named `Diamon` in `Item`",
            "Minecraft has no item `minecraft:diamon`",
        ]
    );
    let helps: Vec<String> = analysis
        .diagnostics
        .iter()
        .filter_map(|d| d.help.clone())
        .collect();
    assert!(
        helps
            .iter()
            .filter(|h| h.as_str() == "did you mean `Item::Diamond`?")
            .count()
            == 2,
        "{helps:?}"
    );
    assert_eq!(
        warnings(&analysis),
        vec!["use `Item::Diamond` instead of a string"]
    );

    let hover = ide::hover(&analysis, FileId(0), at(&text, "Diamond, 2") + 2).unwrap();
    assert!(
        hover
            .markdown
            .contains("The vanilla item `minecraft:diamond`"),
        "{}",
        hover.markdown
    );
    assert!(
        hover
            .markdown
            .contains("Rust: `sand::vanilla::Item::Diamond`")
    );
}

#[test]
fn vanilla_values_lower_to_identities() {
    let analysis = analyze_one(&pack(
        "#[on(use_item = Item::CarrotOnAStick)]\nfn f() {\n    give_item(Item::Diamond, 2);\n    play_sound(Sound::EntityPlayerLevelup);\n}\n",
    ));
    let pack = compile(&analysis);
    let f = function(&pack, "demo/function/f");
    assert!(f.contains("give @s minecraft:diamond 2"), "{f}");
    assert!(
        f.contains("playsound minecraft:entity.player.levelup"),
        "{f}"
    );
    let load = function(&pack, "demo/function/__sand_lifecycle_load");
    assert!(
        load.contains("minecraft.used:minecraft.carrot_on_a_stick"),
        "{load}"
    );
}

#[test]
fn vanilla_values_complete_after_the_registry_and_inside_arguments() {
    let text = pack("fn f() {\n    give_item(Item::);\n    add_effect();\n}\n");
    let analysis = analyze_one(&text);
    let after = ide::completions(&analysis, FileId(0), at(&text, "Item::)") + 6);
    let diamond = after
        .iter()
        .find(|c| c.label == "Diamond")
        .expect("Diamond offered");
    assert_eq!(diamond.detail.as_deref(), Some("minecraft:diamond"));
    assert!(after.len() > 1000, "every item is offered");
    let inside = ide::completions(&analysis, FileId(0), at(&text, "add_effect()") + 11);
    assert!(
        inside.iter().any(|c| c.label == "Effect::Speed"),
        "typed values in the argument"
    );
    assert!(
        !inside.iter().any(|c| c.label == "Item::Diamond"),
        "only the expected registry"
    );
}

#[test]
fn completion_still_works_inside_code_that_does_not_parse_yet() {
    // A missing semicolon makes the whole function unparseable.
    let text = pack("fn f() {\n    let x = 1\n    \n}\n");
    let analysis = analyze_one(&text);
    assert!(analysis.has_errors());
    let offset = text.find("1\n    \n").unwrap() + 6;
    let labels: Vec<String> = ide::completions(&analysis, FileId(0), offset)
        .into_iter()
        .map(|c| c.label)
        .collect();
    for expected in ["let", "while", "give_item", "chat!"] {
        assert!(
            labels.iter().any(|l| l == expected),
            "missing {expected} in {labels:?}"
        );
    }
    assert!(
        !labels.iter().any(|l| l == "player state"),
        "not item templates"
    );
}

#[test]
fn names_follow_one_convention_with_suggested_fixes() {
    let analysis = analyze_one(&pack(
        r#"
const maxHealth: number = 40;
player state wallet_state { Coins: number }
enum phase { idle, Active }
item ruby_sword { base: Item::DiamondSword }
recipe ruby_recipe { shapeless: [Item::Redstone], result: Item::Emerald }
tag ores: Item { values: [Item::IronOre] }
fn BuyHeart(itemCount: number) {
    let TotalCost = itemCount;
    let _Unused = 1;
    Wallet.Coins = TotalCost;
}
"#,
    ));
    let fixes: Vec<(String, String)> = analysis
        .diagnostics
        .iter()
        .filter(|d| d.code == "naming")
        .map(|d| (d.message.clone(), d.help.clone().unwrap_or_default()))
        .collect();
    for (message, help) in [
        (
            "constant `maxHealth` should have a SCREAMING_SNAKE_CASE name",
            "rename it to `MAX_HEALTH`",
        ),
        (
            "state `wallet_state` should have a PascalCase name",
            "rename it to `WalletState`",
        ),
        (
            "field `Coins` should have a snake_case name",
            "rename it to `coins`",
        ),
        (
            "enum `phase` should have a PascalCase name",
            "rename it to `Phase`",
        ),
        (
            "variant `idle` should have a PascalCase name",
            "rename it to `Idle`",
        ),
        (
            "custom item `ruby_sword` should have a PascalCase name",
            "rename it to `RubySword`",
        ),
        (
            "recipe `ruby_recipe` should have a PascalCase name",
            "rename it to `RubyRecipe`",
        ),
        (
            "tag `ores` should have a PascalCase name",
            "rename it to `Ores`",
        ),
        (
            "function `BuyHeart` should have a snake_case name",
            "rename it to `buy_heart`",
        ),
        (
            "local variable `itemCount` should have a snake_case name",
            "rename it to `item_count`",
        ),
        (
            "local variable `TotalCost` should have a snake_case name",
            "rename it to `total_cost`",
        ),
        (
            "local variable `_Unused` should have a snake_case name",
            "rename it to `_unused`",
        ),
    ] {
        assert!(
            fixes.contains(&(message.to_string(), help.to_string())),
            "missing {message:?} / {help:?} in {fixes:#?}"
        );
    }
    assert!(
        !fixes.iter().any(|(m, _)| m.contains("`Active`")),
        "already PascalCase"
    );
}
