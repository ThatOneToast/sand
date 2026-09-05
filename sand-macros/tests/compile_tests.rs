#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_basic.rs");
    t.pass("tests/cases/pass_multiple_commands.rs");
    t.pass("tests/cases/pass_plain_stmts.rs");
    t.pass("tests/cases/pass_attribute_typed.rs");
    t.pass("tests/cases/pass_public_api_tiers.rs");
    t.pass("tests/cases/pass_canonical_command_foundations.rs");
    t.pass("tests/cases/pass_validated_command_media.rs");
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
    t.compile_fail("tests/cases/fail_sand_event_transport_private.rs");
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
    t.compile_fail("tests/cases/fail_selector_wrong_predicate_id.rs");
    // Typed selector filters preserve Target category/cardinality capabilities.
    t.pass("tests/cases/pass_typed_target_filters.rs");
    t.compile_fail("tests/cases/fail_entity_target_gamemode.rs");
    t.compile_fail("tests/cases/fail_player_target_entity_type.rs");
    t.compile_fail("tests/cases/fail_entity_target_filters_do_not_narrow.rs");
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
fn state_derive_compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_entity_state.rs");
    t.pass("tests/cases/pass_entity_archetype.rs");
    t.pass("tests/cases/pass_state_scopes_bound_views.rs");
    t.pass("tests/cases/pass_state_visibility.rs");
    t.pass("tests/cases/pass_state_components_bundles.rs");
    t.pass("tests/cases/pass_direct_state_queries.rs");
    t.compile_fail("tests/cases/fail_entity_state_tuple.rs");
    t.compile_fail("tests/cases/fail_entity_state_unknown_wrapper.rs");
    t.compile_fail("tests/cases/fail_entity_state_bad_namespace.rs");
    t.compile_fail("tests/cases/fail_entity_state_invalid_bounds.rs");
    t.compile_fail("tests/cases/fail_state_invalid_scope.rs");
    t.compile_fail("tests/cases/fail_state_auto_tick_score.rs");
    t.compile_fail("tests/cases/fail_state_invalid_criterion.rs");
    t.compile_fail("tests/cases/fail_state_private_bound_visibility.rs");
    t.compile_fail("tests/cases/fail_state_bundle_scope_mismatch.rs");
    t.compile_fail("tests/cases/fail_state_bundle_cycle.rs");
    t.compile_fail("tests/cases/fail_state_migration_gap.rs");
    t.compile_fail("tests/cases/fail_state_query_contradiction.rs");
    t.compile_fail("tests/cases/fail_state_query_nested_contradiction.rs");
    t.compile_fail("tests/cases/fail_state_query_scope_mismatch.rs");
    t.compile_fail("tests/cases/fail_state_bundle_target_scope.rs");
    t.compile_fail("tests/cases/fail_global_state_query.rs");
    t.compile_fail("tests/cases/fail_system_parameter_count.rs");
    t.compile_fail("tests/cases/fail_system_parameter_kind.rs");
    t.compile_fail("tests/cases/fail_system_grouped_parameter_kind.rs");
    t.compile_fail("tests/cases/fail_system_query_shadowing.rs");
    t.compile_fail("tests/cases/fail_system_cadence.rs");
    t.compile_fail("tests/cases/fail_system_event_signature.rs");
    t.compile_fail("tests/cases/fail_system_return_type.rs");
    t.pass("tests/cases/pass_state_data_entity_scope.rs");
    t.compile_fail("tests/cases/fail_entity_state_enum_duplicate.rs");
    t.compile_fail("tests/cases/fail_entity_state_enum_payload.rs");
    t.compile_fail("tests/cases/fail_entity_marker_health_capability.rs");
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
    t.compile_fail("tests/cases/fail_run_fn_path_only_no_namespace.rs");
}

#[test]
fn api_contract_compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_api_contracts.rs");
    t.pass("tests/cases/pass_api_members.rs");
    t.compile_fail("tests/cases/fail_api_missing_parameter.rs");
    t.compile_fail("tests/cases/fail_api_nonexistent_parameter.rs");
    t.compile_fail("tests/cases/fail_api_duplicate_field.rs");
    t.compile_fail("tests/cases/fail_api_unknown_field.rs");
    t.compile_fail("tests/cases/fail_api_malformed_list.rs");
    t.compile_fail("tests/cases/fail_api_unsupported_item.rs");
    t.compile_fail("tests/cases/fail_api_duplicate_identity.rs");
    t.compile_fail("tests/cases/fail_api_missing_field.rs");
    t.compile_fail("tests/cases/fail_api_unknown_field_doc.rs");
    t.compile_fail("tests/cases/fail_api_missing_variant.rs");
    t.compile_fail("tests/cases/fail_api_unknown_variant_doc.rs");
    t.compile_fail("tests/cases/fail_api_missing_variant_field.rs");
    t.compile_fail("tests/cases/fail_api_unknown_variant_field.rs");
    t.compile_fail("tests/cases/fail_api_duplicate_variant_field.rs");
    t.compile_fail("tests/cases/fail_api_members_on_function.rs");
    t.compile_fail("tests/cases/fail_api_public_tuple_field.rs");
}
