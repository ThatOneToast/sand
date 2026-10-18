use sand::advanced::compiler::*;
use std::collections::BTreeMap;

const FIXTURE: &[u8] = include_bytes!("../../tests/fixtures/program/flows.sand.json");

fn program() -> Program {
    Compiler::decode(FIXTURE, &BTreeMap::new()).unwrap()
}
fn decode(value: serde_json::Value) -> Program {
    Compiler::decode(&serde_json::to_vec(&value).unwrap(), &BTreeMap::new()).unwrap()
}
fn forbidden_collection() -> sand::command::Actions {
    panic!("portable flows must not collect Rust inventory or invoke author callbacks")
}
sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        owner: module_path!(), context: ExecutionContext::Server,
        path: "forbidden_collection", make: forbidden_collection,
    }
}

#[test]
fn owned_flows_match_json_and_declaration_permutations_without_inventory() {
    let expected = Compiler::compile(&program()).unwrap();
    let mut constructed = program();
    constructed.modules[0].flows[1] = Flow {
        id: "demo:round".parse().unwrap(),
        state: FieldReference {
            state: "demo:world".parse().unwrap(),
            field: "phase".into(),
        },
        transitions: vec![FlowTransition {
            from: 0,
            to: 1,
            when: None,
            priority: 0,
        }],
        hooks: vec![FlowHook {
            phase: FlowHookPhase::Tick,
            state: 1,
            cadence: 2,
            function: FunctionReference::Internal {
                id: "demo:world_tick".parse().unwrap(),
            },
        }],
    };
    assert_eq!(
        Compiler::compile(&constructed).unwrap().resources,
        expected.resources
    );
    constructed.modules[0].states.reverse();
    constructed.modules[0].functions.reverse();
    constructed.modules[0].flows.reverse();
    assert_eq!(
        Compiler::compile(&constructed).unwrap().resources,
        expected.resources
    );
    let encoded = serde_json::to_vec(&constructed).unwrap();
    let decoded = Compiler::decode(&encoded, &BTreeMap::new()).unwrap();
    assert_eq!(
        Compiler::compile(&decoded).unwrap().resources,
        expected.resources
    );
    let outputs: String = expected
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, body)| String::from_utf8(body.clone()).unwrap())
        .collect();
    assert!(outputs.contains("return run function demo:__sand_transition/flow_"));
    assert!(outputs.lines().any(
        |line| line.contains("scoreboard players add #sand_demo_world_")
            && line.contains(" __sf_")
            && line.ends_with(" 1")
    ));
    assert!(
        outputs
            .lines()
            .any(|line| line.contains("scoreboard players add @s __sf_") && line.ends_with(" 1"))
    );
    assert!(!outputs.contains("forbidden_collection"));
}

#[test]
fn invalid_flow_contracts_have_module_and_field_locations() {
    let source: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    let cases = [
        (
            "/modules/0/flows/1/hooks/0/function/id",
            serde_json::json!("demo:active"),
            "SAND_PROGRAM_CONTEXT",
            "/flows/1/hooks/0/function",
        ),
        (
            "/modules/0/flows/0/hooks/0/function/id",
            serde_json::json!("demo:missing"),
            "SAND_PROGRAM_REFERENCE",
            "/flows/0/hooks/0/function",
        ),
        (
            "/modules/0/flows/0/state/field",
            serde_json::json!("missing"),
            "SAND_PROGRAM_REFERENCE",
            "/flows/0/state",
        ),
        (
            "/modules/0/flows/0/transitions/0/to",
            serde_json::json!(0),
            "SAND_PROGRAM_FLOW",
            "/flows/0/transitions/0",
        ),
        (
            "/modules/0/flows/0/hooks/0/cadence",
            serde_json::json!(2),
            "SAND_PROGRAM_FLOW",
            "/flows/0/hooks/0/cadence",
        ),
        (
            "/modules/0/flows/1/hooks/0/cadence",
            serde_json::json!(0),
            "SAND_PROGRAM_FLOW",
            "/flows/1/hooks/0/cadence",
        ),
        (
            "/modules/0/flows/1/hooks/0/cadence",
            serde_json::json!(4294967295_u32),
            "SAND_PROGRAM_FLOW",
            "/flows/1/hooks/0/cadence",
        ),
        (
            "/modules/0/flows/1/id",
            serde_json::json!("demo:encounter"),
            "SAND_PROGRAM_ID",
            "/flows/1/id",
        ),
    ];
    for (pointer, value, code, expected_pointer) in cases {
        let mut changed = source.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        let errors = Compiler::compile(&decode(changed)).unwrap_err();
        assert!(
            errors.iter().any(|error| error.code == code
                && error.module == "demo:encounters"
                && error.pointer == expected_pointer),
            "{pointer}: {errors:?}"
        );
    }
}

#[test]
fn guards_bounds_and_conflicts_use_shared_validation() {
    let mut program = program();
    program.modules[0].flows[1].transitions[0].when =
        program.modules[0].flows[0].transitions[0].when.clone();
    assert!(
        Compiler::compile(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_CONTEXT"
                && error.pointer == "/flows/1/transitions/0/when/score")
    );
    program.modules[0].flows[1].transitions[0].when = None;
    program.modules[0].states[0].fields[0].bounds = Some((0, 1));
    assert!(
        Compiler::compile(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.pointer == "/flows/0/transitions/1")
    );
    program.modules[0].states[0].fields[0].bounds = None;
    program.modules[0].flows[0].transitions[0].when = None;
    program.modules[0].flows[0].transitions[0].priority = 0;
    let errors = Compiler::compile(&program).unwrap_err();
    assert!(
        errors.iter().any(|error| error.code == "SAND_PROGRAM_FLOW"
            && error.module == "demo:encounters"
            && error.pointer == "/flows/0"
            && error.message.contains("conflicting transitions")),
        "{errors:?}"
    );
}

#[test]
fn untrusted_json_rejects_unknown_hook_phases_and_noninteger_transitions() {
    let source: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    for (pointer, value) in [
        (
            "/modules/0/flows/0/hooks/0/phase",
            serde_json::json!("callback"),
        ),
        (
            "/modules/0/flows/0/transitions/0/to",
            serde_json::json!(true),
        ),
        (
            "/modules/0/flows/0/transitions/0/to",
            serde_json::json!(1.5),
        ),
        (
            "/modules/0/flows/0/transitions/0/to",
            serde_json::json!(2147483648_i64),
        ),
    ] {
        let mut changed = source.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(
            Compiler::decode(&serde_json::to_vec(&changed).unwrap(), &BTreeMap::new()).is_err()
        );
    }
}
