use sand::advanced::compiler::*;
use std::collections::BTreeMap;
const COUNTER: &[u8] = include_bytes!("../../tests/fixtures/program/counter.sand.json");

fn operation(action: Action) -> Operation {
    Operation {
        action,
        origin: None,
    }
}
fn counter() -> Program {
    Program {
        format: "sand.program".into(),
        format_version: 1,
        target: ProgramTarget {
            minecraft: "26.2".into(),
        },
        pack: Pack {
            namespace: "demo".into(),
            description: "Player counter".into(),
        },
        requires: vec![],
        modules: vec![Module {
            id: "demo:counter_module".parse().unwrap(),
            states: vec![State {
                id: "demo:counter".parse().unwrap(),
                scope: "player".into(),
                revision: 1,
                fields: vec![ScoreField {
                    name: "value".into(),
                    default: 0,
                }],
            }],
            functions: vec![
                Function {
                    id: "demo:tick".parse().unwrap(),
                    context: ExecutionContext::Server,
                    body: vec![operation(Action::Players {
                        body: vec![operation(Action::Call {
                            function: FunctionReference::Internal {
                                id: "demo:increment".parse().unwrap(),
                            },
                        })],
                    })],
                },
                Function {
                    id: "demo:increment".parse().unwrap(),
                    context: ExecutionContext::Player,
                    body: vec![operation(Action::ScoreAdd {
                        score: FieldReference {
                            state: "demo:counter".parse().unwrap(),
                            field: "value".into(),
                        },
                        value: 1,
                    })],
                },
            ],
            load: vec![],
            tick: vec![],
            tags: vec![TagMembership {
                tag: "minecraft:tick".parse().unwrap(),
                function: FunctionReference::Internal {
                    id: "demo:tick".parse().unwrap(),
                },
            }],
        }],
    }
}
fn golden() -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &std::path::Path, path: &std::path::Path, map: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, map);
            } else {
                map.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/program/golden");
    let mut map = BTreeMap::new();
    visit(&root, &root, &mut map);
    map
}
// Any accidental inventory read in embedded compilation fails this test binary.
fn forbidden_schema() -> sand_core::entity::StateSchema {
    panic!("portable compiler read inventory")
}
fn forbidden_provision() -> Vec<String> {
    panic!("portable compiler invoked a callback")
}
fn forbidden_body(_: &'static str) -> Vec<String> {
    panic!("portable compiler invoked a callback")
}
fn forbidden_migration(_: &'static str, _: u32, _: u32) -> Vec<String> {
    panic!("portable compiler invoked a callback")
}
inventory::submit!(sand_core::StateHookDescriptor {
    schema: forbidden_schema,
    provision: forbidden_provision,
    initialize: forbidden_body,
    tick: forbidden_body,
    reconcile: forbidden_body,
    cleanup: forbidden_body,
    migrate: forbidden_migration
});

