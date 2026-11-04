use super::{Lifetime, identity, runtime};
use crate::{
    IntoCommands,
    condition::Condition,
    entity::{FixedPoint, PlayerKind, StatCurve},
    ir::{Actions, Cmd, NumericWrite},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
};
use sand_commands::{
    Actionbar, CommandError, CommandResult, ObjectiveName, ScoreHolder, Target, TextComponent,
};

/// Player-owned screen presentation composed with the canonical [`Lifetime`].
///
/// Register the completed definition, then call [`Hud::start`] in player context.
/// Every instance owns a distinct native bossbar, visible only to its owner.
/// Progress reads canonical State expressions in owner context each loaded tick;
/// values round to integer units and clamp to `0..=max`. Arithmetic failure
/// preserves the previous native value. Use `Lifetime::requires` for required
/// player State. The supplied lifetime's callbacks and cadence are preserved.
///
/// Minecraft provides one actionbar slot per player. The most recently started
/// Sand HUD in the export pack claims that slot, across definition namespaces;
/// older HUDs stay silent
/// and cannot clear it. Ending the newer HUD clears the slot without restoring
/// the older HUD. Commands from other packs can overwrite the same native slot.
/// Actionbar lease identities never wrap or reuse; exhaustion rejects starts.
///
/// Cancellation, expiry and owner loss remove native bossbars even when the
/// owner is offline. Reload drains a compact persistent list of active bars,
/// including unloaded instances. HUD capacity must be at most 128; the list has
/// one entry per live instance and retains no history. Timers and updates pause
/// with unloaded roots. Removing the datapack cannot run a cleanup callback.
/// Ordinary stop cleanup waits for an unloaded root to load again; reload's
/// bossbar drain does not require any root to be loaded.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Hud", module = "sand::systems",
summary = "Player-owned actionbar and bossbar presentation with bounded cleanup.",
context = "Composes Lifetime, canonical text and State expressions. Each instance owns a native bossbar; actionbars use a pack-shared newest-claim lease across definition namespaces. Capacity is at most 128.",
minecraft = "Native bossbars are private to their player owner and removed on stop or reload, including offline owners. Actionbar updates pause with unloaded roots.",
use_when = ["Showing owner-bound gameplay status"], avoid_when = ["Multiple independent actionbars must be visible simultaneously"],
example = "use sand::systems::Hud;",
)]
pub struct Hud {
    lifetime: Lifetime<PlayerKind>,
    actionbar: Option<TextComponent>,
    bossbar: Option<(TextComponent, StatCurve, u32)>,
}
impl Hud {
    /// Define screen presentation for a player lifetime. Add at least one
    /// actionbar or bossbar before registration. Capacity must be `1..=128`.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Hud::new", module = "sand::systems", kind = "method",
summary = "Define screen presentation for a player lifetime.", context = "The lifetime supplies ownership, duration, callbacks and capacity; configure at least one channel. Capacity is limited to 128 for bounded reload cleanup.",
minecraft = "Uses the canonical player-owned lifetime runtime and native screen channels.",
use_when = ["Showing owner-bound gameplay status"], avoid_when = ["An entity or shared server bossbar is required"],
params(lifetime = "Player lifetime with capacity at most 128."), returns = "An empty HUD definition.",
example = "use sand::systems::{Hud,Lifetime};\nfn demonstrate(lifetime: Lifetime) { let _hud = Hud::new(lifetime); }",
)]
    pub fn new(lifetime: Lifetime<PlayerKind>) -> Self {
        Self {
            lifetime,
            actionbar: None,
            bossbar: None,
        }
    }
    /// Refresh rich text in the owner's actionbar every loaded tick. This
    /// instance claims the pack's actionbar lease on start. Stopping clears
    /// only a lease still owned by this instance; superseded HUDs do not resume.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Hud::actionbar", module = "sand::systems", kind = "method",
summary = "Configure owner actionbar text and exclusive slot ownership.", context = "Newest HUD claims the pack-shared player slot across definition namespaces. Stale instances neither write nor clear it; ending the winner does not restore older text.",
minecraft = "Refreshes native actionbar text every loaded tick and clears the current lease on stop or reload. Other packs share Minecraft's slot.",
use_when = ["Showing one short owner status line"], avoid_when = ["Independent overlays must coexist in the same actionbar slot"],
params(text = "Canonical rich text evaluated in player context at each refresh."), returns = "The configured HUD.",
example = "use sand::systems::Hud;\nuse sand::text::Text;\nfn demonstrate(hud: Hud) { let _hud = hud.actionbar(Text::new(\"Ready\")); }",
)]
    pub fn actionbar(mut self, text: TextComponent) -> Self {
        self.actionbar = Some(text);
        self
    }
    /// Add a private native bossbar. Text and the value expression evaluate in
    /// player context every loaded tick. Value rounds to integer units and
    /// clamps to `0..=max`; arithmetic failure preserves its previous value.
    /// The maximum must be `1..=i32::MAX`. Native color/style defaults apply.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Hud::bossbar", module = "sand::systems", kind = "method",
