//! Player-owned native dialog sessions with one-shot, generation-bound actions.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    condition::Condition,
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName, TextComponent};
use sand_components::{
    ResourceLocation,
    dialog::{Dialog, DialogAction, DialogButton},
};
use std::sync::Arc;

#[derive(Clone)]
struct Choice {
    label: TextComponent,
    allowed: Condition,
    action: Arc<dyn Fn() -> Actions>,
}

/// A native dialog whose choices belong to one live player session.
///
/// Register the definition before calling `open`. Opening any Menu in the same
/// pack invalidates the player's previous session. Each opening allocates fresh
/// action tokens; old buttons, reloads, disconnects and expired sessions cannot
/// authorize an action. Tokens are routing values, not secrets: every choice
/// rechecks its gameplay condition immediately before consuming the session.
/// A choice consumes the session before user code runs; reopen or open another
/// Menu explicitly to navigate. Escape/close has no reliable native callback,
/// so timeout and explicit `close` own cleanup. Definitions are static; choices
/// remain visible even when their authorization condition is false.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Menu", module = "sand::systems",
summary = "A native dialog with player-owned, expiring, one-shot sessions.", context = "Register the Menu before use. Fresh tokens reject stale buttons; session invalidation precedes callbacks. Conditions remain authoritative even when buttons are visible.",
minecraft = "Native dialogs and player scores; reload and disconnect invalidate sessions. Escape closes the screen but session expiry is timer-based. Token exhaustion rejects opening rather than recycling values.",
use_when = ["Building survival-accessible gameplay menus"], avoid_when = ["Cancellable container slot clicks or unverified free-text input"],
example = "use sand::systems::Menu;")]
pub struct Menu {
    pub(super) id: ResourceLocation,
    title: TextComponent,
    timeout: u32,
    valid: Option<Condition>,
    choices: Vec<Choice>,
    page_size: usize,
    pub(super) object: Option<(ResourceLocation, f32)>,
}
impl Menu {
    /// Define a menu with an online-tick timeout in 1..=2147483647.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Menu::new", module = "sand::systems",
summary = "Define a native menu and its session timeout.", context = "Register the Menu before use. Fresh tokens reject stale buttons; session invalidation precedes callbacks. Conditions remain authoritative even when buttons are visible.",
minecraft = "Native dialogs and player scores; reload and disconnect invalidate sessions. Escape closes the screen but session expiry is timer-based. Token exhaustion rejects opening rather than recycling values.",
use_when = ["Building survival-accessible gameplay menus"], avoid_when = ["Cancellable container slot clicks or unverified free-text input"],
params(id = "Unique registered menu identity.", title = "Native dialog title.", timeout = "Positive online-tick lifetime."),
returns = "The configured menu or retained player-context actions.",
example = "use sand::systems::Menu;")]
    pub fn new(id: ResourceLocation, title: TextComponent, timeout: crate::state::Ticks) -> Self {
        Self {
            id,
            title,
            timeout: timeout.get(),
            valid: None,
            choices: vec![],
            page_size: 64,
            object: None,
        }
    }
    /// Append a visible choice, its authoritative player-context guard and action.
    /// The callback executes as the choosing player. Unbound menus use the
    /// player position; `Interactable::opens_menu` uses the bound object position.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Menu::choice", module = "sand::systems",
summary = "Append a choice with server-side authorization.", context = "Register the Menu before use. Fresh tokens reject stale buttons; session invalidation precedes callbacks. Conditions remain authoritative even when buttons are visible.",
minecraft = "Native dialogs and player scores; reload and disconnect invalidate sessions. Escape closes the screen but session expiry is timer-based. Token exhaustion rejects opening rather than recycling values.",
use_when = ["Building survival-accessible gameplay menus"], avoid_when = ["Cancellable container slot clicks or unverified free-text input"],
params(label = "Visible choice label.", allowed = "Player-context condition rechecked before execution.", action = "Player-context gameplay actions."),
returns = "The configured menu or retained player-context actions.",
example = "use sand::systems::Menu;")]
    pub fn choice<R: IntoCommands>(
        mut self,
        label: TextComponent,
        allowed: Condition,
        action: impl Fn() -> R + 'static,
    ) -> Self {
        self.choices.push(Choice {
            label,
            allowed,
            action: Arc::new(move || action().into_commands()),
        });
        self
    }
    /// Require a player-context condition at opening, on each online tick and
    /// immediately before dispatch. Failure invalidates the session and clears
    /// its screen. Use canonical predicates for world/quest eligibility; a
    /// relative condition is evaluated at the player's current position.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Menu::while_open", module = "sand::systems",
