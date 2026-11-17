use sand::advanced::compiler::{Action, Compiler, DataOperand, Program};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn field(name: &str) -> Value {
    json!({"state":"data:values", "field":name})
}
fn stored(name: &str) -> Value {
    json!({"field":field(name)})
}
fn document() -> Value {
    json!({
        "format":"sand.program", "format_version":1, "target":{"minecraft":"26.2"},
        "pack":{"namespace":"data", "description":"Owned storage regression"},
        "modules":[{"id":"data:module", "states":[{
            "id":"data:values", "scope":"player", "revision":1,
            "fields":[{"name":"index"}, {"name":"result"}],
            "data_fields":[
                {"name":"text", "default":{"type":"list","value":[]}},
                {"name":"other", "default":{"type":"list","value":[]}}
            ]
        }], "functions":[{"id":"data:run", "context":"player", "body":[
            {"action":{"op":"data_modify", "target":stored("text"), "operator":{"kind":"set"}, "source":{"kind":"value", "value":{"type":"list", "value":[{"type":"string", "value":"$(uuid)\n\0🌍"}]}}}},
            {"action":{"op":"push_frame", "fields":[field("text"),field("index")]}},
            {"action":{"op":"data_modify", "target":stored("other"), "operator":{"kind":"append"}, "source":{"kind":"reference", "value":{"field":field("text"), "path":[{"kind":"index", "index":field("index")}]}}}},
            {"action":{"op":"data_read", "score":field("index"), "source":{"field":field("text"), "path":[{"kind":"index", "index":field("index")}]}}},
            {"action":{"op":"data_equal", "score":field("result"), "left":stored("text"), "right":stored("other")}},
            {"action":{"op":"pop_frame", "fields":[field("text"),field("index")]}},
            {"action":{"op":"message", "audience":"player", "channel":"chat", "text":[{"kind":"stored","value":field("text")}]}},
            {"action":{"op":"data_remove", "target":{"field":field("other"), "path":[{"kind":"index", "index":0}]}}}
        ]}]}]
    })
}
fn decode(document: Value) -> Program {
    Compiler::decode(document.to_string().as_bytes(), &BTreeMap::new()).unwrap()
}

#[test]
fn owned_storage_exports_safe_macros_frames_and_plain_text_deterministically() {
    let program = decode(document());
    let pack = Compiler::compile(&program).unwrap();
    assert_eq!(
        pack.resources,
        Compiler::compile(&program).unwrap().resources
    );
    let commands = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .flat_map(|(_, bytes)| std::str::from_utf8(bytes).unwrap().lines())
        .collect::<Vec<_>>();
    assert!(
        commands
            .iter()
            .any(|line| line.contains("stack[-1].f0 set from storage"))
    );
    assert!(
        commands
            .iter()
            .any(|line| line.contains("\\u000a\\u0000🌍"))
    );
    assert!(
        commands
            .iter()
            .any(|line| line.contains("\"interpret\":false") && line.contains("__program_text"))
    );
    for line in commands.iter().filter(|line| line.starts_with('$')) {
        assert!(
            !line.contains("$(uuid)\\u000a"),
            "authored text entered a macro template: {line}"
        );
    }
}

#[test]
fn storage_references_preserve_context_types_and_checked_projections() {
    let mut invalid = Vec::new();
    let mut server = document();
    server["modules"][0]["functions"][0]["context"] = json!("server");
    invalid.push(server);
    let mut duplicate = document();
    duplicate["modules"][0]["states"][0]["data_fields"][0]["name"] = json!("index");
    invalid.push(duplicate);
    let mut member = document();
    member["modules"][0]["functions"][0]["body"][0]["action"]["target"]["path"] =
        json!([{"kind":"member", "name":"x run say unsafe"}]);
    invalid.push(member);
    let mut score = document();
    score["modules"][0]["functions"][0]["body"][3]["action"]["score"] = field("text");
    invalid.push(score);
    let mut negative = document();
    negative["modules"][0]["functions"][0]["body"][0]["action"]["target"]["path"] =
        json!([{"kind":"index", "index":-1}]);
    invalid.push(negative);
    for invalid in invalid {
        assert!(Compiler::compile(&decode(invalid)).is_err());
    }
    let mut raw = decode(document());
    let Action::DataModify { source, .. } = &mut raw.modules[0].functions[0].body[0].action else {
        unreachable!()
    };
    *source = DataOperand::Value {
        value: sand::data::NbtValue::List(vec![sand::data::NbtValue::raw("[]")]),
    };
    assert!(Compiler::compile(&raw).is_err());
}
