# Version support

Sand targets Minecraft Java 26.x and newer. A version below 26 is outside the
project's supported history and is rejected by configuration and version APIs.

The latest verified version is `26.2` (`data_fmt=107`). Sand keeps exact
profiles for known 26.x releases, including the schema distinction between
26.1 and 26.2. Function macros and the current command representation are the
baseline throughout this supported era; there is no older-syntax lowering
path.

Later calendar versions such as 27.x can be represented by the architecture,
but Sand does not claim their schemas are verified. `VersionProfile::resolve()`
uses conservative capabilities for an unknown future release, and
`VersionProfile::resolve_strict()` fails when exact generated Minecraft data is
required.

`sand.toml` accepts `mc_version = "latest"` or a supported explicit version such
as `mc_version = "26.2"`. Malformed and pre-26 values fail with an actionable
diagnostic.

See [`sand::version`](https://docs.rs/sand) for `MinecraftVersion`,
`VersionProfile`, and `VersionFeature`, and [Vanilla
Limitations](vanilla-limitations.md) for constraints Sand cannot work around.
