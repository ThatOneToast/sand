//! Checked, conservative SNBT subset for legacy selector conversion.
//! Unsupported syntax stays a compatibility value rather than being promoted
//! into a checked score holder. Explicit raw NBT builders remain escape hatches.

pub(super) fn supported_compound(text: &str) -> bool {
    let mut parser = Parser { rest: text };
    text.starts_with('{') && parser.value(0) && parser.rest.is_empty()
}

struct Parser<'a> {
    rest: &'a str,
}

impl Parser<'_> {
    fn whitespace(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\t']);
    }

    fn take(&mut self, character: char) -> bool {
        self.whitespace();
        if let Some(rest) = self.rest.strip_prefix(character) {
            self.rest = rest;
            true
        } else {
            false
        }
    }

    fn character(&mut self) -> Option<char> {
        let character = self.rest.chars().next()?;
        self.rest = &self.rest[character.len_utf8()..];
        Some(character)
    }

    fn hex_escape(&mut self, digits: usize) -> Option<u32> {
        let value = self.rest.get(..digits)?;
        if !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(value, 16).ok()?;
        self.rest = &self.rest[digits..];
        Some(value)
    }

    fn escape(&mut self) -> Option<u32> {
        match self.character()? {
            c @ ('\'' | '"' | '\\') => Some(c as u32),
            'b' => Some(8),
            's' => Some(32),
            't' => Some(9),
            'n' => Some(10),
            'f' => Some(12),
            'r' => Some(13),
            'x' => self.hex_escape(2),
            'u' => self.hex_escape(4),
            'U' => self.hex_escape(8).filter(|value| *value <= 0x10FFFF),
            'N' => {
                if self.character() != Some('{') {
                    return None;
                }
                let end = self.rest.find('}')?;
                let point = named_character(&self.rest[..end]);
                self.rest = &self.rest[end + 1..];
                point
            }
            _ => None,
        }
    }

    fn quoted_units(&mut self) -> Option<Vec<u16>> {
        let quote @ ('\'' | '"') = self.character()? else {
            return None;
        };
        let mut units = Vec::new();
        while let Some(character) = self.character() {
            if character.is_control() {
                return None;
            }
            if character == quote {
                return Some(units);
            }
            let point = if character == '\\' {
                self.escape()?
            } else {
                character as u32
            };
            if point <= 0xffff {
                units.push(point as u16);
            } else {
                units.push(0xd800 + ((point - 0x10000) >> 10) as u16);
                units.push(0xdc00 + ((point - 0x10000) & 0x3ff) as u16);
            }
        }
        None
    }

    fn quoted(&mut self) -> bool {
        self.quoted_units().is_some()
    }

    fn boolean(&mut self, depth: usize) -> bool {
        if depth >= 64 {
            return false;
        }
        self.whitespace();
        let token = self.token().to_owned();
        if token == "bool" && self.take('(') {
            self.boolean(depth + 1) && self.take(')')
        } else {
            token.eq_ignore_ascii_case("true")
                || token.eq_ignore_ascii_case("false")
                || integer(&token, 32).is_some()
                || float(&token)
        }
    }

    fn uuid(&mut self) -> bool {
        self.whitespace();
        let units = if self.rest.starts_with(['\'', '"']) {
            let Some(units) = self.quoted_units() else {
                return false;
            };
            units
        } else {
            let token = self.token();
            if !token.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
                return false;
            }
            token.encode_utf16().collect()
        };
        valid_uuid(&units) && self.take(')')
    }

    fn token(&mut self) -> &str {
        let end = self
            .rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+' | '.')))
            .unwrap_or(self.rest.len());
        let (token, rest) = self.rest.split_at(end);
        self.rest = rest;
        token
    }

    fn value(&mut self, depth: usize) -> bool {
        if depth >= 64 {
            return false;
        }
        self.whitespace();
        match self.rest.chars().next() {
            Some('{') => {
                self.rest = &self.rest[1..];
                if self.take('}') {
                    return true;
                }
                loop {
                    self.whitespace();
                    let key = if self.rest.starts_with(['\'', '"']) {
                        self.quoted()
                    } else {
                        !self.token().is_empty()
                    };
                    if !key || !self.take(':') || !self.value(depth + 1) {
                        return false;
                    }
                    if self.take('}') {
                        return true;
                    }
                    if !self.take(',') {
                        return false;
                    }
                    if self.take('}') {
                        return true;
                    }
                }
            }
            Some('[') => {
                self.rest = &self.rest[1..];
                self.whitespace();
                let array_type = if self.rest.as_bytes().get(1) == Some(&b';') {
                    let kind = self.rest.as_bytes()[0];
                    if !matches!(kind, b'B' | b'I' | b'L') {
                        return false;
                    }
                    self.rest = &self.rest[2..];
                    Some(kind)
                } else {
                    None
                };
                if self.take(']') {
                    return true;
                }
                loop {
                    self.whitespace();
                    let valid = if let Some(kind) = array_type {
                        let token = self.token();
                        let max_bits = match kind {
                            b'B' => 8,
                            b'I' => 32,
                            b'L' => 64,
                            _ => unreachable!(),
                        };
                        integer(token, max_bits).is_some_and(|bits| bits <= max_bits)
                    } else {
                        self.value(depth + 1)
                    };
                    if !valid {
                        return false;
                    }
                    if self.take(']') {
                        return true;
                    }
                    if !self.take(',') {
                        return false;
                    }
                    if self.take(']') {
                        return true;
                    }
                }
            }
            Some('\'' | '"') => self.quoted(),
            Some(_) => {
                let token = self.token().to_owned();
                if token == "bool" && self.take('(') {
                    self.boolean(depth + 1) && self.take(')')
                } else if token == "uuid" && self.take('(') {
                    self.uuid()
                } else {
                    scalar(&token)
                }
            }
            None => false,
        }
    }
}

