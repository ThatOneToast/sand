//! Owned frontend-neutral gameplay model, shared by construction and decoding.
mod data;
mod envelope;
mod flow;
mod function;
mod gameplay;
mod operation;
mod resources;
mod state;
pub use data::{BlockDataAddress, DataOperand, DataProjection, DataReference};
pub use envelope::{Module, ModuleSource, Pack, Program, ProgramTarget};
pub use flow::{Flow, FlowHook, FlowHookPhase, FlowTransition};
pub use function::{ExecutionContext, Function, FunctionReference, TagMembership};
pub use gameplay::{
    Amount, Audience, BiomeMatches, BlockMatches, BlockPosition, Condition, DimensionMatches,
    EntitiesExist, EntityData, EntityFlag, EntityQuery, EntitySort, EventHook, FlagSet, ItemHeld,
    MessageChannel, PlayerEvent, PlayerGameMode, Relation, ScaledField, ScoreOperand,
    ScoreOperator, ScoreRelation, StateAttached, TeleportDestination, TextColor, TextPart,
    TextStyle, Weather,
};
pub use operation::{AbilityCommand, Action, Operation, SessionCommand};
pub use resources::{
    AbilityCost, AbilityDefinition, Advancement, AdvancementCriterion, AdvancementDisplay, BossBar,
    BossBarColor, BossBarStyle, CombatCritical, CombatDefinition, CookingMethod, CustomBiome,
    CustomItem, DimensionDefinition, FlatDimension, HudBossBar, HudDefinition,
    InventoryTransferDefinition, JobDefinition, LootItem, LootPool, LootTable, MenuChoice,
    MenuDefinition, MenuView, NoiseDimension, NoticeDialog, ProgressionBranch,
    ProgressionDefinition, ProgressionObjective, ProjectileDefinition, Rarity, Recipe,
    RecipeOutput, ResourceTag, SceneContent, SceneDefinition, SceneKeyframe, ScenePart,
    SessionDefinition, StateCooldown, TagKind, WorldPredicate, WrittenBookContent,
};
pub use state::{DataField, FieldReference, ScoreComparison, ScoreField, State};
