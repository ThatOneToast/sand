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
                let mut i = 1;
                while command[i] != "run" {
                    let negate = command[i] == "unless";
                    assert!(matches!(command[i], "if" | "unless"));
                    assert_eq!(command[i + 1], "score");
                    let left = self.get(command[i + 2], command[i + 3]);
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
                    if condition == negate {
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
            for right in [1, 2, 3, 10, 1000, i32::MAX] {
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