// Java's name lookup uses canonical names, historical control names, and
// block-based names for algorithmic ranges. Unicode's loose name matching
// alone accepts spellings that Minecraft rejects (for example S_N_O_W_M_A_N).
fn named_character(name: &str) -> Option<u32> {
    let name = name.trim_matches(' ').to_ascii_uppercase();
    if let Some(index) = super::java_names::CONTROL_NAMES
        .iter()
        .position(|entry| *entry == name)
    {
        return Some(if index < 32 {
            index as u32
        } else {
            index as u32 + 95
        });
    }
    if let Some((block, hexadecimal)) = name.rsplit_once(' ')
        && let Ok(point) = u32::from_str_radix(hexadecimal, 16)
        && name == format!("{block} {point:X}")
        && super::java_names::BLOCK_NAMES
            .iter()
            .any(|(prefix, first, last)| block == *prefix && (*first..=*last).contains(&point))
    {
        return Some(point);
    }
    let character = unicode_names2::character(&name)?;
    if super::java_names::BLOCK_NAMES
        .iter()
        .any(|(_, first, last)| (*first..=*last).contains(&(character as u32)))
    {
        return None;
    }
    unicode_names2::name(character)
        .is_some_and(|canonical| canonical.to_string() == name)
        .then_some(character as u32)
}

// Match UUID.fromString's accepted short groups, optional plus signs, and
// Character.digit UTF-16 hexadecimal parsing, including its signed-long bound.
fn valid_uuid(units: &[u16]) -> bool {
    if units.len() > 36 {
        return false;
    }
    let parts: Vec<_> = units.split(|unit| *unit == u16::from(b'-')).collect();
    parts.len() == 5
        && parts.iter().all(|part| {
            let part = part.strip_prefix(&[u16::from(b'+')]).unwrap_or(part);
            if part.is_empty() {
                return false;
            }
            part.iter()
                .try_fold(0_i64, |value, unit| {
                    let digit =
                        super::java_names::HEX_DIGITS
                            .iter()
                            .find_map(|(first, last, start)| {
                                (*first..=*last)
                                    .contains(unit)
                                    .then(|| u32::from(unit - first) + u32::from(*start))
                            })?;
                    value.checked_mul(16)?.checked_add(i64::from(digit))
                })
                .is_some()
        })
}

// Numeric digit groups allow repeated separators only between digits.
fn digits(text: &str, radix: u32) -> Option<String> {
    if text.is_empty() || text.starts_with('_') || text.ends_with('_') {
        return None;
    }
    text.chars()
        .all(|c| c == '_' || c.is_digit(radix))
        .then(|| text.replace('_', ""))
}

// Array context supplies the type for unsuffixed values and permits widening
// explicitly smaller integer types. Binary/hex default to unsigned parsing;
// the resulting NBT value is still stored in the corresponding signed type.
fn integer(token: &str, implied_bits: u8) -> Option<u8> {
    let negative = token.starts_with('-');
    let number = token.strip_prefix(['-', '+']).unwrap_or(token);
    let (number, radix) = if let Some(number) = number
        .strip_prefix("0x")
        .or_else(|| number.strip_prefix("0X"))
    {
        (number, 16)
    } else if let Some(number) = number
        .strip_prefix("0b")
        .or_else(|| number.strip_prefix("0B"))
    {
        (number, 2)
    } else {
        (number, 10)
    };
    // Greedy hexadecimal digits include b/B: 0xFFb is an int, not a byte.
    let end = number
        .find(|c: char| c != '_' && !c.is_digit(radix))
        .unwrap_or(number.len());
    let magnitude = digits(&number[..end], radix)?;
    if radix == 10 && magnitude.len() > 1 && magnitude.starts_with('0') {
        return None;
    }
    let suffix = number[end..].to_ascii_lowercase();
    let (bits, signed) = match suffix.as_str() {
        "" => (implied_bits, radix == 10),
        "b" => (8, radix == 10),
        "s" => (16, radix == 10),
        "i" => (32, radix == 10),
        "l" => (64, radix == 10),
        "sb" => (8, true),
        "ss" => (16, true),
        "si" => (32, true),
        "sl" => (64, true),
        "ub" => (8, false),
        "us" => (16, false),
        "ui" => (32, false),
        "ul" => (64, false),
        _ => return None,
    };
    if negative && !signed {
        return None;
    }
    let magnitude = u64::from_str_radix(&magnitude, radix).ok()?;
    let bound = if signed {
        (1_u64 << (bits - 1)) - u64::from(!negative)
    } else {
        u64::MAX >> (64 - bits)
    };
    (magnitude <= bound).then_some(bits)
}

