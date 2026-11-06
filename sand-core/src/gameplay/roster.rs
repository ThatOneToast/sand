//! Explicit bounded online membership over canonical player State.
use super::{identity, runtime};
use crate::{
    IntoCommands,
    condition::Condition,
    entity::{
        EntityContext, EntityEnum, EntityEnumValue, EntityScore, EntityStateField, PlayerKind,
        StateFieldKind,
    },
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
    state::{StateDescriptor, StateScope},
};
use sand_commands::{CommandError, CommandResult, ExecuteOp, ObjectiveName, Selector};
use sand_components::ResourceLocation;

/// One explicitly registered arena's bounded online roster.
///
/// Membership and roles live in ordinary player State. Reuse the same membership
/// field across arenas for exclusive membership: successful admission transfers
/// the player, while rejection preserves their previous membership and role.
/// Roles are an author-defined canonical State enum (teams and spectators need
/// no separate parallel model). All roles count toward capacity.
///
/// Open starts a fresh generation; close and reload invalidate it. Generation
/// IDs never recycle. Disconnect invalidates membership even before the next
/// tick, using the native leave-game statistic; reconnect requires admission.
/// Death and dimension changes do not implicitly change membership. Compose
/// explicit policy from events, Session and Lifetime; this type does not own
/// tasks, teleport players, change game modes, or reserve offline seats.
///
/// Stored scores may remain after invalidation. Always use contains/is/each for
/// authorization rather than treating a nonzero membership score as membership.
/// Do not mutate the membership or role fields from competing systems. each
/// rejects synchronous roster mutations and nested iteration sharing its field;
/// gameplay callbacks retain their own return frames. Online players are scanned
/// on demand, with at most capacity valid members under the supported API.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster", module = "sand::systems", summary = "Bounded online arena membership over canonical player State.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], example = "use sand::systems::Roster;")]
pub struct Roster<T: EntityEnumValue> {
    id: ResourceLocation,
    member: EntityScore<i32>,
    role: EntityEnum<T>,
    capacity: u32,
}
impl<T: EntityEnumValue> Roster<T> {
    /// Define a registered arena. Capacity must be 1..=128. Both fields must
    /// belong to registered player State; membership must be a default-zero,
    /// non-ticking dummy Score whose bounds include 0..=i32::MAX.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::new", module = "sand::systems", summary = "Define a bounded arena roster.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], params(id = "Registered arena identity.", member = "Canonical player membership Score.", role = "Canonical player role enum.", capacity = "Maximum online members including spectators, 1..=128."), returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn new(
        id: ResourceLocation,
        member: EntityScore<i32>,
        role: EntityEnum<T>,
        capacity: u32,
    ) -> Self {
        Self {
            id,
            member,
            role,
            capacity,
        }
    }

    /// Invalidate the old generation and admit a new round. Token exhaustion
    /// closes the arena instead of recycling identities. Does not start Session.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::open", module = "sand::systems", summary = "Start a fresh roster generation.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn open(&self) -> Actions {
        identity::call(&self.id, "roster_open")
    }

    /// Stop admission and invalidate all current memberships, including offline
    /// players. Does not invoke callbacks or clean unrelated owned gameplay.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::close", module = "sand::systems", summary = "Invalidate an arena roster.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn close(&self) -> Actions {
        identity::call(&self.id, "roster_close")
    }

    /// Admit or transfer the executing player after checking attached State,
    /// current generation and capacity. Existing members can change roles even
    /// when full. Rejection leaves the player's previous fields unchanged.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::join", module = "sand::systems", summary = "Admit or transfer a player into a typed role.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], params(role = "Role committed on successful admission."), returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn join(&self, role: T) -> Actions {
        identity::call(&self.id, &format!("roster_join_{}", role.encode()))
    }

    /// Clear this arena's membership for the executing player. Other arenas
    /// sharing the field are unaffected. The role remains an ordinary saved value.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::leave", module = "sand::systems", summary = "Leave this arena without affecting another arena.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn leave(&self) -> Actions {
        identity::call(&self.id, "roster_leave")
    }

    /// Authoritative online-player membership, including schema, generation,
    /// role validity and disconnect-statistic checks. Does not mutate State.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::contains", module = "sand::systems", summary = "Test authoritative current player membership.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn contains(&self) -> Condition {
        Condition::function_internal(identity::resource(&self.id, "roster_contains").to_string())
    }

    /// Authoritative membership in one typed role, such as a team or spectator.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::is", module = "sand::systems", summary = "Test membership in one typed role.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], params(role = "Required canonical role."), returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn is(&self, role: T) -> Condition {
        Condition::all([self.contains(), self.role.is(role).condition()])
    }

    /// Test whether this arena has any valid online member, optionally in one
    /// role. Runs from server or entity context without rebinding the caller.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::has_members", module = "sand::systems", summary = "Test whether an arena has any current online members.", context = "Uses the same State, generation and connection validation as contains, with an optional canonical role filter.", minecraft = "An early-return function scans online players and returns on the first valid member.", use_when = ["Ending empty rounds or requiring a fighter before start"], avoid_when = ["Counting offline reservations"], params(role = "Optional canonical role filter."), returns = "A server-context or entity-context Condition.", example = "use sand::systems::Roster;")]
    pub fn has_members(&self, role: Option<T>) -> Condition {
        let suffix = role.map_or_else(
            || "roster_any".to_owned(),
            |role| format!("roster_any_{}", role.encode()),
        );
        Condition::function_internal(identity::resource(&self.id, &suffix).to_string())
    }

    /// Run once as each valid online member, at that player's position, optionally
    /// filtered by role. Roster mutation and nested iteration in the same field
    /// domain are no-ops until the whole call completes. Use separate later events
    /// for membership transitions; callback returns cannot strand the lock.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Roster::each", module = "sand::systems", summary = "Visit current online members under a membership mutation lock.", context = "Explicit registered arenas share canonical player membership and enum fields. Successful transfers are exclusive within that field domain; reload and disconnect invalidate authorization.", minecraft = "Uses bounded admission, monotonic generation IDs and native leave-game snapshots. No forced chunks, task ownership or offline seats.", use_when = ["Composing bounded multiplayer rounds from State and Session"], avoid_when = ["Dynamic matchmaking or automatic reconnect restoration"], params(role = "Optional typed role filter.", action = "Isolated player-context gameplay actions."), returns = "The configured roster, retained gameplay actions, or authoritative membership condition.", example = "use sand::systems::Roster;")]
    pub fn each<R: IntoCommands>(
        &self,
        role: Option<T>,
        action: impl FnOnce(EntityContext<PlayerKind>) -> R,
    ) -> Actions {
        let predicate = role.map_or_else(|| self.contains(), |role| self.is(role));
        let callback = predicate.guard_actions(
            false,
            &[],
            runtime::isolated(action(EntityContext::default()).into_commands()),
        );
        let mut body = format!(
            "execute if function {} run return 0",
            identity::resource(&self.id, "roster_locked")
        )
        .into_commands();
        body.extend([
            identity::call(&self.id, "roster_lock"),
            Cmd::Execute {
                operations: vec![
                    ExecuteOp::As(Selector::all_players()),
                    ExecuteOp::At(Selector::self_()),
                ],
                run: Box::new(Cmd::AnonymousFunction {
                    prefix: "sand/roster/member".into(),
                    body: callback,
                }),
            }
            .into_commands(),
            identity::call(&self.id, "roster_unlock"),
        ]);
        runtime::isolated(body)
    }

    fn schemas(&self) -> CommandResult<Vec<&'static StateDescriptor<'static>>> {
        let error = |field, detail| CommandError::new("roster", field, detail);
        if !(1..=128).contains(&self.capacity) {
            return Err(error("capacity", "requires 1..=128 online members"));
        }
        let mut schemas = Vec::new();
        for reference in [self.member.field_reference(), self.role.field_reference()] {
            let schema = inventory::iter::<StateDescriptor>
                .into_iter()
                .find(|s| s.id == reference.component)
                .ok_or_else(|| error("fields", "requires registered canonical player State"))?;
            if schema.scope != StateScope::Player {
                return Err(error("fields", "requires player-scoped State"));
            }
            let field = schema
                .fields
                .iter()
                .find(|f| {
                    ObjectiveName::logical(f.objective).as_str() == reference.objective
                        && f.field == reference.descriptor
                })
                .ok_or_else(|| error("fields", "field metadata does not match registered State"))?;
            if field.auto_tick || field.criterion != "dummy" {
                return Err(error("fields", "requires non-ticking dummy fields"));
            }
            schemas.push(schema);
        }
        let member = self.member.descriptor();
        if member.kind != StateFieldKind::Score
            || member.default != 0
            || member
                .bounds
                .is_some_and(|(min, max)| min > 0 || max < i32::MAX)
        {
            return Err(error(
                "member",
                "requires a default-zero Score covering 0..=i32::MAX",
            ));
        }
        if T::ENCODINGS.is_empty()
            || T::ENCODINGS.len() > 256
            || self.member.objective() == self.role.objective()
        {
            return Err(error(
                "role",
                "requires a distinct finite State enum with 1..=256 roles",
            ));
        }
        Ok(schemas)
    }

    fn build(&self, part: &str) -> CommandResult<Actions> {
        let schemas = self.schemas()?;
        let namespace = identity::pack_namespace()?;
        let id = crate::function::resolve_export_location(&self.id)?;
        let domain = self.member.objective();
        let runtime_score = ObjectiveName::logical(format!("sand roster {id}"));
        let sequence = ObjectiveName::logical(format!("sand roster {namespace} sequence"));
        let lock = ObjectiveName::logical(format!("sand roster {namespace} {domain} lock"));
        let seen = ObjectiveName::logical(format!("sand roster {namespace} {domain} connection"));
        let leave = ObjectiveName::logical(format!("sand roster {namespace} disconnect"));
        crate::function::request_numeric_objectives([
            runtime_score.clone(),
            sequence.clone(),
            lock.clone(),
            seen.clone(),
        ])?;
        let f = |suffix: &str| identity::resource(&self.id, &format!("roster_{suffix}"));
        let member = self.member.objective();
        let mut ready =
            "execute unless entity @s[type=minecraft:player] run return 0".into_commands();
        for schema in schemas {
            ready.extend([format!(
                "execute unless score @s {} matches {} run return 0",
                ObjectiveName::logical(schema.presence_objective),
                schema.version
            )]);
        }
        match part {
            "load" => {
                return Ok(format!(
                    "scoreboard objectives add {leave} minecraft.custom:minecraft.leave_game
scoreboard players set #generation {runtime_score} 0
scoreboard players set #busy {lock} 0
execute unless score #next {sequence} matches 0.. run scoreboard players set #next {sequence} 0"
                )
                .into_commands());
            }
            "locked" => {
                return Ok(format!(
                    "execute if score #busy {lock} matches 1 run return 1
return 0"
                )
                .into_commands());
            }
            "lock" => return Ok(format!("scoreboard players set #busy {lock} 1").into_commands()),
            "unlock" => return Ok(format!("scoreboard players set #busy {lock} 0").into_commands()),
            "contains" => {
                ready.extend([format!("execute unless score #generation {runtime_score} matches 1.. run return 0"), format!("execute unless score @s {member} = #generation {runtime_score} run return 0"), format!("execute unless score @s {leave} = @s {seen} run return 0")]);
                for role in T::ENCODINGS {
                    ready.extend([format!(
                        "execute if score @s {} matches {} run return 1",
                        self.role.objective(),
                        role.score
                    )]);
                }
                ready.extend(["return 0"]);
                return Ok(ready);
            }
            _ => {}
        }
        if part == "any" || part.starts_with("any_") {
            let filter = part.strip_prefix("any_").map_or_else(String::new, |role| {
                format!(" if score @s {} matches {role}", self.role.objective())
            });
            return Ok(format!(
                "execute as @a if function {}{filter} run return 1\nreturn 0",
                f("contains")
            )
            .into_commands());
        }
        let mut actions =
            format!("execute if score #busy {lock} matches 1 run return 0").into_commands();
        if part == "open" || part == "close" {
            actions.extend([format!(
                "scoreboard players set #generation {runtime_score} 0"
            )]);
            if part == "open" {
                actions.extend([format!("execute unless score #next {sequence} matches 0..2147483646 run return 0"), format!("scoreboard players add #next {sequence} 1"), format!("scoreboard players operation #generation {runtime_score} = #next {sequence}")]);
            }
            return Ok(actions);
        }
        actions.extend([ready]);
        if part == "leave" {
            actions.extend([
                format!(
                    "execute unless score #generation {runtime_score} matches 1.. run return 0"
                ),
                format!(
                    "execute unless score @s {member} = #generation {runtime_score} run return 0"
                ),
            ]);
            actions.extend([self.member.bind_to("@s", false).set(0)]);
        } else if let Some(role) = part.strip_prefix("join_") {
            let role: i32 = role.parse().expect("registered role encoding");
            actions.extend([format!("execute unless score #generation {runtime_score} matches 1.. run return 0"), format!("scoreboard players set #count {runtime_score} 0"), format!("execute as @a if function {} run scoreboard players add #count {runtime_score} 1", f("contains")), format!("execute unless function {} if score #count {runtime_score} matches {}.. run return 0", f("contains"), self.capacity)]);
            // Player State has no entity dirty reconciliation; both schemas were
            // validated above. Retain the canonical Score setter for membership.
            actions.extend([
                format!("scoreboard players operation @s {member} = #generation {runtime_score}"),
                format!("scoreboard players set @s {} {role}", self.role.objective()),
                format!("scoreboard players add @s {leave} 0"),
                format!("scoreboard players operation @s {seen} = @s {leave}"),
            ]);
        }
        Ok(actions)
    }
}
impl<T: EntityEnumValue> IntoDatapack for Roster<T> {
    fn into_datapack(self) -> DatapackRegistration {
        let mut registration = DatapackRegistration::new().lifecycle(LifecycleContribution::load(
            format!("function {}", identity::resource(&self.id, "roster_load")),
        ));
        let parts = [
            "load", "open", "close", "leave", "contains", "locked", "lock", "unlock", "any",
        ]
        .into_iter()
        .map(str::to_owned)
        .chain(
            T::ENCODINGS
                .iter()
                .flat_map(|r| [format!("join_{}", r.score), format!("any_{}", r.score)]),
        );
        for part in parts {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.id,
                &format!("roster_{part}"),
                move || definition.build(&part),
            ));
        }
        registration
    }
}
