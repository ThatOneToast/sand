mod support;
use sand_lang::library::Library;
use support::*;

#[test]
fn time_and_spatial_libraries_compile_as_normal_owned_state_values() {
    let fixture = format!(
        "{}\nmod spatial;\nmod time;\n",
        include_str!("fixtures/time_spatial.sand")
    );
    let analysis = analyze(&[
        ("pack.sand", &fixture),
        ("spatial.sand", Library::Spatial.source()),
        ("time.sand", Library::Time.source()),
    ]);
    let pack = compile(&analysis);
    assert!(pack.resources.iter().any(|(path, body)| {
        path.ends_with(".mcfunction")
            && String::from_utf8_lossy(body).contains("tp @s ~$(x) ~$(y) ~$(z)")
    }));
    assert_eq!("time".parse::<Library>().unwrap(), Library::Time);
    assert_eq!("spatial".parse::<Library>().unwrap(), Library::Spatial);
}
