//! Executes the emitted arithmetic subset with Minecraft i32 score semantics.
use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct Machine(
    BTreeMap<String, i32>,
    BTreeMap<String, Vec<String>>,
    Option<(bool, i32)>,
    BTreeMap<String, f64>,
    BTreeMap<String, Vec<f64>>,
    BTreeMap<String, String>,
    BTreeMap<String, BTreeMap<String, i32>>,
    String,
    std::collections::BTreeSet<String>,
);
impl Machine {
    fn get(&self, holder: &str, objective: &str) -> i32 {
        self.0[&format!("{holder} {objective}")]
    }
    fn native_value(&self, key: &str) -> Option<f64> {
        self.3.get(key).copied().or_else(|| {
            key.strip_suffix("[0]")
                .and_then(|list| self.4.get(list)?.first().copied())
        })
    }

    fn run(&mut self, command: &[&str]) -> bool {
        let resolved = (!self.7.is_empty()).then(|| {
            command
                .iter()
                .map(|word| {
                    if *word == "@s" {
                        self.7.clone()
                    } else {
                        (*word).to_owned()
                    }
                })
                .collect::<Vec<_>>()
        });
        let references = resolved
            .as_ref()
            .map(|words| words.iter().map(String::as_str).collect::<Vec<_>>());
        let command = references.as_deref().unwrap_or(command);
        match command[0] {
            "return" => {
                let success = match command[1] {
                    "fail" => (false, 0),
                    "run" => {
                        assert_eq!(command[2], "function");
                        self.call_result(command[3])
                    }
                    number => (true, number.parse().unwrap()),
                };
                self.2 = Some(success);
                false
            }
            "function" => {
                self.call(command[1]);
                true
            }
            "execute" => {
                if command[1] == "as" {
                    assert_eq!(command[3], "run");
                    if self.6.contains_key(command[2]) || self.8.contains(command[2]) {
                        let previous = std::mem::replace(&mut self.7, command[2].to_owned());
                        self.run(&command[4..]);
                        self.7 = previous;
                    }
                    return true;
                }
                if command[1] == "store" {
                    let mut stores = Vec::new();
                    let mut i = 1;
                    while command[i] == "store" {
                        assert_eq!(command[i + 2], "score");
                        stores.push((
                            command[i + 1],
                            format!("{} {}", command[i + 3], command[i + 4]),
                        ));
                        i += 5;
                    }
                    assert_eq!(command[i], "run");
                    let inner = &command[i + 1..];
                    let (success, result) = if inner[0] == "function" {
                        self.call_result(inner[1])
                    } else if inner[0] == "clear" {
                        assert_eq!(inner[3], "0", "numeric reads must never remove items");
                        let count = self
                            .6
                            .get(inner[1])
                            .and_then(|items| items.get(inner[2]))
                            .copied()
                            .unwrap_or(0);
                        (count != 0, count)
                    } else if inner[0] == "data" {
                        assert_eq!(inner[1], "get");
                        let key = format!("{} {} {}", inner[2], inner[3], inner[4]);
                        if inner.len() == 5 {
                            let list = self.4.get(&key);
                            (list.is_some(), list.map_or(0, |values| values.len() as i32))
                        } else {
                            let value = self.native_value(&key);
                            (
                                value.is_some(),
                                value.map_or(0, |value| {
                                    (value * inner[5].parse::<f64>().unwrap()).floor() as i32
                                }),
                            )
                        }
                    } else if inner.starts_with(&["scoreboard", "players", "get"]) {
                        let value = self.0.get(&format!("{} {}", inner[3], inner[4])).copied();
                        (value.is_some(), value.unwrap_or(0))
                    } else {
                        assert_eq!(&inner[..3], &["scoreboard", "players", "operation"]);
                        // Vanilla scoreboard operations create missing source scores as zero.
                        self.0
                            .entry(format!("{} {}", inner[6], inner[7]))
                            .or_insert(0);
                        let success = self.run(inner);
                        (
                            success,
                            if success {
                                self.get(inner[3], inner[4])
                            } else {
                                0
                            },
                        )
                    };
                    for (kind, key) in stores {
                        self.0.insert(
                            key,
                            if kind == "success" {
                                i32::from(success)
                            } else {
                                result
                            },
                        );
                    }
                    return true;
                }
                let mut i = 1;
                while command[i] != "run" {
                    let negate = command[i] == "unless";
                    assert!(matches!(command[i], "if" | "unless"));
                    if command[i + 1] == "entity" {
                        assert_eq!(command[i + 2], "@s[type=minecraft:player]");
                        if self.6.contains_key(&self.7) == negate {
                            return true;
                        }
                        i += 3;
                        continue;
                    }
                    if command[i + 1] == "data" {
                        assert_eq!(command[i + 2], "storage");
                        let (path, predicate) = command[i + 4].split_once('{').unwrap();
                        let expected = predicate
                            .strip_prefix("text:\"")
                            .unwrap()
                            .strip_suffix("\"}")
                            .unwrap();
                        let key = format!("storage {} {path}.text", command[i + 3]);
                        let matched = self.5.get(&key).is_some_and(|text| text == expected);
                        if matched == negate {
                            return true;
                        }
                        i += 5;
                        continue;
                    }
                    if command[i + 1] == "function" {
                        let (success, result) = self.call_result(command[i + 2]);
                        i += 3;
                        if (success && result != 0) == negate {
                            return true;
                        }
                        continue;
                    }
                    assert_eq!(command[i + 1], "score");
                    let left = self
                        .0
                        .get(&format!("{} {}", command[i + 2], command[i + 3]))
                        .copied();
                    let present = left.is_some();
                    let left = left.unwrap_or(0);
                    let condition = if command[i + 4] == "matches" {
                        let range = command[i + 5];
                        i += 6;
                        if let Some((min, max)) = range.split_once("..") {
                            (min.is_empty() || left >= min.parse::<i32>().unwrap())
                                && (max.is_empty() || left <= max.parse::<i32>().unwrap())
                        } else {
                            left == range.parse::<i32>().unwrap()
                        }
                    } else {
                        let right = self.get(command[i + 5], command[i + 6]);
                        let matched = match command[i + 4] {
                            "=" => left == right,
                            ">" => left > right,
                            ">=" => left >= right,
                            "<" => left < right,
                            "<=" => left <= right,
                            other => panic!("unsupported comparison {other}"),
                        };
                        i += 7;
                        matched
                    };
                    if (condition && present) == negate {
                        return true;
                    }
                }
                self.run(&command[i + 1..])
            }
            "data" => {
                assert_eq!(&command[1..3], &["modify", "storage"]);
                let key = format!("storage {} {}", command[3], command[4]);
                match &command[5..7] {
                    ["set", "value"] => {
                        assert_eq!(command[7], "{values:[]}");
                        self.4.insert(format!("{key}.values"), Vec::new());
                        self.5.remove(&format!("{key}.text"));
                    }
                    ["append", "from"] => {
                        let source = format!("{} {} {}", command[7], command[8], command[9]);
                        let values = if let Some(value) = self.native_value(&source) {
                            vec![value]
                        } else {
                            self.4.get(&source).cloned().unwrap_or_default()
                        };
                        self.4.get_mut(&key).unwrap().extend(values);
                    }
                    ["set", "string"] => {
                        let source = format!("{} {} {}", command[7], command[8], command[9]);
                        if let Some(value) = self.native_value(&source) {
                            self.5.insert(
                                key,
                                if value.is_nan() {
                                    "NaNd".into()
                                } else {
                                    format!("{value}d")
                                },
                            );
                        }
                    }
                    other => panic!("unsupported data mutation {other:?}"),
                }
                true
            }
            "scoreboard" => {
                assert_eq!(command[1], "players");
                let key = format!("{} {}", command[3], command[4]);
                let value = match command[2] {
                    "set" => command[5].parse().unwrap(),
                    "add" => self.0[&key].wrapping_add(command[5].parse().unwrap()),
                    "operation" => {
                        let right = self.get(command[6], command[7]);
                        if command[5] == "=" {
                            right
                        } else {
                            let left = self.0[&key];
                            match command[5] {
                                "+=" => left.wrapping_add(right),
                                "-=" => left.wrapping_sub(right),
                                "*=" => left.wrapping_mul(right),
                                "/=" | "%=" => {
                                    assert_ne!(right, 0);
                                    let (left, right) = (i64::from(left), i64::from(right));
                                    let mut quotient = left / right;
                                    if left % right != 0 && (left < 0) != (right < 0) {
                                        quotient -= 1;
                                    }
                                    if command[5] == "/=" {
                                        quotient as i32
                                    } else {
                                        (left - quotient * right) as i32
                                    }
                                }
                                other => panic!("unsupported operation {other}"),
                            }
                        }
                    }
                    other => panic!("unsupported scoreboard command {other}"),
                };
                self.0.insert(key, value);
                true
            }
            other => panic!("unsupported command {other}"),
        }
    }
    fn call(&mut self, function: &str) -> bool {
        self.call_result(function).0
    }
    fn call_result(&mut self, function: &str) -> (bool, i32) {
        let commands = self
            .1
            .get(function)
            .expect("generated function exists")
            .clone();
        let parent_return = self.2.take();
        let completed = self.execute(&commands);
        let success = self.2.take().unwrap_or((completed, i32::from(completed)));
        self.2 = parent_return;
        success
    }
    fn execute(&mut self, commands: &[String]) -> bool {
        commands
            .iter()
            .all(|line| self.run(&line.split_whitespace().collect::<Vec<_>>()))
    }
}