#[test]
fn constructed_decoded_and_split_programs_match_complete_golden() {
    let expected = golden();
    assert_eq!(Compiler::compile(&counter()).unwrap().resources, expected);
    let decoded = Compiler::decode(COUNTER, &BTreeMap::new()).unwrap();
    assert_eq!(Compiler::compile(&decoded).unwrap().resources, expected);
    let modules = BTreeMap::from([
        (
            "state.sand.json".into(),
            include_bytes!("../../tests/fixtures/program/split/state.sand.json").to_vec(),
        ),
        (
            "tick.sand.json".into(),
            include_bytes!("../../tests/fixtures/program/split/tick.sand.json").to_vec(),
        ),
    ]);
    let split = Compiler::decode(
        include_bytes!("../../tests/fixtures/program/split/program.sand.json"),
        &modules,
    )
    .unwrap();
    assert_eq!(Compiler::compile(&split).unwrap().resources, expected);
}
#[test]
fn failed_repeated_and_concurrent_compilation_is_isolated() {
    let expected = golden();
    let mut invalid = counter();
    invalid.modules[0].functions[1].context = ExecutionContext::Server;
    assert!(Compiler::compile(&invalid).is_err());
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..3 {
                    assert_eq!(Compiler::compile(&counter()).unwrap().resources, golden());
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(Compiler::compile(&counter()).unwrap().resources, expected);
}
#[test]
fn identities_ignore_declaration_order_and_source_origins() {
    let mut program = counter();
    program.modules[0].functions.reverse();
    program.modules[0].functions[0].body[0].origin =
        Some(serde_json::json!({"node":"node-a", "layout":[12,34]}));
    assert_eq!(Compiler::compile(&program).unwrap().resources, golden());
}
#[test]
fn unreachable_context_errors_keep_module_pointer_and_origin() {
    let mut program = counter();
    let function = &mut program.modules[0].functions[1];
    function.id = "demo:unreachable".parse().unwrap();
    function.context = ExecutionContext::Server;
    function.body[0].origin = Some(serde_json::json!({"node":"bad","port":"score"}));
    let errors = Compiler::check(&program).unwrap_err();
    let error = errors
        .iter()
        .find(|error| error.code == "SAND_PROGRAM_CONTEXT")
        .unwrap();
    assert_eq!(error.module, "demo:counter_module");
    assert_eq!(error.pointer, "/functions/1/body/0/action/score");
    assert_eq!(error.origin.as_ref().unwrap()["node"], "bad");
}
#[test]
fn branch_uses_one_guard_and_returns_before_otherwise() {
    let mut program = counter();
    let score = FieldReference {
        state: "demo:counter".parse().unwrap(),
        field: "value".into(),
    };
    program.modules[0].functions[1].body = vec![operation(Action::Branch {
        condition: ScoreComparison {
            score: score.clone(),
            min: Some(0),
            max: Some(0),
        },
        then: vec![operation(Action::ScoreSet {
            score: score.clone(),
            value: 1,
        })],
        otherwise: vec![operation(Action::ScoreSet { score, value: 2 })],
    })];
    let output = Compiler::compile(&program).unwrap().resources;
    let (_, body) = output
        .iter()
        .find(|(path, _)| path.ends_with("/branch.mcfunction"))
        .unwrap();
    let body = std::str::from_utf8(body).unwrap();
    assert!(body.contains(" run return run function "));
    assert_eq!(body.matches("execute if score").count(), 1);
    assert!(body.lines().nth(1).unwrap().ends_with("/otherwise"));
}
#[test]
fn strict_decode_and_constructed_limits_are_enforced() {
    for input in [br#"{"format":"sand.program","format":"sand.program"}"#.as_slice(),
        br#"{"format":"sand.program","format_version":1,"target":{"minecraft":"26.2","minecraft":"26.2"},"pack":{},"modules":[]}"#.as_slice()] {
        assert!(Compiler::decode(input,&BTreeMap::new()).unwrap_err()[0].message.contains("duplicate"));
    }
    let mut program = counter();
    let mut body = Vec::new();
    for _ in 0..34 {
        body = vec![operation(Action::Players { body })];
    }
    program.modules[0].functions[0].body = body;
    assert!(
        Compiler::check(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_LIMIT")
    );
    for target in ["latest", "bogus", "1.21", "27.9"] {
        let mut program = counter();
        program.target.minecraft = target.into();
        assert!(
            Compiler::check(&program)
                .unwrap_err()
                .iter()
                .any(|error| error.code == "SAND_PROGRAM_TARGET")
        );
    }
}

#[test]
fn decode_errors_locate_nested_fields_and_duplicate_keys() {
    let mut value: serde_json::Value = serde_json::from_slice(COUNTER).unwrap();
    value["modules"][0]["functions"][1]["body"][0]["action"]["value"] = true.into();
    value["modules"][0]["functions"][1]["body"][0]["origin"] =
        serde_json::json!({"node":"bad_value"});
    let error = Compiler::decode(&serde_json::to_vec(&value).unwrap(), &BTreeMap::new())
        .unwrap_err()
        .remove(0);
    assert_eq!(error.module, "demo:counter_module");
    assert!(
        error.pointer.starts_with("/functions/1/body/0/action"),
        "{}",
        error.pointer
    );
    assert_eq!(error.origin.unwrap()["node"], "bad_value");
    let error = Compiler::decode(
        br#"{"target":{"minecraft":"26.2","minecraft":"26.1"}}"#,
        &BTreeMap::new(),
    )
    .unwrap_err()
    .remove(0);
    assert_eq!(error.pointer, "/target/minecraft");
}

#[test]
fn capabilities_external_context_and_tag_entries_are_checked() {
    let mut program = counter();
    program.modules[0].functions[0].body = vec![operation(Action::Raw {
        command: "say explicit".into(),
    })];
    assert!(
        Compiler::check(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_CAPABILITY")
    );
    program.requires.push("raw_commands".into());
    Compiler::check(&program).unwrap();
    program.modules[0].functions[0].body = vec![operation(Action::Call {
        function: FunctionReference::External {
            id: "other:player".parse().unwrap(),
            context: ExecutionContext::Player,
        },
    })];
    assert!(
        Compiler::check(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_CONTEXT")
    );
    program.modules[0].functions[0].body.clear();
    program.modules[0].tags.push(TagMembership {
        tag: "demo:player_actions".parse().unwrap(),
        function: FunctionReference::Internal {
            id: "demo:increment".parse().unwrap(),
        },
    });
    Compiler::check(&program).unwrap();
    program.modules[0].tags.last_mut().unwrap().tag = "minecraft:load".parse().unwrap();
    assert!(
        Compiler::check(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_CONTEXT")
    );
}

#[test]
fn portable_raw_validation_does_not_inherit_a_rust_export_registry() {
    let _scope = sand_commands::export_registry::ExportRegistryGuard::enter().unwrap();
    let line = sand_commands::Inventory::of(sand_commands::Target::self_())
        .set(sand_commands::ItemSlot::AnyHotbar, "demo:marker");
    let profile = sand_commands::CommandProfile::new("26.2", false);
    assert!(sand_commands::render::validate_collected_line(&line, &profile).is_err());
    let mut program = counter();
    program.requires.push("raw_commands".into());
    program.modules[0].functions[0].body = vec![operation(Action::Raw {
        command: line.clone(),
    })];
    let pack = Compiler::compile(&program).unwrap();
    assert_eq!(
        pack.resources["data/demo/function/tick.mcfunction"],
        line.into_bytes()
    );
}

#[test]
fn input_and_generated_output_limits_fail_before_publication() {
    let oversized = vec![b' '; 1024 * 1024 + 1];
    assert_eq!(
        Compiler::decode(&oversized, &BTreeMap::new()).unwrap_err()[0].code,
        "SAND_PROGRAM_LIMIT"
    );
    let mut program = counter();
    program.modules = vec![program.modules[0].clone(); 65];
    assert_eq!(
        Compiler::check(&program).unwrap_err()[0].code,
        "SAND_PROGRAM_LIMIT"
    );

    let mut program = counter();
    program.requires.push("raw_commands".into());
    program.modules[0].functions[0].body = vec![
        operation(Action::Raw {
            command: format!("say {}", "x".repeat(20_000))
        });
        60
    ];
    let error = Compiler::compile(&program).unwrap_err().remove(0);
    assert_eq!(error.code, "SAND_PROGRAM_LIMIT");
    assert!(error.message.contains("generated output"));

    let mut program = counter();
    program.modules[0].functions = (0..6000)
        .map(|index| Function {
            id: format!("demo:f{index}").parse().unwrap(),
            context: ExecutionContext::Server,
            body: vec![operation(Action::Players { body: vec![] })],
        })
        .collect();
    program.modules[0].tags.clear();
    let error = Compiler::compile(&program).unwrap_err().remove(0);
    assert_eq!(error.code, "SAND_PROGRAM_LIMIT");
    assert!(error.message.contains("generated output"));
}

#[test]
fn external_references_cannot_alias_declared_functions() {
    for context in [ExecutionContext::Server, ExecutionContext::Player] {
        let reference = FunctionReference::External {
            id: "demo:increment".parse().unwrap(),
            context,
        };
        for entry in ["call", "load", "tick", "custom"] {
            let mut program = counter();
            let pointer = if entry == "call" {
                program.modules[0].functions[0].body = vec![Operation {
                    action: Action::Call {
                        function: reference.clone(),
                    },
                    origin: Some(serde_json::json!({"node":"alias"})),
                }];
                "/functions/0/body/0/action/function"
            } else {
                let tag = match entry {
                    "load" => "minecraft:load",
                    "tick" => "minecraft:tick",
                    _ => "demo:custom",
                };
                program.modules[0].tags = vec![TagMembership {
                    tag: tag.parse().unwrap(),
                    function: reference.clone(),
                }];
                "/tags/0/function"
            };
            let errors = Compiler::check(&program).unwrap_err();
            let error = errors
                .iter()
                .find(|error| error.code == "SAND_PROGRAM_REFERENCE")
                .unwrap();
            assert_eq!(error.module, "demo:counter_module");
            assert_eq!(error.pointer, pointer);
            if entry == "call" {
                assert_eq!(error.origin.as_ref().unwrap()["node"], "alias");
            }
        }
    }
}

#[test]
fn compiled_resources_reject_file_ancestor_collisions_in_either_order() {
    for reverse in [false, true] {
        let mut program = counter();
        let module = &mut program.modules[0];
        module.functions = ["demo:foo", "demo:foo.mcfunction/bar"]
            .into_iter()
            .map(|id| Function {
                id: id.parse().unwrap(),
                context: ExecutionContext::Server,
                body: vec![],
            })
            .collect();
        module.tags.clear();
        if reverse {
            module.functions.reverse();
        }
        let errors = Compiler::check(&program).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.code == "SAND_PROGRAM_COLLISION")
        );
        assert!(Compiler::compile(&program).is_err());
    }
}

#[test]
fn lifecycle_helpers_are_owned_by_the_pack_namespace() {
    let mut program = counter();
    program.modules[0].functions.clear();
    program.modules[0].tags.clear();
    program.modules[0].tick = vec![operation(Action::Players {
        body: vec![operation(Action::ScoreAdd {
            score: FieldReference {
                state: "demo:counter".parse().unwrap(),
                field: "value".into(),
            },
            value: 1,
        })],
    })];
    let mut helper_paths = Vec::new();
    for namespace in ["first", "second"] {
        program.pack.namespace = namespace.into();
        let pack = Compiler::compile(&program).unwrap();
        let (path, body) = pack
            .resources
            .iter()
            .find(|(path, _)| path.contains("/__sand_program/"))
            .unwrap();
        assert!(path.starts_with(&format!("data/{namespace}/function/")));
        assert!(
            String::from_utf8_lossy(body)
                .starts_with(&format!("function {namespace}:__sand_lifecycle_init\n"))
        );
        let tick = String::from_utf8_lossy(
            &pack.resources[&format!("data/{namespace}/function/__sand_lifecycle_tick.mcfunction")],
        );
        assert!(tick.contains(&format!(
            "execute as @a run function {namespace}:__sand_program/"
        )));
        helper_paths.push(path.clone());
    }
    assert_ne!(helper_paths[0], helper_paths[1]);
}
