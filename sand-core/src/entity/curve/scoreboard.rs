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

/// Validated execution storage for one numeric lowering operation.
///
/// Entity derivations use the current executor. Server-context gameplay
/// actions can use a single fake-player holder without inventing an entity.
#[derive(Clone, Copy)]
pub(crate) struct NumericContext<'a> {
    owner: &'a ResourceLocation,
    holder: &'a sand_commands::ScoreHolder,
}

impl<'a> NumericContext<'a> {
    pub(crate) fn new(
        owner: &'a ResourceLocation,
        holder: &'a sand_commands::ScoreHolder,
    ) -> Result<Self, EntityDiagnostic> {
        holder
            .validate_single(&sand_commands::CommandProfile::unprofiled())
            .map_err(|error| EntityDiagnostic::InvalidRawExtension {
                archetype: owner.to_string(),
                extension: "numeric execution context".into(),
                detail: error.to_string(),
            })?;
        Ok(Self { owner, holder })
    }
}

pub(crate) struct RenderedCurve {
    pub(crate) commands: Vec<String>,
    pub(crate) records: Vec<crate::component::ComponentRecord>,
    pub(crate) functions: Vec<String>,
    pub(crate) objectives: Vec<String>,
}

pub(crate) fn render_lowered_curve(
    context: NumericContext<'_>,
    path: &str,
    lowered: &LoweredCurve,
) -> Result<RenderedCurve, EntityDiagnostic> {
    let holder = context.holder;
    let mut commands = Vec::new();
    let mut records = Vec::new();
    let mut functions = Vec::new();
    let mut objectives = BTreeSet::new();
    for (index, operation) in lowered.operations().iter().enumerate() {
        match operation {
            LoweredCurveOperation::SetConstant { destination, value } => {
                commands.push(format!(
                    "scoreboard players set {holder} {destination} {}",
                    scoreboard_value(context, lowered.target_objective(), value.units())?
                ));
            }
            LoweredCurveOperation::Copy {
                destination,
                source,
            } => commands.push(format!(
                "scoreboard players operation {holder} {destination} = {holder} {source}"
            )),
            LoweredCurveOperation::ScoreToFixed {
                destination,
                source,
                source_holder,
                source_scale,
                target_scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(context, lowered, *overflow)?;
                let source_holder = source_holder.clone().unwrap_or_else(|| holder.to_string());
                // A missing optional State score makes the copy fail without
                // updating its target. Clear persistent scratch first so a
                // later evaluation cannot inherit a previous source value.
                commands.push(format!("scoreboard players set {holder} {destination} 0"));
                commands.push(format!(
                    "scoreboard players operation {holder} {destination} = {source_holder} {source}"
                ));
                append_scale_conversion(
                    context,
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
                require_scoreboard_overflow(context, lowered, *overflow)?;
                append_checked_arithmetic(
                    context,
                    &mut objectives,
                    &mut commands,
                    destination,
                    &format!("{holder} {source}"),
                    false,
                    index,
                );
            }
            LoweredCurveOperation::MultiplyFixed {
                destination,
                factor,
                scale,
                rounding,
                overflow,
            } => {
                require_scoreboard_overflow(context, lowered, *overflow)?;
                append_checked_arithmetic(
                    context,
                    &mut objectives,
                    &mut commands,
                    destination,
                    &format!("{holder} {factor}"),
                    true,
                    index,
                );
                append_scaled_division(
                    context,
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
                require_scoreboard_overflow(context, lowered, *overflow)?;
                commands.push(format!(
                    "scoreboard players operation {holder} {destination} = {holder} {numerator}"
                ));
                let constant = constant_objective(context, "curve_scale", *scale)?;
                objectives.insert(constant.clone());
                commands.push(format!("scoreboard players set #value {constant} {scale}"));
                append_checked_arithmetic(
                    context,
                    &mut objectives,
                    &mut commands,
                    destination,
                    &format!("#value {constant}"),
                    true,
                    index,
                );
                append_score_division(
                    context,
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
                let minimum =
                    scoreboard_value(context, lowered.target_objective(), minimum.units())?;
                let maximum =
                    scoreboard_value(context, lowered.target_objective(), maximum.units())?;
                commands.push(format!(
                    "execute if score {holder} {destination} matches ..{minimum} run scoreboard players set {holder} {destination} {minimum}"
                ));
                commands.push(format!(
                    "execute if score {holder} {destination} matches {maximum}.. run scoreboard players set {holder} {destination} {maximum}"
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
                    context,
                    lowered.target_objective(),
                    below.units(),
                )?];
                for (minimum, value) in bands {
                    boundaries.push(scoreboard_value(
                        context,
                        lowered.target_objective(),
                        minimum.units(),
                    )?);
                    values.push(scoreboard_value(
                        context,
                        lowered.target_objective(),
                        value.units(),
                    )?);
                }
                let base = format!("{path}/select_{index}");
                build_threshold_tree(
                    context,
                    &base,
                    input,
                    destination,
                    &boundaries,
                    &values,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", context.owner.namespace()));
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
                        context,
                        lowered.target_objective(),
                        maximum.units(),
                    )?);
                    values.push(source.clone());
                }
                values.push(fallback.clone());
                let base = format!("{path}/piecewise_{index}");
                build_piecewise_tree(
                    context,
                    &base,
                    input,
                    destination,
                    &boundaries,
                    &values,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", context.owner.namespace()));
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
                        scoreboard_value(context, lowered.target_objective(), *key)?,
                        scoreboard_value(context, lowered.target_objective(), value.units())?,
                    ));
                }
                let fallback =
                    scoreboard_value(context, lowered.target_objective(), fallback.units())?;
                let base = format!("{path}/lookup_{index}");
                build_exact_tree(
                    context,
                    &base,
                    input,
                    destination,
                    &encoded,
                    fallback,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", context.owner.namespace()));
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
                        scoreboard_value(context, lowered.target_objective(), value.units())?,
                    ));
                }
                let fallback =
                    scoreboard_value(context, lowered.target_objective(), fallback.units())?;
                let base = format!("{path}/enum_{index}");
                build_exact_tree(
                    context,
                    &base,
                    input,
                    destination,
                    &encoded,
                    fallback,
                    &mut records,
                    &mut functions,
                );
                commands.push(format!("function {}:{base}", context.owner.namespace()));
            }
            LoweredCurveOperation::SelectFlag {
                destination,
                input,
                disabled,
                enabled,
            } => {
                commands.push(format!(
                    "execute if score {holder} {input} matches 0 run scoreboard players set {holder} {destination} {}",
                    scoreboard_value(context, lowered.target_objective(), disabled.units())?
                ));
                commands.push(format!(
                    "execute unless score {holder} {input} matches 0 run scoreboard players set {holder} {destination} {}",
                    scoreboard_value(context, lowered.target_objective(), enabled.units())?
                ));
            }
            LoweredCurveOperation::Custom {
                destination,
                callback,
                inputs: _,
            } => {
                let callback = callback.parse::<FunctionId>().map_err(|error| {
                    EntityDiagnostic::InvalidRawExtension {
                        archetype: context.owner.to_string(),
                        extension: lowered.target_objective().into(),
                        detail: format!(
                            "custom curve callback must be a canonical function ID: {error}"
                        ),
                    }
                })?;
                commands.push(format!(
                    "execute store result score {holder} {destination} run function {callback}"
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
    context: NumericContext<'_>,
    path: &str,
    input: &str,
    destination: &str,
    entries: &[(i32, i32)],
    fallback: i32,
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    let holder = context.holder;
    functions.push(path.to_owned());
    let commands = if entries.len() <= 1 {
        let mut commands = vec![format!(
            "scoreboard players set {holder} {destination} {fallback}"
        )];
        if let Some((key, value)) = entries.first() {
            commands.push(format!(
                "execute if score {holder} {input} matches {key} run scoreboard players set {holder} {destination} {value}"
            ));
        }
        commands
    } else {
        let middle = entries.len() / 2;
        let (key, value) = entries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_exact_tree(
            context,
            &left,
            input,
            destination,
            &entries[..middle],
            fallback,
            records,
            functions,
        );
        build_exact_tree(
            context,
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
                "execute if score {holder} {input} matches ..{} run function {}:{left}",
                key - 1,
                context.owner.namespace()
            ));
        }
        commands.push(format!(
            "execute if score {holder} {input} matches {key} run scoreboard players set {holder} {destination} {value}"
        ));
        if key < i32::MAX {
            commands.push(format!(
                "execute if score {holder} {input} matches {}.. run function {}:{right}",
                key + 1,
                context.owner.namespace()
            ));
        }
        commands
    };
    records.push(ComponentRecord::function(
        context.owner.namespace(),
        path,
        commands,
    ));
}