#[test]
fn emitted_checked_arithmetic_matches_wide_integer_oracle() {
    let owner = "test:arithmetic".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let context = NumericContext::new(&owner, &holder).unwrap();
    let mut values = vec![
        i32::MIN,
        i32::MIN + 1,
        -1_073_741_824,
        -46341,
        -46340,
        -1000,
        -3,
        -2,
        -1,
        0,
        1,
        2,
        3,
        1000,
        46340,
        46341,
        1_073_741_823,
        i32::MAX - 1,
        i32::MAX,
    ];
    let mut seed = 712367821_u32;
    for _ in 0..64 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        values.push(seed as i32);
    }
    for multiply in [false, true] {
        let mut commands = Vec::new();
        append_checked_arithmetic(
            context,
            &mut BTreeSet::new(),
            &mut commands,
            "left",
            "@s right",
            multiply,
            0,
        );
        for &left in &values {
            for &right in &values {
                let wide = if multiply {
                    i64::from(left) * i64::from(right)
                } else {
                    i64::from(left) + i64::from(right)
                };
                let expected = i32::try_from(wide).ok();
                let mut machine = Machine::default();
                machine.0.insert("@s left".into(), left);
                machine.0.insert("@s right".into(), right);
                let success = machine.execute(&commands);
                assert_eq!(success, expected.is_some(), "{left} {multiply:?} {right}");
                assert_eq!(machine.get("@s", "left"), expected.unwrap_or(left));
            }
        }
    }
}

