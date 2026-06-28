# Version Capabilities

Sand's opt-in systems are Cargo features, not runtime toggles:

| Feature | Capability |
|---|---|
| `systems-damage` | cumulative-stat `DamageTracker` |
| `systems-cooldowns` | registered cooldown tick support |
| `systems-lifecycle` | manual join/death/respawn lifecycle helpers |
| `systems-player-data` | manual `PlayerSchema` builder helpers; implies lifecycle |
| `systems-movement` | push, launch, speed, slow helpers |
| `systems-inventory` | inventory checks and mutations |
| `systems-entities` | interaction-entity builder |
| `systems-all` | all optional systems |

`SandStorage`, `#[function]`, `#[component]`, and `#[event]` are provided by `sand-macros`; they are not Cargo feature gates. Minecraft itself remains version-sensitive: commands, item components, dialogs, interaction entities, and advancement JSON must be supported by the configured Minecraft version. Use `VersionProfile` for explicit component/version decisions.

`systems-player-data` currently exposes the `PlayerSchema` builder. It groups
score/flag/cooldown defaults and storage descriptors, but users still call
`define_all()` from load and `init_player(selector)` from join or first-join
handlers. Automatic lifecycle/export wiring is future #47/#68 work.

<div class="sand-version"><strong>Upgrade discipline.</strong> Never infer support from a Java API alone. Rebuild, inspect generated output, and test with the target server/client version when upgrading Minecraft.</div>

Feature support belongs in version profiles, not scattered string checks.

Common capability checks include dialogs, function macro behavior, registry
features, and data-component support. When an example depends on a gated feature,
call out the target Minecraft version near the example.
