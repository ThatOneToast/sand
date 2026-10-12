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

    fn escape(&mut self) -> bool {
        match self.character() {
            Some('\'' | '"' | '\\' | 'b' | 's' | 't' | 'n' | 'f' | 'r') => true,
            Some('x') => self.hex_escape(2).is_some(),
            // Preserve Java UTF-16 escape units without decoding into Rust chars.
            Some('u') => self.hex_escape(4).is_some(),
            Some('U') => self.hex_escape(8).is_some_and(|value| value <= 0x10FFFF),
            Some('N') => {
                if self.character() != Some('{') {
                    return false;
                }
                let Some(end) = self.rest.find('}') else {
                    return false;
                };
                let valid = named_character(&self.rest[..end]);
                self.rest = &self.rest[end + 1..];
                valid
            }
            _ => false,
        }
    }

    fn quoted(&mut self) -> bool {
        let Some(quote @ ('\'' | '"')) = self.character() else {
            return false;
        };
        while let Some(character) = self.character() {
            if character.is_control() {
                return false;
            }
            if character == '\\' {
                if !self.escape() {
                    return false;
                }
            } else if character == quote {
                return true;
            }
        }
        false
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
                        decimal_integer(token, max_bits).is_some_and(|bits| bits <= max_bits)
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
            Some(_) => scalar(self.token()),
            None => false,
        }
    }
}

// Java's name lookup uses canonical names, historical control names, and
// block-based names for algorithmic ranges. Unicode's loose name matching
// alone accepts spellings that Minecraft rejects (for example S_N_O_W_M_A_N).
fn named_character(name: &str) -> bool {
    let name = name.trim_matches(' ').to_ascii_uppercase();
    if super::java_names::CONTROL_NAMES.contains(&name.as_str()) {
        return true;
    }
    if let Some((block, hexadecimal)) = name.rsplit_once(' ')
        && let Ok(point) = u32::from_str_radix(hexadecimal, 16)
        && name == format!("{block} {point:X}")
        && super::java_names::BLOCK_NAMES
            .iter()
            .any(|(prefix, first, last)| block == *prefix && (*first..=*last).contains(&point))
    {
        return true;
    }
    let Some(character) = unicode_names2::character(&name) else {
        return false;
    };
    if super::java_names::BLOCK_NAMES
        .iter()
        .any(|(_, first, last)| (*first..=*last).contains(&(character as u32)))
    {
        return false;
    }
    unicode_names2::name(character).is_some_and(|canonical| canonical.to_string() == name)
}

// Decimal integer subset of current SNBT. Array context supplies the type
// for unsuffixed values and allows explicitly smaller integer element types.
fn decimal_integer(token: &str, implied_bits: u8) -> Option<u8> {
    let (digits, bits) = match token.as_bytes().last()? {
        b'b' | b'B' => (&token[..token.len() - 1], 8),
        b's' | b'S' => (&token[..token.len() - 1], 16),
        b'i' | b'I' => (&token[..token.len() - 1], 32),
        b'l' | b'L' => (&token[..token.len() - 1], 64),
        _ => (token, implied_bits),
    };
    let unsigned = digits.strip_prefix(['-', '+']).unwrap_or(digits);
    if unsigned.is_empty() || !unsigned.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if unsigned.len() > 1 && unsigned.starts_with('0') && unsigned.bytes().any(|c| c != b'0') {
        return None;
    }
    let number = digits.parse::<i64>().ok()?;
    let valid = match bits {
        8 => i8::try_from(number).is_ok(),
        16 => i16::try_from(number).is_ok(),
        32 => i32::try_from(number).is_ok(),
        64 => true,
        _ => false,
    };
    valid.then_some(bits)
}

fn scalar(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    // Plain identifiers are strings/booleans. Numeric-looking tokens must be
    // a supported decimal number, never an unchecked malformed numeric literal.
    if token.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
        return true;
    }
    if decimal_integer(token, 32).is_some() {
        return true;
    }
    match token.as_bytes().last().copied() {
        Some(b'f' | b'F') => token[..token.len() - 1]
            .parse::<f32>()
            .is_ok_and(f32::is_finite),
        Some(b'd' | b'D') => token[..token.len() - 1]
            .parse::<f64>()
            .is_ok_and(f64::is_finite),
        _ => token.contains(['.', 'e', 'E']) && token.parse::<f64>().is_ok_and(f64::is_finite),
    }
}

#[cfg(test)]
mod tests {
    use crate::{CommandProfile, ScoreHolder};

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
