//! The `execute` command chain builder.
//!
//! # Examples
//! ```rust,ignore
//! use sand_commands::{Execute, Selector, BlockPos};
//!
//! // execute as @a at @s run kill @s
//! Execute::new()
//!     .as_(Selector::all_players())
//!     .at(Selector::self_())
//!     .run("kill @s");
//!
//! // execute if entity @a[tag=ready] run say ready!
//! Execute::new()
//!     .if_entity(Selector::all_players().tag("ready"))
//!     .run("say ready!");
//! ```

use std::fmt;

use crate::Build;
use crate::RawCommand;
use crate::coord::{BlockPos, Rotation, Vec3};
use crate::error::{CommandError, CommandResult};
use crate::execute_args::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
use crate::execute_ir::{ConditionIr, ExecuteOp, ExecuteStoreTarget};
use crate::nbt::DataTarget;
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::scoreboard::{ScoreCmp, ScoreHolder};
use crate::selector::Selector;
use crate::validate;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Execute",
    aliases = ["sand::cmd::Execute", "sand::prelude::Execute", "sand::prelude::cmd::Execute"],
    module = "sand::command",
    summary = "Builder for the `execute` command chain. Call builder methods to add sub-commands, then call [`run`](Execute::run) or [`run_raw`](Execute::run_raw) to complete the command.",
    context = "Builder for the `execute` command chain. Call builder methods to add sub-commands, then call [`run`](Execute::run) or [`run_raw`](Execute::run_raw) to complete the command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Call builder methods to add sub-commands, then call [`run`](Execute::run) or [`run_raw`](Execute::run_raw) to complete the command.",
    use_when = ["Call builder methods to add sub-commands, then call [`run`](Execute::run) or [`run_raw`](Execute::run_raw) to complete the command."],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Execute;",
)]
/// Builder for the `execute` command chain.
///
/// Call builder methods to add sub-commands, then call [`run`](Execute::run) or
/// [`run_raw`](Execute::run_raw) to complete the command.
#[derive(Debug, Clone, Default)]
#[must_use = "execute builders must be completed with `run`, `try_run`, or `run_raw`"]
pub struct Execute {
    operations: Vec<ExecuteOp>,
    checks: Vec<ExecuteCheck>,
}

#[derive(Debug, Clone)]
enum ExecuteCheck {
    Selector {
        index: usize,
        kind: &'static str,
        value: Selector,
    },
    Vec3 {
        index: usize,
        kind: &'static str,
        value: Vec3,
    },
    BlockPos {
        index: usize,
        kind: &'static str,
        value: BlockPos,
    },
    Rotation {
        index: usize,
        kind: &'static str,
        value: Rotation,
    },
    Slot {
        index: usize,
        kind: &'static str,
        value: ItemSlot,
    },
    Finite {
        index: usize,
        kind: &'static str,
        field: &'static str,
        value: f64,
    },
    Resource {
        index: usize,
        kind: &'static str,
        field: &'static str,
        value: String,
        allow_tag: bool,
    },
    Holder {
        index: usize,
        kind: &'static str,
        value: ScoreHolder,
    },
    SingleHolder {
        index: usize,
        kind: &'static str,
        value: ScoreHolder,
    },
    Objective {
        index: usize,
        kind: &'static str,
        value: String,
    },
    ScoreRange {
        index: usize,
        kind: &'static str,
        value: String,
    },
}

