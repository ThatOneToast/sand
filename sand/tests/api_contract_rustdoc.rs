use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn representative_facade_rustdoc_is_substantive_and_local() {
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

    let root = target.join("doc/sand");
    for (relative, required) in [
        ("predicate/struct.Predicate.html", "Minecraft behavior"),
        ("entity/struct.EntityArchetype.html", "lifecycle definition"),
        ("state/struct.ScoreVar.html", "scoreboard"),
        ("command/struct.Vec3.html", "Minecraft"),
        (
            "resource_ref/struct.PredicateId.html",
            "predicate resource identifier",
        ),
        ("vanilla/enum.Item.html", "minecraft:item"),
        ("command/struct.Say.html", "say"),
        (
            "attr.function.html",
            "Registers a Rust function as an exported Minecraft function",
        ),
        ("derive.State.html", "definition-owned typed field handles"),
        ("build/struct.World.html", "world"),
    ] {
        let html = page(&root, relative);
        assert!(html.contains(required), "{relative} lacks {required:?}");
        assert!(
            !pointer_is_primary_content(&html),
            "{relative} falls back to CLI-only documentation"
        );
    }

    for (relative, anchor, required) in [
        (
            "entity/struct.EntityArchetype.html",
            "method.new",
            "Minecraft behavior",
        ),
        ("state/struct.ScoreVar.html", "method.clamp", "Parameters"),
        ("command/struct.Vec3.html", "method.new", "Returns"),
        (
            "component/struct.Advancement.html",
            "method.parent",
            "Minecraft behavior",
        ),
        (
            "resourcepack/struct.Color.html",
            "method.from_u32",
            "Minecraft behavior",
        ),
        (
            "component/struct.AdvancementDisplay.html",
            "method.show_toast",
            "toast",
        ),
        ("vanilla/enum.Item.html", "variant.Stone", "stone"),
    ] {
        let html = page(&root, relative);
        let section = anchor_section(&html, anchor)
            .unwrap_or_else(|| panic!("{relative} has no rendered anchor {anchor}"));
        assert!(
            section.contains(required),
            "{relative}#{anchor} lacks {required:?}"
        );
        assert!(!section.contains("sand api show"));
    }

    fs::remove_dir_all(&target).expect("remove isolated rustdoc target");
}

fn page(root: &Path, relative: &str) -> String {
    let path = root.join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn anchor_section<'a>(html: &'a str, anchor: &str) -> Option<&'a str> {
    let marker = format!("id=\"{anchor}\"");
    let start = html.find(&marker)?;
    let rest = &html[start + marker.len()..];
    let item_kind = anchor.split_once('.')?.0;
    let next_item = format!("id=\"{item_kind}.");
    let end = rest.find(&next_item).unwrap_or(rest.len());
    Some(&rest[..end])
}

fn pointer_is_primary_content(html: &str) -> bool {
    html.contains("sand api show")
        && !html.contains("Minecraft behavior")
        && !html.contains("Context")
}
