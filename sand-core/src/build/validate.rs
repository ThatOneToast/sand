//! Build-time validation of a [`super::sand_build::SandBuild`] before it is
//! lowered to datapack resources.
//!
//! Combines structural checks (ranges, duplicate slots, non-empty layer
//! stacks) with a registry-level check against `minecraft:`-namespaced
//! biome references (see [`super::registry`]), generated per Minecraft
//! version from real data generator output (issue #356). This is not a
//! full audit against every world-generation registry (structures, noise
//! settings, etc.) or a real Minecraft server load — see
//! [`super::registry`]'s module docs and the PR that introduced this module
//! for the tracked follow-up.

use super::context::BuildContext;
use super::dimension::DimensionSlot;
use super::generator::{BiomeSource, Generator};
use super::registry::{CODEGEN_MINECRAFT_VERSION, is_known_vanilla_biome};
use super::sand_build::SandBuild;
use sand_macros::api;

/// Substring shared by every biome-registry diagnostic
/// ([`is_known_vanilla_biome`] failures), used to attach the version-context
/// note in [`validate_for_context`] only when it's actually relevant.
const BIOME_DIAGNOSTIC_MARKER: &str = "not a known vanilla biome";

/// A single validation failure, naming the offending builder call/field so
/// the diagnostic is actionable without re-reading the whole build script.
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::BuildDiagnostic",
    module = "sand::build",
    summary = "BuildDiagnostic reports one validation failure from SandBuild::validate, naming the offending builder call/field.",
    context = "Collected (not short-circuited) so a build script sees every problem in one run rather than fixing them one at a time.",
    minecraft = "Points at the specific dimension/border/generator call that produced invalid world data, before any file is written.",
    use_when = ["Reporting or displaying why a SandBuild failed validation"],
    avoid_when = ["Representing a runtime Minecraft error; this is build-time only"],
    fields(location = "Which builder call/field this points at, e.g. \"World::border\".", message = "Human-readable description of the problem."),
    example = "for d in built.validate().unwrap_err() { eprintln!(\"{d}\"); }"
)]
pub struct BuildDiagnostic {
    /// Which builder call/field this points at, e.g. `"World::border"` or
    /// `"Dimension[minecraft:overworld].generator"`.
    pub location: String,
    /// Human-readable description of the problem.
    pub message: String,
}

impl std::fmt::Display for BuildDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.location, self.message)
    }
}

/// Vanilla's maximum world border diameter, in blocks
/// (`2 * 29_999_984`, matching `WorldBorder#MAX_SIZE`).
const MAX_BORDER_DIAMETER: f64 = 59_999_968.0;

