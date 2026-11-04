# Companion resource packs

Sand can package optional client assets alongside a datapack. Start in an
existing project:

```sh
sand add resourcepack
```

This creates `resourcepack/assets/<namespace>/` and an author README, then opts
in through `sand.toml`. Repeating the command preserves configuration and assets.

```toml
[resource_pack]
source = "resourcepack"
# description = "My companion assets"
# pack_format = [97, 1] # explicit resource format override, normally unnecessary
```

`source` is a project-relative directory outside `dist`. Put assets under native
Minecraft paths, for example `resourcepack/assets/demo/sounds.json`,
`resourcepack/assets/demo/sounds/chime.ogg`, or
`resourcepack/assets/demo/textures/item/icon.png`. Optional `pack.png` belongs at
the source root. A root `README.md` is ignored. Sand generates `pack.mcmeta` from
configuration, so do not provide a source `pack.mcmeta`.

```sh
sand build
sand build --release --format json
```

For namespace `demo`, the outputs are separate:

| Artifact | Folder | Release archive |
| --- | --- | --- |
| Datapack | `dist/demo/` | `dist/demo.zip` |
| Client resources | `dist/demo-resources/` | `dist/demo-resources.zip` |

The optional JSON `resource_pack` result reports its output, exact format,
asset count, write/change/removal counts and, for release builds, archive path.
Projects without `[resource_pack]` retain datapack-only builds and omit that
result. Neither `sand add resourcepack` nor `sand build` installs a client pack,
downloads assets or configures server resource-pack delivery.

## Formats and authoring references

Resource formats differ from datapack formats. Sand's verified Mojang
`version.json` metadata gives 84.0 for 26.1 and 26.1.2, 88.0 for 26.2, and 97.1
for 26.3. The minor revision is retained in exact `min_format`/`max_format`
bounds. An unknown target requires an explicit resource `pack_format`; Sand
does not copy the datapack format or guess a future resource format.

Use existing typed Sand references to name authored assets: rich text font IDs,
sound resource IDs, and `CustomItem::custom_model_data` for client model
selection. Gameplay `custom_data` identifies an item for predicates and events;
it does not define its appearance. Packaging does not create models or convert
legacy model schemas.

## Validation and output ownership

Sand validates paths, JSON syntax and the pack icon's PNG signature before
publishing. Asset files retain their exact bytes. Symlinks, special files,
traversal, invalid resource paths, malformed JSON and conflicting generated
metadata are rejected. Models, textures, sounds, fonts, cross-file references
and visual appearance need validation in a Minecraft client; successful
packaging is not a client codec or rendering proof.

Both output folders are staged before either is replaced. On an ordinary
publication error, including a failed second installation, Sand restores the
previous folders. Ownership manifests prune only unchanged generated files,
preserve unrelated files and reject edits to managed output. Sources remain the
place to edit assets. Release archives are deterministic and each archive is
replaced through a temporary file; folder and archive publication are separate
steps. Crash atomicity and concurrent writes to one destination are not claimed.
