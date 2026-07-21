#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_basic.rs");
    t.pass("tests/cases/pass_multiple_commands.rs");
    t.pass("tests/cases/pass_plain_stmts.rs");
    t.pass("tests/cases/pass_attribute_typed.rs");
    t.pass("tests/cases/pass_public_api_tiers.rs");
    t.pass("tests/cases/pass_canonical_command_foundations.rs");
    t.pass("tests/cases/pass_component.rs");
    t.pass("tests/cases/pass_component_dialog.rs");
    t.pass("tests/cases/pass_event_generic.rs");
    t.pass("tests/cases/pass_event_level_up.rs");
    t.pass("tests/cases/pass_damage_event.rs");
    t.pass("tests/cases/pass_event_generic_used_dash_wand.rs");
    t.pass("tests/cases/pass_canonical_event_docs.rs");
    t.pass("tests/cases/pass_sand_event_tick_dispatch.rs");
    t.pass("tests/cases/pass_sand_event_generic_family.rs");
    t.pass("tests/cases/pass_sand_event_chain_dispatch.rs");
    t.pass("tests/cases/pass_sand_event_while_dispatch.rs");
    t.pass("tests/cases/pass_sand_event_multi_parent_dispatch.rs");
    t.pass("tests/cases/pass_sand_event_within_dispatch.rs");
    t.compile_fail("tests/cases/fail_sand_event_chain_non_sand_event_parent.rs");
    t.compile_fail("tests/cases/fail_sand_event_while_non_persistent.rs");
    t.compile_fail("tests/cases/fail_sand_event_after_any_empty.rs");
    t.compile_fail("tests/cases/fail_sand_event_after_all_non_event.rs");
    t.compile_fail("tests/cases/fail_sand_event_after_any_arity_nine.rs");
    t.compile_fail("tests/cases/fail_sand_event_within_non_sand_event_parent.rs");
    t.compile_fail("tests/cases/fail_with_params.rs");
    t.compile_fail("tests/cases/fail_empty_body.rs");
    t.compile_fail("tests/cases/fail_raw_string.rs");
    t.compile_fail("tests/cases/fail_unsupported_if.rs");
    t.compile_fail("tests/cases/fail_non_command.rs");
    t.compile_fail("tests/cases/fail_component_with_params.rs");
    t.compile_fail("tests/cases/fail_event_generic_missing_type.rs");
    t.compile_fail("tests/cases/fail_event_generic_not_advancement.rs");
    t.compile_fail("tests/cases/fail_damage_event_non_damage.rs");
    t.compile_fail("tests/cases/fail_damage_direct_many_targets.rs");
    t.compile_fail("tests/cases/fail_selector_implicit_narrowing.rs");
    t.compile_fail("tests/cases/fail_scoreboard_string_operation.rs");
    t.compile_fail("tests/cases/fail_event_too_many_params.rs");
    t.compile_fail("tests/cases/fail_advancement_event_marker_field_not_runtime.rs");
    t.pass("tests/cases/pass_run_fn.rs");
    t.pass("tests/cases/pass_sand_storage.rs");
    t.pass("tests/cases/pass_sand_storage_custom_path.rs");
    t.compile_fail("tests/cases/fail_sand_storage_tuple_struct.rs");
    t.compile_fail("tests/cases/fail_sand_storage_missing_attr.rs");
}

#[test]
fn readme_quickstart_compile_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_readme_quickstart.rs");
}

#[test]
fn public_api_tier_compile_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_public_api_tiers.rs");
}

#[test]
fn vanilla_public_api_compile_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_vanilla_public_api.rs");
}

#[test]
fn recipe_fixture_compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_recipe_basic_state.rs");
    t.pass("tests/cases/pass_recipe_custom_item.rs");
    t.pass("tests/cases/pass_recipe_advancement_event.rs");
    t.compile_fail("tests/cases/fail_recipe_block_tag.rs");
    t.compile_fail("tests/cases/fail_recipe_raw_string_typed_path.rs");
}

#[test]
fn resource_path_validation_compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_function_path.rs");
    t.pass("tests/cases/pass_function_namespaced.rs");
    t.pass("tests/cases/pass_component_tag_valid.rs");
    t.pass("tests/cases/pass_run_fn_valid.rs");
    t.compile_fail("tests/cases/fail_function_empty.rs");
    t.compile_fail("tests/cases/fail_function_uppercase.rs");
    t.compile_fail("tests/cases/fail_function_spaces.rs");
    t.compile_fail("tests/cases/fail_function_bad_namespace.rs");
    t.compile_fail("tests/cases/fail_function_missing_path.rs");
    t.compile_fail("tests/cases/fail_function_multi_colon.rs");
    t.compile_fail("tests/cases/fail_component_tag_invalid.rs");
    t.compile_fail("tests/cases/fail_run_fn_empty.rs");
}
