use sand::prelude::*;
use sand::systems::Menu;

#[datapack_component]
fn menu() -> Menu {
    Menu::new(
        "menus:shop".parse().unwrap(),
        Text::new("Shop"),
        Ticks::new(200),
    )
    .while_open(Wallet::coins.matches(1..).unwrap().condition())
    .choice(Text::new("Buy"), Condition::all([]), || {
        cmd::say("PURCHASE")
    })
    .choice(Text::new("Locked"), Condition::any([]), || {
        cmd::say("DENIED")
    })
}

fn body<'a>(pack: &'a serde_json::Value, suffix: &str) -> &'a str {
    system_body(pack, &format!("shop/menu_{suffix}"))
}

#[test]
fn sessions_export_fresh_tokens_and_consume_before_gameplay_on_all_targets() {
    for version in ["26.1", "26.2", "26.3"] {
        let exported = sand::advanced::try_export_pack_json("menus", version).unwrap();
        assert_eq!(
            exported,
            sand::advanced::try_export_pack_json("menus", version).unwrap()
        );
        let pack: serde_json::Value = serde_json::from_str(&exported).unwrap();
        let open = body(&pack, "page_0");
        assert!(open.contains("with storage menus:shop menu"));
        assert_eq!(open.matches("scoreboard players add #next").count(), 2);
        assert!(body(&pack, "admit").contains("matches 0..2147483580 run return run function"));
        assert!(open.contains("data remove storage menus:shop menu"));
        let show = body(&pack, "show_0");
        assert!(show.starts_with("$dialog show @s "));
        assert!(show.contains("$(a0)"));
        assert!(show.contains("$(a1)"));
        let dispatch = body(&pack, "dispatch");
        assert!(
            dispatch.contains(
                "run return run function menus:__sand_system/lifetime/shop/menu_action_0"
            )
        );
        assert!(dispatch.contains("matches 1.. run return 0"));
        let action = body(&pack, "action_0");
        assert!(
            action.starts_with("function menus:__sand_system/lifetime/shop/menu_close\n"),
            "{action}"
        );
        let tick = body(&pack, "tick");
        assert!(tick.contains("unless score @s"));
        let load = body(&pack, "load");
        assert!(load.contains("minecraft.custom:minecraft.leave_game"));
        assert!(load.contains("scoreboard players reset *"));
        assert!(load.contains("execute unless score #next"));
    }
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "menus", scope = player)]
struct Wallet {
    #[state(default = 10)]
    coins: Score,
    claimed: Flag,
}

#[datapack_component]
fn reward() -> sand::systems::Purchase {
    sand::systems::Purchase::new(
        "menus:reward".parse().unwrap(),
        sand::component::ItemStack::new(ItemId::minecraft("written_book").unwrap()).component(
            sand::component::ItemComponent::WrittenBook(
                sand::component::WrittenBook::new("Quest journal", "Sand")
                    .page(Text::new("The gate is now unlocked.")),
            ),
        ),
        sand::inventory::InventoryIndex::new(0).unwrap(),
    )
    .cost(Wallet::coins, 3)
    .claim_once(Wallet::claimed)
    .on_success(|| cmd::say("REWARD_COMMITTED"))
}

#[datapack_component]
fn object() -> sand::systems::Interactable {
    sand::systems::Interactable::new(sand::systems::Lifetime::new(
        "menus:object".parse().unwrap(),
        Ticks::new(1200),
    ))
    .bounds(1.0, 2.0, 5.0)
    .on_use(|| menu().open())
    .on_attack(|| cmd::say("OBJECT_ATTACK"))
}

fn system_body<'a>(pack: &'a serde_json::Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["path"] == format!("__sand_system/lifetime/{path}"))
        .unwrap_or_else(|| panic!("missing exported gameplay function: {path}"))["content"]
        .as_str()
        .expect("exported function content must be text")
}

#[test]
fn rewards_check_placement_before_charging_and_hitboxes_consume_before_callback() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack: serde_json::Value =
            serde_json::from_str(&sand::advanced::try_export_pack_json("menus", version).unwrap())
                .unwrap();
        let buy = system_body(&pack, "reward/purchase_buy");
        let place = buy
            .find("execute unless function menus:__sand_system/lifetime/reward/purchase_place")
            .unwrap();
        let charge = buy
            .find(&format!(
                "scoreboard players remove @s {} 3",
                Wallet::coins.objective()
            ))
            .unwrap();
        let success = buy
            .find("function menus:__sand_system/lifetime/reward/purchase_success")
            .unwrap();
        assert!(place < charge && charge < success, "{buy}");
        let placement = system_body(&pack, "reward/purchase_place");
        assert!(
            placement.starts_with(
                "return run item replace entity @s hotbar.0 with minecraft:written_book["
            ),
            "{placement}"
        );
        assert!(placement.contains("written_book_content="));
        let interaction = system_body(&pack, "object/interaction_use");
        assert!(interaction.starts_with("data remove entity @s interaction\n$execute as @a[nbt={UUID:$(player)},distance=..5,limit=1]"));
        let cleanup = system_body(&pack, "object/interaction_destroy");
        assert!(cleanup.ends_with("kill @s"));
        assert!(system_body(&pack, "object/interaction_remove").contains("=$(instance)"));
    }
}