impl Execute {
    /// Create a new `Execute` builder with no sub-commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::new",
        aliases = ["sand::cmd::Execute::new", "sand::prelude::Execute::new", "sand::prelude::cmd::Execute::new"],
        module = "sand::command",
        kind = "method",
        summary = "Create a new `Execute` builder with no sub-commands.",
        context = "Create a new `Execute` builder with no sub-commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A newly constructed `Execute` configured to create a new `Execute` builder with no sub-commands.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let execute = sand::command::Execute::new();\n}",
    )]
    pub fn new() -> Self {
        Self {
            operations: vec![],
            checks: vec![],
        }
    }

    fn next_index(&self) -> usize {
        self.operations.len()
    }

    fn check_selector(&mut self, kind: &'static str, value: &Selector) {
        self.checks.push(ExecuteCheck::Selector {
            index: self.next_index(),
            kind,
            value: value.clone(),
        });
    }

    fn check_vec3(&mut self, kind: &'static str, value: &Vec3) {
        self.checks.push(ExecuteCheck::Vec3 {
            index: self.next_index(),
            kind,
            value: value.clone(),
        });
    }

    fn check_block_pos(&mut self, kind: &'static str, value: &BlockPos) {
        self.checks.push(ExecuteCheck::BlockPos {
            index: self.next_index(),
            kind,
            value: value.clone(),
        });
    }

    fn check_objective(&mut self, kind: &'static str, value: &str) {
        self.checks.push(ExecuteCheck::Objective {
            index: self.next_index(),
            kind,
            value: value.to_string(),
        });
    }

    fn check_resource(
        &mut self,
        kind: &'static str,
        field: &'static str,
        value: &str,
        allow_tag: bool,
    ) {
        self.checks.push(ExecuteCheck::Resource {
            index: self.next_index(),
            kind,
            field,
            value: value.to_string(),
            allow_tag,
        });
    }

    fn check_single_holder(&mut self, kind: &'static str, value: impl Into<String>) -> String {
        let value = value.into();
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind,
            value: ScoreHolder::from_compat(value.clone()),
        });
        value
    }

    // ── Context sub-commands ──────────────────────────────────────────────────

    /// `as <selector>` — change the executing entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::as_",
        aliases = ["sand::cmd::Execute::as_", "sand::prelude::Execute::as_", "sand::prelude::cmd::Execute::as_"],
        module = "sand::command",
        kind = "method",
        summary = "`as <selector>` — change the executing entity.",
        context = "`as <selector>` — change the executing entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `as <selector>` — change the executing entity form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `as <selector>` — change the executing entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector)  {\n    let updated_execute = execute_value.as_(selector);\n}",
    )]
    pub fn as_(mut self, selector: Selector) -> Self {
        self.check_selector("as", &selector);
        self.operations.push(ExecuteOp::As(selector));
        self
    }

    /// `at <selector>` — change position and rotation to match the selected entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::at",
        aliases = ["sand::cmd::Execute::at", "sand::prelude::Execute::at", "sand::prelude::cmd::Execute::at"],
        module = "sand::command",
        kind = "method",
        summary = "`at <selector>` — change position and rotation to match the selected entity.",
        context = "`at <selector>` — change position and rotation to match the selected entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `at <selector>` — change position and rotation to match the selected entity form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `at <selector>` — change position and rotation to match the selected entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector)  {\n    let updated_execute = execute_value.at(selector);\n}",
    )]
    pub fn at(mut self, selector: Selector) -> Self {
        self.check_selector("at", &selector);
        self.operations.push(ExecuteOp::At(selector));
        self
    }

    /// `positioned <pos>` — change execution position to the given coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::positioned",
        aliases = ["sand::cmd::Execute::positioned", "sand::prelude::Execute::positioned", "sand::prelude::cmd::Execute::positioned"],
        module = "sand::command",
        kind = "method",
        summary = "`positioned <pos>` — change execution position to the given coordinates.",
        context = "`positioned <pos>` — change execution position to the given coordinates. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`pos` supplies the pos value used to emit the documented `positioned <pos>` — change execution position to the given coordinates form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `positioned <pos>` — change execution position to the given coordinates form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::Vec3)  {\n    let updated_execute = execute_value.positioned(pos);\n}",
    )]
    pub fn positioned(mut self, pos: Vec3) -> Self {
        self.check_vec3("positioned", &pos);
        self.operations.push(ExecuteOp::Positioned(pos));
        self
    }

    /// `positioned as <selector>` — change position to match the selected entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::positioned_as",
        aliases = ["sand::cmd::Execute::positioned_as", "sand::prelude::Execute::positioned_as", "sand::prelude::cmd::Execute::positioned_as"],
        module = "sand::command",
        kind = "method",
        summary = "`positioned as <selector>` — change position to match the selected entity.",
        context = "`positioned as <selector>` — change position to match the selected entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `positioned as <selector>` — change position to match the selected entity form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `positioned as <selector>` — change position to match the selected entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector)  {\n    let updated_execute = execute_value.positioned_as(selector);\n}",
    )]
    pub fn positioned_as(mut self, selector: Selector) -> Self {
        self.check_selector("positioned_as", &selector);
        self.operations.push(ExecuteOp::PositionedAs(selector));
        self
    }

    /// `rotated <rotation>` — change execution rotation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::rotated",
        aliases = ["sand::cmd::Execute::rotated", "sand::prelude::Execute::rotated", "sand::prelude::cmd::Execute::rotated"],
        module = "sand::command",
        kind = "method",
        summary = "`rotated <rotation>` — change execution rotation.",
        context = "`rotated <rotation>` — change execution rotation. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(rotation = "`rotation` supplies the rotation value used to emit the documented `rotated <rotation>` — change execution rotation form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `rotated <rotation>` — change execution rotation form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, rotation: sand::command::Rotation)  {\n    let updated_execute = execute_value.rotated(rotation);\n}",
    )]
    pub fn rotated(mut self, rotation: Rotation) -> Self {
        self.checks.push(ExecuteCheck::Rotation {
            index: self.next_index(),
            kind: "rotated",
            value: rotation.clone(),
        });
        self.operations.push(ExecuteOp::Rotated(rotation));
        self
    }

    /// `rotated as <selector>` — change rotation to match the selected entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::rotated_as",
        aliases = ["sand::cmd::Execute::rotated_as", "sand::prelude::Execute::rotated_as", "sand::prelude::cmd::Execute::rotated_as"],
        module = "sand::command",
        kind = "method",
        summary = "`rotated as <selector>` — change rotation to match the selected entity.",
        context = "`rotated as <selector>` — change rotation to match the selected entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `rotated as <selector>` — change rotation to match the selected entity form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `rotated as <selector>` — change rotation to match the selected entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector)  {\n    let updated_execute = execute_value.rotated_as(selector);\n}",
    )]
    pub fn rotated_as(mut self, selector: Selector) -> Self {
        self.check_selector("rotated_as", &selector);
        self.operations.push(ExecuteOp::RotatedAs(selector));
        self
    }

    /// `facing <pos>` — rotate execution to face a position in the world.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::facing",
        aliases = ["sand::cmd::Execute::facing", "sand::prelude::Execute::facing", "sand::prelude::cmd::Execute::facing"],
        module = "sand::command",
        kind = "method",
        summary = "`facing <pos>` — rotate execution to face a position in the world.",
        context = "`facing <pos>` — rotate execution to face a position in the world. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`pos` supplies the pos value used to emit the documented `facing <pos>` — rotate execution to face a position in the world form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `facing <pos>` — rotate execution to face a position in the world form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::Vec3)  {\n    let updated_execute = execute_value.facing(pos);\n}",
    )]
    pub fn facing(mut self, pos: Vec3) -> Self {
        self.check_vec3("facing", &pos);
        self.operations.push(ExecuteOp::Facing(pos));
        self
    }

    /// `facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::facing_entity",
        aliases = ["sand::cmd::Execute::facing_entity", "sand::prelude::Execute::facing_entity", "sand::prelude::cmd::Execute::facing_entity"],
        module = "sand::command",
        kind = "method",
        summary = "`facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point.",
        context = "`facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point form.", anchor = "`anchor` supplies the anchor value used to emit the documented `facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `facing entity <selector> <anchor>` — rotate execution to face an entity's anchor point form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector, anchor: sand::command::Anchor)  {\n    let updated_execute = execute_value.facing_entity(selector, anchor);\n}",
    )]
    pub fn facing_entity(mut self, selector: Selector, anchor: Anchor) -> Self {
        self.check_selector("facing_entity", &selector);
        self.operations.push(ExecuteOp::FacingEntity {
            target: selector,
            anchor,
        });
        self
    }

    /// `in <dimension>` — change dimension for subsequent commands.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::in_",
        aliases = ["sand::cmd::Execute::in_", "sand::prelude::Execute::in_", "sand::prelude::cmd::Execute::in_"],
        module = "sand::command",
        kind = "method",
        summary = "`in <dimension>` — change dimension for subsequent commands.",
        context = "`in <dimension>` — change dimension for subsequent commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(dimension = "`dimension` supplies the dimension value used to emit the documented `in <dimension>` — change dimension for subsequent commands form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `in <dimension>` — change dimension for subsequent commands form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, dimension: impl Into < String >)  {\n    let updated_execute = execute_value.in_(dimension);\n}",
    )]
    pub fn in_(mut self, dimension: impl Into<String>) -> Self {
        let dimension = dimension.into();
        self.check_resource("in", "dimension", &dimension, false);
        self.operations.push(ExecuteOp::In(dimension));
        self
    }

    /// `align <axes>` — snap coordinates to the block grid along specified axes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::align",
        aliases = ["sand::cmd::Execute::align", "sand::prelude::Execute::align", "sand::prelude::cmd::Execute::align"],
        module = "sand::command",
        kind = "method",
        summary = "`align <axes>` — snap coordinates to the block grid along specified axes.",
        context = "`align <axes>` — snap coordinates to the block grid along specified axes. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(axes = "`axes` supplies the axes value used to emit the documented `align <axes>` — snap coordinates to the block grid along specified axes form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `align <axes>` — snap coordinates to the block grid along specified axes form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, axes: sand::command::Swizzle)  {\n    let updated_execute = execute_value.align(axes);\n}",
    )]
    pub fn align(mut self, axes: Swizzle) -> Self {
        self.operations.push(ExecuteOp::Align(axes));
        self
    }

    /// `positioned over <heightmap>` — snap y-coordinate to the top of the given heightmap (1.19.4+).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::positioned_over",
        aliases = ["sand::cmd::Execute::positioned_over", "sand::prelude::Execute::positioned_over", "sand::prelude::cmd::Execute::positioned_over"],
        module = "sand::command",
        kind = "method",
        summary = "`positioned over <heightmap>` — snap y-coordinate to the top of the given heightmap (1.19.4+).",
        context = "`positioned over <heightmap>` — snap y-coordinate to the top of the given heightmap (1.19.4+). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(heightmap = "`heightmap` supplies the heightmap value used to emit the documented `positioned over <heightmap>` — snap y-coordinate to the top of the given heightmap (1.19.4+) form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `positioned over <heightmap>` — snap y-coordinate to the top of the given heightmap (1.19.4+) form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, heightmap: impl Into < String >)  {\n    let updated_execute = execute_value.positioned_over(heightmap);\n}",
    )]
    pub fn positioned_over(mut self, heightmap: impl Into<String>) -> Self {
        self.operations
            .push(ExecuteOp::PositionedOver(heightmap.into()));
        self
    }

    /// `anchored <anchor>` — change the anchor point for position calculations.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::anchored",
        aliases = ["sand::cmd::Execute::anchored", "sand::prelude::Execute::anchored", "sand::prelude::cmd::Execute::anchored"],
        module = "sand::command",
        kind = "method",
        summary = "`anchored <anchor>` — change the anchor point for position calculations.",
        context = "`anchored <anchor>` — change the anchor point for position calculations. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(anchor = "`anchor` supplies the anchor value used to emit the documented `anchored <anchor>` — change the anchor point for position calculations form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `anchored <anchor>` — change the anchor point for position calculations form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, anchor: sand::command::Anchor)  {\n    let updated_execute = execute_value.anchored(anchor);\n}",
    )]
    pub fn anchored(mut self, anchor: Anchor) -> Self {
        self.operations.push(ExecuteOp::Anchored(anchor));
        self
    }

    /// `on <relation>` — follow an entity relationship chain.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::on",
        aliases = ["sand::cmd::Execute::on", "sand::prelude::Execute::on", "sand::prelude::cmd::Execute::on"],
        module = "sand::command",
        kind = "method",
        summary = "`on <relation>` — follow an entity relationship chain.",
        context = "`on <relation>` — follow an entity relationship chain. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(relation = "`relation` supplies the relation value used to emit the documented `on <relation>` — follow an entity relationship chain form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `on <relation>` — follow an entity relationship chain form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, relation: impl Into < String >)  {\n    let updated_execute = execute_value.on(relation);\n}",
    )]
    pub fn on(mut self, relation: impl Into<String>) -> Self {
        self.operations.push(ExecuteOp::On(relation.into()));
        self
    }

    /// `summon <entity_type>` — summon an entity and execute as it immediately.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::summon",
        aliases = ["sand::cmd::Execute::summon", "sand::prelude::Execute::summon", "sand::prelude::cmd::Execute::summon"],
        module = "sand::command",
        kind = "method",
        summary = "`summon <entity_type>` — summon an entity and execute as it immediately.",
        context = "`summon <entity_type>` — summon an entity and execute as it immediately. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(entity_type = "`entity_type` supplies the entity type value used to emit the documented `summon <entity_type>` — summon an entity and execute as it immediately form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `summon <entity_type>` — summon an entity and execute as it immediately form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, entity_type: impl sand::command::IntoEntityType)  {\n    let updated_execute = execute_value.summon(entity_type);\n}",
    )]
    pub fn summon(mut self, entity_type: impl crate::selector::IntoEntityType) -> Self {
        let entity_type = entity_type.into_entity_type();
        self.check_resource("summon", "entity_type", &entity_type, false);
        self.operations.push(ExecuteOp::Summon(entity_type));
        self
    }

    // ── Condition sub-commands ────────────────────────────────────────────────

    /// `if entity <selector>` — execute only if the selector matches at least one entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_entity",
        aliases = ["sand::cmd::Execute::if_entity", "sand::prelude::Execute::if_entity", "sand::prelude::cmd::Execute::if_entity"],
        module = "sand::command",
        kind = "method",
        summary = "`if entity <selector>` — execute only if the selector matches at least one entity.",
        context = "`if entity <selector>` — execute only if the selector matches at least one entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `if entity <selector>` — execute only if the selector matches at least one entity form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if entity <selector>` — execute only if the selector matches at least one entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector)  {\n    let updated_execute = execute_value.if_entity(selector);\n}",
    )]
    pub fn if_entity(mut self, selector: Selector) -> Self {
        self.check_selector("if_entity", &selector);
        self.operations
            .push(ExecuteOp::If(ConditionIr::Entity(selector)));
        self
    }

    /// `unless entity <selector>` — execute only if the selector matches NO entities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_entity",
        aliases = ["sand::cmd::Execute::unless_entity", "sand::prelude::Execute::unless_entity", "sand::prelude::cmd::Execute::unless_entity"],
        module = "sand::command",
        kind = "method",
        summary = "`unless entity <selector>` — execute only if the selector matches NO entities.",
        context = "`unless entity <selector>` — execute only if the selector matches NO entities. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `unless entity <selector>` — execute only if the selector matches NO entities form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless entity <selector>` — execute only if the selector matches NO entities form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector)  {\n    let updated_execute = execute_value.unless_entity(selector);\n}",
    )]
    pub fn unless_entity(mut self, selector: Selector) -> Self {
        self.check_selector("unless_entity", &selector);
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::Entity(selector)));
        self
    }

    /// `if entity @s[team=<team>]` — continue only if the current entity is on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_on_team",
        aliases = ["sand::cmd::Execute::if_on_team", "sand::prelude::Execute::if_on_team", "sand::prelude::cmd::Execute::if_on_team"],
        module = "sand::command",
        kind = "method",
        summary = "`if entity @s[team=<team>]` — continue only if the current entity is on the given team.",
        context = "`if entity @s[team=<team>]` — continue only if the current entity is on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the team value used to emit the documented `if entity @s[team=<team>]` — continue only if the current entity is on the given team form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if entity @s[team=<team>]` — continue only if the current entity is on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, team: impl Into < String >)  {\n    let updated_execute = execute_value.if_on_team(team);\n}",
    )]
    pub fn if_on_team(mut self, team: impl Into<String>) -> Self {
        self.operations
            .push(ExecuteOp::If(ConditionIr::Team(team.into())));
        self
    }

    /// `unless entity @s[team=<team>]` — skip if the current entity is on the given team.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_on_team",
        aliases = ["sand::cmd::Execute::unless_on_team", "sand::prelude::Execute::unless_on_team", "sand::prelude::cmd::Execute::unless_on_team"],
        module = "sand::command",
        kind = "method",
        summary = "`unless entity @s[team=<team>]` — skip if the current entity is on the given team.",
        context = "`unless entity @s[team=<team>]` — skip if the current entity is on the given team. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(team = "`team` supplies the team value used to emit the documented `unless entity @s[team=<team>]` — skip if the current entity is on the given team form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless entity @s[team=<team>]` — skip if the current entity is on the given team form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, team: impl Into < String >)  {\n    let updated_execute = execute_value.unless_on_team(team);\n}",
    )]
    pub fn unless_on_team(mut self, team: impl Into<String>) -> Self {
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::Team(team.into())));
        self
    }

    /// `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score",
        aliases = ["sand::cmd::Execute::if_score", "sand::prelude::Execute::if_score", "sand::prelude::cmd::Execute::if_score"],
        module = "sand::command",
        kind = "method",
        summary = "`if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal.",
        context = "`if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`a` provides the Minecraft target selection used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal form.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal form.", b = "`b` provides the Minecraft target selection used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal form.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue only if the two scores are equal form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: sand::command::Selector, a_obj: impl Into < String >, b: sand::command::Selector, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.if_score(a, a_obj, b, b_obj);\n}",
    )]
    pub fn if_score(
        mut self,
        a: Selector,
        a_obj: impl Into<String>,
        b: Selector,
        b_obj: impl Into<String>,
    ) -> Self {
        self.check_selector("if_score.left", &a);
        self.check_selector("if_score.right", &b);
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "if_score.left",
            value: ScoreHolder::entity(a.clone()),
        });
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "if_score.right",
            value: ScoreHolder::entity(b.clone()),
        });
        let a_obj = a_obj.into();
        let b_obj = b_obj.into();
        self.check_objective("if_score.left", &a_obj);
        self.check_objective("if_score.right", &b_obj);
        self.operations
            .push(ExecuteOp::If(ConditionIr::ScoreCompare {
                left: ScoreHolder::entity(a),
                left_objective: a_obj,
                op: ScoreCmp::Eq,
                right: ScoreHolder::entity(b),
                right_objective: b_obj,
            }));
        self
    }

    /// `unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score",
        aliases = ["sand::cmd::Execute::unless_score", "sand::prelude::Execute::unless_score", "sand::prelude::cmd::Execute::unless_score"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal.",
        context = "`unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(primary_selector = "`primary_selector` provides the Minecraft target selection used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal form.", primary = "`primary` supplies the primary value used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal form.", secondary_selector = "`secondary_selector` provides the Minecraft target selection used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal form.", secondary = "`secondary` supplies the secondary value used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if two scores are equal form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, primary_selector: sand::command::Selector, primary: impl Into < String >, secondary_selector: sand::command::Selector, secondary: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score(primary_selector, primary, secondary_selector, secondary);\n}",
    )]
    pub fn unless_score(
        mut self,
        primary_selector: Selector,
        primary: impl Into<String>,
        secondary_selector: Selector,
        secondary: impl Into<String>,
    ) -> Self {
        self.check_selector("unless_score.left", &primary_selector);
        self.check_selector("unless_score.right", &secondary_selector);
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "unless_score.left",
            value: ScoreHolder::entity(primary_selector.clone()),
        });
        self.checks.push(ExecuteCheck::SingleHolder {
            index: self.next_index(),
            kind: "unless_score.right",
            value: ScoreHolder::entity(secondary_selector.clone()),
        });
        let primary = primary.into();
        let secondary = secondary.into();
        self.check_objective("unless_score.left", &primary);
        self.check_objective("unless_score.right", &secondary);
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::ScoreCompare {
                left: ScoreHolder::entity(primary_selector),
                left_objective: primary,
                op: ScoreCmp::Eq,
                right: ScoreHolder::entity(secondary_selector),
                right_objective: secondary,
            }));
        self
    }

    /// `if block <pos> <block>` — execute only if the block at `pos` matches.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_block",
        aliases = ["sand::cmd::Execute::if_block", "sand::prelude::Execute::if_block", "sand::prelude::cmd::Execute::if_block"],
        module = "sand::command",
        kind = "method",
        summary = "`if block <pos> <block>` — execute only if the block at `pos` matches.",
        context = "`if block <pos> <block>` — execute only if the block at `pos` matches. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`if block <pos> <block>` — execute only if the block at `pos` matches.", block = "`block` provides the block value or block predicate used to emit the documented `if block <pos> <block>` — execute only if the block at `pos` matches form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if block <pos> <block>` — execute only if the block at `pos` matches form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, block: impl Into < String >)  {\n    let updated_execute = execute_value.if_block(pos, block);\n}",
    )]
    pub fn if_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.check_block_pos("if_block", &pos);
        self.operations.push(ExecuteOp::If(ConditionIr::Block {
            position: pos,
            block: block.into(),
        }));
        self
    }

    /// `unless block <pos> <block>` — execute only if the block at `pos` does NOT match.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_block",
        aliases = ["sand::cmd::Execute::unless_block", "sand::prelude::Execute::unless_block", "sand::prelude::cmd::Execute::unless_block"],
        module = "sand::command",
        kind = "method",
        summary = "`unless block <pos> <block>` — execute only if the block at `pos` does NOT match.",
        context = "`unless block <pos> <block>` — execute only if the block at `pos` does NOT match. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`unless block <pos> <block>` — execute only if the block at `pos` does NOT match.", block = "`block` provides the block value or block predicate used to emit the documented `unless block <pos> <block>` — execute only if the block at `pos` does NOT match form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless block <pos> <block>` — execute only if the block at `pos` does NOT match form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, block: impl Into < String >)  {\n    let updated_execute = execute_value.unless_block(pos, block);\n}",
    )]
    pub fn unless_block(mut self, pos: BlockPos, block: impl Into<String>) -> Self {
        self.check_block_pos("unless_block", &pos);
        self.operations.push(ExecuteOp::Unless(ConditionIr::Block {
            position: pos,
            block: block.into(),
        }));
        self
    }

    /// `if score <holder> <obj> matches <range>` — execute if a score falls within the range.
    ///
    /// Range can be `"5"` (exact), `"5.."` (5 or more), `"..5"` (5 or less), or `"1..10"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score_matches",
        aliases = ["sand::cmd::Execute::if_score_matches", "sand::prelude::Execute::if_score_matches", "sand::prelude::cmd::Execute::if_score_matches"],
        module = "sand::command",
        kind = "method",
        summary = "`if score <holder> <obj> matches <range>` — execute if a score falls within the range.",
        context = "`if score <holder> <obj> matches <range>` — execute if a score falls within the range. Range can be `\"5\"` (exact), `\"5..\"` (5 or more), `\"..5\"` (5 or less), or `\"1..10\"`.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the holder value used to emit the documented `if score <holder> <obj> matches <range>` — execute if a score falls within the range form.", objective = "`objective` supplies the objective value used to emit the documented `if score <holder> <obj> matches <range>` — execute if a score falls within the range form.", range = "`range` supplies the range value used to emit the documented `if score <holder> <obj> matches <range>` — execute if a score falls within the range form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score <holder> <obj> matches <range>` — execute if a score falls within the range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, holder: impl Into < String >, objective: impl Into < String >, range: impl Into < String >)  {\n    let updated_execute = execute_value.if_score_matches(holder, objective, range);\n}",
    )]
    pub fn if_score_matches(
        mut self,
        holder: impl Into<String>,
        objective: impl Into<String>,
        range: impl Into<String>,
    ) -> Self {
        let holder = self.check_single_holder("if_score_matches", holder);
        let objective = objective.into();
        let range = range.into();
        self.check_objective("if_score_matches", &objective);
        self.checks.push(ExecuteCheck::ScoreRange {
            index: self.next_index(),
            kind: "if_score_matches",
            value: range.clone(),
        });
        self.operations
            .push(ExecuteOp::If(ConditionIr::ScoreMatches {
                holder: ScoreHolder::from_compat(holder),
                objective,
                range,
            }));
        self
    }

    /// `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score_matches",
        aliases = ["sand::cmd::Execute::unless_score_matches", "sand::prelude::Execute::unless_score_matches", "sand::prelude::cmd::Execute::unless_score_matches"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range.",
        context = "`unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the holder value used to emit the documented `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range form.", objective = "`objective` supplies the objective value used to emit the documented `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range form.", range = "`range` supplies the range value used to emit the documented `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score <holder> <obj> matches <range>` — execute if a score falls OUTSIDE the range form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, holder: impl Into < String >, objective: impl Into < String >, range: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score_matches(holder, objective, range);\n}",
    )]
    pub fn unless_score_matches(
        mut self,
        holder: impl Into<String>,
        objective: impl Into<String>,
        range: impl Into<String>,
    ) -> Self {
        let holder = self.check_single_holder("unless_score_matches", holder);
        let objective = objective.into();
        let range = range.into();
        self.check_objective("unless_score_matches", &objective);
        self.checks.push(ExecuteCheck::ScoreRange {
            index: self.next_index(),
            kind: "unless_score_matches",
            value: range.clone(),
        });
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::ScoreMatches {
                holder: ScoreHolder::from_compat(holder),
                objective,
                range,
            }));
        self
    }

    /// `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score_compare",
        aliases = ["sand::cmd::Execute::if_score_compare", "sand::prelude::Execute::if_score_compare", "sand::prelude::cmd::Execute::if_score_compare"],
        module = "sand::command",
        kind = "method",
        summary = "`if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores.",
        context = "`if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`a` supplies the a value used to emit the documented `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores form.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores form.", cmp = "`cmp` supplies the cmp value used to emit the documented `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores form.", b = "`b` supplies the b value used to emit the documented `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores form.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score <a> <a_obj> <cmp> <b> <b_obj>` — compare two scores form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, cmp: sand::command::ScoreCmp, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.if_score_compare(a, a_obj, cmp, b, b_obj);\n}",
    )]
    pub fn if_score_compare(
        mut self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        cmp: ScoreCmp,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        let a = self.check_single_holder("if_score_compare.left", a);
        let b = self.check_single_holder("if_score_compare.right", b);
        let a_obj = a_obj.into();
        let b_obj = b_obj.into();
        self.check_objective("if_score_compare.left", &a_obj);
        self.check_objective("if_score_compare.right", &b_obj);
        self.operations
            .push(ExecuteOp::If(ConditionIr::ScoreCompare {
                left: ScoreHolder::from_compat(a),
                left_objective: a_obj,
                op: cmp,
                right: ScoreHolder::from_compat(b),
                right_objective: b_obj,
            }));
        self
    }

    /// `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score_compare",
        aliases = ["sand::cmd::Execute::unless_score_compare", "sand::prelude::Execute::unless_score_compare", "sand::prelude::cmd::Execute::unless_score_compare"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true.",
        context = "`unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`a` supplies the a value used to emit the documented `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true form.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true form.", cmp = "`cmp` supplies the cmp value used to emit the documented `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true form.", b = "`b` supplies the b value used to emit the documented `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true form.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score <a> <a_obj> <cmp> <b> <b_obj>` — skip if the comparison is true form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, cmp: sand::command::ScoreCmp, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score_compare(a, a_obj, cmp, b, b_obj);\n}",
    )]
    pub fn unless_score_compare(
        mut self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        cmp: ScoreCmp,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        let a = self.check_single_holder("unless_score_compare.left", a);
        let b = self.check_single_holder("unless_score_compare.right", b);
        let a_obj = a_obj.into();
        let b_obj = b_obj.into();
        self.check_objective("unless_score_compare.left", &a_obj);
        self.check_objective("unless_score_compare.right", &b_obj);
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::ScoreCompare {
                left: ScoreHolder::from_compat(a),
                left_objective: a_obj,
                op: cmp,
                right: ScoreHolder::from_compat(b),
                right_objective: b_obj,
            }));
        self
    }

    // ── Score comparison shorthands ───────────────────────────────────────────

    /// `if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score_eq",
        aliases = ["sand::cmd::Execute::if_score_eq", "sand::prelude::Execute::if_score_eq", "sand::prelude::cmd::Execute::if_score_eq"],
        module = "sand::command",
        kind = "method",
        summary = "`if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal.",
        context = "`if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`a` supplies the a value used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal form.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal form.", b = "`b` supplies the b value used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal form.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score <a> <a_obj> = <b> <b_obj>` — continue if scores are equal form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.if_score_eq(a, a_obj, b, b_obj);\n}",
    )]
    pub fn if_score_eq(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Eq, b, b_obj)
    }

    /// `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score_eq",
        aliases = ["sand::cmd::Execute::unless_score_eq", "sand::prelude::Execute::unless_score_eq", "sand::prelude::cmd::Execute::unless_score_eq"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal.",
        context = "`unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`a` supplies the a value used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal form.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal form.", b = "`b` supplies the b value used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal form.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score <a> <a_obj> = <b> <b_obj>` — skip if scores are equal form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score_eq(a, a_obj, b, b_obj);\n}",
    )]
    pub fn unless_score_eq(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Eq, b, b_obj)
    }

    /// `if score ... < ...` — continue if `a` is strictly less than `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score_lt",
        aliases = ["sand::cmd::Execute::if_score_lt", "sand::prelude::Execute::if_score_lt", "sand::prelude::cmd::Execute::if_score_lt"],
        module = "sand::command",
        kind = "method",
        summary = "`if score ... < ...` — continue if `a` is strictly less than `b`.",
        context = "`if score ... < ...` — continue if `a` is strictly less than `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`if score ... < ...` — continue if `a` is strictly less than `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `if score ... < ...` — continue if `a` is strictly less than `b` form.", b = "`if score ... < ...` — continue if `a` is strictly less than `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `if score ... < ...` — continue if `a` is strictly less than `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score ... < ...` — continue if `a` is strictly less than `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.if_score_lt(a, a_obj, b, b_obj);\n}",
    )]
    pub fn if_score_lt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Lt, b, b_obj)
    }

    /// `unless score ... < ...` — skip if `a` is strictly less than `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score_lt",
        aliases = ["sand::cmd::Execute::unless_score_lt", "sand::prelude::Execute::unless_score_lt", "sand::prelude::cmd::Execute::unless_score_lt"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score ... < ...` — skip if `a` is strictly less than `b`.",
        context = "`unless score ... < ...` — skip if `a` is strictly less than `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`unless score ... < ...` — skip if `a` is strictly less than `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `unless score ... < ...` — skip if `a` is strictly less than `b` form.", b = "`unless score ... < ...` — skip if `a` is strictly less than `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `unless score ... < ...` — skip if `a` is strictly less than `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score ... < ...` — skip if `a` is strictly less than `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score_lt(a, a_obj, b, b_obj);\n}",
    )]
    pub fn unless_score_lt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Lt, b, b_obj)
    }

    /// `if score ... <= ...` — continue if `a` is less than or equal to `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score_lte",
        aliases = ["sand::cmd::Execute::if_score_lte", "sand::prelude::Execute::if_score_lte", "sand::prelude::cmd::Execute::if_score_lte"],
        module = "sand::command",
        kind = "method",
        summary = "`if score ... <= ...` — continue if `a` is less than or equal to `b`.",
        context = "`if score ... <= ...` — continue if `a` is less than or equal to `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`if score ... <= ...` — continue if `a` is less than or equal to `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `if score ... <= ...` — continue if `a` is less than or equal to `b` form.", b = "`if score ... <= ...` — continue if `a` is less than or equal to `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `if score ... <= ...` — continue if `a` is less than or equal to `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score ... <= ...` — continue if `a` is less than or equal to `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.if_score_lte(a, a_obj, b, b_obj);\n}",
    )]
    pub fn if_score_lte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Le, b, b_obj)
    }

    /// `unless score ... <= ...` — skip if `a` is less than or equal to `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score_lte",
        aliases = ["sand::cmd::Execute::unless_score_lte", "sand::prelude::Execute::unless_score_lte", "sand::prelude::cmd::Execute::unless_score_lte"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score ... <= ...` — skip if `a` is less than or equal to `b`.",
        context = "`unless score ... <= ...` — skip if `a` is less than or equal to `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`unless score ... <= ...` — skip if `a` is less than or equal to `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `unless score ... <= ...` — skip if `a` is less than or equal to `b` form.", b = "`unless score ... <= ...` — skip if `a` is less than or equal to `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `unless score ... <= ...` — skip if `a` is less than or equal to `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score ... <= ...` — skip if `a` is less than or equal to `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score_lte(a, a_obj, b, b_obj);\n}",
    )]
    pub fn unless_score_lte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Le, b, b_obj)
    }

    /// `if score ... > ...` — continue if `a` is strictly greater than `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score_gt",
        aliases = ["sand::cmd::Execute::if_score_gt", "sand::prelude::Execute::if_score_gt", "sand::prelude::cmd::Execute::if_score_gt"],
        module = "sand::command",
        kind = "method",
        summary = "`if score ... > ...` — continue if `a` is strictly greater than `b`.",
        context = "`if score ... > ...` — continue if `a` is strictly greater than `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`if score ... > ...` — continue if `a` is strictly greater than `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `if score ... > ...` — continue if `a` is strictly greater than `b` form.", b = "`if score ... > ...` — continue if `a` is strictly greater than `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `if score ... > ...` — continue if `a` is strictly greater than `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score ... > ...` — continue if `a` is strictly greater than `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.if_score_gt(a, a_obj, b, b_obj);\n}",
    )]
    pub fn if_score_gt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Gt, b, b_obj)
    }

    /// `unless score ... > ...` — skip if `a` is strictly greater than `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score_gt",
        aliases = ["sand::cmd::Execute::unless_score_gt", "sand::prelude::Execute::unless_score_gt", "sand::prelude::cmd::Execute::unless_score_gt"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score ... > ...` — skip if `a` is strictly greater than `b`.",
        context = "`unless score ... > ...` — skip if `a` is strictly greater than `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`unless score ... > ...` — skip if `a` is strictly greater than `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `unless score ... > ...` — skip if `a` is strictly greater than `b` form.", b = "`unless score ... > ...` — skip if `a` is strictly greater than `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `unless score ... > ...` — skip if `a` is strictly greater than `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score ... > ...` — skip if `a` is strictly greater than `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score_gt(a, a_obj, b, b_obj);\n}",
    )]
    pub fn unless_score_gt(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Gt, b, b_obj)
    }

    /// `if score ... >= ...` — continue if `a` is greater than or equal to `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_score_gte",
        aliases = ["sand::cmd::Execute::if_score_gte", "sand::prelude::Execute::if_score_gte", "sand::prelude::cmd::Execute::if_score_gte"],
        module = "sand::command",
        kind = "method",
        summary = "`if score ... >= ...` — continue if `a` is greater than or equal to `b`.",
        context = "`if score ... >= ...` — continue if `a` is greater than or equal to `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`if score ... >= ...` — continue if `a` is greater than or equal to `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `if score ... >= ...` — continue if `a` is greater than or equal to `b` form.", b = "`if score ... >= ...` — continue if `a` is greater than or equal to `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `if score ... >= ...` — continue if `a` is greater than or equal to `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if score ... >= ...` — continue if `a` is greater than or equal to `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.if_score_gte(a, a_obj, b, b_obj);\n}",
    )]
    pub fn if_score_gte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.if_score_compare(a, a_obj, ScoreCmp::Ge, b, b_obj)
    }

    /// `unless score ... >= ...` — skip if `a` is greater than or equal to `b`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_score_gte",
        aliases = ["sand::cmd::Execute::unless_score_gte", "sand::prelude::Execute::unless_score_gte", "sand::prelude::cmd::Execute::unless_score_gte"],
        module = "sand::command",
        kind = "method",
        summary = "`unless score ... >= ...` — skip if `a` is greater than or equal to `b`.",
        context = "`unless score ... >= ...` — skip if `a` is greater than or equal to `b`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(a = "`unless score ... >= ...` — skip if `a` is greater than or equal to `b`.", a_obj = "`a_obj` supplies the a obj value used to emit the documented `unless score ... >= ...` — skip if `a` is greater than or equal to `b` form.", b = "`unless score ... >= ...` — skip if `a` is greater than or equal to `b`.", b_obj = "`b_obj` supplies the b obj value used to emit the documented `unless score ... >= ...` — skip if `a` is greater than or equal to `b` form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless score ... >= ...` — skip if `a` is greater than or equal to `b` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, a: impl Into < String >, a_obj: impl Into < String >, b: impl Into < String >, b_obj: impl Into < String >)  {\n    let updated_execute = execute_value.unless_score_gte(a, a_obj, b, b_obj);\n}",
    )]
    pub fn unless_score_gte(
        self,
        a: impl Into<String>,
        a_obj: impl Into<String>,
        b: impl Into<String>,
        b_obj: impl Into<String>,
    ) -> Self {
        self.unless_score_compare(a, a_obj, ScoreCmp::Ge, b, b_obj)
    }

    // ── Data / NBT conditions ─────────────────────────────────────────────────

    /// `if data entity <selector> <path>` — continue if entity NBT has a value at `path`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_data_entity",
        aliases = ["sand::cmd::Execute::if_data_entity", "sand::prelude::Execute::if_data_entity", "sand::prelude::cmd::Execute::if_data_entity"],
        module = "sand::command",
        kind = "method",
        summary = "`if data entity <selector> <path>` — continue if entity NBT has a value at `path`.",
        context = "`if data entity <selector> <path>` — continue if entity NBT has a value at `path`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `if data entity <selector> <path>` — continue if entity NBT has a value at `path` form.", path = "`if data entity <selector> <path>` — continue if entity NBT has a value at `path`."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if data entity <selector> <path>` — continue if entity NBT has a value at `path` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector, path: impl Into < String >)  {\n    let updated_execute = execute_value.if_data_entity(selector, path);\n}",
    )]
    pub fn if_data_entity(mut self, selector: Selector, path: impl Into<String>) -> Self {
        self.check_selector("if_data_entity", &selector);
        self.operations.push(ExecuteOp::If(ConditionIr::Data {
            target: DataTarget::entity(selector),
            path: path.into(),
        }));
        self
    }

    /// `unless data entity <selector> <path>` — skip if entity NBT has a value at `path`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_data_entity",
        aliases = ["sand::cmd::Execute::unless_data_entity", "sand::prelude::Execute::unless_data_entity", "sand::prelude::cmd::Execute::unless_data_entity"],
        module = "sand::command",
        kind = "method",
        summary = "`unless data entity <selector> <path>` — skip if entity NBT has a value at `path`.",
        context = "`unless data entity <selector> <path>` — skip if entity NBT has a value at `path`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `unless data entity <selector> <path>` — skip if entity NBT has a value at `path` form.", path = "`unless data entity <selector> <path>` — skip if entity NBT has a value at `path`."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless data entity <selector> <path>` — skip if entity NBT has a value at `path` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector, path: impl Into < String >)  {\n    let updated_execute = execute_value.unless_data_entity(selector, path);\n}",
    )]
    pub fn unless_data_entity(mut self, selector: Selector, path: impl Into<String>) -> Self {
        self.check_selector("unless_data_entity", &selector);
        self.operations.push(ExecuteOp::Unless(ConditionIr::Data {
            target: DataTarget::entity(selector),
            path: path.into(),
        }));
        self
    }

    /// `if data block <pos> <path>` — continue if block NBT has a value at `path`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_data_block",
        aliases = ["sand::cmd::Execute::if_data_block", "sand::prelude::Execute::if_data_block", "sand::prelude::cmd::Execute::if_data_block"],
        module = "sand::command",
        kind = "method",
        summary = "`if data block <pos> <path>` — continue if block NBT has a value at `path`.",
        context = "`if data block <pos> <path>` — continue if block NBT has a value at `path`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`pos` supplies the pos value used to emit the documented `if data block <pos> <path>` — continue if block NBT has a value at `path` form.", path = "`if data block <pos> <path>` — continue if block NBT has a value at `path`."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if data block <pos> <path>` — continue if block NBT has a value at `path` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, path: impl Into < String >)  {\n    let updated_execute = execute_value.if_data_block(pos, path);\n}",
    )]
    pub fn if_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.check_block_pos("if_data_block", &pos);
        self.operations.push(ExecuteOp::If(ConditionIr::Data {
            target: DataTarget::block(pos),
            path: path.into(),
        }));
        self
    }

    /// `unless data block <pos> <path>` — skip if block NBT has a value at `path`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_data_block",
        aliases = ["sand::cmd::Execute::unless_data_block", "sand::prelude::Execute::unless_data_block", "sand::prelude::cmd::Execute::unless_data_block"],
        module = "sand::command",
        kind = "method",
        summary = "`unless data block <pos> <path>` — skip if block NBT has a value at `path`.",
        context = "`unless data block <pos> <path>` — skip if block NBT has a value at `path`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`pos` supplies the pos value used to emit the documented `unless data block <pos> <path>` — skip if block NBT has a value at `path` form.", path = "`unless data block <pos> <path>` — skip if block NBT has a value at `path`."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless data block <pos> <path>` — skip if block NBT has a value at `path` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, path: impl Into < String >)  {\n    let updated_execute = execute_value.unless_data_block(pos, path);\n}",
    )]
    pub fn unless_data_block(mut self, pos: BlockPos, path: impl Into<String>) -> Self {
        self.check_block_pos("unless_data_block", &pos);
        self.operations.push(ExecuteOp::Unless(ConditionIr::Data {
            target: DataTarget::block(pos),
            path: path.into(),
        }));
        self
    }

    /// `if data storage <source> <path>` — continue if storage has a value at `path`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_data_storage",
        aliases = ["sand::cmd::Execute::if_data_storage", "sand::prelude::Execute::if_data_storage", "sand::prelude::cmd::Execute::if_data_storage"],
        module = "sand::command",
        kind = "method",
        summary = "`if data storage <source> <path>` — continue if storage has a value at `path`.",
        context = "`if data storage <source> <path>` — continue if storage has a value at `path`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(source = "`source` supplies the source value used to emit the documented `if data storage <source> <path>` — continue if storage has a value at `path` form.", path = "`if data storage <source> <path>` — continue if storage has a value at `path`."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if data storage <source> <path>` — continue if storage has a value at `path` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, source: impl Into < String >, path: impl Into < String >)  {\n    let updated_execute = execute_value.if_data_storage(source, path);\n}",
    )]
    pub fn if_data_storage(mut self, source: impl Into<String>, path: impl Into<String>) -> Self {
        let source = source.into();
        self.check_resource("if_data_storage", "storage", &source, false);
        self.operations.push(ExecuteOp::If(ConditionIr::Data {
            target: DataTarget::storage(source),
            path: path.into(),
        }));
        self
    }

    /// `unless data storage <source> <path>` — skip if storage has a value at `path`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_data_storage",
        aliases = ["sand::cmd::Execute::unless_data_storage", "sand::prelude::Execute::unless_data_storage", "sand::prelude::cmd::Execute::unless_data_storage"],
        module = "sand::command",
        kind = "method",
        summary = "`unless data storage <source> <path>` — skip if storage has a value at `path`.",
        context = "`unless data storage <source> <path>` — skip if storage has a value at `path`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(source = "`source` supplies the source value used to emit the documented `unless data storage <source> <path>` — skip if storage has a value at `path` form.", path = "`unless data storage <source> <path>` — skip if storage has a value at `path`."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless data storage <source> <path>` — skip if storage has a value at `path` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, source: impl Into < String >, path: impl Into < String >)  {\n    let updated_execute = execute_value.unless_data_storage(source, path);\n}",
    )]
    pub fn unless_data_storage(
        mut self,
        source: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        let source = source.into();
        self.check_resource("unless_data_storage", "storage", &source, false);
        self.operations.push(ExecuteOp::Unless(ConditionIr::Data {
            target: DataTarget::storage(source),
            path: path.into(),
        }));
        self
    }

    // ── World conditions ──────────────────────────────────────────────────────

    /// `if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_biome",
        aliases = ["sand::cmd::Execute::if_biome", "sand::prelude::Execute::if_biome", "sand::prelude::cmd::Execute::if_biome"],
        module = "sand::command",
        kind = "method",
        summary = "`if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+).",
        context = "`if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+).", biome = "`biome` supplies the biome value used to emit the documented `if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+) form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if biome <pos> <biome>` — continue if the biome at `pos` matches (1.19.4+) form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, biome: impl Into < String >)  {\n    let updated_execute = execute_value.if_biome(pos, biome);\n}",
    )]
    pub fn if_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        let biome = biome.into();
        self.check_block_pos("if_biome", &pos);
        self.check_resource("if_biome", "biome", &biome, true);
        self.operations.push(ExecuteOp::If(ConditionIr::Biome {
            position: pos,
            biome,
        }));
        self
    }

    /// `unless biome <pos> <biome>` — skip if the biome at `pos` matches.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_biome",
        aliases = ["sand::cmd::Execute::unless_biome", "sand::prelude::Execute::unless_biome", "sand::prelude::cmd::Execute::unless_biome"],
        module = "sand::command",
        kind = "method",
        summary = "`unless biome <pos> <biome>` — skip if the biome at `pos` matches.",
        context = "`unless biome <pos> <biome>` — skip if the biome at `pos` matches. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`unless biome <pos> <biome>` — skip if the biome at `pos` matches.", biome = "`biome` supplies the biome value used to emit the documented `unless biome <pos> <biome>` — skip if the biome at `pos` matches form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless biome <pos> <biome>` — skip if the biome at `pos` matches form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, biome: impl Into < String >)  {\n    let updated_execute = execute_value.unless_biome(pos, biome);\n}",
    )]
    pub fn unless_biome(mut self, pos: BlockPos, biome: impl Into<String>) -> Self {
        let biome = biome.into();
        self.check_block_pos("unless_biome", &pos);
        self.check_resource("unless_biome", "biome", &biome, true);
        self.operations.push(ExecuteOp::Unless(ConditionIr::Biome {
            position: pos,
            biome,
        }));
        self
    }

    /// `if dimension <dimension>` — continue if executing in the given dimension (1.21+).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_dimension",
        aliases = ["sand::cmd::Execute::if_dimension", "sand::prelude::Execute::if_dimension", "sand::prelude::cmd::Execute::if_dimension"],
        module = "sand::command",
        kind = "method",
        summary = "`if dimension <dimension>` — continue if executing in the given dimension (1.21+).",
        context = "`if dimension <dimension>` — continue if executing in the given dimension (1.21+). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(dimension = "`dimension` supplies the dimension value used to emit the documented `if dimension <dimension>` — continue if executing in the given dimension (1.21+) form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if dimension <dimension>` — continue if executing in the given dimension (1.21+) form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, dimension: impl Into < String >)  {\n    let updated_execute = execute_value.if_dimension(dimension);\n}",
    )]
    pub fn if_dimension(mut self, dimension: impl Into<String>) -> Self {
        let dimension = dimension.into();
        self.check_resource("if_dimension", "dimension", &dimension, false);
        self.operations
            .push(ExecuteOp::If(ConditionIr::Dimension(dimension)));
        self
    }

    /// `unless dimension <dimension>` — skip if executing in the given dimension (1.21+).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_dimension",
        aliases = ["sand::cmd::Execute::unless_dimension", "sand::prelude::Execute::unless_dimension", "sand::prelude::cmd::Execute::unless_dimension"],
        module = "sand::command",
        kind = "method",
        summary = "`unless dimension <dimension>` — skip if executing in the given dimension (1.21+).",
        context = "`unless dimension <dimension>` — skip if executing in the given dimension (1.21+). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(dimension = "`dimension` supplies the dimension value used to emit the documented `unless dimension <dimension>` — skip if executing in the given dimension (1.21+) form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless dimension <dimension>` — skip if executing in the given dimension (1.21+) form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, dimension: impl Into < String >)  {\n    let updated_execute = execute_value.unless_dimension(dimension);\n}",
    )]
    pub fn unless_dimension(mut self, dimension: impl Into<String>) -> Self {
        let dimension = dimension.into();
        self.check_resource("unless_dimension", "dimension", &dimension, false);
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::Dimension(dimension)));
        self
    }

    /// `if loaded <pos>` — continue only if the chunk at `pos` is fully loaded.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_loaded",
        aliases = ["sand::cmd::Execute::if_loaded", "sand::prelude::Execute::if_loaded", "sand::prelude::cmd::Execute::if_loaded"],
        module = "sand::command",
        kind = "method",
        summary = "`if loaded <pos>` — continue only if the chunk at `pos` is fully loaded.",
        context = "`if loaded <pos>` — continue only if the chunk at `pos` is fully loaded. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`if loaded <pos>` — continue only if the chunk at `pos` is fully loaded."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if loaded <pos>` — continue only if the chunk at `pos` is fully loaded form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos)  {\n    let updated_execute = execute_value.if_loaded(pos);\n}",
    )]
    pub fn if_loaded(mut self, pos: BlockPos) -> Self {
        self.check_block_pos("if_loaded", &pos);
        self.operations
            .push(ExecuteOp::If(ConditionIr::Loaded(pos)));
        self
    }

    /// `unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_loaded",
        aliases = ["sand::cmd::Execute::unless_loaded", "sand::prelude::Execute::unless_loaded", "sand::prelude::cmd::Execute::unless_loaded"],
        module = "sand::command",
        kind = "method",
        summary = "`unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded.",
        context = "`unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless loaded <pos>` — skip if the chunk at `pos` is NOT fully loaded form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos)  {\n    let updated_execute = execute_value.unless_loaded(pos);\n}",
    )]
    pub fn unless_loaded(mut self, pos: BlockPos) -> Self {
        self.check_block_pos("unless_loaded", &pos);
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::Loaded(pos)));
        self
    }

    /// `if items entity <selector> <slot> <item>` — execute if an entity has a matching item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_items_entity",
        aliases = ["sand::cmd::Execute::if_items_entity", "sand::prelude::Execute::if_items_entity", "sand::prelude::cmd::Execute::if_items_entity"],
        module = "sand::command",
        kind = "method",
        summary = "`if items entity <selector> <slot> <item>` — execute if an entity has a matching item.",
        context = "`if items entity <selector> <slot> <item>` — execute if an entity has a matching item. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `if items entity <selector> <slot> <item>` — execute if an entity has a matching item form.", slot = "`slot` supplies the slot value used to emit the documented `if items entity <selector> <slot> <item>` — execute if an entity has a matching item form.", item = "`item` provides the item value or item predicate used to emit the documented `if items entity <selector> <slot> <item>` — execute if an entity has a matching item form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if items entity <selector> <slot> <item>` — execute if an entity has a matching item form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector, slot: sand::command::ItemSlot, item: impl Into < String >)  {\n    let updated_execute = execute_value.if_items_entity(selector, slot, item);\n}",
    )]
    pub fn if_items_entity(
        mut self,
        selector: Selector,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.check_selector("if_items_entity", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "if_items_entity",
            value: slot.clone(),
        });
        self.operations
            .push(ExecuteOp::If(ConditionIr::ItemsEntity {
                target: selector,
                slot,
                item: item.into(),
            }));
        self
    }

    /// `unless items entity <selector> <slot> <item>` — skip if the entity has the item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_items_entity",
        aliases = ["sand::cmd::Execute::unless_items_entity", "sand::prelude::Execute::unless_items_entity", "sand::prelude::cmd::Execute::unless_items_entity"],
        module = "sand::command",
        kind = "method",
        summary = "`unless items entity <selector> <slot> <item>` — skip if the entity has the item.",
        context = "`unless items entity <selector> <slot> <item>` — skip if the entity has the item. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `unless items entity <selector> <slot> <item>` — skip if the entity has the item form.", slot = "`slot` supplies the slot value used to emit the documented `unless items entity <selector> <slot> <item>` — skip if the entity has the item form.", item = "`item` provides the item value or item predicate used to emit the documented `unless items entity <selector> <slot> <item>` — skip if the entity has the item form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless items entity <selector> <slot> <item>` — skip if the entity has the item form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector, slot: sand::command::ItemSlot, item: impl Into < String >)  {\n    let updated_execute = execute_value.unless_items_entity(selector, slot, item);\n}",
    )]
    pub fn unless_items_entity(
        mut self,
        selector: Selector,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.check_selector("unless_items_entity", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "unless_items_entity",
            value: slot.clone(),
        });
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::ItemsEntity {
                target: selector,
                slot,
                item: item.into(),
            }));
        self
    }

    /// `if items block <pos> <slot> <item>` — execute if a block container has a matching item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_items_block",
        aliases = ["sand::cmd::Execute::if_items_block", "sand::prelude::Execute::if_items_block", "sand::prelude::cmd::Execute::if_items_block"],
        module = "sand::command",
        kind = "method",
        summary = "`if items block <pos> <slot> <item>` — execute if a block container has a matching item.",
        context = "`if items block <pos> <slot> <item>` — execute if a block container has a matching item. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`pos` supplies the pos value used to emit the documented `if items block <pos> <slot> <item>` — execute if a block container has a matching item form.", slot = "`slot` supplies the slot value used to emit the documented `if items block <pos> <slot> <item>` — execute if a block container has a matching item form.", item = "`item` provides the item value or item predicate used to emit the documented `if items block <pos> <slot> <item>` — execute if a block container has a matching item form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if items block <pos> <slot> <item>` — execute if a block container has a matching item form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, slot: sand::command::ItemSlot, item: impl Into < String >)  {\n    let updated_execute = execute_value.if_items_block(pos, slot, item);\n}",
    )]
    pub fn if_items_block(
        mut self,
        pos: BlockPos,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.check_block_pos("if_items_block", &pos);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "if_items_block",
            value: slot.clone(),
        });
        self.operations.push(ExecuteOp::If(ConditionIr::ItemsBlock {
            position: pos,
            slot,
            item: item.into(),
        }));
        self
    }

    /// `unless items block <pos> <slot> <item>` — skip if the block container has the item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_items_block",
        aliases = ["sand::cmd::Execute::unless_items_block", "sand::prelude::Execute::unless_items_block", "sand::prelude::cmd::Execute::unless_items_block"],
        module = "sand::command",
        kind = "method",
        summary = "`unless items block <pos> <slot> <item>` — skip if the block container has the item.",
        context = "`unless items block <pos> <slot> <item>` — skip if the block container has the item. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(pos = "`pos` supplies the pos value used to emit the documented `unless items block <pos> <slot> <item>` — skip if the block container has the item form.", slot = "`slot` supplies the slot value used to emit the documented `unless items block <pos> <slot> <item>` — skip if the block container has the item form.", item = "`item` provides the item value or item predicate used to emit the documented `unless items block <pos> <slot> <item>` — skip if the block container has the item form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless items block <pos> <slot> <item>` — skip if the block container has the item form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, pos: sand::command::BlockPos, slot: sand::command::ItemSlot, item: impl Into < String >)  {\n    let updated_execute = execute_value.unless_items_block(pos, slot, item);\n}",
    )]
    pub fn unless_items_block(
        mut self,
        pos: BlockPos,
        slot: ItemSlot,
        item: impl Into<String>,
    ) -> Self {
        self.check_block_pos("unless_items_block", &pos);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "unless_items_block",
            value: slot.clone(),
        });
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::ItemsBlock {
                position: pos,
                slot,
                item: item.into(),
            }));
        self
    }

    /// `if predicate <predicate>` — execute if a loot table predicate evaluates to true.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_predicate",
        aliases = ["sand::cmd::Execute::if_predicate", "sand::prelude::Execute::if_predicate", "sand::prelude::cmd::Execute::if_predicate"],
        module = "sand::command",
        kind = "method",
        summary = "`if predicate <predicate>` — execute if a loot table predicate evaluates to true.",
        context = "`if predicate <predicate>` — execute if a loot table predicate evaluates to true. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(predicate = "`predicate` provides the predicate that must match used to emit the documented `if predicate <predicate>` — execute if a loot table predicate evaluates to true form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if predicate <predicate>` — execute if a loot table predicate evaluates to true form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, predicate: impl Into < String >)  {\n    let updated_execute = execute_value.if_predicate(predicate);\n}",
    )]
    pub fn if_predicate(mut self, predicate: impl Into<String>) -> Self {
        let predicate = predicate.into();
        self.check_resource("if_predicate", "predicate", &predicate, false);
        self.operations
            .push(ExecuteOp::If(ConditionIr::Predicate(predicate)));
        self
    }

    /// Append a legacy raw execute fragment (e.g. from `Objective::if_matches`).
    ///
    /// This compatibility method creates [`ExecuteOp::Raw`]. Sand preserves
    /// the fragment verbatim and cannot structurally validate or version-check
    /// it. Prefer typed condition methods for new code.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_",
        aliases = ["sand::cmd::Execute::if_", "sand::prelude::Execute::if_", "sand::prelude::cmd::Execute::if_"],
        module = "sand::command",
        kind = "method",
        summary = "Append a legacy raw execute fragment (e.g. from `Objective::if_matches`).",
        context = "Append a legacy raw execute fragment (e.g. from `Objective::if_matches`). This compatibility method creates [`ExecuteOp::Raw`]. Sand preserves the fragment verbatim and cannot structurally validate or version-check it. Prefer typed condition methods for new code.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(condition = "`condition` provides the condition that gates the operation used to append a legacy raw execute fragment (e.g. from `Objective::if_matches`)."),
        returns = "The `Execute` value with the documented change applied to append a legacy raw execute fragment (e.g. from `Objective::if_matches`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, condition: impl Into < String >)  {\n    let updated_execute = execute_value.if_(condition);\n}",
    )]
    pub fn if_(mut self, condition: impl Into<String>) -> Self {
        self.operations.push(ExecuteOp::Raw(condition.into()));
        self
    }

    /// Append an explicitly opaque execute subcommand.
    ///
    /// Raw operations are not parsed, optimized, rewritten, or version-checked.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::raw_operation",
        aliases = ["sand::cmd::Execute::raw_operation", "sand::prelude::Execute::raw_operation", "sand::prelude::cmd::Execute::raw_operation"],
        module = "sand::command",
        kind = "method",
        summary = "Append an explicitly opaque execute subcommand. Raw operations are not parsed, optimized, rewritten, or version-checked.",
        context = "Append an explicitly opaque execute subcommand. Raw operations are not parsed, optimized, rewritten, or version-checked. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(fragment = "`fragment` supplies the fragment value used to append an explicitly opaque execute subcommand. Raw operations are not parsed, optimized, rewritten, or version-checked."),
        returns = "The `Execute` value with the documented change applied to append an explicitly opaque execute subcommand. Raw operations are not parsed, optimized, rewritten, or version-checked.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, fragment: impl Into < String >)  {\n    let updated_execute = execute_value.raw_operation(fragment);\n}",
    )]
    pub fn raw_operation(mut self, fragment: impl Into<String>) -> Self {
        self.operations.push(ExecuteOp::Raw(fragment.into()));
        self
    }

    // ── Items conditions (1.20.5+) ────────────────────────────────────────────

    /// `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item.
    ///
    /// Accepts any type that converts to [`ItemSlot`], including wildcard
    /// variants such as `ItemSlot::AnyHotbar`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::if_items",
        aliases = ["sand::cmd::Execute::if_items", "sand::prelude::Execute::if_items", "sand::prelude::cmd::Execute::if_items"],
        module = "sand::command",
        kind = "method",
        summary = "`if items entity <selector> <slot> <item>` — execute if the slot holds a matching item.",
        context = "`if items entity <selector> <slot> <item>` — execute if the slot holds a matching item. Accepts any type that converts to [`ItemSlot`], including wildcard variants such as `ItemSlot::AnyHotbar`.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item form.", slot = "`slot` supplies the slot value used to emit the documented `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item form.", item = "`item` provides the item value or item predicate used to emit the documented `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `if items entity <selector> <slot> <item>` — execute if the slot holds a matching item form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector, slot: impl Into < sand::command::ItemSlot >, item: impl Into < String >)  {\n    let updated_execute = execute_value.if_items(selector, slot, item);\n}",
    )]
    pub fn if_items(
        mut self,
        selector: Selector,
        slot: impl Into<ItemSlot>,
        item: impl Into<String>,
    ) -> Self {
        let slot = slot.into();
        self.check_selector("if_items", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "if_items",
            value: slot.clone(),
        });
        self.operations
            .push(ExecuteOp::If(ConditionIr::ItemsEntity {
                target: selector,
                slot,
                item: item.into(),
            }));
        self
    }

    /// `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::unless_items",
        aliases = ["sand::cmd::Execute::unless_items", "sand::prelude::Execute::unless_items", "sand::prelude::cmd::Execute::unless_items"],
        module = "sand::command",
        kind = "method",
        summary = "`unless items entity <selector> <slot> <item>` — execute if the slot does NOT match.",
        context = "`unless items entity <selector> <slot> <item>` — execute if the slot does NOT match. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match form.", slot = "`slot` supplies the slot value used to emit the documented `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match form.", item = "`item` provides the item value or item predicate used to emit the documented `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `unless items entity <selector> <slot> <item>` — execute if the slot does NOT match form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, selector: sand::command::Selector, slot: impl Into < sand::command::ItemSlot >, item: impl Into < String >)  {\n    let updated_execute = execute_value.unless_items(selector, slot, item);\n}",
    )]
    pub fn unless_items(
        mut self,
        selector: Selector,
        slot: impl Into<ItemSlot>,
        item: impl Into<String>,
    ) -> Self {
        let slot = slot.into();
        self.check_selector("unless_items", &selector);
        self.checks.push(ExecuteCheck::Slot {
            index: self.next_index(),
            kind: "unless_items",
            value: slot.clone(),
        });
        self.operations
            .push(ExecuteOp::Unless(ConditionIr::ItemsEntity {
                target: selector,
                slot,
                item: item.into(),
            }));
        self
    }

    // ── Store sub-commands ────────────────────────────────────────────────────

    /// `store result score <holder> <objective>` — capture the `run` result into a score.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::store_result_score",
        aliases = ["sand::cmd::Execute::store_result_score", "sand::prelude::Execute::store_result_score", "sand::prelude::cmd::Execute::store_result_score"],
        module = "sand::command",
        kind = "method",
        summary = "`store result score <holder> <objective>` — capture the `run` result into a score.",
        context = "`store result score <holder> <objective>` — capture the `run` result into a score. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the holder value used to emit the documented `store result score <holder> <objective>` — capture the `run` result into a score form.", objective = "`objective` supplies the objective value used to emit the documented `store result score <holder> <objective>` — capture the `run` result into a score form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `store result score <holder> <objective>` — capture the `run` result into a score form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, holder: sand::command::ScoreHolder, objective: impl Into < String >)  {\n    let updated_execute = execute_value.store_result_score(holder, objective);\n}",
    )]
    pub fn store_result_score(mut self, holder: ScoreHolder, objective: impl Into<String>) -> Self {
        let objective = objective.into();
        self.checks.push(ExecuteCheck::Holder {
            index: self.next_index(),
            kind: "store_result_score",
            value: holder.clone(),
        });
        self.checks.push(ExecuteCheck::Objective {
            index: self.next_index(),
            kind: "store_result_score",
            value: objective.clone(),
        });
        self.operations
            .push(ExecuteOp::StoreResult(ExecuteStoreTarget::Score {
                holder,
                objective,
            }));
        self
    }

    /// `store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::store_success_score",
        aliases = ["sand::cmd::Execute::store_success_score", "sand::prelude::Execute::store_success_score", "sand::prelude::cmd::Execute::store_success_score"],
        module = "sand::command",
        kind = "method",
        summary = "`store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails.",
        context = "`store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the holder value used to emit the documented `store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails form.", objective = "`objective` supplies the objective value used to emit the documented `store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `store success score <holder> <objective>` — store 1 if `run` succeeds, 0 if it fails form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, holder: sand::command::ScoreHolder, objective: impl Into < String >)  {\n    let updated_execute = execute_value.store_success_score(holder, objective);\n}",
    )]
    pub fn store_success_score(
        mut self,
        holder: ScoreHolder,
        objective: impl Into<String>,
    ) -> Self {
        let objective = objective.into();
        self.checks.push(ExecuteCheck::Holder {
            index: self.next_index(),
            kind: "store_success_score",
            value: holder.clone(),
        });
        self.checks.push(ExecuteCheck::Objective {
            index: self.next_index(),
            kind: "store_success_score",
            value: objective.clone(),
        });
        self.operations
            .push(ExecuteOp::StoreSuccess(ExecuteStoreTarget::Score {
                holder,
                objective,
            }));
        self
    }

    /// `store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::store_result_nbt",
        aliases = ["sand::cmd::Execute::store_result_nbt", "sand::prelude::Execute::store_result_nbt", "sand::prelude::cmd::Execute::store_result_nbt"],
        module = "sand::command",
        kind = "method",
        summary = "`store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT.",
        context = "`store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to emit the documented `store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT form.", path = "`path` provides the typed resource identifier or location used to emit the documented `store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT form.", kind = "`kind` supplies the kind value used to emit the documented `store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT form.", scale = "`scale` supplies the scale value used to emit the documented `store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `store result nbt <target> <path> <type> <scale>` — write the `run` result into NBT form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, target: sand::data::DataTarget, path: impl Into < String >, kind: sand::command::NbtStoreKind, scale: f64)  {\n    let updated_execute = execute_value.store_result_nbt(target, path, kind, scale);\n}",
    )]
    pub fn store_result_nbt(
        mut self,
        target: DataTarget,
        path: impl Into<String>,
        kind: NbtStoreKind,
        scale: f64,
    ) -> Self {
        self.checks.push(ExecuteCheck::Finite {
            index: self.next_index(),
            kind: "store_result_nbt",
            field: "scale",
            value: scale,
        });
        self.operations
            .push(ExecuteOp::StoreResult(ExecuteStoreTarget::Nbt {
                target,
                path: path.into(),
                kind,
                scale,
            }));
        self
    }

    /// `store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::store_success_nbt",
        aliases = ["sand::cmd::Execute::store_success_nbt", "sand::prelude::Execute::store_success_nbt", "sand::prelude::cmd::Execute::store_success_nbt"],
        module = "sand::command",
        kind = "method",
        summary = "`store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT.",
        context = "`store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to emit the documented `store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT form.", path = "`path` provides the typed resource identifier or location used to emit the documented `store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT form.", kind = "`kind` supplies the kind value used to emit the documented `store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT form.", scale = "`scale` supplies the scale value used to emit the documented `store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `store success nbt <target> <path> <type> <scale>` — write 1/0 (success/fail) into NBT form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, target: sand::data::DataTarget, path: impl Into < String >, kind: sand::command::NbtStoreKind, scale: f64)  {\n    let updated_execute = execute_value.store_success_nbt(target, path, kind, scale);\n}",
    )]
    pub fn store_success_nbt(
        mut self,
        target: DataTarget,
        path: impl Into<String>,
        kind: NbtStoreKind,
        scale: f64,
    ) -> Self {
        self.checks.push(ExecuteCheck::Finite {
            index: self.next_index(),
            kind: "store_success_nbt",
            field: "scale",
            value: scale,
        });
        self.operations
            .push(ExecuteOp::StoreSuccess(ExecuteStoreTarget::Nbt {
                target,
                path: path.into(),
                kind,
                scale,
            }));
        self
    }

    /// `store result bossbar <id> value` — write the `run` result into a bossbar's current value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::store_result_bossbar",
        aliases = ["sand::cmd::Execute::store_result_bossbar", "sand::prelude::Execute::store_result_bossbar", "sand::prelude::cmd::Execute::store_result_bossbar"],
        module = "sand::command",
        kind = "method",
        summary = "`store result bossbar <id> value` — write the `run` result into a bossbar's current value.",
        context = "`store result bossbar <id> value` — write the `run` result into a bossbar's current value. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to emit the documented `store result bossbar <id> value` — write the `run` result into a bossbar's current value form.", attribute = "`attribute` supplies the attribute value used to emit the documented `store result bossbar <id> value` — write the `run` result into a bossbar's current value form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `store result bossbar <id> value` — write the `run` result into a bossbar's current value form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, id: impl Into < String >, attribute: impl Into < String >)  {\n    let updated_execute = execute_value.store_result_bossbar(id, attribute);\n}",
    )]
    pub fn store_result_bossbar(
        mut self,
        id: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        self.operations
            .push(ExecuteOp::StoreResult(ExecuteStoreTarget::Bossbar {
                id: id.into(),
                attribute: attribute.into(),
            }));
        self
    }

    /// `store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::store_success_bossbar",
        aliases = ["sand::cmd::Execute::store_success_bossbar", "sand::prelude::Execute::store_success_bossbar", "sand::prelude::cmd::Execute::store_success_bossbar"],
        module = "sand::command",
        kind = "method",
        summary = "`store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute.",
        context = "`store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to emit the documented `store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute form.", attribute = "`attribute` supplies the attribute value used to emit the documented `store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute form."),
        returns = "The `Execute` value with the documented change applied to emit the documented `store success bossbar <id> <attribute>` — write success/failure into a bossbar attribute form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, id: impl Into < String >, attribute: impl Into < String >)  {\n    let updated_execute = execute_value.store_success_bossbar(id, attribute);\n}",
    )]
    pub fn store_success_bossbar(
        mut self,
        id: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        self.operations
            .push(ExecuteOp::StoreSuccess(ExecuteStoreTarget::Bossbar {
                id: id.into(),
                attribute: attribute.into(),
            }));
        self
    }

    // ── Terminal ──────────────────────────────────────────────────────────────

    /// Compatibility renderer for `run <command>`.
    ///
    /// This retains the historical infallible string API. Prefer [`try_run`](Self::try_run)
    /// for typed terminal commands; exported compatibility output is validated
    /// again with function context before files are accepted.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::run",
        aliases = ["sand::cmd::Execute::run", "sand::prelude::Execute::run", "sand::prelude::cmd::Execute::run"],
        module = "sand::command",
        kind = "method",
        summary = "Compatibility renderer for `run <command>`. This retains the historical infallible string API. Prefer [`try_run`](Self::try_run) for typed terminal commands; exported compatibility output is validated again with function context before files are accepted.",
        context = "Compatibility renderer for `run <command>`. This retains the historical infallible string API. Prefer [`try_run`](Self::try_run) for typed terminal commands; exported compatibility output is validated again with function context before files are accepted. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "This retains the historical infallible string API. Prefer [`try_run`](Self::try_run) for typed terminal commands; exported compatibility output is validated again with function context before files are accepted.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cmd = "`cmd` supplies the cmd value used to use compatibility renderer for `run <command>`. This retains the historical infallible string API. Prefer [`try_run`](Self::try_run) for typed terminal commands; exported compatibility output is validated again with function context before files are accepted."),
        returns = "The rendered Minecraft command text produced to use compatibility renderer for `run <command>`. This retains the historical infallible string API. Prefer [`try_run`](Self::try_run) for typed terminal commands; exported compatibility output is validated again with function context before files are accepted.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, cmd: impl fmt::Display)  {\n    let command = execute_value.run(cmd);\n}",
    )]
    pub fn run(self, cmd: impl fmt::Display) -> String {
        self.finish(cmd)
    }

    /// Validate the whole execute chain and a typed terminal command before
    /// rendering. Errors identify the failing execute subcommand.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::try_run",
        aliases = ["sand::cmd::Execute::try_run", "sand::prelude::Execute::try_run", "sand::prelude::cmd::Execute::try_run"],
        module = "sand::command",
        kind = "method",
        summary = "Validate the whole execute chain and a typed terminal command before rendering. Errors identify the failing execute subcommand.",
        context = "Validate the whole execute chain and a typed terminal command before rendering. Errors identify the failing execute subcommand. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cmd = "`cmd` supplies the cmd value used to validate the whole execute chain and a typed terminal command before rendering. Errors identify the failing execute subcommand."),
        returns = "On success, the value produced to validate the whole execute chain and a typed terminal command before rendering. Errors identify the failing execute subcommand; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, cmd: & impl sand::command::RenderCommand)  {\n    let try_run = execute_value.try_run(cmd);\n}",
    )]
    pub fn try_run(self, cmd: &impl RenderCommand) -> CommandResult<String> {
        let profile = CommandProfile::unprofiled();
        self.validate(&profile)?;
        let cmd = cmd
            .render(&profile)
            .map_err(|e| e.with_context("Execute::run command"))?;
        let line = format!("{} run {cmd}", self.build());
        crate::execute_ir::register_line(&line, &self.operations);
        Ok(line)
    }

    /// Like [`run`](Execute::run) but more explicit about accepting raw strings.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::run_raw",
        aliases = ["sand::cmd::Execute::run_raw", "sand::prelude::Execute::run_raw", "sand::prelude::cmd::Execute::run_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Like [`run`](Execute::run) but more explicit about accepting raw strings.",
        context = "Like [`run`](Execute::run) but more explicit about accepting raw strings. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cmd = "`cmd` supplies the cmd value used to use like [`run`](Execute::run) but more explicit about accepting raw strings."),
        returns = "The string value produced to use like [`run`](Execute::run) but more explicit about accepting raw strings.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, cmd: impl fmt::Display)  {\n    let run_raw = execute_value.run_raw(cmd);\n}",
    )]
    pub fn run_raw(self, cmd: impl fmt::Display) -> String {
        self.finish(cmd)
    }

    /// Validate the typed execute chain, then append an explicitly raw terminal
    /// command. The raw text bypasses typed grammar modeling but must remain one
    /// `.mcfunction`-safe line without a leading slash.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::try_run_raw",
        aliases = ["sand::cmd::Execute::try_run_raw", "sand::prelude::Execute::try_run_raw", "sand::prelude::cmd::Execute::try_run_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Validate the typed execute chain, then append an explicitly raw terminal command. The raw text bypasses typed grammar modeling but must remain one `.mcfunction`-safe line without a leading slash.",
        context = "Validate the typed execute chain, then append an explicitly raw terminal command. The raw text bypasses typed grammar modeling but must remain one `.mcfunction`-safe line without a leading slash. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cmd = "`cmd` supplies the cmd value used to validate the typed execute chain, then append an explicitly raw terminal command. The raw text bypasses typed grammar modeling but must remain one `.mcfunction`-safe line without a leading slash."),
        returns = "On success, the value produced to validate the typed execute chain, then append an explicitly raw terminal command. The raw text bypasses typed grammar modeling but must remain one `.mcfunction`-safe line without a leading slash; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, cmd: sand::command::RawCommand)  {\n    let try_run_raw = execute_value.try_run_raw(cmd);\n}",
    )]
    pub fn try_run_raw(self, cmd: RawCommand) -> CommandResult<String> {
        let profile = CommandProfile::unprofiled();
        self.validate(&profile)?;
        let cmd = cmd.as_str();
        if cmd.contains(['\0', '\n', '\r']) || cmd.trim_start().starts_with('/') {
            return Err(CommandError::new(
                "Execute::try_run_raw",
                "command",
                "raw terminal commands must be a single line without a leading `/`",
            ));
        }
        let line = format!("{} run {cmd}", self.build());
        crate::execute_ir::register_line(&line, &self.operations);
        Ok(line)
    }

    /// Run a named function: `execute ... run function <namespace:path>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::run_fn",
        aliases = ["sand::cmd::Execute::run_fn", "sand::prelude::Execute::run_fn", "sand::prelude::cmd::Execute::run_fn"],
        module = "sand::command",
        kind = "method",
        summary = "Run a named function: `execute ... run function <namespace:path>`.",
        context = "Run a named function: `execute ... run function <namespace:path>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(function = "`function` provides the callback invoked by this operation used to run a named function: `execute ... run function <namespace:path>`."),
        returns = "The string value produced to run a named function: `execute ... run function <namespace:path>`.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(execute_value: sand::command::Execute, function: impl fmt::Display)  {\n    let run_fn = execute_value.run_fn(function);\n}",
    )]
    pub fn run_fn(self, function: impl fmt::Display) -> String {
        self.finish(format!("function {function}"))
    }

    /// Append a typed operation. Intended for Sand's higher-level builders.
    pub(crate) fn with_operation(mut self, operation: ExecuteOp) -> Self {
        self.operations.push(operation);
        self
    }

    /// Borrow the ordered operation IR.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Execute::operations",
        aliases = ["sand::cmd::Execute::operations", "sand::prelude::Execute::operations", "sand::prelude::cmd::Execute::operations"],
        module = "sand::command",
        kind = "method",
        summary = "Borrow the ordered operation IR.",
        context = "Borrow the ordered operation IR. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `& [ExecuteOp]` value produced to borrow the ordered operation IR.",
        example = "use sand::prelude::*;\n\nfn demonstrate(execute_value: &sand::command::Execute)  {\n    let operations = execute_value.operations();\n}",
    )]
    pub fn operations(&self) -> &[ExecuteOp] {
        &self.operations
    }

    fn finish(self, command: impl fmt::Display) -> String {
        let line = format!("{} run {command}", self.build());
        crate::execute_ir::register_line(&line, &self.operations);
        line
    }
}