#[test]
fn emitted_rounding_handles_minimum_score_without_intermediate_overflow() {
    let owner = "test:division".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let context = NumericContext::new(&owner, &holder).unwrap();
    for rounding in [
        RoundingPolicy::Floor,
        RoundingPolicy::TowardZero,
        RoundingPolicy::Ceiling,
        RoundingPolicy::NearestTiesAwayFromZero,
        RoundingPolicy::NearestTiesToEven,
    ] {
        let mut commands = Vec::new();
        append_score_division(
            context,
            &mut BTreeSet::new(),
            &mut commands,
            "value",
            "divisor",
            rounding,
            0,
        )
        .unwrap();
        for left in [i32::MIN, i32::MIN + 1, -15, -5, -1, 0, 1, 5, 15, i32::MAX] {
            for right in [
                i32::MIN,
                -i32::MAX,
                -1000,
                -10,
                -3,
                -2,
                -1,
                1,
                2,
                3,
                10,
                1000,
                i32::MAX,
            ] {
                let ratio = f64::from(left) / f64::from(right);
                let expected = match rounding {
                    RoundingPolicy::Floor => ratio.floor(),
                    RoundingPolicy::TowardZero => ratio.trunc(),
                    RoundingPolicy::Ceiling => ratio.ceil(),
                    RoundingPolicy::NearestTiesAwayFromZero => ratio.round(),
                    RoundingPolicy::NearestTiesToEven => ratio.round_ties_even(),
                } as i32;
                let mut machine = Machine::default();
                machine.0.insert("@s value".into(), left);
                machine.0.insert("@s divisor".into(), right);
                if left == i32::MIN && right == -1 {
                    assert!(!machine.execute(&commands));
                    assert_eq!(machine.get("@s", "value"), left);
                    continue;
                }
                assert!(machine.execute(&commands));
                assert_eq!(
                    machine.get("@s", "value"),
                    expected,
                    "{left} / {right}, {rounding:?}"
                );
            }
        }
    }
}

#[test]
fn missing_numeric_source_aborts_without_committing_or_reusing_scratch() {
    use crate::entity::{EntityScore, EntityStateField, FixedPoint, StatCurve};
    let owner = "test:missing".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let input = EntityScore::<i32>::__new(
        "test",
        "combat",
        "source",
        crate::entity::state::StateFieldKind::Score,
        0,
        None,
    );
    let lowered = StatCurve::from(input.bind())
        .lower_scoreboard("result", "test:missing", FixedPoint::default())
        .unwrap();
    let output = render_lowered_curve(
        NumericContext::new(&owner, &holder).unwrap(),
        "missing",
        &lowered,
    )
    .unwrap();
    let mut machine = Machine::default();
    machine.0.insert("@s result".into(), 123);
    assert!(!machine.execute(&output.commands));
    assert_eq!(machine.get("@s", "result"), 123);
    assert!(!machine.0.contains_key(&format!("@s {}", input.objective())));
    machine.0.insert(format!("@s {}", input.objective()), 2);
    assert!(machine.execute(&output.commands));
    assert_eq!(machine.get("@s", "result"), 2000);
    machine.0.remove(&format!("@s {}", input.objective()));
    assert!(!machine.execute(&output.commands));
    assert_eq!(machine.get("@s", "result"), 2000);
}

