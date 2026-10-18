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
            flows: vec![],
            id: "demo:counter_module".parse().unwrap(),
            states: vec![State {
                id: "demo:counter".parse().unwrap(),
                scope: "player".into(),
                revision: 1,
                fields: vec![ScoreField {
                    name: "value".into(),
                    default: 0,
                    bounds: None,
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

#[test]
fn integer_defaults_preserve_the_complete_canonical_pack_across_frontends() {
    const INIT: &str = "data/demo/function/__sand_lifecycle_init.mcfunction";
    for default in [i32::MIN, -20, 0, 100, i32::MAX] {
        let mut program = counter();
        program.requires.push("state_defaults".into());
        program.modules[0].states[0].fields[0].default = default;
        let mut expected = golden();
        let body = String::from_utf8(expected[INIT].clone()).unwrap();
        let original = "run scoreboard players set @s sffe99bc0114fb8b 0";
        assert_eq!(body.matches(original).count(), 1);
        expected.insert(
            INIT.into(),
            body.replace(
                original,
                &format!("run scoreboard players set @s sffe99bc0114fb8b {default}"),
            )
            .into_bytes(),
        );
        assert_eq!(Compiler::compile(&program).unwrap().resources, expected);
        let decoded =
            Compiler::decode(&serde_json::to_vec(&program).unwrap(), &BTreeMap::new()).unwrap();
        assert_eq!(Compiler::compile(&decoded).unwrap().resources, expected);
    }
}

#[test]
fn defaults_reject_non_integer_and_out_of_range_json() {
    for invalid in [
        serde_json::json!(true),
        serde_json::json!(1.5),
        serde_json::json!(2147483648_i64),
        serde_json::json!(-2147483649_i64),
    ] {
        let mut document = serde_json::to_value(counter()).unwrap();
        document["modules"][0]["states"][0]["fields"][0]["default"] = invalid;
        assert!(
            Compiler::decode(&serde_json::to_vec(&document).unwrap(), &BTreeMap::new()).is_err()
        );
    }
}

#[test]
fn advertised_capabilities_are_accepted_as_requirements() {
    let capabilities = Compiler::capabilities();
    let mut program = counter();
    program.requires = capabilities["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert!(
        program
            .requires
            .iter()
            .any(|value| value == "state_defaults")
    );
    Compiler::check(&program).unwrap();
    program.requires.push("unknown_future_capability".into());
    assert!(
        Compiler::check(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_CAPABILITY")
    );
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

#[test]
fn long_owner_ids_generate_bounded_distinct_and_order_independent_helpers() {
    let mut program = counter();
    let players = operation(Action::Players { body: vec![] });
    let module = &mut program.modules[0];
    module.id = format!("demo:{}/{}", "a".repeat(100), "b".repeat(100))
        .parse()
        .unwrap();
    module.functions.clear();
    module.tags.clear();
    module.load = vec![players.clone()];
    module.tick = vec![players.clone()];
    for suffix in ["first", "second"] {
        module.functions.push(Function {
            id: format!("demo:{}/{suffix}", "a".repeat(180))
                .parse()
                .unwrap(),
            context: ExecutionContext::Server,
            body: vec![players.clone()],
        });
    }
    let expected = Compiler::compile(&program).unwrap().resources;
    let helpers: Vec<_> = expected
        .keys()
        .filter(|path| path.contains("/__sand_program/"))
        .collect();
    assert_eq!(helpers.len(), 4);
    for path in helpers {
        assert!(
            path.split('/').all(|segment| segment.len() <= 255),
            "{path}"
        );
        assert!(path.len() < 1024, "{path}");
    }
    program.modules[0].functions.reverse();
    assert_eq!(Compiler::compile(&program).unwrap().resources, expected);
}

#[test]
fn authored_resource_names_include_the_extension_in_the_filesystem_limit() {
    let mut program = counter();
    program.modules[0].tags.clear();
    program.modules[0].functions = vec![Function {
        id: format!("demo:{}", "x".repeat(244)).parse().unwrap(),
        context: ExecutionContext::Server,
        body: vec![],
    }];
    Compiler::check(&program).unwrap();
    program.modules[0].functions[0].id = format!("demo:{}", "x".repeat(245)).parse().unwrap();
    let errors = Compiler::check(&program).unwrap_err();
    assert!(errors.iter().any(|error| error.code == "SAND_PROGRAM_PATH"));
}

#[test]
fn complete_resource_path_limits_are_reported_and_enforced() {
    let limit = Compiler::capabilities()["limits"]["resource_path_bytes"]
        .as_u64()
        .unwrap() as usize;
    let mut program = counter();
    let overhead = "data/demo/function/".len() + ".mcfunction".len();
    let count = limit - overhead;
    let path = format!(
        "{}{}",
        "a/".repeat((count - 1) / 2),
        "x".repeat(1 + (count - 1) % 2)
    );
    program.modules[0].tags.clear();
    program.modules[0].functions = vec![Function {
        id: format!("demo:{path}").parse().unwrap(),
        context: ExecutionContext::Server,
        body: vec![],
    }];
    Compiler::check(&program).unwrap();
    for path in [format!("{path}x"), format!("{}x", "a/".repeat(2500))] {
        program.modules[0].functions[0].id = format!("demo:{path}").parse().unwrap();
        let errors = Compiler::compile(&program).unwrap_err();
        assert!(errors.iter().any(|error| error.code == "SAND_PROGRAM_PATH"));
        assert!(Compiler::check(&program).is_err());
    }
}

#[test]
fn format_diagnostics_identify_each_invalid_envelope_field() {
    for (format, version, pointers) in [
        ("wrong", 1, vec!["/format"]),
        ("sand.program", 2, vec!["/format_version"]),
        ("wrong", 2, vec!["/format", "/format_version"]),
    ] {
        let mut program = counter();
        program.format = format.into();
        program.format_version = version;
        let errors = Compiler::check(&program).unwrap_err();
        let actual: Vec<_> = errors
            .iter()
            .filter(|error| error.code == "SAND_PROGRAM_VERSION")
            .map(|error| error.pointer.as_str())
            .collect();
        assert_eq!(actual, pointers);
    }
}

#[test]
fn resource_paths_reject_windows_reserved_segments_on_every_host() {
    let mut program = counter();
    program.modules[0].tags.clear();
    program.modules[0].functions = vec![Function {
        id: "demo:safe".parse().unwrap(),
        context: ExecutionContext::Server,
        body: vec![],
    }];
    for path in [
        "con",
        "prn.extra",
        "aux/entry",
        "nul",
        "com1",
        "com9/entry",
        "lpt1",
        "lpt9.more",
        "folder./entry",
    ] {
        program.modules[0].functions[0].id = format!("demo:{path}").parse().unwrap();
        let errors = Compiler::check(&program).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.code == "SAND_PROGRAM_PATH"
                    && error.message.contains("non-portable")),
            "{path}: {errors:?}"
        );
        assert!(Compiler::compile(&program).is_err());
    }
    for path in ["console", "com0", "com10", "lpt0", "null", "folder/entry"] {
        program.modules[0].functions[0].id = format!("demo:{path}").parse().unwrap();
        Compiler::check(&program).unwrap();
    }
    program.modules[0].functions[0].id = "con:entry".parse().unwrap();
    assert!(Compiler::check(&program).is_err());
}

/// Runtime reports retain logical ownership without publishing synthetic resources.
#[test]
fn runtime_report_preserves_tick_contributors_and_unknown_fork_cost() {
    let pack = Compiler::compile(&counter()).unwrap();
    let report = serde_json::to_value(&pack.runtime_report).unwrap();
    assert_eq!(report["schema_version"], 1);
    let tick = report["tick"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["reference"] == "demo:tick")
        .unwrap();
    assert_eq!(
        tick["contributors"],
        serde_json::json!(["demo:counter_module"])
    );
    assert_eq!(
        report["functions"]["demo:tick"]["owners"],
        serde_json::json!(["demo:counter_module"])
    );
    assert!(tick["evaluations"]["maximum"].is_null());
    assert!(
        !tick["evaluations"]["unknown"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !pack
            .resources
            .keys()
            .any(|path| path.contains("runtime_report"))
    );
    let again = Compiler::compile(&counter()).unwrap();
    assert_eq!(report, serde_json::to_value(again.runtime_report).unwrap());
}

/// Generated branches/player bodies and load spans retain their authored module.
#[test]
fn runtime_report_attributes_helpers_and_load_contributions() {
    let mut program = counter();
    let call = operation(Action::Call {
        function: FunctionReference::Internal {
            id: "demo:increment".parse().unwrap(),
        },
    });
    program.modules[0].functions[0].body = vec![operation(Action::Players {
        body: vec![call.clone(), call],
    })];
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
    program.modules[0].load = vec![operation(Action::Call {
        function: FunctionReference::Internal {
            id: "demo:tick".parse().unwrap(),
        },
    })];
    program.modules[0].tick = program.modules[0].load.clone();
    let pack = Compiler::compile(&program).unwrap();
    let report = serde_json::to_value(&pack.runtime_report).unwrap();
    let helpers: Vec<_> = report["functions"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(id, _)| id.starts_with("demo:__sand_program/"))
        .collect();
    assert!(helpers.len() >= 4);
    for (_, cost) in helpers {
        assert_eq!(cost["owners"], serde_json::json!(["demo:counter_module"]));
    }
    for phase in ["load", "tick"] {
        let id = format!("demo:__sand_lifecycle_{phase}");
        assert_eq!(
            report["functions"][&id]["owners"],
            serde_json::json!(["demo:counter", "demo:counter_module"])
        );
        let entry = report[phase]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["reference"] == id)
            .unwrap();
        assert_eq!(
            entry["contributors"],
            serde_json::json!(["demo:counter", "demo:counter_module"])
        );
    }
    let spans = report["provenance"]["lifecycle_owners"]["demo:__sand_lifecycle_load"]
        .as_array()
        .unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0]["owner"], "demo:counter_module");
    let line = spans[0]["first_line"].as_u64().unwrap() as usize;
    let body =
        std::str::from_utf8(&pack.resources["data/demo/function/__sand_lifecycle_load.mcfunction"])
            .unwrap();
    assert_eq!(body.lines().nth(line - 1), Some("function demo:tick"));
}

/// State-only modules still contribute load, initialization, and tick infrastructure.
#[test]
fn automatic_state_lifecycle_keeps_state_and_module_origins() {
    let pack = Compiler::compile(&counter()).unwrap();
    let report = serde_json::to_value(&pack.runtime_report).unwrap();
    let owners = serde_json::json!(["demo:counter", "demo:counter_module"]);
    for path in [
        "__sand_lifecycle_load",
        "__sand_lifecycle_init",
        "__sand_lifecycle_tick",
    ] {
        assert_eq!(
            report["functions"][format!("demo:{path}")]["owners"],
            owners
        );
    }
    for phase in ["load", "tick"] {
        let entry = report[phase]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["reference"] == format!("demo:__sand_lifecycle_{phase}"))
            .unwrap();
        assert_eq!(entry["contributors"], owners);
    }
}

#[test]
fn bounded_writes_match_complete_constructed_and_decoded_exports() {
    for bounds in [(0, 100), (-10, -5), (7, 7), (i32::MIN, i32::MAX)] {
        let mut program = counter();
        program.requires.push("state_bounds".into());
        program.modules[0].states[0].fields[0].bounds = Some(bounds);
        let score = FieldReference {
            state: "demo:counter".parse().unwrap(),
            field: "value".into(),
        };
        program.modules[0].functions[1].body = vec![
            operation(Action::ScoreSet {
                score: score.clone(),
                value: i32::MAX,
            }),
            operation(Action::ScoreAdd { score, value: -200 }),
        ];
        let mut expected = golden();
        let path = "data/demo/function/increment.mcfunction";
        let original = String::from_utf8(expected[path].clone()).unwrap();
        let init = original.lines().next().unwrap();
        let mut lines = vec![init.to_owned()];
        for command in [
            format!("scoreboard players set @s sffe99bc0114fb8b {}", bounds.1),
            "scoreboard players remove @s sffe99bc0114fb8b 200".into(),
        ] {
            lines.push(command);
            if bounds.0 != i32::MIN {
                lines.push(format!("execute if score @s sffe99bc0114fb8b matches ..{} run scoreboard players set @s sffe99bc0114fb8b {}", bounds.0 - 1, bounds.0));
            }
            if bounds.1 != i32::MAX {
                lines.push(format!("execute if score @s sffe99bc0114fb8b matches {}.. run scoreboard players set @s sffe99bc0114fb8b {}", bounds.1 + 1, bounds.1));
            }
        }
        expected.insert(path.into(), lines.join("\n").into_bytes());
        assert_eq!(Compiler::compile(&program).unwrap().resources, expected);
        let decoded =
            Compiler::decode(&serde_json::to_vec(&program).unwrap(), &BTreeMap::new()).unwrap();
        assert_eq!(Compiler::compile(&decoded).unwrap().resources, expected);
    }
}

#[test]
fn invalid_bounds_are_source_located_and_strictly_typed() {
    let mut program = counter();
    program.modules[0].states[0].fields[0].bounds = Some((100, 0));
    let errors = Compiler::check(&program).unwrap_err();
    assert!(
        errors.iter().any(|error| error.code == "SAND_PROGRAM_STATE"
            && error.pointer == "/states/0/fields/0/bounds")
    );
    for invalid in [
        serde_json::json!([0]),
        serde_json::json!([0, 1, 2]),
        serde_json::json!([false, 100]),
        serde_json::json!([0, 2147483648_i64]),
    ] {
        let mut document = serde_json::to_value(counter()).unwrap();
        document["modules"][0]["states"][0]["fields"][0]["bounds"] = invalid;
        assert!(
            Compiler::decode(&serde_json::to_vec(&document).unwrap(), &BTreeMap::new()).is_err()
        );
    }
}

#[test]
fn minimum_signed_addition_splits_before_applying_bounds() {
    let mut program = counter();
    program.modules[0].states[0].fields[0].bounds = Some((-2, 2));
    if let Action::ScoreAdd { value, .. } = &mut program.modules[0].functions[1].body[0].action {
        *value = i32::MIN;
    }
    let pack = Compiler::compile(&program).unwrap();
    assert_eq!(
        String::from_utf8(pack.resources["data/demo/function/increment.mcfunction"].clone())
            .unwrap(),
        "function demo:__sand_lifecycle_init\nscoreboard players remove @s sffe99bc0114fb8b 2147483647\nscoreboard players remove @s sffe99bc0114fb8b 1\nexecute if score @s sffe99bc0114fb8b matches ..-3 run scoreboard players set @s sffe99bc0114fb8b -2\nexecute if score @s sffe99bc0114fb8b matches 3.. run scoreboard players set @s sffe99bc0114fb8b 2"
    );
}

#[test]
fn global_state_initializes_without_players_and_uses_one_shared_holder() {
    let mut program = counter();
    program.requires.push("global_state".into());
    program.modules[0].states[0].scope = "global".into();
    program.modules[0].states[0].fields[0].default = 10;
    program.modules[0].functions[1].context = ExecutionContext::Server;
    let expected = Compiler::compile(&program).unwrap().resources;
    let decoded =
        Compiler::decode(&serde_json::to_vec(&program).unwrap(), &BTreeMap::new()).unwrap();
    assert_eq!(Compiler::compile(&decoded).unwrap().resources, expected);
    assert!(!expected.contains_key("data/demo/function/__sand_lifecycle_init.mcfunction"));
    let holder = sand_core::GlobalStateHolder::new("demo:counter");
    let holder = holder.as_str();
    let load =
        String::from_utf8(expected["data/demo/function/__sand_lifecycle_load.mcfunction"].clone())
            .unwrap();
    assert!(load.contains(&format!("execute unless score {holder} sffe99bc0114fb8b matches -2147483648.. run scoreboard players set {holder} sffe99bc0114fb8b 10")));
    assert!(!load.contains("@s"));
    assert_eq!(
        String::from_utf8(expected["data/demo/function/increment.mcfunction"].clone()).unwrap(),
        format!("scoreboard players add {holder} sffe99bc0114fb8b 1")
    );
    // Global ownership never relaxes player-self requirements for another State.
    program.modules[0].states[0].scope = "player".into();
    assert!(
        Compiler::check(&program)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SAND_PROGRAM_CONTEXT")
    );
}

#[test]
fn global_load_mutations_follow_canonical_initialization() {
    let mut program = counter();
    program.modules[0].states[0].scope = "global".into();
    program.modules[0].states[0].fields[0].default = 10;
    program.modules[0].load = vec![operation(Action::ScoreAdd {
        score: FieldReference {
            state: "demo:counter".parse().unwrap(),
            field: "value".into(),
        },
        value: 7,
    })];
    let pack = Compiler::compile(&program).unwrap();
    let load = String::from_utf8(
        pack.resources["data/demo/function/__sand_lifecycle_load.mcfunction"].clone(),
    )
    .unwrap();
    let holder = sand_core::GlobalStateHolder::new("demo:counter");
    let initialized = load
        .find(&format!(
            "run scoreboard players set {} sffe99bc0114fb8b 10",
            holder.as_str()
        ))
        .unwrap();
    let mutated = load
        .find(&format!(
            "scoreboard players add {} sffe99bc0114fb8b 7",
            holder.as_str()
        ))
        .unwrap();
    assert!(initialized < mutated, "{load}");
}
