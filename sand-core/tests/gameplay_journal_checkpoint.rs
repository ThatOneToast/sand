use sand::component::IntoCommands;
use sand::entity::EntityText;
use sand::prelude::*;
use sand::systems::{Checkpoint, Journal, Lifetime, Progression, Session};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EntityStateEnum)]
enum JournalStage {
    Started = 0,
    Finished = 1,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace="journal_checkpoint",scope=player)]
struct Live {
    #[state(min = 0, max = 10)]
    health: Score,
    #[state(min = 0, max = 2)]
    phase: Score,
    #[state(default = "JournalStage::Started")]
    status: EntityEnum<JournalStage>,
    won: Flag,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace="journal_checkpoint",scope=player,version=2,migrate(from=1,to=2))]
struct Saved {
    #[state(min = 0, max = 10)]
    health: Score,
    #[state(min = 0, max = 2)]
    phase: Score,
    valid: Flag,
}
#[state_lifecycle]
impl StateLifecycle for Saved {
    fn migrate(ctx: StateMigrate) -> Vec<String> {
        Saved::valid.bind_to(ctx.holder(), false).disable()
    }
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace="journal_checkpoint",scope=player)]
struct Policy {
    invalidate: Flag,
    detach_state: Flag,
    prepared: Score,
    restored: Score,
    issued: Score,
    rejected: Score,
}
#[allow(dead_code)]
#[derive(State)]
#[state(namespace="journal_checkpoint",scope=entity)]
struct NameState {
    count: Score,
    flag: Flag,
}
#[derive(Archetype)]
#[archetype(id="journal_checkpoint:named",entity=MarkerKind,configure=Self::configure)]
#[allow(dead_code)]
struct Named {
    state: NameState,
}
impl Named {
    fn configure(archetype: EntityArchetype<MarkerKind>) -> EntityArchetype<MarkerKind> {
        archetype.name(
            EntityName::new()
                .text(Text::new("quote \" \\ $(missing)\n"))
                .state(NameState::count, ChatColor::Gold)
                .flag_state(
                    NameState::flag,
                    "off \" $(missing)",
                    "on \\ $(missing)",
                    ChatColor::Green,
                ),
        )
    }
}
#[datapack_component]
fn round() -> Session {
    Session::new(
        Lifetime::new("journal_checkpoint:round".parse().unwrap(), Ticks::new(200))
            .requires::<Live>(),
        Progression::new(
            "journal_checkpoint:round_phases".parse().unwrap(),
            Live::phase,
        )
        .objective(
            |owner| owner.state(Live::health).matches(10..).unwrap(),
            |_| cmd::say("ROUND_ONE"),
        )
        .objective(
            |owner| owner.state(Live::health).matches(10..).unwrap(),
            |_| cmd::say("ROUND_TWO"),
        ),
    )
}
#[datapack_component]
fn alternate_checkpoint() -> Checkpoint {
    Checkpoint::new(
        "other_checkpoint:shared_saved".parse().unwrap(),
        Saved::valid,
    )
    .field(Live::health, Saved::health)
    .field(Live::phase, Saved::phase)
}

#[datapack_component]
fn checkpoint() -> Checkpoint {
    Checkpoint::new("journal_checkpoint:saved".parse().unwrap(), Saved::valid)
        .field(Live::health, Saved::health)
        .field(Live::phase, Saved::phase)
        .before_restore(|owner| {
            let mut actions = owner.state(Policy::prepared).add(1);
            actions.extend([
                round().cancel(),
                checkpoint().save(),
                checkpoint().restore(),
                checkpoint().clear(),
                alternate_checkpoint().clear(),
                alternate_checkpoint().save(),
                alternate_checkpoint().restore(),
            ]);
            actions.extend([
                when(owner.state(Policy::invalidate).is_enabled())
                    .then(owner.state(Saved::valid).disable()),
                when(owner.state(Policy::detach_state).is_enabled()).then(Live::detach(owner)),
            ]);
            actions
        })
        .after_restore(|owner| {
            let mut actions = owner.state(Policy::restored).add(1);
            actions.extend([
                checkpoint().restore(),
                round().start(),
                cmd::return_cmd(7).into_commands(),
            ]);
            actions
        })
}
#[datapack_component]
fn journal() -> Journal {
    Journal::new(
        "journal_checkpoint:journal".parse().unwrap(),
        "Quest $(missing)",
        "A \"writer\"",
        InventoryIndex::new(0).unwrap(),
    )
    .page(
        EntityText::new()
            .literal("quote \" \\ $(missing)\nHealth: ")
            .score(Live::health)
            .literal(" Status: ")
            .enum_value(Live::status)
            .literal(" Won: ")
            .flag(Live::won, "no \" $(missing)", "yes \\ $(missing)"),
    )
    .replace_owned()
    .on_success(|| Policy::issued.bind().add(1))
    .on_rejected(|| Policy::rejected.bind().add(1))
}
#[function("journal_checkpoint:attach")]
fn attach() {
    Live::attach(EntityContext::default());
    Saved::attach(EntityContext::default());
    Policy::attach(EntityContext::default());
}
#[function("journal_checkpoint:save")]
fn save() {
    checkpoint().save();
}
#[function("journal_checkpoint:restore")]
fn restore() {
    checkpoint().restore();
}
#[function("journal_checkpoint:clear_checkpoint")]
fn clear_checkpoint() {
    checkpoint().clear();
}
#[function("journal_checkpoint:journal")]
fn refresh() {
    journal().refresh();
}
#[function("journal_checkpoint:start")]
fn start() {
    round().start();
}
#[function("journal_checkpoint:named")]
fn named() {
    Named::summon(sand::command::Vec3::absolute(2.0, 200.0, 4.0));
}

