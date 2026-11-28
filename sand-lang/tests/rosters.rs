mod support;
use support::*;

use sand::advanced::compiler::{Compiler, RosterCommand};

const SOURCE: &str = r#"
#![pack(namespace = "roster_probe", minecraft = "26.2")]
enum Role { Fighter = 7, Spectator = -3 }
enum Other { Fighter = 7 }
player state Member { generation: number, role: Role = Role::Spectator, visits: number }
roster Arena { member: Member.generation, role: Member.role, capacity: 1 }
roster Reserve { member: Member.generation, role: Member.role, capacity: 2 }
pub fn start() { roster_open(Arena); roster_open(Reserve); }
pub fn join() { roster_join(Arena, Role::Fighter); }
pub fn standby() { roster_join(Reserve, Role::Spectator); }
pub fn visit() { roster_each(Arena, on_member, role = Role::Fighter); }
pub fn stop() { roster_close(Arena); }
pub fn leave() { roster_leave(Reserve); }
fn on_member() {
    if roster_contains(Arena, role = Role::Fighter) {
        Member.visits += 1;
        roster_join(Reserve, Role::Spectator);
    }
}
pub fn active() {
    if roster_has_members(Arena, role = Role::Fighter) { raw!("say arena active"); }
}
"#;

#[test]
fn rosters_export_shared_admission_and_iteration_and_round_trip() {
    let analysis = analyze_one(SOURCE);
    let pack = compile(&analysis);
    let modules = &analysis.program.as_ref().unwrap().modules;
    assert_eq!(modules[0].rosters[0].roles, [7, -3]);
    let command_text = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, value)| std::str::from_utf8(value).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for required in [
        "minecraft.custom:minecraft.leave_game",
        "roster_join_7",
        "roster_is_7",
        "roster_any_7",
        "roster_locked",
        "roster_unlock",
        "function roster_probe:on_member",
    ] {
        assert!(command_text.contains(required), "{required}");
    }
    let program = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = Compiler::decode(&program, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        Compiler::compile(&decoded).unwrap().resources
    );
}

#[test]
fn roster_requires_typed_roles_and_safe_membership_fields() {
    let invalid = SOURCE.replace(
        "roster_join(Arena, Role::Fighter)",
        "roster_join(Arena, Other::Fighter)",
    );
    let messages = errors(&analyze_one(&invalid));
    assert!(
        messages
            .iter()
            .any(|message| message.contains("expected `Role`, found `Other`")),
        "{messages:?}"
    );

    let mut program = analyze_one(SOURCE).program.unwrap();
    program.modules[0].states[0].fields[0].default = 1;
    program.modules[0]
        .functions
        .iter_mut()
        .find(|function| function.id.to_string() == "roster_probe:join")
        .unwrap()
        .body
        .iter_mut()
        .find_map(|operation| {
            if let sand::advanced::compiler::Action::Roster { command, .. } = &mut operation.action
            {
                *command = RosterCommand::Join { role: 100 };
                Some(())
            } else {
                None
            }
        })
        .unwrap();
    let diagnostics = Compiler::compile(&program).unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("default-zero player State number")
    }));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unknown role encoding"))
    );
}