summary = "Invalidate a session when its gameplay context is no longer eligible.", context = "The player-context guard runs at opening, during online ticks and before action dispatch. Failure closes the session without invoking a choice.", minecraft = "Compiles canonical Conditions into an isolated native function condition; relative queries use the current player position.", use_when = ["Restricting menus to live world or quest contexts"], avoid_when = ["Assuming this captures an arbitrary runtime object position"], params(condition = "Player-context session validity condition."), returns = "The configured menu.", example = "use sand::systems::Menu;")]
    pub fn while_open(mut self, condition: Condition) -> Self {
        self.valid = Some(condition);
        self
    }

    /// Split choices into pages of 1–64 gameplay buttons, with automatic Previous
    /// and Next controls. At most 256 choices may be registered. Navigation
    /// invalidates old buttons and allocates a fresh session; only choices on
    /// the visible page receive tokens. Object bindings survive navigation.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Menu::page_size", module = "sand::systems",
summary = "Paginate choices with generation-bound navigation controls.", context = "Only visible choices receive fresh action values. Navigation retains object binding but invalidates every old-page button and refreshes the timeout.", minecraft = "Uses native multi-action dialogs with compiler-generated page functions and bounded player scores. Supports up to 256 choices and 1–64 choices per page.", use_when = ["Building catalogs and longer gameplay menus"], avoid_when = ["Unbounded dynamically generated choice collections"], params(choices = "Number of gameplay choices per page, in 1..=64."), returns = "The configured menu.", example = "use sand::systems::Menu; fn paginate(menu: Menu) -> Menu { menu.page_size(8) }")]
    pub fn page_size(mut self, choices: usize) -> Self {
        self.page_size = choices;
        self
    }

    /// Open a fresh session for the executing player, replacing any prior menu.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Menu::open", module = "sand::systems",
summary = "Open a fresh menu session for the executing player.", context = "Register the Menu before use. Fresh tokens reject stale buttons; session invalidation precedes callbacks. Conditions remain authoritative even when buttons are visible.",
minecraft = "Native dialogs and player scores; reload and disconnect invalidate sessions. Escape closes the screen but session expiry is timer-based. Token exhaustion rejects opening rather than recycling values.",
use_when = ["Building survival-accessible gameplay menus"], avoid_when = ["Cancellable container slot clicks or unverified free-text input"],
returns = "The configured menu or retained player-context actions.",
example = "use sand::systems::Menu;")]
    pub fn open(&self) -> Actions {
        identity::call(&self.id, "menu_open")
    }
    /// Invalidate this menu's session for the executing player and clear its input.
    /// Does not invalidate a newer session belonging to another menu.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Menu::close", module = "sand::systems",
