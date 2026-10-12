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

    fn quoted(&mut self) -> bool {
        let Some(quote @ ('\'' | '"')) = self.rest.chars().next() else {
            return false;
        };
        self.rest = &self.rest[1..];
        let mut escaped = false;
        for (index, character) in self.rest.char_indices() {
            if character.is_control() {
                return false;
            }
            if escaped {
                if character != quote && character != '\\' {
                    return false;
                }
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote {
                self.rest = &self.rest[index + 1..];
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
                        match kind {
                            b'B' => token
                                .strip_suffix(['b', 'B'])
                                .is_some_and(|v| v.parse::<i8>().is_ok()),
                            b'I' => token.parse::<i32>().is_ok(),
                            b'L' => token
                                .strip_suffix(['l', 'L'])
                                .is_some_and(|v| v.parse::<i64>().is_ok()),
                            _ => false,
                        }
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

fn scalar(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    // Plain identifiers are strings/booleans. Numeric-looking tokens must be
    // a supported decimal number, never an unchecked malformed numeric literal.
    if token.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
        return true;
    }
    match token.as_bytes().last().copied() {
        Some(b'b' | b'B') => token[..token.len() - 1].parse::<i8>().is_ok(),
        Some(b's' | b'S') => token[..token.len() - 1].parse::<i16>().is_ok(),
        Some(b'l' | b'L') => token[..token.len() - 1].parse::<i64>().is_ok(),
        Some(b'f' | b'F') => token[..token.len() - 1]
            .parse::<f32>()
            .is_ok_and(f32::is_finite),
        Some(b'd' | b'D') => token[..token.len() - 1]
            .parse::<f64>()
            .is_ok_and(f64::is_finite),
        _ => {
            token.parse::<i32>().is_ok()
                || (token.contains(['.', 'e', 'E'])
                    && token.parse::<f64>().is_ok_and(f64::is_finite))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CommandProfile, ScoreHolder};

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
            "{a:[I;1b]}",
            "{a:[L;1]}",
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
