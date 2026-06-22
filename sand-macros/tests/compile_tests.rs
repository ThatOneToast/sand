#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_basic.rs");
    t.pass("tests/cases/pass_multiple_commands.rs");
    t.pass("tests/cases/pass_plain_stmts.rs");
    t.pass("tests/cases/pass_attribute_typed.rs");
    t.pass("tests/cases/pass_component.rs");
    t.pass("tests/cases/pass_event_generic.rs");
    t.pass("tests/cases/pass_event_generic_used_dash_wand.rs");
    t.compile_fail("tests/cases/fail_with_params.rs");
    t.compile_fail("tests/cases/fail_empty_body.rs");
    t.compile_fail("tests/cases/fail_raw_string.rs");
    t.compile_fail("tests/cases/fail_unsupported_if.rs");
    t.compile_fail("tests/cases/fail_non_command.rs");
    t.compile_fail("tests/cases/fail_component_with_params.rs");
    t.compile_fail("tests/cases/fail_event_generic_missing_type.rs");
    t.compile_fail("tests/cases/fail_event_generic_not_advancement.rs");
    t.compile_fail("tests/cases/fail_event_too_many_params.rs");
    t.pass("tests/cases/pass_run_fn.rs");
}