#[allow(clippy::too_many_arguments)]
fn build_threshold_tree(
    context: NumericContext<'_>,
    path: &str,
    input: &str,
    destination: &str,
    boundaries: &[i32],
    values: &[i32],
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    let holder = context.holder;
    functions.push(path.to_owned());
    let commands = if boundaries.is_empty() {
        vec![format!(
            "scoreboard players set {holder} {destination} {}",
            values[0]
        )]
    } else {
        let middle = boundaries.len() / 2;
        let boundary = boundaries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_threshold_tree(
            context,
            &left,
            input,
            destination,
            &boundaries[..middle],
            &values[..middle + 1],
            records,
            functions,
        );
        build_threshold_tree(
            context,
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
                "execute if score {holder} {input} matches ..{} run function {}:{left}",
                boundary - 1,
                context.owner.namespace()
            ));
        }
        commands.push(format!(
            "execute if score {holder} {input} matches {boundary}.. run function {}:{right}",
            context.owner.namespace()
        ));
        commands
    };
    records.push(ComponentRecord::function(
        context.owner.namespace(),
        path,
        commands,
    ));
}

#[allow(clippy::too_many_arguments)]
fn build_piecewise_tree(
    context: NumericContext<'_>,
    path: &str,
    input: &str,
    destination: &str,
    boundaries: &[i32],
    values: &[String],
    records: &mut Vec<crate::component::ComponentRecord>,
    functions: &mut Vec<String>,
) {
    let holder = context.holder;
    functions.push(path.to_owned());
    let commands = if boundaries.is_empty() {
        vec![format!(
            "scoreboard players operation {holder} {destination} = {holder} {}",
            values[0]
        )]
    } else {
        let middle = boundaries.len() / 2;
        let boundary = boundaries[middle];
        let left = format!("{path}/l");
        let right = format!("{path}/r");
        build_piecewise_tree(
            context,
            &left,
            input,
            destination,
            &boundaries[..middle],
            &values[..middle + 1],
            records,
            functions,
        );
        build_piecewise_tree(
            context,
            &right,
            input,
            destination,
            &boundaries[middle + 1..],
            &values[middle + 1..],
            records,
            functions,
        );
        let mut commands = vec![format!(
            "execute if score {holder} {input} matches ..{boundary} run function {}:{left}",
            context.owner.namespace()
        )];
        if boundary < i32::MAX {
            commands.push(format!(
                "execute if score {holder} {input} matches {}.. run function {}:{right}",
                boundary + 1,
                context.owner.namespace()
            ));
        }
        commands
    };
    records.push(ComponentRecord::function(
        context.owner.namespace(),
        path,
        commands,
    ));
}

