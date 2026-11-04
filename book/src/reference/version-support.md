# Version support

Sand targets Minecraft Java 26.x and newer. A version below 26 is outside the
project's supported history and is rejected by configuration and version APIs.

The default version is `26.3` (`data_fmt=121`). Sand retains exact export
profiles for 26.1 (including 26.1.1/26.1.2) and 26.2. Generated command and
registry catalogs have audited 26.2 and 26.3 profiles. Function macros and the current command representation are the
baseline throughout this supported era; there is no older-syntax lowering
path.

Later calendar versions such as 27.x can be represented by the architecture,
but Sand does not claim their schemas are verified. `VersionProfile::resolve()`
uses conservative capabilities for an unknown future release, and
`VersionProfile::resolve_strict()` fails when exact generated Minecraft data is
required.

`sand.toml` accepts `mc_version = "latest"` or a supported explicit version such
as `mc_version = "26.3"`. Malformed and pre-26 values fail with an actionable
diagnostic.

See the [`sand::version` implementation](../../../sand-core/src/version.rs) for `MinecraftVersion`,
`VersionProfile`, and `VersionFeature`, and [Vanilla
Limitations](vanilla-limitations.md) for constraints Sand cannot work around.

## 26.3 migration boundary

Schema changes follow Mojang’s [26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3) and the verified server reports.

The generator reads the verified 26.3 server report, including `compute` and
`posteffect`. Generated compute builders distinguish integer/float and
server/entity/block contexts; entity context requires a single target.
Provider arguments currently accept native provider syntax. A top-level inline
constant must use an object, for example `{type:"minecraft:constant",value:3}`;
a bare command token is parsed as a registry reference.

The export backend lowers predicate, advancement-condition, item-modifier,
loot-entry, typed loot-number-provider, and biome-spawning schemas for 26.3. Typed configured features use
`worldgen/feature` and inline configuration. Older targets keep their previous
resource shapes and directories. Exact API-surface profiles remain enforced.

The legacy configured-carver builder fails explicitly on 26.3: removed
replacement/lava settings cannot be silently preserved by renaming its directory.
Use 26.2 for that builder, or explicitly author the new `worldgen/carver` schema
with `RawComponent`. Furnace gameplay remains gated to its verified 26.2 schema;
26.3 changed its native numeric fields. New registries are recorded as raw-only
until typed authoring is implemented. Version support does not imply complete
coverage of every newly added resource shape.

`StatCurve` still uses its established fixed-point backend. The new compute
command is not silently substituted for existing arithmetic: float precision,
rounding, overflow, absent inputs, and failure semantics need equivalent tests
before that optimization is enabled. The intended authoring surface remains
State and gameplay expressions rather than provider JSON.

The vanilla workflow keeps the established 26.2 gameplay proofs and adds a
26.3 resource load/reload audit and compute execution proof. Run the latter with:

```sh
python3 scripts/validate_compute_26_3.py --jar /path/to/26.3/server.jar --output /tmp/sand-compute-proof
```

The proof checks generated builders, integer arithmetic, float rounding, square
root, block/entity contexts, independent entity scores, calculation failure,
and execution again after a confirmed load-tag reload.
