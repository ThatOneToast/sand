use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn rendered_rustdoc_links_every_contract_production_mechanism() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sand is in the workspace");
    let target = std::env::temp_dir().join(format!(
        "sand-api-rustdoc-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", &target)
        .args(["doc", "-p", "sand", "--all-features", "--no-deps"])
        .output()
        .expect("run cargo doc");
    assert!(
        output.status.success(),
        "cargo doc failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    for (relative, canonical) in [
        (
            "predicate/struct.Predicate.html",
            "sand::predicate::Predicate",
        ),
        (
            "entity/struct.EntityArchetype.html",
            "sand::entity::EntityArchetype",
        ),
        ("state/struct.ScoreVar.html", "sand::state::ScoreVar"),
        ("command/struct.Vec3.html", "sand::command::Vec3"),
        (
            "resource_ref/struct.PredicateId.html",
            "sand::predicate::PredicateId",
        ),
        ("vanilla/enum.Item.html", "sand::vanilla::Item"),
        ("command/struct.Say.html", "sand::command::Say"),
        ("attr.function.html", "sand::function"),
        ("derive.State.html", "sand::State"),
    ] {
        let page = target.join("doc/sand").join(relative);
        let html = fs::read_to_string(&page)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", page.display()));
        assert!(
            html.contains("API Contract"),
            "{} has no contract section",
            page.display()
        );
        assert!(
            html.contains(canonical),
            "{} has no canonical contract link for {canonical}",
            page.display()
        );
    }

    fs::remove_dir_all(&target).expect("remove isolated rustdoc target");
}