/// Validates every world/dimension resource a [`SandBuild`] would produce.
/// Returns `Ok(())` if every check passes, or every collected
/// [`BuildDiagnostic`] otherwise (not just the first).
///
/// Not part of the public `sand::build` facade — authors call
/// [`SandBuild::validate`] instead, which delegates here. Kept as a free
/// function (rather than inlined into the method) so `run_and_print` and
/// this module's own tests can call it directly without a `SandBuild`
/// receiver already in hand.
pub(crate) fn validate(build: &SandBuild) -> Result<(), Vec<BuildDiagnostic>> {
    let mut diagnostics = Vec::new();

    if let Some(world) = &build.world {
        if let Some(border) = &world.border {
            if !(border.diameter > 0.0 && border.diameter <= MAX_BORDER_DIAMETER) {
                diagnostics.push(BuildDiagnostic {
                    location: "World::border".to_string(),
                    message: format!(
                        "diameter {} is out of range (0, {MAX_BORDER_DIAMETER}]",
                        border.diameter
                    ),
                });
            }
            if border.damage_per_block < 0.0 {
                diagnostics.push(BuildDiagnostic {
                    location: "World::border".to_string(),
                    message: format!(
                        "damage_per_block {} must not be negative",
                        border.damage_per_block
                    ),
                });
            }
        }

        // Duplicate dimension slots (e.g. two Overworld entries).
        let mut seen = std::collections::HashSet::new();
        for dim in world.dimensions.entries() {
            let key = dim.resource_location();
            if !seen.insert(key.clone()) {
                diagnostics.push(BuildDiagnostic {
                    location: "World::dimensions".to_string(),
                    message: format!("dimension slot '{key}' is defined more than once"),
                });
            }

            match &dim.generator {
                Generator::Flat(flat) => {
                    if flat.layers.is_empty() {
                        diagnostics.push(BuildDiagnostic {
                            location: format!("Dimension[{key}].generator"),
                            message:
                                "FlatGenerator has no layers — a flat world needs at least one"
                                    .to_string(),
                        });
                    }
                    for layer in &flat.layers {
                        if layer.height == 0 {
                            diagnostics.push(BuildDiagnostic {
                                location: format!("Dimension[{key}].generator"),
                                message: format!(
                                    "FlatLayer for block '{}' has height 0",
                                    layer.block
                                ),
                            });
                        }
                    }
                    let total: u32 = flat.layers.iter().map(|l| l.height).sum();
                    if total > 384 {
                        diagnostics.push(BuildDiagnostic {
                            location: format!("Dimension[{key}].generator"),
                            message: format!(
                                "FlatGenerator layers total {total} blocks tall, \
                                 exceeding the maximum build height of 384"
                            ),
                        });
                    }
                    // Registry check (issue #317 §3.4, made version-aware by
                    // #356): a `minecraft:`-namespaced biome must be a real
                    // vanilla biome ID, generated from real data-generator
                    // output for the version sand-core was compiled against.
                    // Modded/datapack namespaces are accepted unconditionally
                    // — see `registry.rs`'s module docs.
                    if flat.biome.namespace() == "minecraft"
                        && !is_known_vanilla_biome(flat.biome.path())
                    {
                        diagnostics.push(BuildDiagnostic {
                            location: format!("Dimension[{key}].generator"),
                            message: format!(
                                "FlatGenerator::biome '{}' {BIOME_DIAGNOSTIC_MARKER}",
                                flat.biome
                            ),
                        });
                    }
                }
                Generator::Noise(noise) => {
                    if let BiomeSource::Fixed(biome) = &noise.biome_source
                        && biome.namespace() == "minecraft"
                        && !is_known_vanilla_biome(biome.path())
                    {
                        diagnostics.push(BuildDiagnostic {
                            location: format!("Dimension[{key}].generator"),
                            message: format!(
                                "NoiseGenerator::single_biome '{biome}' {BIOME_DIAGNOSTIC_MARKER}"
                            ),
                        });
                    }
                }
                Generator::Void | Generator::CustomReference(_) => {}
            }

            if matches!(dim.slot, DimensionSlot::Custom(_))
                && matches!(
                    dim.dimension_type,
                    super::dimension::DimensionType::Overworld
                        | super::dimension::DimensionType::OverworldCaves
                        | super::dimension::DimensionType::Nether
                        | super::dimension::DimensionType::End
                )
            {
                // Not an error — reusing a vanilla dimension type for a
                // custom dimension slot is valid and common — but flagged
                // as an informational note is out of scope for this
                // diagnostic type; left as a structural non-issue.
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Same checks as [`validate`], plus one extra: when the build reports a
/// biome-registry diagnostic *and* `ctx`'s target Minecraft version doesn't
/// match [`CODEGEN_MINECRAFT_VERSION`] (the exact version `sand-core`'s
/// registry data was actually generated for — `SAND_MC_VERSION` if set at
/// `sand-core` compile time, else `sand_version::DEFAULT_CODEGEN_VERSION`),
/// an additional [`BuildDiagnostic`] explains that the
/// biome check reflects the compiled-in registry snapshot, not necessarily
/// `ctx`'s requested version (issue #356 — every generated registry in this
/// crate, including `Block`/`Item`/`EntityType`, shares this same
/// single-version-per-compile limitation; this is not specific to biomes).
///
/// Deliberately does **not** add a diagnostic to an otherwise-passing build
/// just because the versions differ — only clarifies an existing failure,
/// since a version mismatch alone isn't necessarily wrong (most Sand
/// projects build against one pinned version their whole lifetime).
///
/// This is what [`super::run_and_print`] calls; [`SandBuild::validate`]
/// (the plain, context-free check) remains available for direct use.
pub(crate) fn validate_for_context(
    build: &SandBuild,
    ctx: &BuildContext,
) -> Result<(), Vec<BuildDiagnostic>> {
    let Err(mut diagnostics) = validate(build) else {
        return Ok(());
    };

    let compiled_version = CODEGEN_MINECRAFT_VERSION;
    let requested_version = ctx.mc_version();
    let has_biome_diagnostic = diagnostics
        .iter()
        .any(|d| d.message.contains(BIOME_DIAGNOSTIC_MARKER));
    if has_biome_diagnostic
        && requested_version != compiled_version
        && requested_version != "latest"
    {
        diagnostics.push(BuildDiagnostic {
            location: "BuildContext::mc_version".to_string(),
            message: format!(
                "the biome checks above were validated against Minecraft {compiled_version} \
                 (the version sand-core's registry data was generated for), not the requested \
                 target {requested_version} — a biome flagged as unknown may exist in \
                 {requested_version} but not in {compiled_version}, or vice versa"
            ),
        });
    }
    Err(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::dimension::{Dimension, DimensionType, Dimensions};
    use crate::build::generator::{FlatGenerator, FlatLayer};
    use crate::build::world::{World, WorldBorder};
    use sand_components::resource_location::ResourceLocation;

    #[test]
    fn valid_build_passes() {
        let build = SandBuild::new().world(
            World::new()
                .border(WorldBorder::diameter(1000.0))
                .dimensions(Dimensions::new().with(
                    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                        Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
                            ResourceLocation::new("minecraft", "stone").unwrap(),
                            64,
                        )])),
                    ),
                )),
        );
        assert!(validate(&build).is_ok());
    }

    #[test]
    fn oversized_border_is_rejected() {
        let build = SandBuild::new().world(World::new().border(WorldBorder::diameter(1e10)));
        let errs = validate(&build).unwrap_err();
        assert!(errs.iter().any(|d| d.location == "World::border"));
    }

    #[test]
    fn empty_flat_layers_are_rejected_with_a_pointed_location() {
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
                        .generator(Generator::Flat(FlatGenerator::new(vec![]))),
                ),
            ),
        );
        let errs = validate(&build).unwrap_err();
        assert!(
            errs.iter()
                .any(|d| d.location.contains("minecraft:overworld")
                    && d.message.contains("no layers"))
        );
    }

    #[test]
    fn duplicate_dimension_slots_are_rejected() {
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new()
                    .with(Dimension::new(
                        DimensionSlot::Overworld,
                        DimensionType::Overworld,
                    ))
                    .with(Dimension::new(
                        DimensionSlot::Overworld,
                        DimensionType::Overworld,
                    )),
            ),
        );
        let errs = validate(&build).unwrap_err();
        assert!(errs.iter().any(|d| d.message.contains("more than once")));
    }

    #[test]
    fn flat_layers_exceeding_build_height_are_rejected() {
        let build = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(
            Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
                    ResourceLocation::new("minecraft", "stone").unwrap(),
                    400,
                )])),
            ),
        )));
        let errs = validate(&build).unwrap_err();
        assert!(errs.iter().any(|d| d.message.contains("384")));
    }

    #[test]
    fn flat_generator_with_a_misspelled_vanilla_biome_is_rejected() {
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                        Generator::Flat(
                            FlatGenerator::new(vec![FlatLayer::new(
                                ResourceLocation::new("minecraft", "stone").unwrap(),
                                1,
                            )])
                            .biome(ResourceLocation::new("minecraft", "dessert").unwrap()),
                        ),
                    ),
                ),
            ),
        );
        let errs = validate(&build).unwrap_err();
        assert!(
            errs.iter().any(|d| d.message.contains("dessert")
                && d.message.contains("not a known vanilla biome"))
        );
    }

    #[test]
    fn flat_generator_with_a_real_vanilla_biome_passes() {
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                        Generator::Flat(
                            FlatGenerator::new(vec![FlatLayer::new(
                                ResourceLocation::new("minecraft", "sand").unwrap(),
                                1,
                            )])
                            .biome(ResourceLocation::new("minecraft", "desert").unwrap()),
                        ),
                    ),
                ),
            ),
        );
        assert!(validate(&build).is_ok());
    }

    #[test]
    fn flat_generator_with_a_custom_namespaced_biome_is_never_registry_checked() {
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                        Generator::Flat(
                            FlatGenerator::new(vec![FlatLayer::new(
                                ResourceLocation::new("minecraft", "stone").unwrap(),
                                1,
                            )])
                            .biome(ResourceLocation::new("my_pack", "mystic_forest").unwrap()),
                        ),
                    ),
                ),
            ),
        );
        assert!(validate(&build).is_ok());
    }

    #[test]
    fn noise_generator_single_biome_override_is_registry_checked() {
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                        Generator::Noise(
                            crate::build::generator::NoiseGenerator::vanilla(
                                crate::build::generator::VanillaNoiseSettings::Overworld,
                            )
                            .single_biome(
                                ResourceLocation::new("minecraft", "not_a_real_biome").unwrap(),
                            ),
                        ),
                    ),
                ),
            ),
        );
        let errs = validate(&build).unwrap_err();
        assert!(errs.iter().any(|d| d.message.contains("not_a_real_biome")));
    }

    fn biome_build_with_typo() -> SandBuild {
        SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                        Generator::Flat(
                            FlatGenerator::new(vec![FlatLayer::new(
                                ResourceLocation::new("minecraft", "stone").unwrap(),
                                1,
                            )])
                            .biome(ResourceLocation::new("minecraft", "dessert").unwrap()),
                        ),
                    ),
                ),
            ),
        )
    }

    #[test]
    fn validate_for_context_matches_plain_validate_when_versions_match() {
        use crate::build::context::BuildContext;
        use crate::build::profile::BuildProfile;

        let build = biome_build_with_typo();
        let ctx = BuildContext::new(BuildProfile::Dev).with_mc_version(CODEGEN_MINECRAFT_VERSION);
        let errs = validate_for_context(&build, &ctx).unwrap_err();
        // No extra version-context note: the requested version matches the
        // compiled registry, so there's nothing to clarify.
        assert!(
            !errs
                .iter()
                .any(|d| d.location == "BuildContext::mc_version")
        );
    }

    #[test]
    fn validate_for_context_adds_a_note_when_the_target_version_differs() {
        use crate::build::context::BuildContext;
        use crate::build::profile::BuildProfile;

        let build = biome_build_with_typo();
        let ctx = BuildContext::new(BuildProfile::Dev).with_mc_version("1.19.4");
        let errs = validate_for_context(&build, &ctx).unwrap_err();
        assert!(
            errs.iter()
                .any(|d| d.location == "BuildContext::mc_version" && d.message.contains("1.19.4"))
        );
        // The original biome diagnostic is still present alongside the note.
        assert!(
            errs.iter()
                .any(|d| d.message.contains(BIOME_DIAGNOSTIC_MARKER))
        );
    }

    #[test]
    fn validate_for_context_adds_no_note_for_a_passing_build_even_with_a_version_mismatch() {
        use crate::build::context::BuildContext;
        use crate::build::profile::BuildProfile;

        let build = SandBuild::new().world(World::new());
        let ctx = BuildContext::new(BuildProfile::Dev).with_mc_version("1.19.4");
        assert!(validate_for_context(&build, &ctx).is_ok());
    }

    #[test]
    fn validate_for_context_skips_the_note_for_a_non_biome_failure_even_with_a_version_mismatch() {
        use crate::build::context::BuildContext;
        use crate::build::profile::BuildProfile;

        let build = SandBuild::new().world(World::new().border(WorldBorder::diameter(1e10)));
        let ctx = BuildContext::new(BuildProfile::Dev).with_mc_version("1.19.4");
        let errs = validate_for_context(&build, &ctx).unwrap_err();
        assert!(
            !errs
                .iter()
                .any(|d| d.location == "BuildContext::mc_version")
        );
    }
}
