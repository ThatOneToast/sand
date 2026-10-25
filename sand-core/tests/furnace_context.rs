//! Furnace capability exports retain the canonical numeric/branch contracts.
use sand::prelude::*;
use std::cell::Cell;

#[derive(Clone, Copy)]
enum Case {
    Used,
    Empty,
    Discarded,
    Slots,
    Escaped,
    InvalidPosition,
    Macro,
    Return,
}
thread_local! { static CASE: Cell<Case> = const { Cell::new(Case::Used) }; }

fn build() -> Actions {
    let block = WorldContext::current().block(BlockPos::here());
    CASE.with(|case| match case.get() {
        Case::Empty => block.when_furnace(|_| Actions::default()),
        Case::Discarded => {
            let _ = block.when_furnace(|f| when(f.burn_remaining().gt(0))
                .then_all([f.input().clear().unwrap()]));
            Actions::default()
        }
        Case::Slots => block.when_furnace(|f| f.output().clear().unwrap()),
        Case::Escaped => {
            let mut read = None;
            let _ = block.when_furnace(|f| { read = Some(f.cooking_elapsed()); Actions::default() });
            when(read.unwrap().gt(0)).then_all([block.destroy()])
        }
        Case::InvalidPosition => WorldContext::current().block(BlockPos::new(f64::NAN, 0, 0))
            .when_furnace(|f| f.output().clear().unwrap()),
        Case::Macro => block.when_furnace(|_| FunctionMacroArgs::new(["value"]).unwrap().line("say $(value)").unwrap()),
        Case::Return => block.when_furnace(|_| mcfunction![block.replace(vanilla::Block::Smoker); cmd::return_fail();]),
        Case::Used => {
            static BODY: std::sync::OnceLock<Actions> = std::sync::OnceLock::new();
            BODY.get_or_init(|| block.when_furnace(|f| mcfunction![
                when(f.cooking_elapsed().gte(0).and(f.cooking_total().gt(0)).and(f.burn_remaining().gt(0)))
                    .then_all([f.output().clear().unwrap()]);
                f.input().set(ItemStack::new(vanilla::Item::RawIron.into())).unwrap();
                f.fuel().set(ItemStack::new(vanilla::Item::Coal.into())).unwrap();
            ])).clone()
        }
    })
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "inspect", make: build,
    }
}

fn export(case: Case, namespace: &str, version: &str) -> Result<String, String> {
    CASE.with(|value| value.set(case));
    sand::advanced::try_export_pack_json(namespace, version).map_err(|e| e.to_string())
}

#[test]
fn furnace_profiles_and_escaped_reads_are_checked_on_every_export() {
    for version in ["26.2", "26.2.0", "latest"] {
        for case in [Case::Used, Case::Slots, Case::Escaped] {
            export(case, "forge", version).unwrap();
        }
    }
    for version in ["26.1", "26.1.1", "26.1.2", "26.2.1", "27.0"] {
        for case in [Case::Used, Case::Slots, Case::Escaped] {
            let error = export(case, "forge", version).unwrap_err();
            assert!(
                error.contains("requires verified Minecraft 26.2"),
                "{error}"
            );
            assert!(error.contains("forge:"), "{error}");
        }
    }
    CASE.with(|value| value.set(Case::Used));
    sand_core::try_export_components_json("forge").unwrap();
    let error = sand_core::try_export_components_for_version(
        "forge",
        &sand_version::VersionCaps::all_enabled(),
        "26.2",
        true,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("fallback=true"), "{error}");
}

#[test]
fn unused_furnaces_emit_no_helpers_or_profile_requirement() {
    let empty = export(Case::Empty, "unused_forge", "26.1").unwrap();
    let discarded = export(Case::Discarded, "unused_forge", "26.1").unwrap();
    assert_eq!(empty, discarded);
    for needle in [
        "sand/conditions",
        "sand/branches",
        "sand/numeric",
        "cooking_time",
        "lit_time",
    ] {
        assert!(!discarded.contains(needle), "{needle}");
    }
}

#[test]
fn cached_furnace_output_is_deterministic_and_namespace_isolated() {
    let first = export(Case::Used, "forge_a", "26.2").unwrap();
    let other = export(Case::Used, "forge_b", "26.2").unwrap();
    assert!(!other.contains("forge_a:"));
    assert_eq!(first, export(Case::Used, "forge_a", "26.2").unwrap());
    for needle in [
        "cooking_time_spent",
        "cooking_total_time",
        "lit_time_remaining",
        "container.0",
        "container.1",
        "container.2",
        "run return run function",
    ] {
        assert!(first.contains(needle), "{needle}");
    }
    assert!(!first.contains("forceload"));
    assert!(!first.contains("__sand_local:"));
}

#[test]
fn furnace_frames_preserve_return_and_reject_macro_or_position_errors() {
    let output = export(Case::Return, "forge", "26.2").unwrap();
    assert!(output.contains("setblock ~ ~ ~ minecraft:smoker"));
    assert_eq!(
        output
            .matches("run return run function forge:sand/branches/")
            .count(),
        4
    );
    let error = export(Case::Macro, "forge", "26.2").unwrap_err();
    assert!(
        error.contains("macro lines cannot move into a helper"),
        "{error}"
    );
    let error = export(Case::InvalidPosition, "forge", "26.2").unwrap_err();
    assert!(
        error.contains("forge:") && error.contains("position"),
        "{error}"
    );
}