summary = "Configure a private owner bossbar driven by a State expression.", context = "Each lifetime instance has its own native ID. Canonical numeric lowering rounds to integer units and clamps to zero through max; arithmetic failure preserves the previous value.",
minecraft = "Refreshes native bossbar name and value in player context. Removes the bar on stop or reload, including offline owners.",
use_when = ["Displaying owner-specific progress"], avoid_when = ["A shared global bossbar is intended"],
params(name = "Canonical rich text name.", value = "Canonical State accessor, constant or StatCurve evaluated in owner context.", max = "Positive maximum no greater than i32::MAX."), returns = "The configured HUD.",
example = "use sand::systems::Hud;\nuse sand::text::Text;\nfn demonstrate(hud: Hud) { let _hud = hud.bossbar(Text::new(\"Progress\"), 25, 100); }",
)]
    pub fn bossbar(mut self, name: TextComponent, value: impl Into<StatCurve>, max: u32) -> Self {
        self.bossbar = Some((name, value.into(), max));
        self
    }
    /// Start an independent HUD for the current player. Capacity admission
    /// occurs before presentation is created or the actionbar lease is claimed.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Hud::start", module = "sand::systems", kind = "method",
summary = "Start an independent HUD for the current player.", context = "Register the completed definition before calling. Lifetime admission runs before any native presentation is created.",
minecraft = "Creates a player-owned loaded-tick lifetime and its configured native channels.",
use_when = ["Activating owner-bound presentation"], avoid_when = ["No player executor is available"],
returns = "Retained start actions in player context.", example = "use sand::systems::Hud;\nfn demonstrate(hud: Hud) { let _actions = hud.start(); }",
)]
    pub fn start(&self) -> Actions {
        self.lifetime.start()
    }
    /// Stop all instances of this HUD definition belonging to the current
    /// player. Each private bossbar is removed; stale actionbar leases cannot
    /// clear presentation belonging to a newer instance or another player.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Hud::cancel", module = "sand::systems", kind = "method",
