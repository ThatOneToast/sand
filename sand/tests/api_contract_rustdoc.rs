use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use sand_api_contract::ApiKind;

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

    for relative in [
        "resourcepack/struct.Color.html",
        "component/struct.Advancement.html",
    ] {
        let page = target.join("doc/sand").join(relative);
        let html = fs::read_to_string(&page)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", page.display()));
        assert!(
            html.contains("API Contract") && html.contains("sand api show"),
            "{} has no direct contract discovery guidance",
            page.display()
        );
    }

    let family_paths = sand::__private::api_contract::INSTALLED_FAMILY_API_PATHS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let registrations = sand::__private::api_contract::INSTALLED_FACADE_CONTRACTS;
    let mut family_pages = BTreeSet::new();
    let mut family_members = BTreeSet::new();
    for registration in registrations
        .iter()
        .filter(|registration| family_paths.contains(registration.canonical_path))
        .filter(|registration| {
            [
                "sand::component::",
                "sand::condition::",
                "sand::data::",
                "sand::entity::",
                "sand::execute_when::",
                "sand::inventory::",
                "sand::participant::",
                "sand::predicate::",
                "sand::registry::",
                "sand::resource_ref::",
                "sand::resourcepack::",
                "sand::state::",
                "sand::systems::",
                "sand::text::",
                "sand::version::",
            ]
            .iter()
            .any(|prefix| registration.canonical_path.starts_with(prefix))
        })
    {
        let owner = registrations
            .iter()
            .filter(|candidate| {
                page_prefix(candidate.kind).is_some()
                    && (candidate.canonical_path == registration.canonical_path
                        || registration
                            .canonical_path
                            .strip_prefix(candidate.canonical_path)
                            .is_some_and(|suffix| suffix.starts_with("::")))
            })
            .max_by_key(|candidate| candidate.canonical_path.len())
            .unwrap_or_else(|| panic!("no Rustdoc owner for {}", registration.canonical_path));
        family_pages.insert((rustdoc_page(owner), owner.canonical_path));
        if matches!(registration.kind, ApiKind::Method | ApiKind::TraitMethod) {
            family_members.insert((
                rustdoc_page(owner),
                registration
                    .canonical_path
                    .rsplit("::")
                    .next()
                    .expect("member path has a name"),
                registration.canonical_path,
            ));
        }
    }
    for (relative, owner) in family_pages {
        let page = target.join("doc/sand").join(&relative);
        let html = fs::read_to_string(&page).unwrap_or_else(|error| {
            panic!("failed to read {} for {owner}: {error}", page.display())
        });
        assert!(
            html.contains("API Contract") && html.contains("sand api show"),
            "{} ({owner}) has no direct contract discovery guidance",
            page.display()
        );
    }

    for (relative, member, canonical) in family_members {
        let page = target.join("doc/sand").join(&relative);
        let html = fs::read_to_string(&page).unwrap_or_else(|error| {
            panic!("failed to read {} for {canonical}: {error}", page.display())
        });
        let section = rendered_member_section(&html, member, &page, canonical);
        assert!(
            section.contains("class=\"docblock\""),
            "{} exposes {canonical} without member-specific Rustdoc",
            page.display()
        );
    }

    for (relative, member, canonical) in [
        (
            "entity/struct.EntityArchetype.html",
            "new",
            "sand::entity::EntityArchetype::new",
        ),
        (
            "state/struct.ScoreVar.html",
            "clamp",
            "sand::state::ScoreVar::clamp",
        ),
        (
            "command/struct.Vec3.html",
            "new",
            "sand::command::Vec3::new",
        ),
        (
            "component/struct.Advancement.html",
            "parent",
            "sand::component::Advancement::parent",
        ),
        (
            "resourcepack/struct.Color.html",
            "from_u32",
            "sand::resourcepack::Color::from_u32",
        ),
        (
            "command/struct.Actionbar.html",
            "show",
            "sand::command::Actionbar::show",
        ),
        (
            "entity/struct.StatCurve.html",
            "evaluate",
            "sand::entity::StatCurve::evaluate",
        ),
        (
            "component/struct.AdvancementDisplay.html",
            "show_toast",
            "sand::component::AdvancementDisplay::show_toast",
        ),
    ] {
        let page = target.join("doc/sand").join(relative);
        let html = fs::read_to_string(&page)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", page.display()));
        let section = rendered_member_section(&html, member, &page, canonical);
        assert!(
            section.contains("API Contract")
                && section.contains(&format!("sand api show {canonical}")),
            "{} exposes {canonical} without its exact contract lookup",
            page.display(),
        );
    }

    let nbt_path = fs::read_to_string(target.join("doc/sand/data/struct.NbtPath.html"))
        .expect("read rendered NbtPath documentation");
    assert!(
        nbt_path.contains("Borrows the rendered NBT path text without allocating"),
        "NbtPath::as_str must render its member-specific contract prose"
    );

    fs::remove_dir_all(&target).expect("remove isolated rustdoc target");
}

fn rendered_member_section<'a>(
    html: &'a str,
    member: &str,
    page: &Path,
    canonical: &str,
) -> &'a str {
    let method_anchor = format!("id=\"method.{member}\"");
    let trait_anchor = format!("id=\"tymethod.{member}\"");
    let start = [html.find(&method_anchor), html.find(&trait_anchor)]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or_else(|| panic!("{} has no anchor for {canonical}", page.display()));
    let section = &html[start + 1..];
    let end = [section.find("id=\"method."), section.find("id=\"tymethod.")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(section.len());
    &section[..end]
}

fn page_prefix(kind: ApiKind) -> Option<&'static str> {
    Some(match kind {
        ApiKind::Module => "module",
        ApiKind::Struct => "struct",
        ApiKind::Enum => "enum",
        ApiKind::Trait => "trait",
        ApiKind::TypeAlias => "type",
        ApiKind::Function => "fn",
        ApiKind::Constant => "constant",
        ApiKind::Macro => "macro",
        _ => return None,
    })
}

fn rustdoc_page(registration: &sand_api_contract::ApiRegistration) -> String {
    let mut segments = registration
        .canonical_path
        .split("::")
        .skip(1)
        .collect::<Vec<_>>();
    if registration.kind == ApiKind::Module {
        return format!("{}/index.html", segments.join("/"));
    }
    let name = segments.pop().expect("non-module API has a name");
    let filename = format!("{}.{}.html", page_prefix(registration.kind).unwrap(), name);
    if segments.is_empty() {
        filename
    } else {
        format!("{}/{filename}", segments.join("/"))
    }
}