fn float(token: &str) -> bool {
    let suffix = token.as_bytes().last().copied();
    let explicit = matches!(suffix, Some(b'f' | b'F' | b'd' | b'D'));
    let number = if explicit {
        &token[..token.len() - 1]
    } else {
        token
    };
    let unsigned = number.strip_prefix(['-', '+']).unwrap_or(number);
    let (mantissa, exponent) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(m, e)| (m, Some(e)));
    if exponent.is_some_and(|e| digits(e.strip_prefix(['-', '+']).unwrap_or(e), 10).is_none()) {
        return false;
    }
    let valid = if let Some((whole, fraction)) = mantissa.split_once('.') {
        (!whole.is_empty() || !fraction.is_empty())
            && (whole.is_empty() || digits(whole, 10).is_some())
            && (fraction.is_empty() || digits(fraction, 10).is_some())
    } else {
        (explicit || exponent.is_some()) && digits(mantissa, 10).is_some()
    };
    if !valid {
        return false;
    }
    let normalized = number.replace('_', "");
    if matches!(suffix, Some(b'f' | b'F')) {
        normalized.parse::<f32>().is_ok_and(f32::is_finite)
    } else {
        normalized.parse::<f64>().is_ok_and(f64::is_finite)
    }
}

fn scalar(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    // Numeric-looking tokens cannot fall back to unquoted strings.
    token.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        || integer(token, 32).is_some()
        || float(token)
}

#[cfg(test)]
mod tests {
    use crate::{CommandProfile, ScoreHolder};

    #[test]
    fn conversion_literals_match_vanilla_26_2() {
        for value in [
            "bool(1)",
            "bool(false)",
            "bool(TRUE)",
            "bool(bool(-1))",
            "bool(0xFF)",
            "bool ( 1 )",
            "bool(1.5)",
            r#"uuid("١-１-Ｆ-1-1")"#,
            r#"uuid("\N{DIGIT ONE}-1-1-1-1")"#,
            r#"uuid("00000000-0000-0000-0000-000000000000")"#,
            r#"uuid("1-1-1-1-1")"#,
            r#"uuid("+1-1-1-1-1")"#,
            r#"uuid("fffffffff-1-1-1-1")"#,
            r#"uuid("\u0031-1-1-1-1")"#,
            "uuid(abcdef-1-1-1-1)",
        ] {
            let selector = format!("@s[nbt={{value:{value}}}]");
            ScoreHolder::compat(selector.clone())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap_or_else(|error| panic!("{selector}: {error}"));
        }
        for value in [
            r#"bool("true")"#,
            "bool(1,2)",
            "BOOL(1)",
            "bool(NaN)",
            "bool({})",
            "bool()",
            "bool(true",
            "uuid(1)",
            "uuid(1-1-1-1-1)",
            r#"uuid("00000000000000000000000000000000")"#,
            r#"uuid("8000000000000000-1-1-1-1")"#,
            r#"uuid("1-1-1-1-1", "1-1-1-1-1")"#,
            r#"uuid("\uD800-1-1-1-1")"#,
            "[I;bool(true)]",
        ] {
            let selector = format!("@s[nbt={{value:{value}}}]");
            assert!(
                ScoreHolder::compat(selector.clone())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{selector}"
            );
        }
    }

