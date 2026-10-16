//! Minecraft NBT path nodes, shared by typed data paths and resource text.

/// Parse every node and require complete consumption, including match compounds.
pub(super) fn valid(mut rest: &str) -> bool {
    let mut root = true;
    while !rest.is_empty() {
        match rest.as_bytes()[0] {
            b'{' if root => {
                if !crate::selector::snbt::consume_compound(&mut rest) {
                    return false;
                }
            }
            b'[' => {
                rest = &rest[1..];
                if rest.starts_with('{') {
                    if !crate::selector::snbt::consume_compound(&mut rest) {
                        return false;
                    }
                } else if !rest.starts_with(']') {
                    let end = rest
                        .find(|c: char| !c.is_ascii_digit() && c != '-')
                        .unwrap_or(rest.len());
                    if rest[..end].parse::<i32>().is_err() {
                        return false;
                    }
                    rest = &rest[end..];
                }
                let Some(next) = rest.strip_prefix(']') else {
                    return false;
                };
                rest = next;
            }
            b'"' | b'\'' => {
                if !quoted_key(&mut rest) {
                    return false;
                }
                if rest.starts_with('{') && !crate::selector::snbt::consume_compound(&mut rest) {
                    return false;
                }
            }
            _ => {
                let end = rest
                    .find([' ', '"', '\'', '[', ']', '.', '{', '}'])
                    .unwrap_or(rest.len());
                if end == 0 {
                    return false;
                }
                rest = &rest[end..];
                if rest.starts_with('{') && !crate::selector::snbt::consume_compound(&mut rest) {
                    return false;
                }
            }
        }
        root = false;
        if !rest.is_empty() && !rest.starts_with(['[', '{']) {
            let Some(next) = rest.strip_prefix('.') else {
                return false;
            };
            rest = next;
        }
    }
    true
}

/// Brigadier path keys only escape their delimiter or a backslash, unlike SNBT strings.
fn quoted_key(rest: &mut &str) -> bool {
    let delimiter = rest.as_bytes()[0] as char;
    let mut chars = rest[1..].char_indices();
    let mut nonempty = false;
    while let Some((index, character)) = chars.next() {
        if character == delimiter {
            *rest = &rest[index + 2..];
            return nonempty;
        }
        if character == '\\'
            && !chars
                .next()
                .is_some_and(|(_, c)| c == delimiter || c == '\\')
        {
            return false;
        }
        nonempty = true;
    }
    false
}
