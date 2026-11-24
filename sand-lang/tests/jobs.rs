mod support;

use support::*;

const JOB: &str = include_str!("../../examples/lang_supply_job/pack.sand");

#[test]
fn jobs_use_the_shared_lifetime_and_pass_the_index_to_the_callback() {
    let analysis = analyze_one(JOB);
    let pack = compile(&analysis);
    for expected in [
        "job_dispatch",
        "function supply_job:inspect_supply",
        "function supply_job:finish_inspection",
    ] {
        assert!(
            pack.resources.iter().any(|(name, content)| {
                name.ends_with(".mcfunction") && String::from_utf8_lossy(content).contains(expected)
            }),
            "missing {expected}"
        );
    }
    let program = analysis.program.unwrap();
    let [job] = program
        .modules
        .iter()
        .flat_map(|module| &module.jobs)
        .collect::<Vec<_>>()[..]
    else {
        panic!("expected one job");
    };
    assert_eq!(job.per_tick, Some(2));
    assert!(job.index.field.ends_with("_index"));
    let portable = serde_json::to_vec(&program).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&portable, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn jobs_reject_wrong_callback_signatures_and_budgets() {
    for (from, to, expected) in [
        (
            "inspect_supply(index: number)",
            "inspect_supply(index: float)",
            "a job step takes one number and returns no value",
        ),
        ("per_tick: 2", "per_tick: 65", "from 1 to 64"),
    ] {
        let analysis = analyze_one(&JOB.replace(from, to));
        let errors = analysis.compile().expect_err("invalid job compiled");
        assert!(
            errors
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{errors:?}"
        );
    }
}
