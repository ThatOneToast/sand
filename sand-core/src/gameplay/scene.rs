use super::{Lifetime, identity, runtime};
use crate::{
    IntoCommands,
    entity::{EntityKind, PlayerKind},
    ir::Actions,
    registration::{DatapackRegistration, IntoDatapack},
};
use sand_commands::{CommandError, CommandResult, ObjectiveName};
use sand_components::{BlockId, effect::Ticks};

/// Local display pose: translation, scale and rotation about the Y axis.
/// Minecraft applies scale, then yaw, then translation. Values describe visual
/// geometry only; display entities have no gameplay collision. Default is identity.
#[derive(Clone, Copy, Debug)]
#[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Transform", module = "sand::systems",
summary = "A local block-display pose.", context = "Stores translation, scale and yaw for native display interpolation. This visual pose does not change entity position or gameplay collision.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
example = "use sand::systems::Transform;",
)]
pub struct Transform {
    translation: [f32; 3],
    scale: [f32; 3],
    yaw: f32,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            scale: [1.0; 3],
            yaw: 0.0,
        }
    }
}
impl Transform {
    /// Set local x, y and z translation.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Transform::translated", module = "sand::systems", kind = "method",
summary = "Set local x, y and z translation.", context = "Configures a native display transformation. Values are validated during export; transform changes are interpolated by Minecraft clients.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(translation = "Finite local translation in blocks."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::Transform;\nfn demonstrate() { let _pose = Transform::default().translated([0.0,1.0,0.0]); }",
)]
    pub fn translated(mut self, translation: [f32; 3]) -> Self {
        self.translation = translation;
        self
    }
    /// Set local x, y and z scale.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Transform::scaled", module = "sand::systems", kind = "method",
summary = "Set local x, y and z scale.", context = "Configures a native display transformation. Values are validated during export; transform changes are interpolated by Minecraft clients.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(scale = "Finite nonnegative scale factors; zero hides an axis."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::Transform;\nfn demonstrate() { let _pose = Transform::default().scaled([2.0,2.0,2.0]); }",
)]
    pub fn scaled(mut self, scale: [f32; 3]) -> Self {
        self.scale = scale;
        self
    }
    /// Set local rotation about the Y axis.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Transform::yaw", module = "sand::systems", kind = "method",
summary = "Set local rotation about the Y axis.", context = "Configures a native display transformation. Values are validated during export; transform changes are interpolated by Minecraft clients.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(degrees = "Finite rotation in degrees, applied after scale."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::Transform;\nfn demonstrate() { let _pose = Transform::default().yaw(90.0); }",
)]
    pub fn yaw(mut self, degrees: f32) -> Self {
        self.yaw = degrees;
        self
    }
    fn validate(&self) -> CommandResult<()> {
        if self
            .translation
            .iter()
            .chain(self.scale.iter())
            .any(|v| !v.is_finite())
            || !self.yaw.is_finite()
            || self.scale.iter().any(|v| *v < 0.0)
        {
            return Err(CommandError::new(
                "scene",
                "transform",
                "requires finite translation and yaw, and finite nonnegative scale",
            ));
        }
        Ok(())
    }
    fn snbt(&self) -> String {
        let array = |values: &[f32]| {
            values
                .iter()
                .map(|v| format!("{v}f"))
                .collect::<Vec<_>>()
                .join(",")
        };
        let angle = (f64::from(self.yaw).rem_euclid(360.0).to_radians() / 2.0) as f32;
        format!(
            "{{translation:[{}],scale:[{}],left_rotation:[0f,{}f,0f,{}f],right_rotation:[0f,0f,0f,1f]}}",
            array(&self.translation),
            array(&self.scale),
            angle.sin(),
            angle.cos()
        )
    }
}
#[derive(Clone)]
struct Part {
    name: String,
    block: BlockId,
    pose: Transform,
}
#[derive(Clone)]
struct Frame {
    name: String,
    at: Ticks,
    duration: Ticks,
    pose: Transform,
}
/// Owned flat scene of named block displays using each block's default state.
/// Parts share the root's entity position; visual offsets live in Transform,
/// so ordinary chunk unloads keep the root and displays together. Lifetime
/// cleanup removes the parts. Do not mutate or remove private runtime entities.
///
/// Keyframes start interpolation at a specified loaded tick. They run once;
/// the final pose persists until cleanup. This is native client interpolation,
/// not a server-side collision or physics simulation. Lifetime callbacks and
/// cadence are preserved. Following uses the owner's position, not rotation.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Scene", module = "sand::systems",
summary = "An owned scene of named animated block displays.", context = "Composes Lifetime with flat visual parts and one-shot native interpolation. Private displays share the root position and are removed on lifetime cleanup.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
example = "use sand::systems::Scene;",
)]
pub struct Scene<K: EntityKind = PlayerKind> {
    lifetime: Lifetime<K>,
    parts: Vec<Part>,
    frames: Vec<Frame>,
    follow: bool,
}
impl<K: EntityKind> Scene<K> {
    /// Define an initially empty scene at the lifetime origin.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Scene::new", module = "sand::systems", kind = "method",
summary = "Define an initially empty scene at the lifetime origin.", context = "Uses Lifetime ownership and loaded-tick cleanup. Part names must be unique and keyframes on the same part may not overlap. Export validates the complete definition.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(lifetime = "Ownership, capacity, duration, callbacks and cleanup."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Scene,Lifetime,Transform};\nuse sand::prelude::Ticks;\nfn demonstrate(lifetime: Lifetime) { let _scene = Scene::new(lifetime); }",
)]
    pub fn new(lifetime: Lifetime<K>) -> Self {
        Self {
            lifetime,
            parts: Vec::new(),
            frames: Vec::new(),
            follow: false,
        }
    }
    /// Add a uniquely named block-display part.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Scene::part", module = "sand::systems", kind = "method",