impl Validate for Execute {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        if self.operations.is_empty() {
            return Err(CommandError::new(
                "Execute",
                "subcommands",
                "execute chains require at least one subcommand",
            ));
        }
        for (index, operation) in self.operations.iter().enumerate() {
            operation.validate_version(index, profile)?;
        }
        for check in &self.checks {
            let (index, kind, result) = match check {
                ExecuteCheck::Selector { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Vec3 { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::BlockPos { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Rotation { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Slot { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::Finite {
                    index,
                    kind,
                    field,
                    value,
                } => (
                    *index,
                    *kind,
                    validate::finite(*value, "Execute", field).map(|_| ()),
                ),
                ExecuteCheck::Resource {
                    index,
                    kind,
                    field,
                    value,
                    allow_tag,
                } => (
                    *index,
                    *kind,
                    validate::resource_location_shape(
                        if *allow_tag {
                            value.strip_prefix('#').unwrap_or(value)
                        } else {
                            value
                        },
                        "Execute",
                        field,
                    )
                    .map(|_| ()),
                ),
                ExecuteCheck::Holder { index, kind, value } => {
                    (*index, *kind, value.validate(profile))
                }
                ExecuteCheck::SingleHolder { index, kind, value } => {
                    (*index, *kind, value.validate_single(profile))
                }
                ExecuteCheck::Objective { index, kind, value } => {
                    let result = validate::no_whitespace_or_control(value, "Execute", "objective")
                        .and_then(|_| {
                            if value.len() <= 16 {
                                Ok(value.as_str())
                            } else {
                                Err(CommandError::new(
                                    "Execute",
                                    "objective",
                                    "objective names cannot exceed 16 characters",
                                ))
                            }
                        })
                        .map(|_| ());
                    (*index, *kind, result)
                }
                ExecuteCheck::ScoreRange { index, kind, value } => {
                    (*index, *kind, validate_score_range(value))
                }
            };
            result.map_err(|e| e.with_context(format!("Execute subcommand {index} `{kind}`")))?;
        }
        Ok(())
    }
}

fn validate_score_range(value: &str) -> CommandResult<()> {
    validate::non_empty(value, "Execute", "score_range")?;
    let parse = |bound: &str| -> CommandResult<Option<i32>> {
        if bound.is_empty() {
            Ok(None)
        } else {
            bound.parse::<i32>().map(Some).map_err(|_| {
                CommandError::new(
                    "Execute",
                    "score_range",
                    format!("invalid integer bound `{bound}`"),
                )
            })
        }
    };
    let (min, max) = if let Some((min, max)) = value.split_once("..") {
        if max.contains("..") {
            return Err(CommandError::new(
                "Execute",
                "score_range",
                "range contains more than one `..`",
            ));
        }
        (parse(min)?, parse(max)?)
    } else {
        let exact = parse(value)?;
        (exact, exact)
    };
    if min.is_none() && max.is_none() {
        return Err(CommandError::new(
            "Execute",
            "score_range",
            "range must contain at least one bound",
        ));
    }
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(CommandError::new(
            "Execute",
            "score_range",
            format!("range lower bound `{min}` exceeds upper bound `{max}`"),
        ));
    }
    Ok(())
}

impl RenderCommand for Execute {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.build()
    }
}

impl Build for Execute {
    /// Return the current partial execute chain (without a `run` clause).
    ///
    /// Useful for embedding in `execute store` prefixes or debugging.
    fn build(&self) -> String {
        if self.operations.is_empty() {
            "execute".to_string()
        } else {
            format!(
                "execute {}",
                self.operations
                    .iter()
                    .map(ExecuteOp::render)
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
    }
}

impl fmt::Display for Execute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<Execute> for String {
    fn from(v: Execute) -> Self {
        v.build()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_run() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .run_raw("say hi");
        assert_eq!(s, "execute as @a run say hi");
    }

    #[test]
    fn ordered_context_operations_render_from_typed_ir() {
        let command = Execute::new()
            .as_(Selector::all_players())
            .at(Selector::self_())
            .positioned(Vec3::new(
                crate::Coord::rel_n(1.0),
                crate::Coord::rel_n(2.0),
                crate::Coord::rel_n(3.0),
            ))
            .positioned_as(Selector::nearest_player())
            .rotated(Rotation::new(
                crate::Coord::rel_n(10.0),
                crate::Coord::rel_n(20.0),
            ))
            .rotated_as(Selector::self_())
            .facing(Vec3::absolute(0.0, 64.0, 0.0))
            .facing_entity(Selector::nearest_player(), Anchor::Eyes)
            .anchored(Anchor::Feet)
            .in_("minecraft:the_nether")
            .align(Swizzle::xyz())
            .on("attacker")
            .run_raw("say ordered");
        assert_eq!(
            command,
            "execute as @a at @s positioned ~1 ~2 ~3 positioned as @p rotated ~10 ~20 rotated as @s facing 0 64 0 facing entity @p eyes anchored feet in minecraft:the_nether align xyz on attacker run say ordered"
        );
    }

    #[test]
    fn raw_operation_is_distinguishable_and_verbatim() {
        let execute = Execute::new().raw_operation("if custom future:syntax");
        assert!(
            matches!(execute.operations(), [ExecuteOp::Raw(fragment)] if fragment == "if custom future:syntax")
        );
        assert_eq!(
            execute.run_raw("say opaque"),
            "execute if custom future:syntax run say opaque"
        );
    }

    #[test]
    fn item_condition_capability_is_profile_aware_at_export_boundary() {
        let line = Execute::new()
            .if_items(Selector::self_(), ItemSlot::MainHand, "minecraft:diamond")
            .run_raw("say found");

        crate::render::validate_collected_line(&line, &CommandProfile::new("1.20.5", false))
            .unwrap();
        let error =
            crate::render::validate_collected_line(&line, &CommandProfile::new("1.20.4", false))
                .unwrap_err();
        assert_eq!(error.code, "SAND-COMMAND-VERSION");
        assert_eq!(error.field, "operation");
        assert!(error.message.contains("ExecuteItemCondition"), "{error}");
        assert!(error.message.contains("Minecraft 1.20.5+"), "{error}");
    }

    #[test]
    fn raw_item_condition_keeps_user_owned_version_semantics() {
        let line = Execute::new()
            .raw_operation("if items entity @s weapon.mainhand minecraft:diamond")
            .run_raw("say opaque");
        assert_eq!(
            crate::render::validate_collected_line(&line, &CommandProfile::new("1.20.4", false),)
                .unwrap(),
            line
        );
    }

    #[test]
    fn empty_compatibility_chain_is_rejected_before_export() {
        let line = Execute::new().run_raw("say unreachable");
        assert_eq!(line, "execute run say unreachable");
        let error = crate::render::validate_collected_line(&line, &CommandProfile::unprofiled())
            .unwrap_err();
        assert_eq!(error.code, "SAND-COMMAND-EXECUTE-EMPTY");
        assert_eq!(error.field, "operations");
    }

    #[test]
    fn chained_conditions() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .at(Selector::self_())
            .if_score_matches("@s", "playtime", "100..")
            .run_raw("say milestone!");
        assert_eq!(
            s,
            "execute as @a at @s if score @s playtime matches 100.. run say milestone!"
        );
    }

    #[test]
    fn score_compare_ops() {
        let eq = Execute::new()
            .if_score_eq("@s", "mana", "@s", "max_mana")
            .run_raw("say full");
        assert_eq!(eq, "execute if score @s mana = @s max_mana run say full");

        let lt = Execute::new()
            .if_score_lt("@s", "health", "#const", "ten")
            .run_raw("say low");
        assert_eq!(lt, "execute if score @s health < #const ten run say low");

        let lte = Execute::new()
            .if_score_lte("@s", "health", "#const", "ten")
            .run_raw("say lte");
        assert_eq!(lte, "execute if score @s health <= #const ten run say lte");

        let gte = Execute::new()
            .if_score_gte("@s", "mana", "@s", "cost")
            .run_raw("say can cast");
        assert_eq!(gte, "execute if score @s mana >= @s cost run say can cast");

        let unless = Execute::new()
            .unless_score_gt("@s", "mana", "#const", "zero")
            .run_raw("say no mana");
        assert_eq!(
            unless,
            "execute unless score @s mana > #const zero run say no mana"
        );
    }

    #[test]
    fn if_data_conditions() {
        let s = Execute::new()
            .if_data_entity(Selector::self_(), "Custom.ready")
            .run_raw("say ready");
        assert_eq!(s, "execute if data entity @s Custom.ready run say ready");

        let s = Execute::new()
            .if_data_storage("my_pack:state", "phase")
            .run_raw("say has phase");
        assert_eq!(
            s,
            "execute if data storage my_pack:state phase run say has phase"
        );
    }

    #[test]
    fn world_conditions() {
        let s = Execute::new()
            .if_biome(BlockPos::here(), "minecraft:desert")
            .run_raw("say desert");
        assert_eq!(s, "execute if biome ~ ~ ~ minecraft:desert run say desert");

        let s = Execute::new()
            .if_loaded(BlockPos::here())
            .run_raw("say loaded");
        assert_eq!(s, "execute if loaded ~ ~ ~ run say loaded");

        let s = Execute::new()
            .if_dimension("minecraft:the_nether")
            .run_raw("say nether");
        assert_eq!(
            s,
            "execute if dimension minecraft:the_nether run say nether"
        );

        assert!(
            Execute::new()
                .if_dimension("the_nether")
                .try_run_raw(RawCommand::new("say no"))
                .is_err()
        );
        assert!(
            Execute::new()
                .if_biome(BlockPos::here(), "#minecraft:is_overworld")
                .try_run_raw(RawCommand::new("say yes"))
                .is_ok()
        );
    }

    #[test]
    fn positioned_over_test() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .positioned_as(Selector::self_())
            .positioned_over("world_surface")
            .run_raw("say ground");
        assert_eq!(
            s,
            "execute as @a positioned as @s positioned over world_surface run say ground"
        );
    }

    #[test]
    fn store_result() {
        let s = Execute::new()
            .store_result_score(ScoreHolder::entity(Selector::self_()), "my_score")
            .run_raw("data get entity @s Health");
        assert_eq!(
            s,
            "execute store result score @s my_score run data get entity @s Health"
        );
    }

    // ── Additional execute golden tests ───────────────────────────────────────

    #[test]
    fn anchored_eyes() {
        let s = Execute::new().anchored(Anchor::Eyes).run_raw("say looking");
        assert_eq!(s, "execute anchored eyes run say looking");
    }

    #[test]
    fn anchored_feet() {
        let s = Execute::new().anchored(Anchor::Feet).run_raw("say feet");
        assert_eq!(s, "execute anchored feet run say feet");
    }

    #[test]
    fn in_dimension() {
        let s = Execute::new()
            .in_("minecraft:the_nether")
            .run_raw("say nether");
        assert_eq!(s, "execute in minecraft:the_nether run say nether");
    }

    #[test]
    fn rotated_as() {
        let s = Execute::new()
            .rotated_as(Selector::self_())
            .run_raw("tp @s ~ ~ ~");
        assert_eq!(s, "execute rotated as @s run tp @s ~ ~ ~");
    }

    #[test]
    fn facing_entity() {
        let s = Execute::new()
            .facing_entity(Selector::nearest_player(), Anchor::Eyes)
            .run_raw("say facing");
        assert_eq!(s, "execute facing entity @p eyes run say facing");
    }

    #[test]
    fn if_predicate_chain() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .if_predicate("my_pack:is_sneaking")
            .run_raw("say sneaking");
        assert_eq!(
            s,
            "execute as @a if predicate my_pack:is_sneaking run say sneaking"
        );
    }

    #[test]
    fn store_result_nbt_entity() {
        let s = Execute::new()
            .store_result_nbt(
                crate::nbt::DataTarget::Entity(Selector::self_()),
                "Custom.kills",
                NbtStoreKind::Int,
                1.0,
            )
            .run_raw("scoreboard players get @s kills");
        assert_eq!(
            s,
            "execute store result entity @s Custom.kills int 1 run scoreboard players get @s kills"
        );
    }

    #[test]
    fn store_success_score() {
        let s = Execute::new()
            .store_success_score(ScoreHolder::entity(Selector::self_()), "result_obj")
            .if_entity(Selector::all_entities().entity_type("minecraft:zombie"))
            .run_raw("say zombies");
        assert_eq!(
            s,
            "execute store success score @s result_obj if entity @e[type=minecraft:zombie] run say zombies"
        );
    }

    #[test]
    fn run_fn_formats_correctly() {
        let s = Execute::new()
            .as_(Selector::all_players())
            .run_fn("my_pack:on_tick");
        assert_eq!(s, "execute as @a run function my_pack:on_tick");
    }

    #[test]
    fn summon_subcommand() {
        let s = Execute::new()
            .summon("minecraft:armor_stand")
            .run_raw("say spawned");
        assert_eq!(s, "execute summon minecraft:armor_stand run say spawned");
    }

    #[test]
    fn unless_entity() {
        let s = Execute::new()
            .unless_entity(Selector::all_players().tag("ready"))
            .run_raw("say not ready");
        assert_eq!(s, "execute unless entity @a[tag=ready] run say not ready");
    }

    #[test]
    fn unless_block_condition() {
        let s = Execute::new()
            .unless_block(BlockPos::here(), "minecraft:air")
            .run_raw("say blocked");
        assert_eq!(
            s,
            "execute unless block ~ ~ ~ minecraft:air run say blocked"
        );
    }

    #[test]
    fn try_build_reports_execute_subcommand_context() {
        let execute = Execute::new().positioned(Vec3::absolute(f64::NAN, 0.0, 0.0));
        let error = execute.try_build().unwrap_err().to_string();
        assert!(
            error.contains("Execute subcommand 0 `positioned`"),
            "{error}"
        );
        assert!(error.contains("finite"), "{error}");
    }

    #[test]
    fn try_build_rejects_invalid_slot_and_scale() {
        assert!(
            Execute::new()
                .if_items(Selector::self_(), ItemSlot::Hotbar(9), "minecraft:stone")
                .try_build()
                .is_err()
        );
        assert!(
            Execute::new()
                .store_result_nbt(
                    DataTarget::Entity(Selector::self_()),
                    "x",
                    NbtStoreKind::Double,
                    f64::INFINITY
                )
                .try_build()
                .is_err()
        );
    }

    #[test]
    fn try_build_validates_score_holders_objectives_and_ranges() {
        let profile = CommandProfile::unprofiled();
        let many = Execute::new().if_score_matches("@a", "mana", "1..");
        let error = many.render(&profile).unwrap_err().to_string();
        assert!(error.contains("if_score_matches"), "{error}");
        assert!(error.contains("exactly one holder"), "{error}");

        assert!(
            Execute::new()
                .if_score_matches("@s", "objective_is_too_long", "1..")
                .render(&profile)
                .is_err()
        );
        assert!(
            Execute::new()
                .if_score_matches("@s", "mana", "5..1")
                .render(&profile)
                .is_err()
        );
    }

    #[test]
    fn try_run_raw_preserves_advanced_syntax_but_validates_the_chain() {
        let command = Execute::new()
            .as_(Selector::all_players())
            .try_run_raw(RawCommand::new("modded command syntax"))
            .unwrap();
        assert_eq!(command, "execute as @a run modded command syntax");
        assert!(
            Execute::new()
                .as_(Selector::all_players().limit(0))
                .try_run_raw(RawCommand::new("modded command syntax"))
                .is_err()
        );
        assert!(
            Execute::new()
                .as_(Selector::all_players())
                .try_run_raw(RawCommand::new("/say no"))
                .is_err()
        );
    }
}
