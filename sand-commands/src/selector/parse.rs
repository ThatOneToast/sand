//! Parse supported legacy selector text into the canonical typed selector.
//! Unknown grammar stays rejected at checked score-holder boundaries.
use super::*;

pub(super) fn selector(value: &str) -> Option<Selector> {
    if value.chars().any(char::is_control) {
        return None;
    }
    let (base, arguments) = if let Some((base, arguments)) = value.split_once('[') {
        (base, arguments.strip_suffix(']')?)
    } else {
        (value, "")
    };
    let mut selector = match base {
        "@s" => Selector::self_(),
        "@p" => Selector::nearest_player(),
        "@n" => Selector::nearest_entity(),
        "@r" => Selector::random_player(),
        "@a" => Selector::all_players(),
        "@e" => Selector::all_entities(),
        _ => return None,
    };
    if arguments.is_empty() {
        return Some(selector);
    }
    for argument in split_arguments(arguments)? {
        let (key, value) = argument.split_once('=')?;
        let key = key.trim();
        let value = value.trim();
        let negated = value.strip_prefix('!');
        selector.args.push(match key {
            "tag" => negated.map_or_else(
                || SelectorArg::Tag(value.into()),
                |v| SelectorArg::NotTag(v.into()),
            ),
            "team" => negated.map_or_else(
                || SelectorArg::Team(value.into()),
                |v| SelectorArg::NotTeam(v.into()),
            ),
            "name" => match negated {
                Some(value) => SelectorArg::NotName(parse_name(value)?),
                None => SelectorArg::Name(parse_name(value)?),
            },
            "type" => negated.map_or_else(
                || SelectorArg::Type(entity_type(value)),
                |v| SelectorArg::NotType(entity_type(v)),
            ),
            "limit" => SelectorArg::Limit(value.parse().ok()?),
            "sort" => SelectorArg::Sort(match value {
                "nearest" => SortOrder::Nearest,
                "furthest" => SortOrder::Furthest,
                "random" => SortOrder::Random,
                "arbitrary" => SortOrder::Arbitrary,
                _ => return None,
            }),
            "distance" => SelectorArg::Distance(value.into()),
            "level" => SelectorArg::Level(value.into()),
            "x_rotation" => SelectorArg::XRotation(value.into()),
            "y_rotation" => SelectorArg::YRotation(value.into()),
            "gamemode" => SelectorArg::Gamemode(value.into()),
            "scores" => SelectorArg::Scores(value.strip_prefix('{')?.strip_suffix('}')?.into()),
            "advancements" => SelectorArg::Advancements(advancements(value)?),
            "nbt" => SelectorArg::Nbt(value.into()),
            "predicate" => SelectorArg::Predicate(negated.map_or_else(
                || resource_location(value),
                |v| format!("!{}", resource_location(v)),
            )),
            "x" => SelectorArg::X(value.parse().ok()?),
            "y" => SelectorArg::Y(value.parse().ok()?),
            "z" => SelectorArg::Z(value.parse().ok()?),
            "dx" => SelectorArg::Dx(value.parse().ok()?),
            "dy" => SelectorArg::Dy(value.parse().ok()?),
            "dz" => SelectorArg::Dz(value.parse().ok()?),
            _ => return None,
        });
    }
    Some(selector)
}

fn resource_location(value: &str) -> String {
    if value.contains(':') {
        value.into()
    } else {
        format!("minecraft:{value}")
    }
}

fn entity_type(value: &str) -> String {
    value.strip_prefix('#').map_or_else(
        || resource_location(value),
        |tag| format!("#{}", resource_location(tag)),
    )
}

fn advancements(value: &str) -> Option<std::collections::BTreeMap<String, AdvancementMatch>> {
    let inner = value.strip_prefix('{')?.strip_suffix('}')?.trim();
    let mut filters = std::collections::BTreeMap::new();
    if inner.is_empty() {
        return Some(filters);
    }
    for entry in split_arguments(inner)? {
        let (name, progress) = entry.split_once('=')?;
        let name = resource_location(name.trim());
        validate::resource_location_shape(&name, "Selector", "advancements").ok()?;
        let progress = progress.trim();
        let progress = if let Some(criteria) = progress.strip_prefix('{') {
            let criteria = criteria.strip_suffix('}')?.trim();
            let mut matches = std::collections::BTreeMap::new();
            if !criteria.is_empty() {
                for criterion in split_arguments(criteria)? {
                    let (name, done) = criterion.rsplit_once('=')?;
                    let name = parse_name(name.trim())?;
                    let done = done.trim().parse::<bool>().ok()?;
                    if matches.insert(name, done).is_some() {
                        return None;
                    }
                }
            }
            AdvancementMatch::Criteria(matches)
        } else {
            AdvancementMatch::Complete(progress.parse().ok()?)
        };
        if filters.insert(name, progress).is_some() {
            return None;
        }
    }
    Some(filters)
}