#[test]
fn unbound_inputs_keep_the_caller_when_scratch_uses_a_fake_holder() {
    use crate::entity::{FixedPoint, StatCurve};
    let owner = "test:caller".parse().unwrap();
    let scratch = sand_commands::ScoreHolder::fake("#value");
    let caller = sand_commands::ScoreHolder::self_();
    let context = NumericContext::new(&owner, &scratch)
        .unwrap()
        .with_input_holder(&caller)
        .unwrap();
    for curve in [
        StatCurve::input_raw("mana"),
        StatCurve::stepped(StatCurve::input_raw("mana"), vec![(1.0, 2.0)], 0.0),
        StatCurve::lookup_raw("mana", vec![(1, 2.0)], 0.0),
        StatCurve::enum_mapping_raw("mana", vec![(1, 2.0)], 0.0),
        StatCurve::flag_mapping_raw("mana", 0.0, 2.0),
    ] {
        let lowered = curve
            .lower_scoreboard("result", "test:caller", FixedPoint::default())
            .unwrap();
        let output = render_lowered_curve(context, "caller", &lowered).unwrap();
        assert!(
            output
                .commands
                .iter()
                .any(|line| line.contains("store result score #value ")
                    && line.ends_with("run scoreboard players get @s mana")),
            "{curve:?}"
        );
        assert!(
            output
                .commands
                .iter()
                .any(|line| line.starts_with("execute unless score #value ")
                    && line.ends_with("matches 1 run return fail"))
        );
        assert!(
            !output
                .commands
                .iter()
                .any(|line| line.ends_with("= #value mana"))
        );
    }
}

#[test]
fn random_source_is_resolved_once_for_read_and_success() {
    use crate::entity::{EntityStateField, FixedPoint, FixedScore, StatCurve};
    let owner = "test:random_read".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let field = FixedScore::__new("test", "combat", "source", 100, 0, None);
    let lowered = StatCurve::from(field.bind_to("@r", false))
        .lower_scoreboard("result", "test:random_read", FixedPoint::default())
        .unwrap();
    let output = render_lowered_curve(
        NumericContext::new(&owner, &holder).unwrap(),
        "random_read",
        &lowered,
    )
    .unwrap();
    let text = output.commands.join("\n");
    assert_eq!(text.matches("@r").count(), 1);
    assert!(
        output
            .commands
            .iter()
            .any(|line| line.starts_with("execute store success score ")
                && line.ends_with(&format!(
                    "run scoreboard players get @r {}",
                    field.objective()
                )))
    );
}

#[test]
fn piecewise_evaluates_only_the_selected_arm_and_propagates_failure() {
    use crate::entity::{FixedPoint, StatCurve};
    let owner = "test:piecewise".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let context = NumericContext::new(&owner, &holder).unwrap();
    let missing = StatCurve::input_raw("missing");
    let overflow = StatCurve::multiply([StatCurve::input_raw("large"), StatCurve::constant(2.0)]);
    for bad in [missing, overflow] {
        for bad_is_fallback in [false, true] {
            let good = StatCurve::piecewise(
                StatCurve::input_raw("selector"),
                vec![(0.0, StatCurve::constant(7.0))],
                StatCurve::constant(9.0),
            );
            let (arm, fallback) = if bad_is_fallback {
                (good, bad.clone())
            } else {
                (bad.clone(), good)
            };
            let curve =
                StatCurve::piecewise(StatCurve::input_raw("selector"), vec![(0.0, arm)], fallback);
            let lowered = curve
                .lower_scoreboard("result", "test:piecewise", FixedPoint::default())
                .unwrap();
            let rendered = render_lowered_curve(context, "piecewise", &lowered).unwrap();
            let mut machine = Machine::default();
            for record in rendered.records {
                assert_eq!(record.dir, "function");
                machine.1.insert(
                    format!("{}:{}", record.namespace, record.path),
                    record.content.lines().map(str::to_owned).collect(),
                );
            }
            machine.0.insert("@s result".into(), 123);
            machine.0.insert("@s large".into(), i32::MAX);
            machine
                .0
                .insert("@s selector".into(), if bad_is_fallback { 0 } else { 1 });
            assert!(
                machine.execute(&rendered.commands),
                "unselected bad arm must not execute"
            );
            let expected = if bad_is_fallback { 7000 } else { 9000 };
            assert_eq!(machine.get("@s", "result"), expected);
            machine
                .0
                .insert("@s selector".into(), if bad_is_fallback { 1 } else { 0 });
            assert!(
                !machine.execute(&rendered.commands),
                "selected bad arm must abort"
            );
            assert_eq!(
                machine.get("@s", "result"),
                expected,
                "failed assignment must preserve destination"
            );
        }
    }
}

