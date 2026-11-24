mod support;
use support::*;

#[test]
fn function_values_are_typed_and_flow_through_owned_storage() {
    compile(&analyze_one(
        r#"
#![pack(namespace = "callbacks", minecraft = "26.2")]
struct Entry { transform: fn(number) -> string }
global state Actions { saved: fn(number) -> string, batch: Vec<Entry>, n: number }
fn first(value: number) -> string { return value.to_string(); }
fn second(value: number) -> string { return "s" + value.to_string(); }
fn notify(value: number) { Actions.n = value; }
fn run_with<T, R>(callback: fn(T) -> R, argument: T) -> R { return callback(argument); }
fn callback_for(flag: bool) -> fn(number) -> string {
    return match flag { true => first, false => second };
}
fn run(flag: bool) {
    Actions.saved = callback_for(flag);
    let selected = Actions.saved;
    let label = run_with(selected, 3);
    Actions.batch = [Entry { transform: selected }];
    let also = Actions.batch[0].transform;
    let another = also(4);
    let mut callbacks: Vec<fn(number) -> string> = [first];
    let popped = callbacks.pop_or(second);
    let popped_result = popped(5);
    let fixed: [fn(number) -> string; 1] = [first];
    let indexed = fixed.get_or(0, second);
    let indexed_result = indexed(6);
    let action: fn(number) = notify;
    action(5);
    let empty: fn(number) -> string;
    let default = empty(0);
}
"#,
    ));
}

#[test]
fn function_values_reject_incompatible_signatures_and_executors() {
    for (body, message) in [
        ("let callback: fn(string) -> number = plain;", "expected"),
        (
            "let callback = player_only; let value = callback();",
            "without a player or entity executor",
        ),
        ("let number = 3; number();", "does not hold a function"),
        ("let callback = plain; callback(true);", "expected"),
        (
            "let callback = plain; let bad = callback as number;",
            "cannot be cast",
        ),
    ] {
        let diagnostics = errors(&analyze_one(&format!(
            r#"
#![pack(namespace = "callbacks")]
fn plain(value: number) -> number {{ return value; }}
#[player] fn player_only() -> number {{ return 1; }}
fn run() {{ {body} }}
"#
        )));
        assert!(
            diagnostics.iter().any(|error| error.contains(message)),
            "{body}: {diagnostics:?}"
        );
    }
}

#[test]
fn move_closures_capture_owned_values_and_compose_with_generic_calls() {
    compile(&analyze_one(
        r#"
#![pack(namespace = "callbacks", minecraft = "26.2")]
struct Label { text: string }
global state Actions { save: Vec<fn(number) -> string> }
fn apply<T, R>(callback: fn(T) -> R, argument: T) -> R { return callback(argument); }
fn capture(label: Label) -> fn(number) -> string {
    return move |value: number| -> string { return label.text + value.to_string(); };
}
fn make<T>(value: T) -> fn() -> T {
    return move || -> T { return value; };
}
fn wrapped(value: number) -> fn() -> fn() -> number {
    return move || -> fn() -> number {
        return move || -> number { return value; };
    };
}
fn run() {
    let mut origin = Label { text: "initial" };
    let retained = capture(origin);
    origin.text = "changed";
    let message = apply(retained, 7);
    Actions.save = [retained];
    let stored = Actions.save[0];
    let recovered = stored(3);
    let generic = make("hello");
    let text = generic();
    let outer = wrapped(11);
    let inner = outer();
    let result = inner();
}
"#,
    ));
}

#[test]
fn captured_bindings_are_immutable_and_bad_closure_syntax_is_reported() {
    for (body, message) in [
        ("let mut n = 3; let cb = move || { n += 1; };", "immutable"),
        (
            "let cb = move || -> number { let a = ; return 1; };",
            "expected",
        ),
    ] {
        let source = format!("#![pack(namespace = \"callbacks\")] fn run() {{ {body} }}");
        let diagnostics = errors(&analyze_one(&source));
        assert!(
            diagnostics.iter().any(|error| error.contains(message)),
            "{body}: {diagnostics:?}"
        );
    }
}

#[test]
fn native_callable_fixture_compiles_and_formats() {
    let source = include_str!("fixtures/callables.sand");
    compile(&analyze_one(source));
    let formatted = sand_lang::format::format(source).unwrap();
    assert_eq!(sand_lang::format::format(&formatted).unwrap(), formatted);
    compile(&analyze_one(&formatted));
}

#[test]
fn callable_locals_have_signature_help() {
    let source = r#"#![pack(namespace = "callbacks")]
fn render(callback: fn(number, string) -> string) -> string {
    return callback(3, "label");
}
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    let offset = source.find("3, \"label\"").unwrap() + 3;
    let help = sand_lang::ide::signature_help(&analysis, sand_lang::FileId(0), offset).unwrap();
    assert_eq!(help.label, "callback(arg1: number, arg2: string) -> string");
    assert_eq!(help.active, 1);
}

#[test]
fn executor_scoped_callables_dispatch_as_the_invoking_owner() {
    use sand::advanced::compiler::ExecutionContext;

    let analysis = analyze_one(
        r#"
#![pack(namespace = "callbacks", minecraft = "26.2")]
player state Journal { total: number, pending: player fn(number) }
fn deposit(n: number) { Journal.total += n; }
fn add_tagged() { add_tag("visited"); }
fn pure(n: number) -> number { return n + 1; }
fn with_player<T>(work: player fn(T), value: T) { work(value); }
fn saved(delta: number) -> player fn(number) {
    return move |n: number| { deposit(n + delta); };
}
#[load]
fn install() {
    for player in players() { Journal.pending = saved(2); }
}
#[tick]
fn tick() {
    for player in players() {
        let callback = Journal.pending;
        with_player(callback, 3);
        with_player(deposit, 1);
        let read: player fn(number) -> number = pure;
        deposit(read(4));
    }
    for zombie in entities(type = Entity::Zombie) {
        let visit: entity fn() = add_tagged;
        visit();
    }
}
"#,
    );
    compile(&analysis);
    let functions = analysis
        .program
        .as_ref()
        .unwrap()
        .modules
        .iter()
        .flat_map(|m| &m.functions);
    let dispatchers: Vec<_> = functions
        .filter(|f| f.id.path().starts_with("sand.lang_call/"))
        .map(|f| f.context)
        .collect();
    assert!(dispatchers.contains(&ExecutionContext::Player));
    assert!(dispatchers.contains(&ExecutionContext::Entity));
}

#[test]
fn executor_contracts_validate_targets_and_invocation_separately() {
    for (source, cause) in [
        (
            "fn install() { let callback: entity fn() = player_only; }",
            "an entity callback cannot require a player executor",
        ),
        (
            "#[load] fn run() { let callback: player fn() = pure; callback(); }",
            "without a player",
        ),
        (
            "#[tick] fn run() { let callback: entity fn() = pure; callback(); }",
            "on the server",
        ),
        (
            "fn run() { for zombie in entities(type = Entity::Zombie) { let callback: player fn() = player_only; callback(); } }",
            "loops run as entities that may not be players",
        ),
        (
            "fn install() { let callback: entity fn() = move || { player_only(); }; }",
            "an entity callback cannot require a player executor",
        ),
    ] {
        let analysis = analyze_one(&format!(
            r#"
#![pack(namespace = "callbacks", minecraft = "26.2")]
#[player] fn player_only() {{ chat!("hello"); }}
fn pure() {{}}
{source}
"#
        ));
        let messages = errors(&analysis);
        assert!(
            messages.iter().any(|e| e.contains(cause)),
            "{source}: {messages:?}"
        );
    }
}