/// Decode a complete Brigadier string into the canonical literal name.
/// Only the active quote delimiter and backslash may be escaped.
fn parse_name(value: &str) -> Option<String> {
    let Some(delimiter @ ('\'' | '"')) = value.chars().next() else {
        return value
            .chars()
            .all(unquoted_name_character)
            .then(|| value.into());
    };
    let mut result = String::new();
    let mut escaped = false;
    let mut characters = value[1..].chars();
    while let Some(character) = characters.next() {
        if escaped {
            if character != delimiter && character != '\\' {
                return None;
            }
            result.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == delimiter {
            return characters.next().is_none().then_some(result);
        } else {
            result.push(character);
        }
    }
    None
}

// Commas inside score maps, SNBT lists/compounds, and quoted strings are values,
// not selector argument separators. Reject unbalanced or trailing syntax.
fn split_arguments(value: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut nested = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '{' => nested.push('}'),
            '[' => nested.push(']'),
            '}' | ']' => {
                if nested.pop()? != character {
                    return None;
                }
            }
            ',' if nested.is_empty() => {
                parts.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if quote.is_some() || !nested.is_empty() || escaped {
        return None;
    }
    parts.push(&value[start..]);
    Some(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScoreHolder;

    #[test]
    fn parsed_single_selectors_retain_supported_filters() {
        for value in [
            "@e[type=minecraft:zombie,limit=1]",
            "@p[tag=ready]",
            "@a[scores={mana=1..,health=..20},limit=1]",
            "@e[nbt={Text:'a,b',Values:[1,2]},limit=1]",
        ] {
            let parsed = selector(value).expect("supported selector syntax");
            parsed.validate(&CommandProfile::unprofiled()).unwrap();
            assert_eq!(parsed.to_string(), value);
            ScoreHolder::compat(value.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
    }

    #[test]
    fn parsing_does_not_turn_ambiguous_or_invalid_text_into_single_holders() {
        for value in [
            "@e[type=minecraft:zombie]",
            "@e[limit=2]",
            "@e[limit=1,limit=2]",
            "@e[nbt={Text:'limit=1'}]",
            "@e[limit=1] run say injected",
            "@e[limit=1]\nkill @s",
            "@e[nbt={Text:'a,b},limit=1]",
            "@e[unknown=true,limit=1]",
            "@e[limit=1,]",
            "@e[nbt={x:[1,2}},limit=1]",
        ] {
            assert!(
                ScoreHolder::compat(value.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{value}"
            );
        }
    }
}

#[cfg(test)]
mod name_tests {
    use super::*;
    use crate::ScoreHolder;

    #[test]
    fn quoted_names_round_trip_as_canonical_literals() {
        for (text, expected) in [
            (
                r#"@e[name="Boss Mob",limit=1]"#,
                r#"@e[name="Boss Mob",limit=1]"#,
            ),
            (
                "@e[name='Boss Mob',limit=1]",
                r#"@e[name="Boss Mob",limit=1]"#,
            ),
            (
                r#"@e[name=!"Friendly Mob",limit=1]"#,
                r#"@e[name=!"Friendly Mob",limit=1]"#,
            ),
            (
                r#"@e[name="Boss \"One\"\\Path",limit=1]"#,
                r#"@e[name="Boss \"One\"\\Path",limit=1]"#,
            ),
            (
                r#"@e[name="Boss,limit=20",limit=1]"#,
                r#"@e[name="Boss,limit=20",limit=1]"#,
            ),
            (r#"@e[name="",limit=1]"#, r#"@e[name="",limit=1]"#),
        ] {
            let parsed = selector(text).unwrap();
            parsed.validate(&CommandProfile::unprofiled()).unwrap();
            assert_eq!(parsed.to_string(), expected);
            ScoreHolder::compat(text.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
            assert_eq!(selector(expected).unwrap().to_string(), expected);
        }
    }

    #[test]
    fn invalid_name_strings_do_not_bypass_compatibility_validation() {
        for text in [
            r#"@e[name="Boss\q",limit=1]"#,
            r#"@e[name="Boss"junk,limit=1]"#,
            r#"@e[name="Boss,limit=1]"#,
            "@e[name=Boss Mob,limit=1]",
            "@e[name=\"Boss\nMob\",limit=1]",
        ] {
            assert!(selector(text).is_none(), "{text}");
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err()
            );
        }
    }

    #[test]
    fn typed_names_are_literals_and_cannot_add_selector_arguments() {
        for name in [
            "Boss Mob",
            "Boss,limit=20",
            "a] run kill @a",
            "Boss \"One\"\\Path",
            "雪",
        ] {
            let target = Selector::all_entities().name(name).limit(1);
            target.validate(&CommandProfile::unprofiled()).unwrap();
            let rendered = target.to_string();
            let parsed = selector(&rendered).unwrap();
            assert_eq!(parsed.args.len(), 2);
            assert!(matches!(&parsed.args[0], SelectorArg::Name(actual) if actual == name));
            assert_eq!(parsed.to_string(), rendered);
        }
        assert!(
            Selector::all_entities()
                .name("bad\nname")
                .validate(&CommandProfile::unprofiled())
                .is_err()
        );
    }
}

#[cfg(test)]
mod nbt_tests {
    use super::*;
    use crate::ScoreHolder;

    #[test]
    fn negated_compounds_preserve_filter_and_single_holder_validation() {
        for text in ["@e[nbt=!{NoAI:1b},limit=1]", "@e[nbt={NoAI:1b},limit=1]"] {
            let parsed = selector(text).unwrap();
            parsed.validate(&CommandProfile::unprofiled()).unwrap();
            assert_eq!(parsed.to_string(), text);
            ScoreHolder::compat(text.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
        for text in [
            "@e[nbt=!!{NoAI:1b},limit=1]",
            "@e[nbt=!1b,limit=1]",
            "@e[nbt=!{NoAI:1b,limit=1]",
        ] {
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
    }
}

#[cfg(test)]
mod advancement_tests {
    use super::*;
    use crate::ScoreHolder;

    #[test]
    fn advancement_completion_and_criterion_filters_are_single_holder_sources() {
        for text in [
            "@a[advancements={minecraft:story/mine_stone=true},limit=1]",
            "@a[advancements={minecraft:story/mine_stone=false},limit=1]",
            "@e[advancements={minecraft:story/mine_stone={get_stone=true}},limit=1]",
            r#"@a[advancements={game:quest={"criterion = one"=false,other=true}},limit=1]"#,
            "@a[advancements={},limit=1]",
            "@a[advancements={game:quest={}},limit=1]",
        ] {
            let parsed = selector(text).unwrap();
            parsed.validate(&CommandProfile::unprofiled()).unwrap();
            assert_eq!(parsed.to_string(), text);
            ScoreHolder::compat(text.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
    }

    #[test]
    fn invalid_advancement_filters_cannot_bypass_validation() {
        for text in [
            "@a[advancements={game:quest=maybe},limit=1]",
            "@a[advancements={game:quest={one=1}},limit=1]",
            "@a[advancements={game:quest=true,game:quest=false},limit=1]",
            "@a[advancements={game:quest={one=true,one=false}},limit=1]",
            "@a[advancements={game:quest={one=true},limit=1]",
            "@a[advancements={bad id=true},limit=1]",
        ] {
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
    }
}

#[cfg(test)]
mod repeated_predicate_tests {
    use super::*;
    use crate::ScoreHolder;

    #[test]
    fn repeated_predicates_remain_valid_single_entity_sources() {
        let text = "@e[predicate=demo:is_hostile,predicate=!demo:is_friendly,limit=1]";
        let parsed = selector(text).unwrap();
        parsed.validate(&CommandProfile::unprofiled()).unwrap();
        assert_eq!(parsed.to_string(), text);
        ScoreHolder::compat(text.into())
            .validate_single(&CommandProfile::unprofiled())
            .unwrap();
    }
}

#[cfg(test)]
mod selector_cardinality_tests {
    use super::*;
    use crate::ScoreHolder;

    #[test]
    fn repeated_native_filters_and_explicit_single_limits_preserve_cardinality() {
        for text in [
            "@e[nbt={OnGround:1b},nbt=!{NoAI:1b},limit=1]",
            "@a[gamemode=!creative,gamemode=!spectator,limit=1]",
            "@p[limit=1,sort=furthest]",
            "@n",
            "@n[type=minecraft:zombie]",
            "@n[limit=1,sort=random]",
            "@r[limit=1,sort=nearest]",
        ] {
            let parsed = selector(text).unwrap();
            parsed.validate(&CommandProfile::unprofiled()).unwrap();
            assert_eq!(parsed.to_string(), text);
            ScoreHolder::compat(text.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
        for text in ["@p[limit=2]", "@r[limit=2]", "@n[limit=2]"] {
            let parsed = selector(text).unwrap();
            parsed.validate(&CommandProfile::unprofiled()).unwrap();
            assert!(!parsed.is_statically_single());
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err()
            );
        }
        for text in [
            "@s[limit=1]",
            "@a[gamemode=creative,gamemode=survival,limit=1]",
        ] {
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err()
            );
        }
    }
}

#[cfg(test)]
mod empty_scores_and_team_tests {
    use super::*;
    use crate::ScoreHolder;

    #[test]
    fn empty_scores_and_repeated_negative_teams_are_valid() {
        for text in [
            "@s[scores={}]",
            "@s[team=red,team=!blue,team=!green]",
            "@s[team=]",
        ] {
            ScoreHolder::compat(text.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
    }

    #[test]
    fn duplicate_positive_teams_and_malformed_score_entries_are_rejected() {
        for text in [
            "@s[team=red,team=blue]",
            "@s[team=,team=red]",
            "@s[scores={broken}]",
            "@s[scores={points=}]",
            "@s[scores={points=1,}]",
        ] {
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
    }
}

#[cfg(test)]
mod integer_range_tests {
    use super::*;
    #[test]
    fn integer_selector_ranges_use_bounded_integer_grammar() {
        for text in [
            "@s[scores={points=-2147483648..2147483647}]",
            "@s[level=0..2147483647]",
            "@s[scores={points=..-1}]",
        ] {
            selector(text)
                .unwrap()
                .validate(&CommandProfile::unprofiled())
                .unwrap();
        }
        for text in [
            "@s[scores={points=1.0}]",
            "@s[scores={points=1e3}]",
            "@s[level=1e3]",
            "@s[level=1.0]",
            "@s[scores={points=2147483648}]",
            "@s[scores={points=-2147483649}]",
            "@s[level=0..2147483648]",
            "@s[scores={points=+1}]",
        ] {
            assert!(
                selector(text)
                    .unwrap()
                    .validate(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
    }
}

#[cfg(test)]
mod default_namespace_tests {
    use super::*;
    use crate::ScoreHolder;
    #[test]
    fn resource_filters_normalize_the_default_minecraft_namespace() {
        for (source, expected) in [
            (
                "@e[type=zombie,limit=1]",
                "@e[type=minecraft:zombie,limit=1]",
            ),
            (
                "@e[type=!#undead,limit=1]",
                "@e[type=!#minecraft:undead,limit=1]",
            ),
            (
                "@s[predicate=ready,predicate=!blocked]",
                "@s[predicate=minecraft:ready,predicate=!minecraft:blocked]",
            ),
            (
                "@s[advancements={story/root=true}]",
                "@s[advancements={minecraft:story/root=true}]",
            ),
        ] {
            let parsed = selector(source).unwrap();
            parsed.validate(&CommandProfile::unprofiled()).unwrap();
            assert_eq!(parsed.to_string(), expected);
            ScoreHolder::compat(source.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
        assert!(
            selector("@s[advancements={story/root=true,minecraft:story/root=false}]").is_none()
        );
    }
}

#[cfg(test)]
mod compound_and_decimal_tests {
    use super::*;
    use crate::ScoreHolder;
    #[test]
    fn floating_ranges_use_brigadier_decimal_grammar() {
        for text in [
            "@s[distance=.5..1.25]",
            "@s[x_rotation=-90.5..90.5]",
            "@s[y_rotation=..180]",
        ] {
            ScoreHolder::compat(text.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
        for text in [
            "@s[distance=1e3]",
            "@s[x_rotation=-1E2]",
            "@s[y_rotation=0..1e3]",
            "@s[distance=+1.5]",
        ] {
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn nbt_filter_consumes_exactly_one_compound() {
        for text in [
            "@s[nbt={}{ }]",
            "@s[nbt={}{}]",
            "@s[nbt=!{}{}]",
            "@s[nbt={} {}]",
        ] {
            assert!(
                ScoreHolder::compat(text.into())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
        for text in [
            "@s[nbt={nested:{},values:[{}]}]",
            r#"@s[nbt={text:"{}{}",nested:{}}]"#,
        ] {
            ScoreHolder::compat(text.into())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
    }
}

#[cfg(test)]
mod score_objective_tests {
    use super::*;
    use crate::ScoreHolder;
    #[test]
    fn score_map_keys_follow_canonical_objective_validation() {
        for objective in [
            "",
            "bad/name",
            "bad:name",
            "bad$name",
            "über",
            "name_that_is_too_long",
        ] {
            let text = format!("@s[scores={{{objective}=1}}]");
            assert!(
                ScoreHolder::compat(text.clone())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
        for objective in ["points", "A_1-2.3+4", "sixteen_chars_16"] {
            let text = format!("@s[scores={{{objective}=1}}]");
            ScoreHolder::compat(text)
                .validate_single(&CommandProfile::unprofiled())
                .unwrap();
        }
    }
}
