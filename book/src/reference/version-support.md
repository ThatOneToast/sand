# Version support

The latest known version is `26.2` (`data_fmt=107`, `res_fmt=88`). Known 26.x
and 1.21.x versions resolve to an exact profile from Sand's bundled version
table. Unknown or future versions use a conservative fallback profile — all
version-gated features (dialogs, jukebox songs, item components, and so on)
are treated as unsupported rather than guessed, so a build against an
unrecognized `mc_version` fails loudly on any feature that needs an exact
profile instead of silently emitting a schema that might be wrong.

`sand.toml`'s `mc_version` accepts `"latest"` (resolves to the anchor above)
or an explicit version string such as `"1.21.4"`. `sand build` fails with an
actionable error for a malformed version string rather than silently falling
back.

See [`sand::version`](https://docs.rs/sand) (`MinecraftVersion`,
`VersionProfile`) for the typed API, and
[Vanilla Limitations](vanilla-limitations.md) for what no version of Sand can
work around.
