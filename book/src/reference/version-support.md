# Version support

The latest known version is `26.2` (`data_fmt=107`, `res_fmt=88`). Known 26.x
and 1.21.x versions resolve to an exact profile from Sand's bundled version
table. Unknown or future versions use a conservative fallback profile — all
version-gated features (dialogs, jukebox songs, item components, and so on)
are treated as unsupported rather than guessed, so a build against an
unrecognized `mc_version` fails loudly on any feature that needs an exact
profile instead of silently emitting a schema that might be wrong.

Function macro lines (`$...` with `$(name)` placeholders) are accepted only
for exact Minecraft 1.20.2+ profiles. Older and conservative fallback
profiles fail at the final function export boundary with the owning function
and line index in the diagnostic.

`sand.toml`'s `mc_version` accepts `"latest"` (resolves to the anchor above)
or an explicit version string such as `"1.21.4"`. `sand build` fails with an
actionable error for a malformed version string rather than silently falling
back.

See [`sand::version`](https://docs.rs/sand) (`MinecraftVersion`,
`VersionProfile`, and `VersionFeature`) for the typed API. Custom export hooks
use `sand::advanced::try_export_components_json`, which resolves capabilities
internally, and
[Vanilla Limitations](vanilla-limitations.md) for what no version of Sand can
work around.

## Typed world/server configuration (`sand::build`)

`sand::build`'s `SandBuild::validate()` (see the book's "Generated World
Resources" chapter) performs structural/range validation — world border
size, duplicate dimension slots, flat generator layer sanity — but does not
currently gate any world-build field on a `VersionFeature` or resolve
world-generation resource references against a version's real registries.
Every `Generator`/`DimensionType` variant this issue shipped (flat, void,
vanilla noise presets, custom references) is stable vanilla behavior across
Sand's supported version range, so there was nothing to gate yet — but a
future dimension/generator feature that *is* version-specific would need an
explicit `VersionFeature` addition here, and a full `sand-vanilla-audit`
registry audit of world-build resources remains a tracked follow-up (see
`ROADMAP.md`).