#[test]
fn native_numeric_read_quantizes_and_preserves_destination_on_failed_read() {
    use crate::entity::{EntityContext, FixedPoint, PlayerKind, StatCurve};
    let owner = "test:native".parse().unwrap();
    let holder = sand_commands::ScoreHolder::fake("#scratch");
    let curve = StatCurve::from(EntityContext::<PlayerKind>::default().living().health());
    let lowered = curve
        .lower_scoreboard("result", "test:native", FixedPoint::default())
        .unwrap();
    let rendered = render_lowered_curve(
        NumericContext::new(&owner, &holder).unwrap(),
        "native",
        &lowered,
    )
    .unwrap();
    let mut machine = Machine::default();
    machine.0.insert("#scratch result".into(), 123);
    for (value, expected) in [
        (2.755, 2755),
        (-2.755, -2755),
        (1.2345, 1234),
        (-1.2345, -1235),
        (0.0, 0),
        (-0.0, 0),
        (f64::MIN_POSITIVE, 0),
        (f64::from_bits(1), 0),
    ] {
        machine.3.insert("entity @s Health".into(), value);
        assert!(machine.execute(&rendered.commands));
        assert_eq!(machine.get("#scratch", "result"), expected);
    }
    machine.0.insert("#scratch result".into(), 123);
    for value in [
        None,
        Some(1.0e20),
        Some(-1.0e20),
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        if let Some(value) = value {
            machine.3.insert("entity @s Health".into(), value);
        } else {
            machine.3.remove("entity @s Health");
        }
        assert!(!machine.execute(&rendered.commands));
        assert_eq!(machine.get("#scratch", "result"), 123);
    }
    machine.3.remove("entity @s Health");
    for values in [vec![], vec![1.0, 2.0]] {
        machine.4.insert("entity @s Health".into(), values);
        assert!(!machine.execute(&rendered.commands));
        assert_eq!(machine.get("#scratch", "result"), 123);
    }
    let text = rendered.commands.join("\n");
    assert!(text.contains("NaNf") && text.contains("NaNd"));
    assert_eq!(text.matches("from entity @s Health").count(), 1);
    let error = curve
        .validate_entity_inputs("test:mob", "health")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("change observation is not supported")
    );
}

#[test]
fn runtime_numeric_comparisons_match_full_range_integer_oracle() {
    use crate::condition::ScoreCompareOp;
    use crate::ir::{Actions, Cmd, NumericComparison};
    let owner = "test:comparison".parse().unwrap();
    for op in [
        ScoreCompareOp::Eq,
        ScoreCompareOp::Gt,
        ScoreCompareOp::Gte,
        ScoreCompareOp::Lt,
        ScoreCompareOp::Lte,
    ] {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
            left: crate::entity::StatCurve::input_raw("left"),
            op,
            right: crate::entity::StatCurve::input_raw("right"),
        }))])
        .lower(&owner)
        .unwrap();
        let mut machine = Machine::default();
        machine.1.insert("test:comparison".into(), body);
        for left in [i32::MIN, -1, 0, 1, i32::MAX] {
            for right in [i32::MIN, -1, 0, 1, i32::MAX] {
                machine.0.insert("@s left".into(), left);
                machine.0.insert("@s right".into(), right);
                let expected = match op {
                    ScoreCompareOp::Eq => left == right,
                    ScoreCompareOp::Gt => left > right,
                    ScoreCompareOp::Gte => left >= right,
                    ScoreCompareOp::Lt => left < right,
                    ScoreCompareOp::Lte => left <= right,
                };
                assert_eq!(
                    machine.call_result("test:comparison"),
                    (true, i32::from(expected)),
                    "{left} {op:?} {right}"
                );
            }
        }
        machine.0.remove("@s right");
        assert_eq!(machine.call_result("test:comparison"), (false, 0));
        for keyword in ["if", "unless"] {
            machine.0.insert("@s selected".into(), 0);
            machine.run(&[
                "execute",
                keyword,
                "function",
                "test:comparison",
                "run",
                "scoreboard",
                "players",
                "set",
                "@s",
                "selected",
                "1",
            ]);
            assert_eq!(
                machine.get("@s", "selected"),
                i32::from(keyword == "unless")
            );
        }
    }
}

