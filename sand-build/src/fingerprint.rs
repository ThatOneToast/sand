//! Shared deterministic content-hashing utility (issue #347).
//!
//! Sand's caches and content-addressed output (the Minecraft/codegen cache,
//! API-contract manifests, and the `dist/` output manifest) all need the
//! same property: given the same bytes, always produce the same fingerprint,
//! on any machine, in any worktree, regardless of absolute path. This module
//! is the single place that owns "how Sand hashes bytes for change
//! detection," so those subsystems don't grow independent, possibly
//! inconsistent hashing implementations.
//!
//! This is a change-detection fingerprint, not a security boundary. SHA-1's
//! cryptographic weaknesses (chosen-prefix collisions) are irrelevant to
//! detecting accidental content drift at Sand's scale, and reusing SHA-1 —
//! already a dependency for verifying Mojang's server jar download in
//! [`crate::download`] — avoids pulling a second hashing crate into the
//! build graph purely for a build-performance feature.

use sha1::{Digest, Sha1};

/// Hex-encoded content fingerprint of `bytes`.
///
/// Deterministic and path-independent: the result depends only on `bytes`
/// themselves, never on where they live on disk, so it is safe to compare
/// across worktrees, machines, and repeated builds.
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Combines already-computed fingerprints (or other stable string inputs)
/// into one fingerprint, in the order given. Used to build a composite
/// fingerprint from multiple structured inputs (e.g. schema version +
/// several file hashes) without inventing a new ad-hoc combination scheme
/// at each call site.
///
/// Each input is length-prefixed before hashing so `["ab", "c"]` and
/// `["a", "bc"]` never collide.
pub fn combine<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = Sha1::new();
    for part in parts {
        hasher.update(part.len().to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_bytes_is_deterministic() {
        assert_eq!(hash_bytes(b"hello"), hash_bytes(b"hello"));
    }

    #[test]
    fn hash_bytes_distinguishes_content() {
        assert_ne!(hash_bytes(b"hello"), hash_bytes(b"world"));
    }

    #[test]
    fn hash_bytes_matches_known_sha1() {
        // Known SHA-1 test vector, same algorithm sand-build already uses to
        // verify Mojang server jar downloads (see download.rs).
        assert_eq!(hash_bytes(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn combine_is_length_prefixed_to_avoid_boundary_collisions() {
        let a = combine(["ab", "c"]);
        let b = combine(["a", "bc"]);
        assert_ne!(
            a, b,
            "combine must not let concatenation boundaries collide"
        );
    }

    #[test]
    fn combine_is_order_sensitive() {
        let a = combine(["a", "b"]);
        let b = combine(["b", "a"]);
        assert_ne!(a, b);
    }

    #[test]
    fn combine_is_deterministic() {
        assert_eq!(combine(["x", "y", "z"]), combine(["x", "y", "z"]));
    }
}
