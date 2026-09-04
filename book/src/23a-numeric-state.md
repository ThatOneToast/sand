# Numeric State: scales, rounding, and arithmetic

Sand has one numeric model for persistent gameplay State:

- `Score` is a whole-number value.
- `FixedScore` is a decimal value.

Both describe **logical gameplay values**. A health value of `20` means twenty
health points and a speed multiplier of `1.25` means one and one quarter. You
do not convert either value to Minecraft scoreboard units in gameplay code.

```rust,ignore
#[derive(State)]
#[state(namespace = "trailforge", scope = living)]
pub struct Stats {
    #[state(default = 20, min = 0, max = 10_000)]
    pub health: Score,

    #[state(default = 2, min = 0, max = 1_000)]
    pub heal_amount: Score,

    #[state(default = 1.25, min = 0, max = 10, scale = 100)]
    pub speed_multiplier: FixedScore,

    #[state(default = 0.10, min = 0, max = 1, scale = 1000)]
    pub resistance: FixedScore,
}
```

## What a scale means

Minecraft scoreboards store signed 32-bit integers. A `FixedScore` scale says
how many of those integer units represent one logical unit:

| Declaration | Logical value | Stored scoreboard value | Precision |
| --- | ---: | ---: | ---: |
| `Score` | `20` | `20` | whole numbers |
| `FixedScore`, `scale = 10` | `1.2` | `12` | tenths |
| `FixedScore`, `scale = 100` | `1.25` | `125` | hundredths |
| `FixedScore`, `scale = 1000` | `1.237` | `1237` | thousandths |

The default `FixedScore` scale is `1_000`. Choose a smaller explicit scale when
the domain has a natural precision, such as hundredths for percentages or
currency-like values. A larger scale preserves more fractional digits but
reduces the largest logical magnitude that fits in a scoreboard. At scale
`1_000`, for example, the raw i32 limit corresponds to roughly ±2.1 million
logical units.

A scale is part of the field's schema and storage format. Changing it after a
pack has shipped changes the interpretation of existing scores, so treat that
as a data migration: multiply or divide stored values in a State migration and
then publish the new schema version.

## Defaults and bounds are logical values

Decimal defaults, minimums, and maximums on `FixedScore` are written in
logical units:

```rust,ignore
#[state(default = 1.25, min = -2.5, max = 8.0, scale = 100)]
pub modifier: FixedScore,
```

This declares a stored default of `125` and inclusive stored bounds of `-250`
through `800`. Bounds are applied after a mutation or derivation has been
converted to the destination representation. They are not applied to each
intermediate term, which avoids changing the meaning of a formula merely
because it was split into several operations.

`Score` defaults and bounds are integers because a `Score` cannot retain a
fraction. Use `FixedScore` if fractions are meaningful even when the current
default happens to be whole.

## Mutating values directly

Bound accessors accept logical values:

```rust,ignore
let stats = Stats::on(entity);
stats.health.add(2);                 // twenty becomes twenty-two
stats.speed_multiplier.add(0.10);   // 1.25 becomes 1.35
stats.speed_multiplier.set(0.875);  // stored as 88 at scale 100
```

A bound numeric field is also a valid source. Both accessors keep the same
current owner, and Sand uses the source and destination scales automatically:

```rust,ignore
let stats = Stats::on(entity);
stats.health.add(stats.heal_amount);
stats.speed_multiplier.subtract(stats.resistance);
```

This works for same-scale and cross-scale pairs, including `Score` to
`FixedScore` and `FixedScore` to `Score`. Conversion rounds only when the
destination cannot retain the source precision.

The final line demonstrates rounding: `0.875 × 100` is exactly `87.5`, so it
becomes `88`. Mutations clamp to the field bounds and, for entity/living State,
mark both the field and its component dirty. Archetype derivations and native
property reconciliation therefore see the change without hand-written
objective names.

## Derivations infer their destination

An `EntityDerivation` takes any numeric State field. The target chooses the
stored representation:

```rust,ignore
let damage = StatCurve::multiply([
    StatCurve::state(Stats::base_damage),
    StatCurve::state(Stats::damage_multiplier),
]);

EntityArchetype::<ZombieKind, Stats>::new(id)
    .derive(EntityDerivation::new("damage", Stats::damage, damage));
```

If `damage` is a `Score`, Sand rounds the final logical result once and stores
a whole number. If it is `FixedScore(scale = 100)`, Sand stores hundredths.
There is no `hundredths()` helper, output-encoding switch, or matching
`FixedPoint` configuration to keep synchronized.

`StatCurve::state` also reads the source field's scale. An expression may mix
`Score`, `FixedScore(scale = 10)`, `FixedScore(scale = 100)`, and
`FixedScore(scale = 1000)`. Sand reduces conversion ratios, creates
deterministic entity-scoped scratch objectives, and emits integer scoreboard
operations. Scratch objectives are compiler-owned and are not State fields.

## Rounding

The canonical policy is **nearest, with exact halves away from zero**:

| Logical result targeting `Score` | Stored result |
| ---: | ---: |
| `1.49` | `1` |
| `1.50` | `2` |
| `-1.49` | `-1` |
| `-1.50` | `-2` |

The same rule is used when a decimal constant is encoded, when a source is
converted between scales, after fixed-point multiplication or division, and
at the final `FixedScore`-to-`Score` boundary. Sand's build-time evaluator and
Minecraft lowering use the same integer quotient/remainder definition so
negative values and exact halves do not depend on host-language behavior.

Rounding necessarily discards information when the destination is less
precise. Keep intermediate and persistent values as `FixedScore` when later
calculations need their fractions; choose `Score` when the gameplay value is
fundamentally integral.

## Overflow and division safety

State bounds and machine overflow are different:

- Declared `min`/`max` bounds define valid gameplay values and clamp the final
  stored result.
- A value that cannot fit the scoreboard's signed i32 representation is a
  backend overflow, even if its mathematical logical value is valid.
- Curve constants are checked during export. Non-finite constants, invalid
  scales, and values that cannot be represented stop the build with a
  diagnostic.
- Runtime division guards against a zero divisor. A zero denominator fails
  that generated derivation function instead of emitting an arbitrary value.
- Scratch names and allocation order are deterministic, so the same schema and
  expression produce the same datapack output.

Multiplication can temporarily require a larger integer than its final result.
Avoid formulas whose intermediate raw value approaches the i32 limit. Prefer
smaller meaningful scales and algebraically smaller terms. Sand reduces
cross-scale ratios before lowering, but Minecraft scoreboards still impose an
i32 ceiling on every runtime intermediate.

## Native Minecraft properties

Native bindings consume the logical meaning of the State field. Keep the
representation decision in the State declaration and pass the typed field:

```rust,ignore
archetype
    .health(HealthBinding::new(Stats::health))
    .attribute(AttributeBinding::new(
        AttributeType::MovementSpeed,
        NumericPropertySource::state(Stats::speed_multiplier),
    ));
```

Do not pre-scale a value because an underlying command happens to use a float,
integer, NBT number, or attribute base. The binding owns that Minecraft
boundary.

## Advanced fixed-point control

`FixedPoint`, `RoundingPolicy`, `OverflowPolicy`, and raw objective APIs remain
available for custom compiler-facing curves and integrations. They describe
the working representation of the lowering engine; they are not additional
State field types. Ordinary State and archetype code should start with
`Score`, `FixedScore`, and typed `StatCurve::state` inputs. Reach for the
advanced controls only when a deliberately different intermediate precision
is part of the gameplay contract, and still let the destination State field
choose what is stored.
