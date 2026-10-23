//! Result-sensitive composition must retain the pre-elision function boundary.
use super::*;
use crate::entity::{EntityStateField, FixedScore};
use crate::function::ExportFunctionRegistryScope;
use crate::ir::{ExecuteOp, ExecuteStoreTarget};

fn field(bounds: Option<(i32, i32)>) -> FixedScore {
    FixedScore::__new("result_probe", "numbers", "value", 1, 0, bounds)
}

fn write(holder: &'static str, bounds: Option<(i32, i32)>, dirty: bool, wrapper: u8) -> Cmd {
    // Start with the actual authoring accessor, not a handwritten scoreboard set.
    let mut node = field(bounds).bind_to(holder, dirty).set(7).0.remove(0);
    if wrapper & 1 != 0 {
        node = Cmd::WithScoreOperands {
            operands: vec![crate::state::ScoreConst::<i32>::new("result_probe_gate", 1).ref_()],
            run: Box::new(node),
        };
    }
    if wrapper & 2 != 0 {
        node = Cmd::WithConditionResources {
            condition: StatCurve::input_raw("probe_gate").gte(0),
            run: Box::new(node),
        };
    }
    node
}

fn gate(node: Cmd) -> Cmd {
    Cmd::Execute {
        operations: vec![ExecuteOp::If(sand_commands::ConditionIr::ScoreMatches {
            holder: ScoreHolder::fake("#gate"),
            objective: "probe_gate".into(),
            range: "1".into(),
        })],
        run: Box::new(node),
    }
}

fn store(node: Cmd, success: bool) -> Cmd {
    let target = ExecuteStoreTarget::Score {
        holder: ScoreHolder::fake("#observed"),
        objective: "probe_result".into(),
    };
    Cmd::Execute {
        operations: vec![if success {
            ExecuteOp::StoreSuccess(target)
        } else {
            ExecuteOp::StoreResult(target)
        }],
        run: Box::new(node),
    }
}

fn drain() -> Vec<crate::component::ComponentRecord> {
    let mut records = Vec::new();
    crate::compiler::export::functions::drain_generated_resources_into(
        &mut records,
        "result_probe",
        None,
        &mut Default::default(),
    )
    .unwrap();
    crate::compiler::export::records::validate_unique_output_identities(&records).unwrap();
    records
}

#[test]
fn nested_result_boundaries_follow_original_cardinality_through_resource_wrappers() {
    for bounds in [
        None,
        Some((i32::MIN, i32::MAX)),
        Some((0, i32::MAX)),
        Some((i32::MIN, 10)),
        Some((0, 10)),
    ] {
        for holder in [
            "#target",
            "@s",
            "*",
            "@r",
            "@e[tag=numeric_probe,limit=1,sort=random]",
        ] {
            for dirty in [false, true] {
                for wrapper in 0..4 {
                    for context in 0..3 {
                        let _scope = ExportFunctionRegistryScope::enter();
                        let node = write(holder, bounds, dirty, wrapper);
                        let nested = match context {
                            0 => store(node, false),
                            1 => store(gate(node), true),
                            _ => Cmd::ReturnRun(Box::new(gate(node))),
                        };
                        crate::function::register_dyn_fn("entry".into(), Actions(vec![nested]));
                        let records = drain();
                        let selected = holder.starts_with('@') && holder != "@s";
                        let clamped =
                            bounds.is_some_and(|b| holder == "*" || b != (i32::MIN, i32::MAX));
                        let sequence = records
                            .iter()
                            .filter(|r| r.path.starts_with("sand/action_sequence/"))
                            .collect::<Vec<_>>();
                        assert_eq!(
                            sequence.len(),
                            usize::from(!selected && (dirty || clamped)),
                            "{holder} {bounds:?} dirty={dirty} wrapper={wrapper} context={context}"
                        );
                        let commits = records
                            .iter()
                            .filter(|r| r.path.starts_with("sand/numeric_commit/"))
                            .collect::<Vec<_>>();
                        assert_eq!(commits.len(), usize::from(selected));
                        for record in sequence.iter().chain(commits.iter()) {
                            assert_eq!(record.content.lines().count(), if dirty { 3 } else { 1 });
                            assert!(!record.content.contains("if score"));
                            assert!(!record.content.contains("return"));
                        }
                        if selected {
                            assert_eq!(
                                records
                                    .iter()
                                    .map(|r| r.content.matches(holder).count())
                                    .sum::<usize>(),
                                1
                            );
                        }
                        if wrapper & 1 != 0 {
                            assert!(!crate::state::score::drain_internal_score_setup().is_empty());
                        }
                    }
                }
            }
        }
    }
}