#[function("menus:attach")]
fn attach() {
    Wallet::attach(EntityContext::<PlayerKind>::default());
}
#[function("menus:open")]
fn open() {
    menu().open();
}
#[function("menus:buy")]
fn buy() {
    reward().buy();
}
#[function("menus:object_start")]
fn object_start() {
    object().start();
}
#[function("menus:object_stop")]
fn object_stop() {
    object().cancel();
}

#[test]
fn export_interaction_runtime_fixture_when_requested() {
    let Some(root) = std::env::var_os("SAND_INTERACTION_PROOF_PACK") else {
        return;
    };
    let version = std::env::var("SAND_INTERACTION_PROOF_VERSION").unwrap_or_else(|_| "26.2".into());
    let root = std::path::PathBuf::from(root);
    std::fs::create_dir_all(&root).unwrap();
    let pack: serde_json::Value =
        serde_json::from_str(&sand::advanced::try_export_pack_json("menus", &version).unwrap())
            .unwrap();
    let format = if version == "26.3" {
        121
    } else if version == "26.2" {
        107
    } else {
        101
    };
    std::fs::write(root.join("pack.mcmeta"), serde_json::json!({"pack":{"description":"Sand interaction proof", "min_format":format,"max_format":format}}).to_string()).unwrap();
    std::fs::write(root.join("fixture.json"), serde_json::json!({"coins":Wallet::coins.objective(),"claimed":Wallet::claimed.objective()}).to_string()).unwrap();
    for record in pack["components"].as_array().unwrap() {
        let dir = record["dir"].as_str().unwrap();
        let extension = if dir == "function" {
            "mcfunction"
        } else {
            "json"
        };
        let path = root
            .join("data")
            .join(record["namespace"].as_str().unwrap())
            .join(dir)
            .join(format!(
                "{}.{}",
                record["path"].as_str().unwrap(),
                extension
            ));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let content = &record["content"];
        std::fs::write(
            path,
            content
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| content.to_string()),
        )
        .unwrap();
    }
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "menus", scope = global)]
struct SharedLoot {
    claimed: Flag,
}

#[datapack_component]
fn shared_reward() -> sand::systems::Purchase {
    sand::systems::Purchase::new(
        "menus:shared_reward".parse().unwrap(),
        sand::component::ItemStack::new(ItemId::minecraft("diamond").unwrap()),
        sand::inventory::InventoryIndex::new(1).unwrap(),
    )
    .claim_once(SharedLoot::claimed)
}

#[function("menus:shared_buy")]
fn shared_buy() {
    shared_reward().buy();
}

#[datapack_component]
fn bound_object() -> sand::systems::Interactable {
    let menu = Menu::new(
        "menus:bound".parse().unwrap(),
        Text::new("Catalog"),
        Ticks::new(200),
    )
    .page_size(2)
    .choice(Text::new("One"), Condition::all([]), || cmd::say("ONE"))
    .choice(Text::new("Two"), Condition::all([]), || cmd::say("TWO"))
    .choice(Text::new("Three"), Condition::all([]), || cmd::say("THREE"));
    sand::systems::Interactable::new(sand::systems::Lifetime::new(
        "menus:bound_object".parse().unwrap(),
        Ticks::new(1200),
    ))
    .bounds(1.0, 2.0, 5.0)
    .opens_menu(menu)
}

#[test]
fn pagination_and_object_binding_preserve_authorization_and_actor_context() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack: serde_json::Value =
            serde_json::from_str(&sand::advanced::try_export_pack_json("menus", version).unwrap())
                .unwrap();
        let page0 = system_body(&pack, "bound/menu_page_0");
        let page1 = system_body(&pack, "bound/menu_page_1");
        assert!(page0.contains("menu.a0"));
        assert!(page0.contains("menu.a1"));
        assert!(page0.contains("menu.a3"));
        assert!(!page0.contains("menu.a2"));
        assert!(page1.contains("menu.a2"));
        assert!(page1.contains("menu.a4"));
        assert!(!page1.contains("menu.a0"));
        let next = system_body(&pack, "bound/menu_next");
        assert!(next.find("menu_clear").unwrap() < next.find("menu_admit").unwrap());
        assert!(!system_body(&pack, "bound/menu_clear").contains("object_call"));
        assert_eq!(system_body(&pack, "bound/menu_open").trim(), "return 0");
        let valid = system_body(&pack, "bound/menu_valid_object");
        assert!(valid.contains("=$(instance)"));
        assert!(valid.contains("distance=..5,limit=1"));
        let invoke = system_body(&pack, "bound/menu_invoke_0");
        assert!(invoke.find("menu_close").unwrap() < invoke.find("$execute at @e").unwrap());
        assert!(!invoke.contains("execute as @e"));
        let capture = system_body(&pack, "bound_object/interaction_menu_capture");
        assert!(capture.contains("menu_event.instance"));
        assert!(
            capture.find("data remove entity @s interaction").unwrap()
                < capture.find("interaction_menu with storage").unwrap()
        );
        assert!(
            system_body(&pack, "bound_object/interaction_menu")
                .contains("menu_open_bound {instance:$(instance)}")
        );
    }
}
