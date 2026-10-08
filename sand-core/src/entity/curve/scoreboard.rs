//! Shared scoreboard lowering for the canonical numeric expression engine.
//!
//! Resource ownership supplies deterministic scratch names and diagnostics;
//! lowering does not require an archetype definition. Gameplay actions and
//! derived properties must use this backend to share scale and rounding rules.

use super::{LoweredCurve, LoweredCurveOperation, OverflowPolicy, RoundingPolicy};
use crate::component::ComponentRecord;
use crate::entity::diagnostic::EntityDiagnostic;
use crate::resource_ref::FunctionId;
use sand_components::ResourceLocation;
use std::collections::BTreeSet;

pub(crate) struct RenderedCurve {
    pub(crate) commands: Vec<String>,
    pub(crate) records: Vec<crate::component::ComponentRecord>,
    pub(crate) functions: Vec<String>,
    pub(crate) objectives: Vec<String>,
}

pub(crate) fn render_lowered_curve(
    owner: &ResourceLocation,
    path: &str,
    lowered: &LoweredCurve,
) -> Result<RenderedCurve, EntityDiagnostic> {
    let mut commands = Vec::new();
    let mut records = Vec::new();
    let mut functions = Vec::new();
    let mut objectives = BTreeSet::new();
    for (index, operation) in lowered.operations().iter().enumerate() {
        match operation {
            LoweredCurveOperation::SetConstant { destination, value } => {
                commands.push(format!(
                    "scoreboard players set @s {destination} {}",
                    scoreboard_value(owner, lowered.target_objective(), value.units())?
                ));
            }
            LoweredCurveOperation::Copy {
                destination,
                source,
            } => commands.push(format!(
                "scoreboard players operation @s {destination} = @s {source}"
            )),
            LoweredCurveOperation::ScoreToFixed {
                destination,
                source,
                source_scale,
                target_scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(owner, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} = @s {source}"
                ));
                append_scale_conversion(
                    owner,
                    &mut objectives,
                    &mut commands,
                    destination,
                    *source_scale,
                    *target_scale,
                    *rounding,
                    index,
                )?;
            }
            LoweredCurveOperation::Add {
                destination,
                source,
                overflow,
            } => {
                require_scoreboard_overflow(owner, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} += @s {source}"
                ));
            }
            LoweredCurveOperation::MultiplyFixed {
                destination,
                factor,
                scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(owner, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} *= @s {factor}"
                ));
                append_scaled_division(
                    owner,
                    &mut objectives,
                    &mut commands,
                    destination,
                    *scale,
                    *rounding,
                    index,
                )?;
            }
            LoweredCurveOperation::RatioFixed {
                destination,
                numerator,
                denominator,
                scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(owner, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation @s {destination} = @s {numerator}"
                ));
                let constant = constant_objective(owner, "curve_scale", *scale)?;
                objectives.insert(constant.clone());
                commands.push(format!("scoreboard players set #value {constant} {scale}"));
                commands.push(format!(
                    "scoreboard players operation @s {destination} *= #value {constant}"
                ));
                append_score_division(
                    owner,
                    &mut objectives,
                    &mut commands,
                    destination,
                    denominator,
                    *rounding,
                    index,
                )?;
            }
            LoweredCurveOperation::Clamp {
                destination,
                minimum,
                maximum,
            } => {
                let minimum = scoreboard_value(owner, lowered.target_objective(), minimum.units())?;
                let maximum = scoreboard_value(owner, lowered.target_objective(), maximum.units())?;
                commands.push(format!(
                    "execute if score @s {destination} matches ..{minimum} run scoreboard players set @s {destination} {minimum}"
                ));
                commands.push(format!(
                    "execute if score @s {destination} matches {maximum}.. run scoreboard players set @s {destination} {maximum}"
                ));
            }
            LoweredCurveOperation::SelectStepped {
                destination,
                input,
                bands,
                below,
            } => {
                let mut boundaries = Vec::new();
                let mut values = vec![scoreboard_value(
                    owner,
                    lowered.target_objective(),
                    below.units(),
                )?];
                for (minimum, value) in bands {
                    boundaries.push(scoreboard_value(
                        owner,
                        lowered.target_objective(),
                        minimum.units(),
                    )?);
                    values.push(scoreboard_value(
                        owner,
                        lowered.target_objective(),
                        value.units(),
                    )?);
                }
                let base = format!("{path}/select_{index}");
                build_threshold_tree(
                    owner,
                    &base,
                    input,
                    destination,
                    &boundaries,
                    &values,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", owner.namespace()));
            }
            LoweredCurveOperation::SelectPiecewise {
                destination,
                input,
                branches,
                fallback,
            } => {
                let mut boundaries = Vec::new();
                let mut values = Vec::new();
                for (maximum, source) in branches {
                    boundaries.push(scoreboard_value(
                        owner,
                        lowered.target_objective(),
                        maximum.units(),
                    )?);
                    values.push(source.clone());
                }
                values.push(fallback.clone());
                let base = format!("{path}/piecewise_{index}");
                build_piecewise_tree(
                    owner,
                    &base,
                    input,
                    destination,
                    &boundaries,
                    &values,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", owner.namespace()));
            }
            LoweredCurveOperation::LookupTable {
                destination,
                input,
                entries,
                fallback,
            } => {
                let mut encoded = Vec::new();
                for (key, value) in entries {
                    encoded.push((
                        scoreboard_value(owner, lowered.target_objective(), *key)?,
                        scoreboard_value(owner, lowered.target_objective(), value.units())?,
                    ));
                }
                let fallback =
                    scoreboard_value(owner, lowered.target_objective(), fallback.units())?;
                let base = format!("{path}/lookup_{index}");
                build_exact_tree(
                    owner,
                    &base,
                    input,
                    destination,
                    &encoded,
                    fallback,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", owner.namespace()));
            }
            LoweredCurveOperation::SelectEnum {
                destination,
                input,
                entries,
                fallback,
            } => {
                let mut encoded = Vec::new();
                for (encoding, value) in entries {
                    encoded.push((
                        *encoding,
                        scoreboard_value(owner, lowered.target_objective(), value.units())?,
                    ));
                }
                let fallback =
                    scoreboard_value(owner, lowered.target_objective(), fallback.units())?;
                let base = format!("{path}/enum_{index}");
                build_exact_tree(
                    owner,
                    &base,
                    input,
                    destination,
                    &encoded,
                    fallback,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", owner.namespace()));
            }
            LoweredCurveOperation::SelectFlag {
                destination,
                input,
                disabled,
                enabled,
            } => {
                commands.push(format!(
                    "execute if score @s {input} matches 0 run scoreboard players set @s {destination} {}",
                    scoreboard_value(owner, lowered.target_objective(), disabled.units())?
                ));
                commands.push(format!(
                    "execute unless score @s {input} matches 0 run scoreboard players set @s {destination} {}",
                    scoreboard_value(owner, lowered.target_objective(), enabled.units())?
                ));
            }
            LoweredCurveOperation::Custom {
                destination,
                callback,
                inputs: _,
            } => {
                let callback = callback.parse::<FunctionId>().map_err(|error| {
                    EntityDiagnostic::InvalidRawExtension {
                        archetype: owner.to_string(),
                        extension: lowered.target_objective().into(),
                        detail: format!(
                            "custom curve callback must be a canonical function ID: {error}"
                        ),
                    }
                })?;
                commands.push(format!(
                    "execute store result score @s {destination} run function {callback}"
                ));
            }
        }
    }
    Ok(RenderedCurve {
        commands,
        records,
        functions,
        objectives: objectives.into_iter().collect(),
    })
}

#[allow(clippy::too_many_arguments)]
fn build_exact_tree(
    owner: &ResourceLocation,
    path: &str,
    input: &str,
    destination: &str,
    entries: &[(i32, i32)],
    fallback: i32,
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    functions.push(path.to_owned());
    let commands = if entries.len() <= 1 {
        let mut commands = vec![format!(
            "scoreboard players set @s {destination} {fallback}"
        )];
        if let Some((key, value)) = entries.first() {
            commands.push(format!(
                "execute if score @s {input} matches {key} run scoreboard players set @s {destination} {value}"
            ));
        }
        commands
    } else {
        let middle = entries.len() / 2;
        let (key, value) = entries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_exact_tree(
            owner,
            &left,
            input,
            destination,
            &entries[..middle],
            fallback,
            records,
            functions,
        );
        build_exact_tree(
            owner,
            &right,
            input,
            destination,
            &entries[middle + 1..],
            fallback,
            records,
            functions,
        );
        let mut commands = Vec::new();
        if key > i32::MIN {
            commands.push(format!(
                "execute if score @s {input} matches ..{} run function {}:{left}",
                key - 1,
                owner.namespace()
            ));
        }
        commands.push(format!(
            "execute if score @s {input} matches {key} run scoreboard players set @s {destination} {value}"
        ));
        if key < i32::MAX {
            commands.push(format!(
                "execute if score @s {input} matches {}.. run function {}:{right}",
                key + 1,
                owner.namespace()
            ));
        }
        commands
    };
    records.push(ComponentRecord::function(owner.namespace(), path, commands));
}

#[allow(clippy::too_many_arguments)]
fn build_threshold_tree(
    owner: &ResourceLocation,
    path: &str,
    input: &str,
    destination: &str,
    boundaries: &[i32],
    values: &[i32],
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    functions.push(path.to_owned());
    let commands = if boundaries.is_empty() {
        vec![format!(
            "scoreboard players set @s {destination} {}",
            values[0]
        )]
    } else {
        let middle = boundaries.len() / 2;
        let boundary = boundaries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_threshold_tree(
            owner,
            &left,
            input,
            destination,
            &boundaries[..middle],
            &values[..middle + 1],
            records,
            functions,
        );
        build_threshold_tree(
            owner,
            &right,
            input,
            destination,
            &boundaries[middle + 1..],
            &values[middle + 1..],
            records,
            functions,
        );
        let mut commands = Vec::new();
        if boundary > i32::MIN {
            commands.push(format!(
                "execute if score @s {input} matches ..{} run function {}:{left}",
                boundary - 1,
                owner.namespace()
            ));
        }
        commands.push(format!(
            "execute if score @s {input} matches {boundary}.. run function {}:{right}",
            owner.namespace()
        ));
        commands
    };
    records.push(ComponentRecord::function(owner.namespace(), path, commands));
}

#[allow(clippy::too_many_arguments)]
fn build_piecewise_tree(
    owner: &ResourceLocation,
    path: &str,
    input: &str,
    destination: &str,
    boundaries: &[i32],
    values: &[String],
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    functions.push(path.to_owned());
    let commands = if boundaries.is_empty() {
        vec![format!(
            "scoreboard players operation @s {destination} = @s {}",
            values[0]
        )]
    } else {
        let middle = boundaries.len() / 2;
        let boundary = boundaries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_piecewise_tree(
            owner,
            &left,
            input,
            destination,
            &boundaries[..middle],
            &values[..middle + 1],
            records,
            functions,
        );
        build_piecewise_tree(
            owner,
            &right,
            input,
            destination,
            &boundaries[middle + 1..],
            &values[middle + 1..],
            records,
            functions,
        );
        let mut commands = vec![format!(
            "execute if score @s {input} matches ..{boundary} run function {}:{left}",
            owner.namespace()
        )];
        if boundary < i32::MAX {
            commands.push(format!(
                "execute if score @s {input} matches {}.. run function {}:{right}",
                boundary + 1,
                owner.namespace()
            ));
        }
        commands
    };
    records.push(ComponentRecord::function(owner.namespace(), path, commands));
}

fn scoreboard_value(
    owner: &ResourceLocation,
    derivation: &str,
    value: i64,
) -> Result<i32, EntityDiagnostic> {
    i32::try_from(value).map_err(|_| EntityDiagnostic::FixedPointOverflow {
        archetype: owner.to_string(),
        derivation: derivation.into(),
        detail: format!("fixed-point unit `{value}` does not fit a Minecraft score"),
    })
}

fn require_scoreboard_overflow(
    owner: &ResourceLocation,
    lowered: &LoweredCurve,
    overflow: OverflowPolicy,
) -> Result<(), EntityDiagnostic> {
    if overflow == OverflowPolicy::Error {
        Ok(())
    } else {
        Err(EntityDiagnostic::UnsupportedProfile {
            archetype: owner.to_string(),
            property: lowered.target_objective().into(),
            profile: "vanilla-scoreboard".into(),
            reason: "runtime saturating arithmetic is not reliably representable".into(),
        })
    }
}

fn append_scaled_division(
    owner: &ResourceLocation,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    divisor: i64,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    let objective = constant_objective(owner, &format!("curve_divisor_{index}"), divisor)?;
    objectives.insert(objective.clone());
    commands.push(format!(
        "scoreboard players set #value {objective} {divisor}"
    ));
    append_score_division(
        owner,
        objectives,
        commands,
        destination,
        &format!("#value {objective}"),
        rounding,
        index,
    )
}

#[allow(clippy::too_many_arguments)] // Lowering keeps each scoreboard operand explicit.
pub(crate) fn append_scale_conversion(
    owner: &ResourceLocation,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    source_scale: i64,
    target_scale: i64,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    debug_assert!(source_scale > 0 && target_scale > 0);
    let common = gcd(source_scale, target_scale);
    let multiplier = target_scale / common;
    let divisor = source_scale / common;

    if multiplier != 1 {
        let objective = constant_objective(
            owner,
            &format!("curve_rescale_multiplier_{index}"),
            multiplier,
        )?;
        objectives.insert(objective.clone());
        commands.push(format!(
            "scoreboard players set #value {objective} {multiplier}"
        ));
        commands.push(format!(
            "scoreboard players operation @s {destination} *= #value {objective}"
        ));
    }
    if divisor != 1 {
        append_scaled_division(
            owner,
            objectives,
            commands,
            destination,
            divisor,
            rounding,
            index,
        )?;
    }
    Ok(())
}

fn gcd(mut left: i64, mut right: i64) -> i64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.abs()
}