/// Export exactly the same authoring actions on base and candidate for the
/// maintained native differential harness. No server is launched by this test.
#[test]
#[ignore = "explicit native fixture export; set SAND_NUMERIC_RESULT_FIXTURE to an absolute JSON path"]
fn export_numeric_result_fixture() {
    let destination = std::env::var("SAND_NUMERIC_RESULT_FIXTURE").unwrap();
    assert!(std::path::Path::new(&destination).is_absolute());
    let _scope = ExportFunctionRegistryScope::enter();
    let mut cases = Vec::new();
    for (bound_name, bounds) in [
        ("bounded", Some((0, 10))),
        ("full", Some((i32::MIN, i32::MAX))),
        ("unbounded", None),
        ("lower", Some((0, i32::MAX))),
    ] {
        for (holder_name, holder) in [
            ("fake", "#target"),
            ("self", "@s"),
            ("wildcard", "*"),
            ("random", "@e[tag=numeric_probe,limit=1,sort=random]"),
        ] {
            for dirty in [false, true] {
                for wrapper in 0..4 {
                    // Keep the live probe bounded: exhaustive cross-products
                    // are already covered by the emitted regression above.
                    if bound_name != "bounded" && (holder_name != "fake" || dirty || wrapper != 0)
                        || wrapper != 0 && (holder_name != "fake" || dirty)
                    {
                        continue;
                    }
                    for context in ["store", "nested", "return", "return_nested"] {
                        for success in [false, true] {
                            let name = format!(
                                "{bound_name}_{holder_name}_{dirty}_{wrapper}_{context}_{success}"
                            );
                            let node = write(holder, bounds, dirty, wrapper);
                            let node = if context.ends_with("nested") {
                                gate(node)
                            } else {
                                node
                            };
                            let node = if context.starts_with("return") {
                                let return_path = format!("return/{name}");
                                crate::function::register_dyn_fn(
                                    return_path.clone(),
                                    Actions(vec![Cmd::ReturnRun(Box::new(node))]),
                                );
                                Cmd::Function(format!("result_probe:{return_path}"))
                            } else {
                                node
                            };
                            crate::function::register_dyn_fn(
                                name.clone(),
                                Actions(vec![store(node, success)]),
                            );
                            cases.push(serde_json::json!({"name":name, "holder":holder_name, "dirty":dirty, "wrapper":wrapper, "context":context, "success":success, "bound":bound_name}));
                        }
                    }
                }
            }
        }
    }
    let mut records = drain();
    let mut setup = crate::state::score::drain_internal_score_setup();
    let field = field(None);
    let component_dirty =
        ObjectiveName::logical("result_probe:numbers.reconcile_dirty").to_string();
    for objective in [
        field.objective(),
        field.dirty_objective(),
        component_dirty.clone(),
        "probe_gate".into(),
        "probe_result".into(),
    ] {
        setup.push(format!("scoreboard objectives add {objective} dummy"));
    }
    records.push(crate::component::ComponentRecord::function(
        "result_probe",
        "setup",
        setup,
    ));
    for record in &mut records {
        record.content =
            crate::compiler::export::functions::resolve_local_refs(&record.content, "result_probe");
    }
    std::fs::write(
        destination,
        serde_json::to_vec_pretty(&serde_json::json!({
            "components": records, "cases": cases, "value": field.objective(),
            "field_dirty": field.dirty_objective(), "component_dirty": component_dirty,
        }))
        .unwrap(),
    )
    .unwrap();
}