summary = "Cancel this definition's HUD instances for the current player.", context = "Canonical lifetime cleanup removes each owned bossbar and releases only an actionbar lease still owned by that instance.",
minecraft = "Native bossbar removal does not require an online player target.",
use_when = ["Ending owner-bound presentation"], avoid_when = ["Cancelling another player's HUD without entering that context"],
returns = "Retained cancellation actions in player context.", example = "use sand::systems::Hud;\nfn demonstrate(hud: Hud) { let _actions = hud.cancel(); }",
)]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl AsRef<Lifetime<PlayerKind>> for Hud {
    fn as_ref(&self) -> &Lifetime<PlayerKind> {
        &self.lifetime
    }
}
impl IntoDatapack for Hud {
    fn into_datapack(mut self) -> DatapackRegistration {
        if self.actionbar.is_some() {
            let available = Condition::function_internal(
                identity::resource(&self.lifetime.id, "hud_available").to_string(),
            );
            self.lifetime.diagnostics.push((
                available.clone(),
                "Actionbar lease identity space is exhausted".into(),
            ));
            self.lifetime.guard = Some(Condition::all(
                self.lifetime.guard.take().into_iter().chain([available]),
            ));
        }
        runtime::hooks(&mut self.lifetime, "hud_init", "hud_cleanup", "hud_step");
        let mut registration =
            self.lifetime
                .clone()
                .into_datapack()
                .lifecycle(LifecycleContribution::load(format!(
                    "function {}",
                    identity::resource(&self.lifetime.id, "hud_load")
                )));
        for part in [
            "hud_init",
            "hud_create",
            "hud_available",
            "hud_allocate",
            "hud_claim",
            "hud_step",
            "hud_owner",
            "hud_render",
            "hud_value",
            "hud_actionbar",
            "hud_clear",
            "hud_cleanup",
            "hud_forget",
            "hud_release",
            "hud_remove_bar",
            "hud_load",
            "hud_drain",
        ] {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.lifetime.id,
                part,
                move || definition.build(part),
            ));
        }
        registration
    }
}
impl Hud {
    fn validate(&self) -> CommandResult<()> {
        if self.actionbar.is_none() && self.bossbar.is_none() {
            return Err(CommandError::new(
                "hud",
                "channels",
                "configure an actionbar or bossbar",
            ));
        }
        if self.lifetime.capacity_limit() > 128 {
            return Err(CommandError::new(
                "hud",
                "capacity",
                "HUD capacity must be at most 128 for bounded reload cleanup",
            ));
        }
        if let Some(text) = &self.actionbar {
            text.try_to_json_value()?;
        }
        if let Some((name, value, max)) = &self.bossbar {
            name.try_to_json_value()?;
            if *max == 0 || *max > i32::MAX as u32 {
                return Err(CommandError::new(
                    "hud",
                    "max",
                    "bossbar maximum must be in 1..=2147483647",
                ));
            }
            value
                .validate(
                    FixedPoint::default(),
                    &self.lifetime.id.to_string(),
                    "bossbar",
                )
                .map_err(|error| CommandError::new("hud", "value", error.to_string()))?;
        }
        Ok(())
    }
    fn build(&self, part: &str) -> CommandResult<Actions> {
        self.validate()?;
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let owner = identity::objective(&id)?;
        // Resource namespaces have independent Lifetime instance counters.
        // The native actionbar slot instead needs one nonreused claim identity
        // for the entire exported pack, including imported HUD definitions.
        let lease = ObjectiveName::logical(format!(
            "sand hud actionbar {}",
            identity::pack_namespace()?
        ));
        let value = ObjectiveName::logical(format!("sand hud value {id}"));
        crate::function::request_numeric_objectives([lease.clone(), value.clone()])?;
        let resource = |part| identity::resource(&id, part).to_string();
        let ledger = resource("hud_ledger");
        let bar = format!("{}/$(instance)", resource("hud_bar"));
        let mut actions = Actions::default();
        let mut lines = Vec::new();
        match part {
            "hud_init" => {
                if self.bossbar.is_some() {
                    lines.extend([
                        format!("execute unless data storage {ledger} active run data modify storage {ledger} active set value []"),
                        format!("function {} with entity @s data", resource("hud_create")),
                    ]);
                }
                if self.actionbar.is_some() {
                    lines.extend(["data modify entity @s data.hud_lease set value 0".into(), format!("function {}", resource("hud_allocate"))]);
                }
                lines.push(format!("function {}", resource("hud_step")));
            }
            "hud_create" => {
                if let Some((name, _, max)) = &self.bossbar {
                    lines.extend([
                        format!("$data modify storage {ledger} active append value {{instance:$(instance)}}"),
                        format!("$bossbar add {bar} {}", macro_text(name)?),
                        format!("$bossbar set {bar} max {max}"),
                    ]);
                }
            }
            "hud_available" => lines.extend([format!("execute if score #next {lease} matches 0..2147483646 run return 1"), "return 0".into()]),
            "hud_allocate" => lines.extend([
                // Recheck after admission: attaching authored State may invoke
                // another HUD before this instance reaches initialization.
                format!("execute unless score #next {lease} matches 0..2147483646 run return 0"),
                format!("scoreboard players add #next {lease} 1"),
                format!("execute store result entity @s data.hud_lease int 1 run scoreboard players get #next {lease}"),
                format!("function {} with entity @s data", resource("hud_claim")),
            ]),
            "hud_claim" => lines.push(format!("$scoreboard players set @a[scores={{{owner}=$(owner)}},limit=1] {lease} $(hud_lease)")),
            "hud_step" => lines.push(format!("function {} with entity @s data", resource("hud_owner"))),
            "hud_owner" => lines.push(format!("$execute as @a[scores={{{owner}=$(owner)}},limit=1] at @s run function {} {{instance:$(instance){}}}", resource("hud_render"), if self.actionbar.is_some() { ",hud_lease:$(hud_lease)" } else { "" })),
            "hud_render" => {
                if let Some((name, _, max)) = &self.bossbar {
                    lines.extend([
                        format!("$bossbar set {bar} players @s"),
                        format!("$bossbar set {bar} name {}", macro_text(name)?),
                        format!("function {}", resource("hud_value")),
                        format!("$execute if score #hud {value} matches 0..{max} store result bossbar {bar} value run scoreboard players get #hud {value}"),
                    ]);
                }
                if self.actionbar.is_some() { lines.push(format!("$execute if score @s {lease} matches 1.. if score @s {lease} matches $(hud_lease) run function {}", resource("hud_actionbar"))); }
            }
            "hud_value" => if let Some((_, curve, max)) = &self.bossbar {
                // Clearing the temporary destination prevents a failed read for
                // one owner from reusing another owner's previously computed value.
                actions.extend([format!("scoreboard players reset #hud {value}").into_commands()]);
                actions.extend([Cmd::NumericWrite(Box::new(NumericWrite {
                    value: curve.clone(), holder: ScoreHolder::fake("#hud"), objective: value,
                    scale: 1, fixed: FixedPoint::default(), bounds: Some((0, *max as i32)), dirty: vec![],
                }))]);
            },
            "hud_actionbar" => if let Some(text) = &self.actionbar { lines.push(Actionbar::show(Target::current_player(), text.clone())); },
            "hud_clear" => {
                lines.extend([Actionbar::show(Target::current_player(), TextComponent::literal("")), format!("scoreboard players reset @s {lease}")]);
            }
            "hud_cleanup" => {
                if self.bossbar.is_some() {
                    lines.push(format!("function {} with entity @s data", resource("hud_forget")));
                }
                if self.actionbar.is_some() { lines.push(format!("function {} with entity @s data", resource("hud_release"))); }
            }
            "hud_release" => lines.push(format!("$execute as @a[scores={{{owner}=$(owner),{lease}=$(hud_lease)}},limit=1] run function {}", resource("hud_clear"))),
            "hud_forget" => lines.extend([format!("$bossbar remove {bar}"), format!("$data remove storage {ledger} active[{{instance:$(instance)}}]")]),
            "hud_remove_bar" => lines.push(format!("$bossbar remove {bar}")),
            "hud_load" => {
                if self.bossbar.is_some() {
                    lines.extend([format!("execute if data storage {ledger} active[0] run function {}", resource("hud_drain")), format!("data remove storage {ledger} active")]);
                }
                if self.actionbar.is_some() { lines.extend([format!("scoreboard players add #next {lease} 0"), format!("execute as @a[scores={{{lease}=1..}}] run function {}", resource("hud_clear"))]); }
            }
            "hud_drain" => lines.extend([
                format!("function {} with storage {ledger} active[0]", resource("hud_remove_bar")),
                format!("data remove storage {ledger} active[0]"),
                format!("execute if data storage {ledger} active[0] run function {}", resource("hud_drain")),
            ]),
            _ => unreachable!(),
        }
        actions.extend([lines.into_commands()]);
        Ok(actions)
    }
}

