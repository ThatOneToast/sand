//! Protocol limits, applied to decoded and directly constructed programs.
pub(super) const DOCUMENT_BYTES: usize = 1024 * 1024;
pub(super) const INPUT_BYTES: usize = 8 * DOCUMENT_BYTES;
pub(super) const MODULES: usize = 64;
pub(super) const JSON_DEPTH: usize = 64;
pub(super) const OP_DEPTH: usize = 32;
pub(super) const DECLARATIONS: usize = 10_000;
pub(super) const OPERATIONS: usize = 100_000;
pub(super) const RESOURCES: usize = 10_000;
pub(super) const OUTPUT_BYTES: usize = 32 * DOCUMENT_BYTES;
// Conservative relative-path budget leaves room for a publication root on
// platforms with a 1024-byte total path limit. Hosts still validate destinations.
pub(super) const RESOURCE_PATH_BYTES: usize = 768;
pub(super) const RESOURCE_SEGMENT_BYTES: usize = 255;