#[test]
fn numeric_conditions_compare_integer_constants_without_rescaling_full_range_scores() {
    use crate::condition::ScoreCompareOp;
    use crate::entity::StatCurve;
    use crate::ir::{Actions, Cmd, NumericComparison};
    for constant in [i32::MIN, -1, 0, 1, i32::MAX] {
        for constant_on_left in [false, true] {
            for op in [
                ScoreCompareOp::Eq,
                ScoreCompareOp::Gt,
                ScoreCompareOp::Gte,
                ScoreCompareOp::Lt,
                ScoreCompareOp::Lte,
            ] {
                let _scope = crate::function::ExportFunctionRegistryScope::enter();
                let source = StatCurve::input_raw("source");
                let literal = StatCurve::constant(f64::from(constant));
                let (left, right) = if constant_on_left {
                    (literal, source)
                } else {
                    (source, literal)
                };
                let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
                    left,
                    op,
                    right,
                }))])
                .lower(&"test:constant_comparison".parse().unwrap())
                .unwrap();
                let mut machine = Machine::default();
                machine.1.insert("test:constant_comparison".into(), body);
                for source in [i32::MIN, -1, 0, 1, i32::MAX] {
                    machine.0.insert("@s source".into(), source);
                    let (left, right) = if constant_on_left {
                        (constant, source)
                    } else {
                        (source, constant)
                    };
                    let expected = match op {
                        ScoreCompareOp::Eq => left == right,
                        ScoreCompareOp::Gt => left > right,
                        ScoreCompareOp::Gte => left >= right,
                        ScoreCompareOp::Lt => left < right,
                        ScoreCompareOp::Lte => left <= right,
                    };
                    assert_eq!(
                        machine.call_result("test:constant_comparison"),
                        (true, i32::from(expected)),
                        "{left} {op:?} {right}"
                    );
                }
            }
        }
    }
}

#[test]
fn numeric_conditions_compare_full_range_integer_constants() {
    use crate::condition::ScoreCompareOp;
    use crate::entity::StatCurve;
    use crate::ir::{Actions, Cmd, NumericComparison};
    for (left, right) in [(i32::MIN, i32::MAX), (i32::MAX, i32::MIN)] {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
            left: StatCurve::constant(f64::from(left)),
            op: ScoreCompareOp::Lt,
            right: StatCurve::constant(f64::from(right)),
        }))])
        .lower(&"test:literal_comparison".parse().unwrap())
        .unwrap();
        let mut machine = Machine::default();
        machine.1.insert("test:literal_comparison".into(), body);
        assert_eq!(
            machine.call_result("test:literal_comparison"),
            (true, i32::from(left < right)),
        );
    }
}

#[test]
fn numeric_conditions_keep_fractional_constant_thresholds() {
    use crate::condition::ScoreCompareOp;
    use crate::entity::StatCurve;
    use crate::ir::{Actions, Cmd, NumericComparison};
    for threshold in [-0.5, 0.5] {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
            left: StatCurve::input_raw("source"),
            op: ScoreCompareOp::Gt,
            right: StatCurve::constant(threshold),
        }))])
        .lower(&"test:fractional_threshold".parse().unwrap())
        .unwrap();
        let mut machine = Machine::default();
        machine.1.insert("test:fractional_threshold".into(), body);
        for value in [-1, 0, 1] {
            machine.0.insert("@s source".into(), value);
            assert_eq!(
                machine.call_result("test:fractional_threshold"),
                (true, i32::from(f64::from(value) > threshold)),
            );
        }
    }
}

#[test]
fn numeric_conditions_use_native_precision_and_fail_without_stale_results() {
    use crate::condition::ScoreCompareOp;
    use crate::entity::{EntityContext, PlayerKind, StatCurve};
    use crate::ir::{Actions, Cmd, NumericComparison};
    let _scope = crate::function::ExportFunctionRegistryScope::enter();
    let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
        left: StatCurve::from(EntityContext::<PlayerKind>::default().living().health()),
        op: ScoreCompareOp::Gte,
        right: StatCurve::constant(2.755),
    }))])
    .lower(&"test:native_comparison".parse().unwrap())
    .unwrap();
    let mut machine = Machine::default();
    machine.1.insert("test:native_comparison".into(), body);
    for (value, expected) in [
        (Some(2.755), (true, 1)),
        (Some(f64::NAN), (false, 0)),
        (Some(f64::INFINITY), (false, 0)),
        (Some(f64::NEG_INFINITY), (false, 0)),
        (Some(-0.0), (true, 0)),
        (Some(2.7549), (true, 0)),
        (None, (false, 0)),
        (Some(1.0e20), (false, 0)),
        (Some(3.0), (true, 1)),
    ] {
        if let Some(value) = value {
            machine.3.insert("entity @s Health".into(), value);
        } else {
            machine.3.remove("entity @s Health");
        }
        assert_eq!(machine.call_result("test:native_comparison"), expected);
    }
}

