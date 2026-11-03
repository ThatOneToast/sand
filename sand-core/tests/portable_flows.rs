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

#[test]
fn copied_transition_hooks_are_bounded_before_materializing_the_plan() {
    let mut program = program();
    program.modules[0].flows.truncate(1);
    let flow = &mut program.modules[0].flows[0];
    flow.transitions = (2..402)
        .map(|from| FlowTransition {
            from,
            to: 1,
            when: None,
            priority: 0,
        })
        .collect();
    flow.hooks = vec![flow.hooks[0].clone(); 400];
    // Only 800 declarations, but 160,000 copied enter calls. Both directly
    // constructed and decoded inputs must fail before allocating that output.
    for input in [
        program.clone(),
        decode(serde_json::to_value(&program).unwrap()),
    ] {
        let errors = Compiler::compile(&input).unwrap_err();
        assert!(
            errors.iter().any(|e| e.code == "SAND_PROGRAM_LIMIT"
                && e.module == "demo:encounters"
                && e.pointer == "/flows/0"),
            "{errors:?}"
        );
    }
}

#[test]
fn expanded_hook_bytes_obey_resource_and_pack_limits_before_lowering() {
    for (transitions, hooks) in [(300, 200), (1, 1800)] {
        let mut program = program();
        program.modules[0].flows.truncate(1);
        let flow = &mut program.modules[0].flows[0];
        flow.transitions = (2..2 + transitions)
            .map(|from| FlowTransition {
                from,
                to: 1,
                when: None,
                priority: 0,
            })
            .collect();
        let mut hook = flow.hooks[0].clone();
        hook.function = FunctionReference::External {
            id: format!("external:{}hook", "leaf/".repeat(120))
                .parse()
                .unwrap(),
            context: ExecutionContext::Player,
        };
        flow.hooks = vec![hook; hooks];
        // These remain below the operation budget, but exceed either the
        // complete pack byte budget or one generated transition's byte budget.
        let errors = Compiler::compile(&program).unwrap_err();
        assert!(
            errors.iter().any(|e| e.code == "SAND_PROGRAM_LIMIT"
                && e.pointer == "/flows/0"
                && e.message.contains("expanded flow hooks")),
            "{errors:?}"
        );
    }
}

#[test]
fn shared_lifecycle_evaluates_flows_before_authored_tick_writes() {
    let mut source: serde_json::Value = serde_json::from_slice(FIXTURE).unwrap();
    source["modules"][0]["tick"] = serde_json::json!([{
        "action": {"op": "score_set", "score": {"state": "demo:world", "field": "phase"}, "value": 2}
    }]);
    let pack = Compiler::compile(&decode(source)).unwrap();
    let tick = String::from_utf8(
        pack.resources["data/demo/function/__sand_lifecycle_tick.mcfunction"].clone(),
    )
    .unwrap();
    let lines: Vec<_> = tick.lines().collect();
    assert_eq!(lines.len(), 4, "{tick}");
    assert_eq!(
        lines[0],
        "execute as @a run function demo:__sand_lifecycle_init"
    );
    assert!(
        lines[1].starts_with("execute as @a at @s run function demo:__sand_transition/flow_"),
        "{tick}"
    );
    assert!(
        lines[2].starts_with("function demo:__sand_transition/flow_"),
        "{tick}"
    );
    assert!(
        lines[3].starts_with("scoreboard players set #sand_demo_world_")
            && lines[3].ends_with(" 2"),
        "{tick}"
    );
}

#[test]
fn player_initializer_calls_emitted_owned_helper_in_each_namespace() {
    let first = Compiler::compile(&program()).unwrap();
    for namespace in ["demo", "isolated", "demo"] {
        let mut input = program();
        input.pack.namespace = namespace.into();
        let pack = Compiler::compile(&input).unwrap();
        let init_path = format!("data/{namespace}/function/__sand_lifecycle_init.mcfunction");
        let init = std::str::from_utf8(&pack.resources[&init_path]).unwrap();
        let target = init.split("run function ").nth(1).unwrap();
        let (actual_namespace, path) = target.split_once(':').unwrap();
        assert_eq!(actual_namespace, namespace);
        assert_eq!(path, "sand/state_init/54af896e");
        let body = &pack.resources[&format!("data/{namespace}/function/{path}.mcfunction")];
        // Preserve the complete pre-fast-path initializer, including suppression
        // on every command and presence publication last.
        assert_eq!(
            body,
            include_bytes!(
                "../../tests/fixtures/program/flows-golden/data/demo/function/sand/state_init/54af896e.mcfunction"
            )
        );
        assert!(
            !pack
                .resources
                .values()
                .any(|bytes| String::from_utf8_lossy(bytes).contains("__sand_local:"))
        );
        let report = serde_json::to_value(&pack.runtime_report).unwrap();
        assert_eq!(
            report["functions"][target]["owners"],
            serde_json::json!(["demo:player"])
        );
        if namespace == "demo" {
            assert_eq!(first.resources, pack.resources);
        }
    }
}
