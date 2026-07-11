#![forbid(unsafe_code)]

//! Shared Minecraft version anchors used by Sand crates that cannot depend on
//! `sand-core` without creating build-time dependency cycles.

/// The latest Minecraft version Sand's bundled version table was verified against.
///
/// This is the **export/profile anchor**: it is the version
/// `VersionProfile::resolve("latest")` resolves to, and it drives pack
/// metadata (`pack.mcmeta`) and version-sensitive feature flags. It is *not*
/// necessarily the same version used to run `sand-build` codegen for local
/// `sand-core` builds/tests — see [`DEFAULT_CODEGEN_VERSION`].
pub const LATEST_KNOWN: &str = "26.2";

/// The default Minecraft version `sand-core/build.rs` uses to run `sand-build`
/// codegen when `SAND_MC_VERSION` is unset.
///
/// This is the **codegen anchor**, kept deliberately separate from
/// [`LATEST_KNOWN`] so the two concerns do not get conflated:
///
/// - [`LATEST_KNOWN`] answers "which version profile do exported packs and
///   feature flags target by default?"
/// - `DEFAULT_CODEGEN_VERSION` answers "which verified, codegen-available
///   Minecraft server jar should local `cargo test -p sand-core --lib` use to
///   generate command/registry/block-state Rust APIs?"
///
/// The value MUST be a verified, codegen-available version: `sand-build` must
/// be able to download/cache its server jar and run the Minecraft data
/// generator to produce non-placeholder `commands.rs`, `registries.rs`, and
/// `block_states.rs`. It need not equal [`LATEST_KNOWN`]; when they differ,
/// [`LATEST_KNOWN`] is the export/profile target and `DEFAULT_CODEGEN_VERSION`
/// is the build-time codegen target.
///
/// If codegen fails, `sand-core/build.rs` fails immediately with an actionable
/// message (no silent placeholders). Set `SAND_ALLOW_PLACEHOLDER_CODEGEN=1` to
/// explicitly opt into placeholder files that compile but fail
/// `generated_api_health`. Changing this value requires confirming the new
/// target is codegen-available in the default local and CI environments.
pub const DEFAULT_CODEGEN_VERSION: &str = "1.21.11";