#[test]
fn numeric_conditions_share_scale_conversion_and_checked_arithmetic() {
    use crate::condition::ScoreCompareOp;
    use crate::entity::{EntityStateField, FixedScore, StatCurve};
    use crate::ir::{Actions, Cmd, NumericComparison};
    let left = FixedScore::__new("test", "comparison", "left", 100, 0, None);
    let right = FixedScore::__new("test", "comparison", "right", 1000, 0, None);
    for arithmetic in [false, true] {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let source = StatCurve::from(left.bind_to("@s", false));
        let source = if arithmetic {
            StatCurve::add([source, StatCurve::constant(0.5)])
        } else {
            source
        };
        let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
            left: source,
            op: ScoreCompareOp::Eq,
            right: StatCurve::from(right.bind_to("@s", false)),
        }))])
        .lower(&"test:scaled_comparison".parse().unwrap())
        .unwrap();
        let mut machine = Machine::default();
        machine.1.insert("test:scaled_comparison".into(), body);
        machine.0.insert(format!("@s {}", left.objective()), 125);
        machine.0.insert(
            format!("@s {}", right.objective()),
            if arithmetic { 1750 } else { 1250 },
        );
        assert_eq!(machine.call_result("test:scaled_comparison"), (true, 1));
        machine.0.insert(format!("@s {}", right.objective()), 1200);
        assert_eq!(machine.call_result("test:scaled_comparison"), (true, 0));
        machine
            .0
            .insert(format!("@s {}", left.objective()), i32::MAX);
        assert_eq!(machine.call_result("test:scaled_comparison"), (false, 0));
    }
}

#[test]
fn numeric_conditions_preserve_scale_exact_decimal_constants() {
    use crate::condition::ScoreCompareOp;
    use crate::entity::curve::{CurveInput, CurveKind, StatCurve};
    use crate::ir::{Actions, Cmd, NumericComparison};
    for constant in [0.29, -0.29, 0.57, -0.57] {
        for constant_on_left in [false, true] {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            let source = StatCurve {
                kind: CurveKind::Input(CurveInput::Score {
                    objective: "source".into(),
                    holder: None,
                    storage_scale: 100,
                    field: None,
                }),
            };
            let literal = StatCurve::constant(constant);
            let (left, right) = if constant_on_left {
                (literal, source)
            } else {
                (source, literal)
            };
            let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
                left,
                op: ScoreCompareOp::Gt,
                right,
            }))])
            .lower(&"test:decimal_comparison".parse().unwrap())
            .unwrap();
            let mut machine = Machine::default();
            machine.1.insert("test:decimal_comparison".into(), body);
            for stored in [
                i32::MIN,
                -58,
                -57,
                -30,
                -29,
                0,
                28,
                29,
                30,
                57,
                58,
                i32::MAX,
            ] {
                machine.0.insert("@s source".into(), stored);
                let logical = f64::from(stored) / 100.0;
                let expected = if constant_on_left {
                    constant > logical
                } else {
                    logical > constant
                };
                assert_eq!(
                    machine.call_result("test:decimal_comparison"),
                    (true, i32::from(expected)),
                    "{stored} versus {constant}, reversed={constant_on_left}"
                );
            }
        }
    }
}

#[test]
fn numeric_conditions_propagate_custom_failure_on_either_operand() {
    use crate::condition::ScoreCompareOp;
    use crate::entity::StatCurve;
    use crate::ir::{Actions, Cmd, NumericComparison};
    for custom_on_left in [false, true] {
        let _scope = crate::function::ExportFunctionRegistryScope::enter();
        let custom = StatCurve::custom(
            "test:custom"
                .parse::<crate::resource_ref::FunctionId>()
                .unwrap(),
            |_, fixed| fixed.encode(0.0, "test", "custom").map_err(Into::into),
        );
        let constant = StatCurve::constant(0.0);
        let (left, right) = if custom_on_left {
            (custom, constant)
        } else {
            (constant, custom)
        };
        let body = Actions(vec![Cmd::NumericComparison(Box::new(NumericComparison {
            left,
            op: ScoreCompareOp::Eq,
            right,
        }))])
        .lower(&"test:custom_comparison".parse().unwrap())
        .unwrap();
        let mut machine = Machine::default();
        machine.1.insert("test:custom_comparison".into(), body);
        for (command, expected) in [
            ("return 0", (true, 1)),
            ("return fail", (false, 0)),
            ("return 1", (true, 0)),
            ("return 0", (true, 1)),
        ] {
            machine.1.insert("test:custom".into(), vec![command.into()]);
            assert_eq!(machine.call_result("test:custom_comparison"), expected);
        }
    }
}