fn append_score_division(
    owner: &ResourceLocation,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    divisor: &str,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    let divisor = if divisor.starts_with("#value ") {
        divisor.to_owned()
    } else {
        format!("@s {divisor}")
    };
    if divisor.starts_with("@s ") {
        commands.push(format!(
            "execute unless score {divisor} matches 1.. run return fail"
        ));
    }
    let scratch = |role: &str| {
        sand_commands::ObjectiveName::logical(format!(
            "{}.division.{index}.{destination}.{role}",
            owner
        ))
        .as_str()
        .to_string()
    };
    let original = scratch("original");
    let product = scratch("product");
    let remainder = scratch("remainder");
    objectives.extend([original.clone(), product.clone(), remainder.clone()]);
    commands.push(format!(
        "scoreboard players operation @s {original} = @s {destination}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {destination} /= {divisor}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {product} = @s {destination}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {product} *= {divisor}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {remainder} = @s {original}"
    ));
    commands.push(format!(
        "scoreboard players operation @s {remainder} -= @s {product}"
    ));
    match rounding {
        RoundingPolicy::Floor => {}
        RoundingPolicy::TowardZero => commands.push(format!(
            "execute if score @s {original} matches ..-1 if score @s {remainder} matches 1.. run scoreboard players add @s {destination} 1"
        )),
        RoundingPolicy::Ceiling => commands.push(format!(
            "execute if score @s {remainder} matches 1.. run scoreboard players add @s {destination} 1"
        )),
        RoundingPolicy::NearestTiesAwayFromZero | RoundingPolicy::NearestTiesToEven => {
            let half = scratch("half_divisor");
            let divisor_parity = scratch("divisor_parity");
            let two = constant_objective(owner, "curve_two", 2)?;
            objectives.extend([half.clone(), divisor_parity.clone(), two.clone()]);
            commands.push(format!("scoreboard players set #value {two} 2"));
            commands.push(format!(
                "scoreboard players operation @s {half} = {divisor}"
            ));
            commands.push(format!(
                "scoreboard players operation @s {half} /= #value {two}"
            ));
            commands.push(format!(
                "scoreboard players operation @s {divisor_parity} = {divisor}"
            ));
            commands.push(format!(
                "scoreboard players operation @s {divisor_parity} %= #value {two}"
            ));
            commands.push(format!(
                "execute if score @s {remainder} > @s {half} run scoreboard players add @s {destination} 1"
            ));
            if rounding == RoundingPolicy::NearestTiesAwayFromZero {
                commands.push(format!(
                    "execute if score @s {original} matches 0.. if score @s {remainder} = @s {half} if score @s {divisor_parity} matches 0 run scoreboard players add @s {destination} 1"
                ));
            } else {
                let parity = scratch("parity");
                objectives.insert(parity.clone());
                commands.push(format!(
                    "scoreboard players operation @s {parity} = @s {destination}"
                ));
                commands.push(format!(
                    "scoreboard players operation @s {parity} %= #value {two}"
                ));
                commands.push(format!(
                    "execute if score @s {remainder} = @s {half} if score @s {divisor_parity} matches 0 unless score @s {parity} matches 0 run scoreboard players add @s {destination} 1"
                ));
            }
        }
    }
    Ok(())
}

fn constant_objective(
    owner: &ResourceLocation,
    role: &str,
    value: i64,
) -> Result<String, EntityDiagnostic> {
    scoreboard_value(owner, role, value)?;
    Ok(
        sand_commands::ObjectiveName::logical(format!("{}.{}.{}", owner, role, value))
            .as_str()
            .to_string(),
    )
}