fn scoreboard_value(
    context: NumericContext<'_>,
    derivation: &str,
    value: i64,
) -> Result<i32, EntityDiagnostic> {
    i32::try_from(value).map_err(|_| EntityDiagnostic::FixedPointOverflow {
        archetype: context.owner.to_string(),
        derivation: derivation.into(),
        detail: format!("fixed-point unit `{value}` does not fit a Minecraft score"),
    })
}

fn require_scoreboard_overflow(
    context: NumericContext<'_>,
    lowered: &LoweredCurve,
    overflow: OverflowPolicy,
) -> Result<(), EntityDiagnostic> {
    if overflow == OverflowPolicy::Error {
        Ok(())
    } else {
        Err(EntityDiagnostic::UnsupportedProfile {
            archetype: context.owner.to_string(),
            property: lowered.target_objective().into(),
            profile: "vanilla-scoreboard".into(),
            reason: "runtime saturating arithmetic is not reliably representable".into(),
        })
    }
}

/// Check Minecraft's wrapping arithmetic before committing the operation.
/// Multiplication is reversible by division exactly when its product fits,
/// apart from MIN * -1 (whose JVM division also wraps), guarded explicitly.
fn append_checked_arithmetic(
    context: NumericContext<'_>,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    operand: &str,
    multiply: bool,
    index: usize,
) {
    let holder = context.holder;
    let scratch = |role| {
        sand_commands::ObjectiveName::logical(format!(
            "{}.checked.{index}.{destination}.{role}",
            context.owner
        ))
        .to_string()
    };
    let left = scratch("left");
    let right = scratch("right");
    let result = scratch("result");
    objectives.extend([left.clone(), right.clone(), result.clone()]);
    commands.push(format!(
        "scoreboard players operation {holder} {left} = {holder} {destination}"
    ));
    commands.push(format!(
        "scoreboard players operation {holder} {right} = {operand}"
    ));
    commands.push(format!(
        "scoreboard players operation {holder} {result} = {holder} {left}"
    ));
    let operation = if multiply { "*=" } else { "+=" };
    commands.push(format!(
        "scoreboard players operation {holder} {result} {operation} {holder} {right}"
    ));
    if multiply {
        let quotient = scratch("quotient");
        objectives.insert(quotient.clone());
        commands.push(format!("execute if score {holder} {left} matches -2147483648 if score {holder} {right} matches -1 run return fail"));
        commands.push(format!(
            "scoreboard players operation {holder} {quotient} = {holder} {result}"
        ));
        commands.push(format!("execute unless score {holder} {right} matches 0 run scoreboard players operation {holder} {quotient} /= {holder} {right}"));
        commands.push(format!("execute unless score {holder} {right} matches 0 unless score {holder} {quotient} = {holder} {left} run return fail"));
    } else {
        commands.push(format!("execute if score {holder} {left} matches 0.. if score {holder} {right} matches 0.. if score {holder} {result} matches ..-1 run return fail"));
        commands.push(format!("execute if score {holder} {left} matches ..-1 if score {holder} {right} matches ..-1 if score {holder} {result} matches 0.. run return fail"));
    }
    commands.push(format!(
        "scoreboard players operation {holder} {destination} = {holder} {result}"
    ));
}