#[function("journal_checkpoint:named_update")]
fn named_update() {
    NameState::count.bind().set(42);
    NameState::flag.bind().enable();
}

#[function("journal_checkpoint:status_finished")]
fn status_finished() {
    Live::status.bind().set(JournalStage::Finished);
    Live::won.bind().enable();
}

fn export(version: &str) -> serde_json::Value {
    serde_json::from_str(
        &sand::advanced::try_export_pack_json("journal_checkpoint", version).unwrap(),
    )
    .unwrap()
}
fn body<'a>(pack: &'a serde_json::Value, path: &str) -> &'a str {
    pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["path"] == path)
        .unwrap_or_else(|| panic!("missing {path}"))["content"]
        .as_str()
        .unwrap()
}
#[test]
fn checkpoint_revalidates_after_preparation_and_commits_before_continuation() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = export(version);
        assert_eq!(pack, export(version));
        let restore = body(
            &pack,
            "__sand_system/lifetime/saved/checkpoint_restore_owner",
        );
        assert!(restore.contains("UUID:$(owner)"));
        assert!(restore.contains("/checkpoint_restore_commit"));
        let commit = body(
            &pack,
            "__sand_system/lifetime/saved/checkpoint_restore_commit",
        );
        assert!(commit.find("matches").unwrap() < commit.find("\nfunction").unwrap());
        assert!(commit.rfind("#accepted").unwrap() < commit.rfind("function").unwrap());
        let save = body(&pack, "__sand_system/lifetime/saved/checkpoint_save");
        let lock = save
            .lines()
            .find(|line| line.starts_with("execute if score #busy "))
            .unwrap()
            .split_whitespace()
            .nth(4)
            .unwrap();
        let alternate = body(
            &pack,
            "__sand_system/lifetime/shared_saved/checkpoint_clear",
        );
        assert!(alternate.contains(&format!("execute if score #busy {lock} matches 1")));
        assert!(save.find("#busy").unwrap() < save.find("function").unwrap());
        assert!(save.rfind("#busy").unwrap() > save.find("function").unwrap());
    }
}
#[test]
fn journals_share_safe_text_materialization_and_inventory_lock() {
    for version in ["26.1", "26.2", "26.3"] {
        let pack = export(version);
        let refresh = body(&pack, "__sand_system/lifetime/journal/journal_refresh");
        assert!(refresh.contains("/journal_place with storage"));
        assert!(refresh.contains("/journal_verify with storage"));
        assert!(refresh.contains("operation.content.pages[0]"));
        assert!(!refresh.contains("$(missing)\nHealth"));
        assert!(refresh.contains(r"$(missing)\nHealth"));
        let place = body(&pack, "__sand_system/lifetime/journal/journal_place");
        assert!(place.contains("written_book"));
        assert!(place.contains("$(content)"));
        assert!(!place.contains("$(missing)"));
        let macros = pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|component| component["content"].as_str())
            .flat_map(str::lines)
            .filter(|line| line.starts_with('$'))
            .collect::<Vec<_>>();
        assert!(macros.iter().any(|line| line.contains("$(n1)")));
        assert!(
            macros.iter().all(|line| !line.contains("$(missing)")),
            "{macros:?}"
        );
        let text = pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|component| component["content"].as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("CustomName set from storage"));
        assert!(text.contains("quote"));
    }
}
#[test]
fn optional_native_fixture() {
    let Some(root) = std::env::var_os("SAND_JOURNAL_CHECKPOINT_PROOF_PACK") else {
        return;
    };
    let version =
        std::env::var("SAND_JOURNAL_CHECKPOINT_PROOF_VERSION").unwrap_or_else(|_| "26.3".into());
    let pack = export(&version);
    let root = std::path::PathBuf::from(root);
    std::fs::create_dir_all(&root).unwrap();
    let format = if version == "26.3" {
        121
    } else if version == "26.2" {
        107
    } else {
        101
    };
    std::fs::write(root.join("pack.mcmeta"),serde_json::to_vec(&serde_json::json!({"pack":{"description":"Journal and checkpoint native proof","min_format":format,"max_format":format}})).unwrap()).unwrap();
    std::fs::write(root.join("fixture.json"),serde_json::to_vec_pretty(&serde_json::json!({"health":Live::health.objective(),"phase":Live::phase.objective(),"status":Live::status.objective(),"won":Live::won.objective(),"saved_health":Saved::health.objective(),"saved_phase":Saved::phase.objective(),"saved_valid":Saved::valid.objective(),"saved_presence":<Saved as sand::__private::StateBundleMember>::presence_requirements()[0].0,"invalidate":Policy::invalidate.objective(),"detach":Policy::detach_state.objective(),"prepared":Policy::prepared.objective(),"restored":Policy::restored.objective(),"issued":Policy::issued.objective(),"rejected":Policy::rejected.objective(),"name_count":NameState::count.objective(),"name_flag":NameState::flag.objective()})).unwrap()).unwrap();
    for component in pack["components"].as_array().unwrap() {
        let dir = component["dir"].as_str().unwrap();
        let extension = if dir == "function" {
            "mcfunction"
        } else {
            "json"
        };
        let path = root
            .join("data")
            .join(component["namespace"].as_str().unwrap())
            .join(dir)
            .join(format!(
                "{}.{}",
                component["path"].as_str().unwrap(),
                extension
            ));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            component["content"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| serde_json::to_string(&component["content"]).unwrap()),
        )
        .unwrap();
    }
}