summary = "Add a uniquely named block-display part.", context = "Uses Lifetime ownership and loaded-tick cleanup. Part names must be unique and keyframes on the same part may not overlap. Export validates the complete definition.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(name = "Unique nonempty local part name.", block = "Typed block identifier; uses the default block state.", pose = "Initial local visual pose."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Scene,Lifetime,Transform};\nuse sand::prelude::Ticks;\nfn demonstrate(scene: Scene) { let _scene = scene.part(\"body\", sand::registry::BlockId::minecraft(\"stone\").unwrap(), Transform::default()); }",
)]
    pub fn part(mut self, name: impl Into<String>, block: BlockId, pose: Transform) -> Self {
        self.parts.push(Part {
            name: name.into(),
            block,
            pose,
        });
        self
    }
    /// Start interpolation toward a pose at a loaded tick.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Scene::keyframe", module = "sand::systems", kind = "method",
summary = "Start interpolation toward a pose at a loaded tick.", context = "Uses Lifetime ownership and loaded-tick cleanup. Part names must be unique and keyframes on the same part may not overlap. Export validates the complete definition.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(part = "Name of an existing part.", at = "Positive loaded tick at which interpolation starts.", duration = "Positive interpolation duration in ticks.", pose = "Target local pose."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Scene,Lifetime,Transform};\nuse sand::prelude::Ticks;\nfn demonstrate(scene: Scene) { let _scene = scene.keyframe(\"body\", Ticks::new(1), Ticks::new(10), Transform::default().yaw(90.0)); }",
)]
    pub fn keyframe(
        mut self,
        part: impl Into<String>,
        at: Ticks,
        duration: Ticks,
        pose: Transform,
    ) -> Self {
        self.frames.push(Frame {
            name: part.into(),
            at,
            duration,
            pose,
        });
        self
    }
    /// Follow the available owner position each loaded tick.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Scene::follow_owner", module = "sand::systems", kind = "method",
summary = "Follow the available owner position each loaded tick.", context = "Uses Lifetime ownership and loaded-tick cleanup. Part names must be unique and keyframes on the same part may not overlap. Export validates the complete definition.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
params(enabled = "True to follow; false to remain at the lifetime origin."),
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Scene,Lifetime,Transform};\nuse sand::prelude::Ticks;\nfn demonstrate(scene: Scene) { let _scene = scene.follow_owner(true); }",
)]
    pub fn follow_owner(mut self, enabled: bool) -> Self {
        self.follow = enabled;
        self
    }
    /// Start an independent scene for the current owner.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Scene::start", module = "sand::systems", kind = "method",
summary = "Start an independent scene for the current owner.", context = "Uses Lifetime ownership and loaded-tick cleanup. Part names must be unique and keyframes on the same part may not overlap. Export validates the complete definition.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Scene,Lifetime,Transform};\nuse sand::prelude::Ticks;\nfn demonstrate(scene: Scene) { let _actions = scene.start(); }",
)]
    pub fn start(&self) -> Actions {
        self.lifetime.start()
    }
    /// Remove all scenes of this definition belonging to the current owner.
    #[sand_macros::api(
