//! Executes the emitted arithmetic subset with Minecraft i32 score semantics.
use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct Machine(BTreeMap<String, i32>);
impl Machine {
    fn get(&self, holder: &str, objective: &str) -> i32 {
        self.0[&format!("{holder} {objective}")]
    }
    fn run(&mut self, command: &[&str]) -> bool {
        match command[0] {
            "return" => false,
            "execute" => {
                if command[1] == "store" {
                    assert_eq!(&command[1..4], &["store", "success", "score"]);
                    assert_eq!(command[6], "run");
                    let inner = &command[7..];
                    assert_eq!(&inner[..3], &["scoreboard", "players", "operation"]);
                    let present = self.0.contains_key(&format!("{} {}", inner[6], inner[7]));
                    let success = present && self.run(inner);
                    self.0
                        .insert(format!("{} {}", command[4], command[5]), i32::from(success));
                    return true;
                }
                let mut i = 1;
                while command[i] != "run" {
                    let negate = command[i] == "unless";
                    assert!(matches!(command[i], "if" | "unless"));
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
                            "<" => left < right,
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
            output.commands.iter().any(|line| line
                .contains("run scoreboard players operation #value ")
                && line.ends_with("= @s mana")),
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
                && line.ends_with(&format!("= @r {}", field.objective())))
    );
}
