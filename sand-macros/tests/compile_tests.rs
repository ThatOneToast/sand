#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_basic.rs");
    t.pass("tests/cases/pass_multiple_commands.rs");
    t.pass("tests/cases/pass_component.rs");
    t.compile_fail("tests/cases/fail_with_params.rs");
    t.compile_fail("tests/cases/fail_empty_body.rs");
    t.pass("tests/cases/pass_str_command.rs");
    t.compile_fail("tests/cases/fail_component_with_params.rs");
    t.pass("tests/cases/pass_run_fn.rs");
}