// Escape a literal macro opening only inside the serialized text. Minecraft
// expands macro placeholders before decoding text, so the native Unicode
// escape preserves authored content without making it an argument name.
fn macro_text(text: &TextComponent) -> CommandResult<String> {
    Ok(text
        .try_to_json_value()?
        .to_string()
        .replace("$(", r"\u0024("))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_components::effect::Ticks;

    fn hud() -> Hud {
        Hud::new(Lifetime::new("test:hud".parse().unwrap(), Ticks::new(20)))
    }
    #[test]
    fn invalid_presentation_is_rejected_before_allocating_runtime_objectives() {
        for (definition, field) in [
            (hud(), "channels"),
            (
                hud().bossbar(TextComponent::literal("Progress"), 1, 0),
                "max",
            ),
            (
                hud().bossbar(TextComponent::literal("Progress"), 1, u32::MAX),
                "max",
            ),
            (
                Hud::new(Lifetime::new("test:hud".parse().unwrap(), Ticks::new(20)).capacity(129))
                    .actionbar(TextComponent::literal("Ready")),
                "capacity",
            ),
            (
                hud().actionbar(TextComponent::literal("Ready").color_hex("#GG0000")),
                "color",
            ),
        ] {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            let error = definition.build("hud_init").unwrap_err();
            assert!(error.field.contains(field), "{error:?}");
            assert!(crate::function::take_numeric_objectives().is_empty());
        }
    }
    #[test]
    fn authored_text_cannot_become_a_runtime_macro_argument() {
        let literal = TextComponent::literal(r"Progress $(literal) \$(another)");
        let encoded = macro_text(&literal).unwrap();
        assert!(!encoded.contains("$("));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),
            literal.try_to_json_value().unwrap()
        );
    }
}
