//! Canonical global State identity shared by derives and owned compiler inputs.

/// Fixed-size compiler identity usable in static descriptors and borrowed exports.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalStateHolder {
    bytes: [u8; 39],
    len: usize,
}

impl GlobalStateHolder {
    /// Preserve the established sanitized prefix and FNV-1a suffix for a State ID.
    pub const fn new(logical: &str) -> Self {
        let source = logical.as_bytes();
        let mut hash = 14_695_981_039_346_656_037u64;
        let mut index = 0;
        while index < source.len() {
            hash ^= source[index] as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
            index += 1;
        }
        let mut bytes = [0; 39];
        let prefix = b"#sand_";
        index = 0;
        while index < prefix.len() {
            bytes[index] = prefix[index];
            index += 1;
        }
        let mut len = prefix.len();
        index = 0;
        while index < source.len() && len < prefix.len() + 24 {
            let byte = source[index];
            // One replacement per Unicode scalar, matching the historical
            // derive implementation. Valid resource IDs are entirely ASCII.
            if byte & 0xc0 != 0x80 {
                bytes[len] = if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.') {
                    byte
                } else {
                    b'_'
                };
                len += 1;
            }
            index += 1;
        }
        bytes[len] = b'_';
        len += 1;
        index = 0;
        while index < 8 {
            bytes[len] = b"0123456789abcdef"[((hash as u32 >> (28 - index * 4)) & 15) as usize];
            len += 1;
            index += 1;
        }
        Self { bytes, len }
    }

    /// Borrow the identity without allocation or global interning.
    pub const fn as_str(&self) -> &str {
        match std::str::from_utf8(self.bytes.split_at(self.len).0) {
            Ok(value) => value,
            Err(_) => panic!("global State holder must be ASCII"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_preserve_existing_global_storage_locations() {
        // Retain the former derive algorithm as a compatibility oracle for
        // existing saved scoreboards, including truncation and Unicode edges.
        for logical in [
            "demo:counter",
            "state_test:global_state",
            "very_long_namespace:very_long_state_name",
            "demo:é/😀",
            "",
        ] {
            let clean: String = logical
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
                        ch
                    } else {
                        '_'
                    }
                })
                .take(24)
                .collect();
            let mut hash = 14_695_981_039_346_656_037u64;
            for byte in logical.bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(1_099_511_628_211);
            }
            let expected = format!("#sand_{clean}_{:08x}", hash as u32);
            assert_eq!(GlobalStateHolder::new(logical).as_str(), expected);
        }
        const HOLDER: GlobalStateHolder = GlobalStateHolder::new("demo:counter");
        const NAME: &str = HOLDER.as_str();
        assert_eq!(NAME, GlobalStateHolder::new("demo:counter").as_str());
    }
}