#[test]
fn inventory_numeric_reads_preserve_owner_zero_and_failure_semantics() {
    use crate::entity::FixedPoint;
    let owner = "test:inventory_count".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let curve = crate::item::ItemLocation::entity(sand_commands::Selector::player("Alice"))
        .count(sand_components::ItemId::minecraft("diamond").unwrap());
    let lowered = curve
        .lower_scoreboard("result", "test:inventory_count", FixedPoint::default())
        .unwrap();
    let rendered = render_lowered_curve(
        NumericContext::new(&owner, &holder).unwrap(),
        "inventory_count",
        &lowered,
    )
    .unwrap();
    let mut machine = Machine::default();
    for record in &rendered.records {
        machine.1.insert(
            format!("test:{}", record.path),
            record.content.lines().map(str::to_owned).collect(),
        );
    }
    machine.6.insert(
        "Bob".into(),
        BTreeMap::from([("minecraft:diamond".into(), 99)]),
    );
    for count in [23, 0, 1, 1024] {
        machine.6.insert(
            "Alice".into(),
            BTreeMap::from([("minecraft:diamond".into(), count)]),
        );
        let before = machine.6.clone();
        assert!(machine.execute(&rendered.commands));
        assert_eq!(machine.get("@s", "result"), count * 1000);
        assert_eq!(machine.6, before, "counting must never mutate inventory");
        assert!(
            !machine.0.contains_key("Alice result"),
            "commit must preserve the caller"
        );
    }
    machine.7 = "Bob".into();
    machine
        .6
        .get_mut("Alice")
        .unwrap()
        .insert("minecraft:diamond".into(), 6);
    assert!(machine.execute(&rendered.commands));
    assert_eq!(
        machine.get("Bob", "result"),
        6000,
        "source ownership must survive a different caller"
    );
    assert!(!machine.0.contains_key("Alice result"));
    machine.7.clear();
    machine
        .6
        .get_mut("Alice")
        .unwrap()
        .insert("minecraft:diamond".into(), i32::MAX);
    machine.0.insert("@s result".into(), 123);
    assert!(!machine.execute(&rendered.commands));
    assert_eq!(
        machine.get("@s", "result"),
        123,
        "scale overflow must preserve the destination"
    );
    machine.6.remove("Alice");
    machine.0.insert("@s result".into(), 123);
    assert!(!machine.execute(&rendered.commands));
    assert_eq!(machine.get("@s", "result"), 123);
    machine.8.insert("Alice".into());
    assert!(!machine.execute(&rendered.commands));
    assert_eq!(
        machine.get("@s", "result"),
        123,
        "non-player must not become zero"
    );
    assert_eq!(
        rendered
            .commands
            .iter()
            .filter(|line| line.contains("execute as Alice"))
            .count(),
        1
    );
    assert!(curve.validate_entity_inputs("test:mob", "health").is_err());
}

#[test]
fn inventory_numeric_reads_reject_plural_targets() {
    use crate::entity::FixedPoint;
    let owner = "test:inventory_many".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let curve = crate::item::ItemLocation::entity(sand_commands::Target::players())
        .count(sand_components::ItemId::minecraft("diamond").unwrap());
    let lowered = curve
        .lower_scoreboard("result", "test:inventory_many", FixedPoint::default())
        .unwrap();
    let error = render_lowered_curve(
        NumericContext::new(&owner, &holder).unwrap(),
        "inventory_many",
        &lowered,
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("inventory count"), "{error}");
}

#[test]
fn inventory_numeric_expression_keeps_two_player_sources_distinct() {
    use crate::entity::{FixedPoint, StatCurve};
    let owner = "test:inventory_sum".parse().unwrap();
    let holder = sand_commands::ScoreHolder::self_();
    let count = |player| {
        crate::item::ItemLocation::entity(sand_commands::Selector::player(player))
            .count(sand_components::ItemId::minecraft("diamond").unwrap())
    };
    let curve = StatCurve::add([count("Alice"), count("Bob")]);
    let lowered = curve
        .lower_scoreboard("result", "test:inventory_sum", FixedPoint::default())
        .unwrap();
    let rendered = render_lowered_curve(
        NumericContext::new(&owner, &holder).unwrap(),
        "inventory_sum",
        &lowered,
    )
    .unwrap();
    let mut machine = Machine::default();
    for record in &rendered.records {
        machine.1.insert(
            format!("test:{}", record.path),
            record.content.lines().map(str::to_owned).collect(),
        );
    }
    machine.6.insert(
        "Alice".into(),
        BTreeMap::from([("minecraft:diamond".into(), 3)]),
    );
    machine.6.insert(
        "Bob".into(),
        BTreeMap::from([("minecraft:diamond".into(), 7)]),
    );
    assert!(machine.execute(&rendered.commands));
    assert_eq!(machine.get("@s", "result"), 10_000);
    machine.6.get_mut("Alice").unwrap().clear();
    assert!(machine.execute(&rendered.commands));
    assert_eq!(machine.get("@s", "result"), 7_000);
    machine.6.remove("Bob");
    assert!(!machine.execute(&rendered.commands));
    assert_eq!(machine.get("@s", "result"), 7_000);
}
