# Typedness and legacy-surface audit

This document records the repository-wide audit performed for the first API
consolidation slice. It is an inventory, not a compatibility plan: removed
surfaces are not retained as aliases.

| Surface | Previous API | Canonical replacement | This slice |
|---|---|---|---|
| state declarations | function-like declaration DSL | `#[derive(State)]` + `#[state(...)]` | removed |
| temporary objectives | separate function-like declaration DSL and export phase | fields in a scoped `State` schema | removed |
| entity schemas | entity-specific derive with associated handles only | `#[derive(State)]` with a concrete bound view | removed and migrated |
| manual objective lifecycle registry | register/drain helpers plus a parallel export phase | derive-emitted immutable lifecycle descriptors | removed |
| function command references | strings, `Display`, and `IntoFunctionRef` | one typed function reference/resolution path | follow-up; #175 open |
| selectors used as command targets | `Selector` and string-like parameters | `EntityTargets`, `PlayerTargets`, `SingleEntity`, `SinglePlayer`, `ScoreHolder` | follow-up |
| resource and registry identifiers | mixed strings and typed IDs | existing typed refs/IDs backed by `ResourceLocation` | follow-up |
| enchantment providers | whole-provider raw JSON only | `EnchantmentProvider`, typed IDs/tags, and typed constant/uniform integer providers | completed common vanilla shapes; #188 |
| storage and NBT paths | strings plus typed paths | `StorageLocation`, `NbtRef`, `NbtPath` | follow-up |

## Public function-like macro inventory

The audit found nine exported function-like macros. The two state/objective
declaration macros were removed in this slice. `all!`, `any!`, `mcfunction!`,
and `run_fn!` require a separate expression/inline-function consolidation.
The three resource-pack declaration macros are deferred with the resource-pack
architecture, as requested.

Internal `macro_rules!` generators used to implement registries, repetitive
event families, and tests are not public declaration languages.

## Retained low-level primitives

`ScoreVar`, `Flag`, `Timer`, `Cooldown`, and `GameState` remain implementation
or advanced command-building primitives because existing systems and state-flow
lowering use their operations directly. They are not the canonical declaration
API, and their parallel lifecycle registration methods have been removed.
Player/global score fields retain custom vanilla criteria and JSON-safe
objective display names through
`#[state(criterion = "...", display_name = "...")]`. Entity/living fields
reject that metadata until archetype descriptor lowering, collision
validation, and dirty-observer integration land in a later #298 slice. Direct
entity/living binding likewise requires archetype-provisioned objectives.

This slice establishes one canonical declaration path for the migrated
surface. It is not complete state-system consolidation.

## Ordered remaining slices

1. Complete #175 around a single typed function-reference resolution path.
2. Canonicalize score holders and player/entity target cardinality.
3. Canonicalize storage/NBT and registry/resource identifiers.
4. Complete #298 entity/living owner lifecycle integration, bundles/presence,
   queries/systems, migrations, and generated
   archetype/global-resource integration.