registry = sand_api_contract, path = "sand::systems::Scene::cancel", module = "sand::systems", kind = "method",
summary = "Remove all scenes of this definition belonging to the current owner.", context = "Uses Lifetime ownership and loaded-tick cleanup. Part names must be unique and keyframes on the same part may not overlap. Export validates the complete definition.",
minecraft = "Uses canonical checked numeric expressions and owned lifecycle functions on all supported targets.",
use_when = ["Authoring reusable bounded gameplay"], avoid_when = ["Work must continue in unloaded chunks"],
returns = "The configured definition or retained runtime actions.", example = "use sand::systems::{Scene,Lifetime,Transform};\nuse sand::prelude::Ticks;\nfn demonstrate(scene: Scene) { let _actions = scene.cancel(); }",
)]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl<K: EntityKind> IntoDatapack for Scene<K> {
    fn into_datapack(mut self) -> DatapackRegistration {
        runtime::hooks(
            &mut self.lifetime,
            "scene_init",
            "scene_cleanup",
            "scene_step",
        );
        let mut registration = self.lifetime.clone().into_datapack();
        let mut parts: Vec<String> = [
            "scene_init",
            "scene_cleanup",
            "scene_remove",
            "scene_step",
            "scene_follow",
            "scene_update",
            "scene_resume",
            "scene_spawn",
            "scene_remove_scores",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        parts.extend((0..self.parts.len()).map(|i| format!("scene_part_{i}")));
        for part in parts {
            let definition = self.clone();
            let location = part.clone();
            registration = registration.component(runtime::Function::new(
                &self.lifetime.id,
                &location,
                move || definition.build(&part),
            ));
        }
        registration
    }
}
impl<K: EntityKind> Scene<K> {
    fn validate(&self) -> CommandResult<()> {
        if self.parts.is_empty() || self.parts.len() > 64 {
            return Err(CommandError::new("scene", "parts", "requires 1..=64 parts"));
        }
        let mut names = std::collections::BTreeSet::new();
        for part in &self.parts {
            if part.name.is_empty() || !names.insert(&part.name) {
                return Err(CommandError::new(
                    "scene",
                    "name",
                    "part names must be nonempty and unique",
                ));
            }
            part.pose.validate()?;
        }
        for frame in &self.frames {
            frame.pose.validate()?;
            if !names.contains(&frame.name) {
                return Err(CommandError::new("scene", "keyframe", "unknown part"));
            }
            if frame.at.get() == 0
                || frame.duration.get() == 0
                || u64::from(frame.at.get()) + u64::from(frame.duration.get()) > i32::MAX as u64
            {
                return Err(CommandError::new(
                    "scene",
                    "keyframe",
                    "positive tick and duration must end within i32::MAX ticks",
                ));
            }
        }
        for name in names {
            let mut frames: Vec<_> = self.frames.iter().filter(|f| &f.name == name).collect();
            frames.sort_by_key(|f| f.at.get());
            if frames
                .windows(2)
                .any(|w| w[0].at.get() + w[0].duration.get() > w[1].at.get())
            {
                return Err(CommandError::new(
                    "scene",
                    "keyframe",
                    "interpolation intervals on a part must not overlap",
                ));
            }
        }
        Ok(())
    }
    fn build(&self, part: &str) -> CommandResult<Actions> {
        self.validate()?;
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let [group, age] =
            ["group", "age"].map(|name| ObjectiveName::logical(format!("sand scene {id} {name}")));
        crate::function::request_numeric_objectives([group.clone(), age.clone()])?;
        let tag = identity::task_tag(&id)?;
        let visual = format!("{tag}.visual");
        let resource = |part: &str| identity::resource(&id, part).to_string();
        let select = |index: Option<usize>| {
            format!(
                "@e[type=minecraft:block_display,tag={},scores={{{group}=$(instance)}}]",
                index
                    .map(|i| format!("{visual}.{i}"))
                    .unwrap_or(visual.clone())
            )
        };
        let mut lines = Vec::new();
        match part {
            "scene_init" => lines.extend([
                format!("scoreboard players set @s {age} 0"),
                format!("function {} with entity @s data", resource("scene_spawn")),
            ]),
            "scene_cleanup" => lines.extend([
                format!("function {} with entity @s data", resource("scene_remove")),
                format!("scoreboard players reset @s {age}"),
            ]),
            "scene_spawn" => {
                for i in 0..self.parts.len() {
                    lines.push(format!("$execute summon minecraft:block_display run function {} {{instance:$(instance)}}",resource(&format!("scene_part_{i}"))));
                }
            }
            "scene_remove" => {
                // Reset each display's private score while its selection is still valid.
                lines.push(format!(
                    "$execute as {} run function {}",
                    select(None),
                    resource("scene_remove_scores")
                ));
            }
            "scene_resume" => return runtime::resolve(&id, "scene_update"),
            "scene_step" => {
                lines.push(format!("execute if score @s {age} matches ..2147483646 run scoreboard players add @s {age} 1"));
                if self.follow {
                    lines.push(format!(
                        "function {} with entity @s data",
                        resource("scene_follow")
                    ));
                    lines.push(format!(
                        "function {} with entity @s data",
                        resource("scene_resume")
                    ));
                } else {
                    lines.push(format!(
                        "function {} with entity @s data",
                        resource("scene_update")
                    ));
                }
            }
            "scene_follow" => {
                let owner = identity::objective(&id)?;
                for selector in identity::selectors(K::LABEL, &owner.to_string(), "$(owner)") {
                    lines.push(format!("$execute as {selector} at @s run tp @e[type=minecraft:marker,tag={tag},scores={{{owner}=$(instance)}},limit=1] ~ ~ ~"));
                }
            }
            "scene_update" => {
                lines.push(format!("$tp {} ~ ~ ~", select(None)));
                for frame in &self.frames {
                    let i = self
                        .parts
                        .iter()
                        .position(|p| p.name == frame.name)
                        .unwrap();
                    lines.push(format!("$execute if score @s {age} matches {} as {} run data merge entity @s {{start_interpolation:0,interpolation_duration:{},transformation:{}}}",frame.at.get(),select(Some(i)),frame.duration.get(),frame.pose.snbt()));
                }
            }
            "scene_remove_scores" => lines.extend([
                format!("scoreboard players reset @s {group}"),
                "kill @s".into(),
            ]),
            _ => {
                let i: usize = part.strip_prefix("scene_part_").unwrap().parse().unwrap();
                let node = &self.parts[i];
                let block_key = if crate::function::uses_26_3_schema() {
                    "id"
                } else {
                    "Name"
                };
                lines.extend([
                    format!("tag @s add {visual}"),
                    format!("tag @s add {visual}.{i}"),
                    format!("$scoreboard players set @s {group} $(instance)"),
                    format!(
                        "data merge entity @s {{block_state:{{{block_key}:{}}},transformation:{}}}",
                        serde_json::to_string(&node.block.to_string()).unwrap(),
                        node.pose.snbt()
                    ),
                ]);
            }
        }
        Ok(lines.into_commands())
    }
}

impl<K: EntityKind> AsRef<Lifetime<K>> for Scene<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        &self.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scene() -> Scene {
        Scene::new(Lifetime::new(
            "test:scene".parse().unwrap(),
            Ticks::new(100),
        ))
    }
    fn populated() -> Scene {
        scene().part(
            "a",
            BlockId::minecraft("stone").unwrap(),
            Transform::default(),
        )
    }
    #[test]
    fn invalid_scenes_are_rejected_before_runtime_allocation() {
        for (definition, field) in [
            (scene(), "parts"),
            (
                populated().part(
                    "a",
                    BlockId::minecraft("stone").unwrap(),
                    Transform::default(),
                ),
                "name",
            ),
            (
                scene().part(
                    "a",
                    BlockId::minecraft("stone").unwrap(),
                    Transform::default().scaled([-1.0; 3]),
                ),
                "transform",
            ),
            (
                scene().part(
                    "a",
                    BlockId::minecraft("stone").unwrap(),
                    Transform::default().yaw(f32::NAN),
                ),
                "transform",
            ),
            (
                populated().keyframe(
                    "missing",
                    Ticks::new(1),
                    Ticks::new(5),
                    Transform::default(),
                ),
                "keyframe",
            ),
            (
                populated().keyframe("a", Ticks::new(0), Ticks::new(5), Transform::default()),
                "keyframe",
            ),
            (
                populated()
                    .keyframe("a", Ticks::new(1), Ticks::new(5), Transform::default())
                    .keyframe("a", Ticks::new(4), Ticks::new(2), Transform::default()),
                "keyframe",
            ),
        ] {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            assert_eq!(definition.build("scene_init").unwrap_err().field, field);
            assert!(crate::function::take_numeric_objectives().is_empty());
        }
        populated()
            .keyframe("a", Ticks::new(6), Ticks::new(5), Transform::default())
            .keyframe("a", Ticks::new(1), Ticks::new(5), Transform::default())
            .validate()
            .unwrap();
    }
}