    #[test]
    fn modern_numeric_literals_match_vanilla_26_2() {
        // These cases were checked with Mojang's 26.2 TagParser.parseCompoundFully.
        for value in [
            "0xCAFE",
            "0XFF",
            "0xFFb",
            "0xFFub",
            "0b10_01",
            "0xAB_CD",
            "1__0",
            "1_2.3_4__5f",
            "1_2e3_4",
            "240ub",
            "-16sb",
            "0xFFFFFFFF",
            "0xfffffffffffffffful",
            "-0x80sb",
            "+0x1",
            "-0b1sb",
            "+1ub",
            "1UB",
            "0xffs",
            "-128sb",
            "255ub",
            "01.0",
            "01f",
            "01e2",
            "1.e2",
            ".1",
            "1.",
            "1e2f",
            "[B;0xff,255ub,-128sb]",
            "[I;0xffub,0xffffus,0xffffffff]",
            "[L;0xffffffffffffffff,0b111ui]",
        ] {
            let text = format!("@s[nbt={{value:{value}}}]");
            ScoreHolder::compat(text.clone())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap_or_else(|error| panic!("{text}: {error}"));
        }
        for value in [
            "0x",
            "0b2",
            "0x100000000",
            "0xffffffffffffffffsl",
            "-0x1",
            "-0b1",
            "-1ub",
            "128sb",
            "256ub",
            "0x80sb",
            "00",
            "0_0",
            "01",
            "1_f",
            "1_.0",
            "1._0",
            "1e_2",
            "1e2_",
            "1e2b",
            "0x_FF",
            "0xff_",
            "1e1000",
            "1e100f",
            "1.0.0",
            "1e2e3",
            "[B;0x100]",
            "[I;0x1ul]",
            "[L;18446744073709551616ul]",
        ] {
            let text = format!("@s[nbt={{value:{value}}}]");
            assert!(
                ScoreHolder::compat(text.clone())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn current_snbt_escapes_are_checked_without_decoding_selector_structure() {
        for value in [
            r"line\nnext",
            r"\b\s\t\n\f\r",
            r"\x42\u2603\U0001F600",
            r"\uD83D\uDE00",
            r"\N{Snowman}",
            r"\N{BACKSPACE}\N{NULL}\N{LINE FEED (LF)}",
            r"\N{CJK UNIFIED IDEOGRAPHS 4E00}\N{HANGUL SYLLABLES AC00}",
            r"\uD800\U0000DC00",
            r"\N{LATIN CAPITAL LETTER A}",
            r"\x7d\u005d,limit=20",
            r"quote\'backslash\\",
        ] {
            let text = format!("@s[nbt={{name:'{value}'}}]");
            ScoreHolder::compat(text.clone())
                .validate_single(&CommandProfile::unprofiled())
                .unwrap_or_else(|error| panic!("{text}: {error}"));
        }
        for value in [
            r"\q",
            r"\x4",
            r"\xGG",
            r"\u123",
            r"\uZZZZ",
            r"\U00110000",
            r"\N{}",
            r"\N{NOT A CHARACTER}",
            r"\N{S_N_O_W_M_A_N}",
            r"\N{LINE FEED}",
            r"\N{CJK UNIFIED IDEOGRAPH-4E00}",
            r"\N{HANGUL SYLLABLE GA}",
            r"\N{SNOWMAN",
            r"\N SNOWMAN",
        ] {
            let text = format!("@s[nbt={{name:'{value}'}}]");
            assert!(
                ScoreHolder::compat(text.clone())
                    .validate_single(&CommandProfile::unprofiled())
                    .is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn compatibility_nbt_requires_complete_key_value_grammar() {
        for nbt in [
            "{NoAI:}",
            "{:1b}",
            "{NoAI}",
            "{NoAI::1b}",
            "{a:1 b:2}",
            "{a:1,,b:2}",
            "{a:[,]}",
            "{a:[1,,2]}",
            "{a:{b:}}",
            "{a:[B;128b]}",
            "{a:[B;1s]}",
            "{a:[I;1L]}",
            "{a:[B;128]}",
            "{a:01}",
            "{a:[I;01]}",
            "{a:'bad\\q'}",
            "{a:0xZZ}",
        ] {
            for negation in ["", "!"] {
                let selector = format!("@s[nbt={negation}{nbt}]");
                assert!(
                    ScoreHolder::compat(selector.clone())
                        .validate_single(&CommandProfile::unprofiled())
                        .is_err(),
                    "{selector}"
                );
            }
        }
        for nbt in [
            "{}",
            "{bytes:[B;1,2,-128,127],ints:[I;1b,2s,3,4i],longs:[L;1,2b,3s,4i,5L]}",
            "{NoAI:1b}",
            "{name:'Boss, Mob',nested:{a:true},list:[1,2]}",
            "{bytes:[B;-128b,127B],ints:[I;-2147483648,2147483647],longs:[L;1L]}",
            "{list:[{},'value',2.5d],value:-1.2e3f}",
            "{key:'quoted\\'value'}",
        ] {
            let selector = format!("@s[nbt={nbt}]");
            let holder = ScoreHolder::compat(selector.clone());
            holder
                .validate_single(&CommandProfile::unprofiled())
                .unwrap_or_else(|error| panic!("{selector}: {error}"));
        }
    }
}