summary = "Invalidate this menu session for the executing player.", context = "Register the Menu before use. Fresh tokens reject stale buttons; session invalidation precedes callbacks. Conditions remain authoritative even when buttons are visible.",
minecraft = "Native dialogs and player scores; reload and disconnect invalidate sessions. Escape closes the screen but session expiry is timer-based. Token exhaustion rejects opening rather than recycling values.",
use_when = ["Building survival-accessible gameplay menus"], avoid_when = ["Cancellable container slot clicks or unverified free-text input"],
returns = "The configured menu or retained player-context actions.",
example = "use sand::systems::Menu;")]
    pub fn close(&self) -> Actions {
        identity::call(&self.id, "menu_close")
    }
}
impl IntoDatapack for Menu {
    fn into_datapack(self) -> DatapackRegistration {
        let mut registration = DatapackRegistration::new();
        for phase in ["load", "tick"] {
            let command = format!(
                "function {}",
                identity::resource(&self.id, &format!("menu_{phase}"))
            );
            registration = registration.lifecycle(if phase == "load" {
                LifecycleContribution::load(command)
            } else {
                LifecycleContribution::tick(command)
            });
        }
        let mut parts = [
            "load",
            "tick",
            "open",
            "open_bound",
            "admit",
            "clear",
            "close",
            "dispatch",
            "valid",
            "valid_object",
            "next",
            "previous",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        parts.extend((0..self.choices.len()).flat_map(|i| {
            [
                format!("action_{i}"),
                format!("invoke_{i}"),
                format!("body_{i}"),
            ]
        }));
        parts.extend(
            (0..self.choices.len().div_ceil(self.page_size.max(1)))
                .flat_map(|i| [format!("show_{i}"), format!("page_{i}")]),
        );
        for part in parts {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.id,
                &format!("menu_{part}"),
                move || definition.build(&part),
            ));
        }
        registration
    }
}
impl Menu {
    fn build(&self, part: &str) -> CommandResult<Actions> {
        if self.timeout == 0
            || self.timeout > i32::MAX as u32
            || self.choices.is_empty()
            || self.choices.len() > 256
            || !(1..=64).contains(&self.page_size)
        {
            return Err(CommandError::new(
                "menu",
                "definition",
                "requires 1..=256 choices, page_size in 1..=64 and timeout in 1..=2147483647",
            ));
        }
        self.title
            .validate_at_path(&sand_commands::CommandProfile::unprofiled(), "menu.title")?;
        for (index, choice) in self.choices.iter().enumerate() {
            choice.label.validate_at_path(
                &sand_commands::CommandProfile::unprofiled(),
                format!("menu.choices[{index}].label"),
            )?;
        }
        if self.title.to_string().contains("$(")
            || self
                .choices
                .iter()
                .any(|choice| choice.label.to_string().contains("$("))
        {
            return Err(CommandError::new(
                "menu",
                "text",
                "literal macro delimiters are not supported in menu text",
            ));
        }
        let id = crate::function::resolve_export_location(&self.id)?;
        let global = |field| {
            ObjectiveName::logical(format!("sand menu {} {field}", id.namespace())).to_string()
        };
        let local =
            |field: &str| ObjectiveName::logical(format!("sand menu {id} {field}")).to_string();
        let sequence = global("sequence");
        let active = global("active");
        let leave = global("leave");
        let session = local("session");
        let observed_leave = local("observed_leave");
        let timer = local("timer");
        let trigger = local("trigger");
        let page = local("page");
        let bound = local("object");
        let pages = self.choices.len().div_ceil(self.page_size);
        let next = self.choices.len();
        let previous = next + 1;
        let tokens = self.choices.len() + if pages > 1 { 2 } else { 0 };
        let token = |i| local(&format!("choice_{i}"));
        let function = |part: &str| identity::resource(&id, &format!("menu_{part}")).to_string();
        let object = self.object.as_ref().map(|(scope, range)| {
            let tag = format!("{}.interaction", identity::task_tag(scope)?);
            let group = super::interactable::group_objective(scope)?;
            Ok::<_, sand_commands::CommandError>(format!("@e[type=minecraft:interaction,tag={tag},scores={{{group}=$(instance)}},distance=..{range},limit=1]"))
        }).transpose()?;
        let mut lines = Vec::new();
        match part {
            "valid" => {
                let mut body = self
                    .valid
                    .as_ref()
                    .map(|condition| {
                        condition.guard_actions(true, &[], Cmd::Return(0).into_commands())
                    })
                    .unwrap_or_default();
                if object.is_some() {
                    body.extend([vec![
                        format!("execute unless score @s {bound} matches 1.. run return 0"),
                        format!("execute store result storage {id} object_check.instance int 1 run scoreboard players get @s {bound}"),
                        format!("return run function {} with storage {id} object_check", function("valid_object")),
                    ].into_commands()]);
                } else {
                    body.extend([Cmd::Return(1).into_commands()]);
                }
                return Ok(body);
            }
            "valid_object" => {
                lines.push(format!("data remove storage {id} object_check"));
                if let Some(object) = &object {
                    lines.push(format!("$execute if entity {object} run return 1"));
                }
                lines.push("return 0".into());
            }
            "load" => {
                for objective in [
                    &sequence,
                    &active,
                    &session,
                    &timer,
                    &observed_leave,
                    &page,
                    &bound,
                ] {
                    lines.push(format!("scoreboard objectives add {objective} dummy"));
                }
                for i in 0..tokens {
                    lines.push(format!("scoreboard objectives add {} dummy", token(i)));
                }
                lines.extend([
                    format!("scoreboard objectives add {trigger} trigger"),
                    format!("scoreboard objectives add {leave} minecraft.custom:minecraft.leave_game"),
                    format!("execute unless score #next {sequence} matches 0.. run scoreboard players set #next {sequence} 0"),
                ]);
                for objective in [&active, &session, &trigger, &timer, &page, &bound] {
                    lines.push(format!("scoreboard players reset * {objective}"));
                }
            }
            "open" | "open_bound" => {
                lines.push("execute unless entity @s[type=minecraft:player] run return 0".into());
                // Bound definitions can only be admitted through the exact hitbox
                // bridge. A plain open must not pick an arbitrary nearby object.
                if (part == "open_bound") != object.is_some() {
                    return Ok(Cmd::Return(0).into_commands());
                }
                lines.push(format!("function {}", function("close")));
                lines.push(format!("scoreboard players set @s {page} 0"));
                if object.is_some() {
                    lines.push(format!("$scoreboard players set @s {bound} $(instance)"));
                }
                lines.push(format!("function {}", function("admit")));
            }
            "admit" => {
                lines.extend([
                    format!("execute at @s unless function {} run return run function {}", function("valid"), function("close")),
                    format!("execute unless score @s {page} matches 0..{} run return run function {}", pages - 1, function("close")),
                    format!("execute unless score #next {sequence} matches 0..{} run return run function {}", i32::MAX as usize - self.page_size - 3, function("close")),
                    format!("scoreboard players add #next {sequence} 1"),
                    format!("scoreboard players operation @s {session} = #next {sequence}"),
                    format!("scoreboard players operation @s {active} = @s {session}"),
                    format!("scoreboard players add @s {leave} 0"),
                    format!("scoreboard players operation @s {observed_leave} = @s {leave}"),
                    format!("scoreboard players set @s {timer} {}", self.timeout),
                    format!("scoreboard players set @s {trigger} 0"),
                    format!("scoreboard players enable @s {trigger}"),
                ]);
                for p in 0..pages {
                    lines.push(format!(
                        "execute if score @s {page} matches {p} run return run function {}",
                        function(&format!("page_{p}"))
                    ));
                }
            }
            "next" | "previous" => {
                let (range, operation) = if part == "next" {
                    (format!("0..{}", pages.saturating_sub(2)), "add")
                } else {
                    (format!("1..{}", pages - 1), "remove")
                };
                if pages == 1 {
                    return Ok(Cmd::Return(0).into_commands());
                }
                lines.extend([
                    format!(
                        "execute unless score @s {page} matches {range} run return run function {}",
                        function("close")
                    ),
                    format!("function {}", function("clear")),
                    format!("scoreboard players {operation} @s {page} 1"),
                    format!("function {}", function("admit")),
                ]);
            }
            "clear" => {
                lines.push(format!(
                    "execute if score @s {active} = @s {session} run dialog clear @s"
                ));
                lines.push(format!("execute if score @s {active} = @s {session} run scoreboard players reset @s {active}"));
                for objective in [&session, &timer, &observed_leave, &trigger] {
                    lines.push(format!("scoreboard players reset @s {objective}"));
                }
                for i in 0..tokens {
                    lines.push(format!("scoreboard players reset @s {}", token(i)));
                }
            }
            "close" => {
                lines.extend([
                    format!("function {}", function("clear")),
                    format!("scoreboard players reset @s {page}"),
                    format!("scoreboard players reset @s {bound}"),
                ]);
            }
            "tick" => {
                let players = format!("@a[scores={{{timer}=1..}}]");
                lines.extend([
                    format!("execute as {players} unless score @s {active} = @s {session} run function {}", function("close")),
                    format!("execute as {players} unless score @s {leave} = @s {observed_leave} run function {}", function("close")),
                    format!("execute as {players} at @s unless function {} run function {}", function("valid"), function("close")),
                    format!("execute as {players} at @s unless score @s {trigger} matches 0 run function {}", function("dispatch")),
                    format!("scoreboard players remove {players} {timer} 1"),
                    format!("execute as @a[scores={{{timer}=0}}] run function {}", function("close")),
                ]);
            }
            "dispatch" => {
                lines.extend([
                    format!(
                        "execute at @s unless function {} run return run function {}",
                        function("valid"),
                        function("close")
                    ),
                    format!("execute unless score @s {active} = @s {session} run return 0"),
                    format!("execute unless score @s {timer} matches 1.. run return 0"),
                    format!("execute unless score @s {leave} = @s {observed_leave} run return 0"),
                ]);
                for i in 0..tokens {
                    let action = if i == next {
                        "next".to_string()
                    } else if i == previous {
                        "previous".to_string()
                    } else {
                        format!("action_{i}")
                    };
                    lines.push(format!(
                        "execute if score @s {trigger} = @s {} run return run function {}",
                        token(i),
                        function(&action)
                    ));
                }
                lines.push(format!("return run function {}", function("close")));
            }
            _ if part.starts_with("page_") => {
                let p: usize = part[5..].parse().expect("registered page");
                let mut visible: Vec<_> = (p * self.page_size
                    ..self.choices.len().min((p + 1) * self.page_size))
                    .collect();
                if p > 0 {
                    visible.push(previous);
                }
                if p + 1 < pages {
                    visible.push(next);
                }
                for i in visible {
                    lines.extend([
                    format!("scoreboard players add #next {sequence} 1"),
                    format!("scoreboard players operation @s {} = #next {sequence}", token(i)),
                    format!("execute store result storage {id} menu.a{i} int 1 run scoreboard players get @s {}", token(i)),
                ]);
                }
                lines.extend([
                    format!(
                        "function {} with storage {id} menu",
                        function(&format!("show_{p}"))
                    ),
                    format!("data remove storage {id} menu"),
                ]);
            }
            _ if part.starts_with("show_") => {
                let p: usize = part[5..].parse().expect("registered page");
                let mut dialog =
                    Dialog::multi_action_local("menu_inline").title(self.title.clone());
                let mut buttons: Vec<_> = (p * self.page_size
                    ..self.choices.len().min((p + 1) * self.page_size))
                    .map(|i| (i, self.choices[i].label.clone()))
                    .collect();
                if p > 0 {
                    buttons.push((previous, TextComponent::literal("Previous")));
                }
                if p + 1 < pages {
                    buttons.push((next, TextComponent::literal("Next")));
                }
                for (i, label) in buttons {
                    dialog = dialog.button(DialogButton::new(label).action(
                        DialogAction::run_command(format!("/trigger {trigger} set $(a{i})")),
                    ));
                }
                lines.push(format!("$dialog show @s {}", dialog.to_json()));
            }
            _ if part.starts_with("body_") => {
                let i: usize = part[5..].parse().expect("registered callback");
                return Ok(runtime::isolated((self.choices[i].action)()));
            }
            _ if part.starts_with("invoke_") => {
                let i: usize = part[7..].parse().expect("registered callback");
                lines.push(format!("data remove storage {id} object_call"));
                lines.push(format!("function {}", function("close")));
                if let Some(object) = &object {
                    lines.push(format!(
                        "$execute at {object} run function {}",
                        function(&format!("body_{i}"))
                    ));
                }
            }
            _ => {
                let i: usize = part
                    .strip_prefix("action_")
                    .expect("registered action")
                    .parse()
                    .expect("action index");
                let mut body = self.choices[i].allowed.guard_actions(
                    true,
                    &[],
                    Cmd::ReturnRun(Box::new(Cmd::Function(function("close")))).into_commands(),
                );
                if object.is_some() {
                    body.extend([vec![format!("execute store result storage {id} object_call.instance int 1 run scoreboard players get @s {bound}"), format!("function {} with storage {id} object_call", function(&format!("invoke_{i}")))].into_commands()]);
                } else {
                    body.extend([
                        format!("function {}", function("close")).into_commands(),
                        runtime::isolated((self.choices[i].action)()),
                    ]);
                }
                return Ok(body);
            }
        }
        Ok(lines.into_commands())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_rich_text_fails_before_runtime_registration() {
        let invalid = || TextComponent::literal("Invalid").color_hex("#12FG00");
        let definition = Menu::new(
            "test:invalid".parse().unwrap(),
            invalid(),
            crate::state::Ticks::new(20),
        )
        .choice(
            TextComponent::literal("Choice"),
            Condition::all([]),
            Actions::default,
        );
        assert!(
            definition
                .build("show")
                .unwrap_err()
                .to_string()
                .contains("menu.title")
        );
        let definition = Menu::new(
            "test:invalid".parse().unwrap(),
            TextComponent::literal("Title"),
            crate::state::Ticks::new(20),
        )
        .choice(invalid(), Condition::all([]), Actions::default);
        assert!(
            definition
                .build("show")
                .unwrap_err()
                .to_string()
                .contains("menu.choices[0].label")
        );
    }
}