fn append_scaled_division(
    context: NumericContext<'_>,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    divisor: i64,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    let objective = constant_objective(context, &format!("curve_divisor_{index}"), divisor)?;
    objectives.insert(objective.clone());
    commands.push(format!(
        "scoreboard players set #value {objective} {divisor}"
    ));
    append_score_division(
        context,
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
    context: NumericContext<'_>,
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
            context,
            &format!("curve_rescale_multiplier_{index}"),
            multiplier,
        )?;
        objectives.insert(objective.clone());
        commands.push(format!(
            "scoreboard players set #value {objective} {multiplier}"
        ));
        append_checked_arithmetic(
            context,
            objectives,
            commands,
            destination,
            &format!("#value {objective}"),
            true,
            index,
        );
    }
    if divisor != 1 {
        append_scaled_division(
            context,
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
    context: NumericContext<'_>,
    objectives: &mut BTreeSet<String>,
    commands: &mut Vec<String>,
    destination: &str,
    divisor: &str,
    rounding: RoundingPolicy,
    index: usize,
) -> Result<(), EntityDiagnostic> {
    let holder = context.holder;
    let dynamic_divisor = !divisor.starts_with("#value ");
    let divisor = if divisor.starts_with("#value ") {
        divisor.to_owned()
    } else {
        format!("{holder} {divisor}")
    };
    if dynamic_divisor {
        commands.push(format!(
            "execute unless score {divisor} matches 1.. run return fail"
        ));
    }
    let scratch = |role: &str| {
        sand_commands::ObjectiveName::logical(format!(
            "{}.division.{index}.{destination}.{role}",
            context.owner
        ))
        .as_str()
        .to_string()
    };
    let original = scratch("original");
    let remainder = scratch("remainder");
    objectives.extend([original.clone(), remainder.clone()]);
    commands.push(format!(
        "scoreboard players operation {holder} {original} = {holder} {destination}"
    ));
    commands.push(format!(
        "scoreboard players operation {holder} {destination} /= {divisor}"
    ));
    // Minecraft's remainder already uses floor division semantics. Computing
    // quotient * divisor can overflow for negative dividends near i32::MIN.
    commands.push(format!(
        "scoreboard players operation {holder} {remainder} = {holder} {original}"
    ));
    commands.push(format!(
        "scoreboard players operation {holder} {remainder} %= {divisor}"
    ));
    match rounding {
        RoundingPolicy::Floor => {}
        RoundingPolicy::TowardZero => commands.push(format!(
            "execute if score {holder} {original} matches ..-1 if score {holder} {remainder} matches 1.. run scoreboard players add {holder} {destination} 1"
        )),
        RoundingPolicy::Ceiling => commands.push(format!(
            "execute if score {holder} {remainder} matches 1.. run scoreboard players add {holder} {destination} 1"
        )),
        RoundingPolicy::NearestTiesAwayFromZero | RoundingPolicy::NearestTiesToEven => {
            let half = scratch("half_divisor");
            let divisor_parity = scratch("divisor_parity");
            let two = constant_objective(context, "curve_two", 2)?;
            objectives.extend([half.clone(), divisor_parity.clone(), two.clone()]);
            commands.push(format!("scoreboard players set #value {two} 2"));
            commands.push(format!(
                "scoreboard players operation {holder} {half} = {divisor}"
            ));
            commands.push(format!(
                "scoreboard players operation {holder} {half} /= #value {two}"
            ));
            commands.push(format!(
                "scoreboard players operation {holder} {divisor_parity} = {divisor}"
            ));
            commands.push(format!(
                "scoreboard players operation {holder} {divisor_parity} %= #value {two}"
            ));
            commands.push(format!(
                "execute if score {holder} {remainder} > {holder} {half} run scoreboard players add {holder} {destination} 1"
            ));
            if rounding == RoundingPolicy::NearestTiesAwayFromZero {
                commands.push(format!(
                    "execute if score {holder} {original} matches 0.. if score {holder} {remainder} = {holder} {half} if score {holder} {divisor_parity} matches 0 run scoreboard players add {holder} {destination} 1"
                ));
            } else {
                let parity = scratch("parity");
                objectives.insert(parity.clone());
                commands.push(format!(
                    "scoreboard players operation {holder} {parity} = {holder} {destination}"
                ));
                commands.push(format!(
                    "scoreboard players operation {holder} {parity} %= #value {two}"
                ));
                commands.push(format!(
                    "execute if score {holder} {remainder} = {holder} {half} if score {holder} {divisor_parity} matches 0 unless score {holder} {parity} matches 0 run scoreboard players add {holder} {destination} 1"
                ));
            }
        }
    }
    Ok(())
}

fn constant_objective(
    context: NumericContext<'_>,
    role: &str,
    value: i64,
) -> Result<String, EntityDiagnostic> {
    scoreboard_value(context, role, value)?;
    Ok(
        sand_commands::ObjectiveName::logical(format!("{}.{}.{}", context.owner, role, value))
            .as_str()
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{FixedPoint, FixedScore, StatCurve};
    use sand_commands::ScoreHolder;

    #[test]
    fn bound_sources_keep_their_holder_and_scale_in_a_separate_working_context() {
        use crate::entity::EntityStateField;
        let owner = "game:calculate".parse().unwrap();
        let input = FixedScore::__new("game", "combat", "power", 100, 125, None);
        let expression = StatCurve::add([
            StatCurve::from(input.bind_to("#left", false)),
            StatCurve::from(input.bind_to("#right", false)),
        ]);
        let working = ScoreHolder::fake("#scratch");
        let lowered = expression
            .lower_scoreboard("result", "game:calculate", FixedPoint::default())
            .unwrap();
        let output = render_lowered_curve(
            NumericContext::new(&owner, &working).unwrap(),
            "numeric/calculate",
            &lowered,
        )
        .unwrap();
        for holder in ["#left", "#right"] {
            assert!(output.commands.iter().any(|line| {
                line.starts_with("scoreboard players operation #scratch ")
                    && line.ends_with(&format!("= {holder} {}", input.objective()))
            }));
        }
        for (index, command) in output.commands.iter().enumerate() {
            if command.contains("= #left ") || command.contains("= #right ") {
                let destination = command.split_whitespace().nth(4).unwrap();
                assert_eq!(
                    output.commands[index - 1],
                    format!("scoreboard players set #scratch {destination} 0"),
                    "an absent source must not reuse a previous evaluation's scratch score"
                );
            }
        }
        assert!(lowered.operations().iter().any(|op| matches!(
            op,
            LoweredCurveOperation::ScoreToFixed {
                source_scale: 100,
                target_scale: 1000,
                ..
            }
        )));
        let keys = expression.inputs();
        assert_eq!(keys.len(), 2);
        let mut inputs = crate::entity::CurveInputs::new();
        for key in keys {
            let value = if key.starts_with("#left ") { 2 } else { 5 };
            inputs
                .insert_score(key, value, FixedPoint::default(), "game:calculate", "sum")
                .unwrap();
        }
        let value = expression
            .evaluate(&inputs, FixedPoint::default(), "game:calculate", "sum")
            .unwrap();
        assert_eq!(value.as_f64(FixedPoint::default()), 7.0);
    }

    #[test]
    fn bound_single_selector_sources_keep_canonical_holder_semantics() {
        use crate::entity::EntityStateField;
        let input = FixedScore::__new("game", "combat", "power", 100, 125, None);
        for holder in [
            "@s",
            "@p",
            "@r",
            "@e[type=minecraft:zombie,limit=1]",
            "@p[tag=ready]",
        ] {
            let expression = StatCurve::from(input.bind_to(holder, false));
            let lowered = expression
                .lower_scoreboard("result", "game:calculate", FixedPoint::default())
                .unwrap();
            let owner = "game:calculate".parse().unwrap();
            let working = ScoreHolder::fake("#scratch");
            let output = render_lowered_curve(
                NumericContext::new(&owner, &working).unwrap(),
                "numeric/calculate",
                &lowered,
            )
            .unwrap();
            assert!(
                output
                    .commands
                    .iter()
                    .any(|line| line.ends_with(&format!("= {holder} {}", input.objective())))
            );
        }
    }

    #[test]
    fn invalid_bound_source_is_rejected_before_command_emission() {
        use crate::entity::EntityStateField;
        let input = FixedScore::__new("game", "combat", "power", 100, 125, None);
        for holder in ["bad holder", "@a", "@s\nkill @s"] {
            let expression = StatCurve::from(input.bind_to(holder, false));
            let error = expression
                .lower_scoreboard("result", "game:calculate", FixedPoint::default())
                .unwrap_err();
            assert!(error.to_string().contains("invalid bound numeric source"));
        }
    }

    #[test]
    fn server_holder_preserves_numeric_operations_and_generated_branches() {
        let owner = "game:calculate".parse().unwrap();
        let input = FixedScore::__new("game", "combat", "power", 100, 125, None);
        let expression = StatCurve::stepped(
            StatCurve::ratio(StatCurve::state(input), StatCurve::constant(3.0)),
            vec![(0.5, 1.0), (1.5, 2.0), (2.5, 3.0)],
            0.0,
        );
        let lowered = expression
            .lower_scoreboard("result", "game:calculate.result", FixedPoint::default())
            .unwrap();
        let entity = ScoreHolder::self_();
        let server = ScoreHolder::fake("#runtime");
        let entity_output = render_lowered_curve(
            NumericContext::new(&owner, &entity).unwrap(),
            "numeric/calculate",
            &lowered,
        )
        .unwrap();
        let server_output = render_lowered_curve(
            NumericContext::new(&owner, &server).unwrap(),
            "numeric/calculate",
            &lowered,
        )
        .unwrap();
        assert!(
            !server_output.records.is_empty(),
            "the expression exercises generated decision helpers"
        );
        assert_eq!(server_output.objectives, entity_output.objectives);
        assert_eq!(server_output.functions, entity_output.functions);
        assert_eq!(
            server_output.commands,
            entity_output
                .commands
                .iter()
                .map(|line| line.replace("@s ", "#runtime "))
                .collect::<Vec<_>>()
        );
        for (server, entity) in server_output.records.iter().zip(&entity_output.records) {
            assert_eq!(server.namespace, "game");
            assert_eq!(server.path, entity.path);
            assert_eq!(server.content, entity.content.replace("@s ", "#runtime "));
            assert!(!server.content.contains("@s"));
        }
    }

    #[test]
    fn dynamic_divisor_guard_is_not_confused_with_the_constant_holder() {
        let owner = "game:divide".parse().unwrap();
        let holder = ScoreHolder::fake("#value");
        let expression = StatCurve::ratio(
            StatCurve::input_raw("numerator"),
            StatCurve::input_raw("denominator"),
        );
        let lowered = expression
            .lower_scoreboard("result", "game:divide", FixedPoint::default())
            .unwrap();
        let output = render_lowered_curve(
            NumericContext::new(&owner, &holder).unwrap(),
            "numeric/divide",
            &lowered,
        )
        .unwrap();
        assert!(
            output
                .commands
                .iter()
                .any(|line| line.starts_with("execute unless score #value ")
                    && line.ends_with("matches 1.. run return fail"))
        );
        assert!(!output.commands.iter().any(|line| line.contains("@s")));
    }

    #[test]
    fn numeric_context_rejects_multiple_or_invalid_holders() {
        let owner = "game:calculate".parse().unwrap();
        for holder in [ScoreHolder::all(), ScoreHolder::fake("invalid holder")] {
            let error = NumericContext::new(&owner, &holder)
                .err()
                .expect("invalid context must fail")
                .to_string();
            assert!(error.contains("game:calculate"), "{error}");
            assert!(error.contains("numeric execution context"), "{error}");
        }
    }
}

#[cfg(test)]
#[path = "scoreboard_tests.rs"]
mod arithmetic_tests;
