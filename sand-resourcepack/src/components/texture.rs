use crate::component::{AssetContent, AssetOutput, ResourcePackComponent};

/// A raw texture registration that copies a PNG from the project source tree
/// into the resource pack.
///
/// `RawTexture` performs no JSON generation — it simply ensures the specified
/// image file ends up at the correct location inside
/// `assets/<namespace>/textures/` so that Minecraft models, GUIs, or other
/// components can reference it.
///
/// # Output files
///
/// | File | Purpose |
/// |---|---|
/// | `assets/<ns>/textures/<dest_path>.png` | The copied texture |
///
/// # Macro
///
/// Prefer the [`texture!`](sand_macros::texture) macro over constructing this
/// struct directly:
///
/// ```rust,ignore
/// use sand_macros::texture;
///
/// texture!(
///     id: "my_pack:item/custom_sword",
///     path: "src/assets/custom_sword.png",
/// );
/// ```
///
/// The `id` field follows the standard resource-location format
/// `<namespace>:<sub_path>`. The namespace in `id` is used as the asset
/// namespace (allowing textures from other namespaces to be overridden), and
/// `<sub_path>` is the path within `textures/`.
///
/// # Placeholder: programmatic generation
///
/// A future API will allow textures to be generated at build time (e.g. solid
/// colour fills, gradients, or simple progress bar strips) without requiring
/// a pre-existing PNG. Those variants will use the `AssetContent::Bytes`
/// path. For now, only `CopyFrom` is wired up.
pub struct RawTexture {
    /// Identifier used in diagnostics.
    pub name: &'static str,

    /// Namespace for the texture asset (usually matches the pack namespace,
    /// but can be any valid Minecraft namespace to override vanilla assets).
    ///
    /// Determines the output path prefix: `assets/<asset_namespace>/textures/`.
    pub asset_namespace: &'static str,

    /// Sub-path within `assets/<asset_namespace>/textures/` (without
    /// extension), e.g. `"item/custom_sword"` or `"block/custom_stone"`.
    pub dest_path: &'static str,

    /// Project-root-relative path to the source PNG.
    ///
    /// Example: `"src/assets/custom_sword.png"`.
    pub src_path: &'static str,
}

impl ResourcePackComponent for RawTexture {
    fn assets(&self, _namespace: &str) -> Vec<AssetOutput> {
        let output_path = format!(
            "assets/{}/textures/{}.png",
            self.asset_namespace, self.dest_path
        );
        vec![AssetOutput {
            path: output_path,
            content: AssetContent::CopyFrom(self.src_path.to_string()),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::AssetContent;

    #[test]
    fn asset_path_and_copy_source() {
        let tex = RawTexture {
            name: "custom_sword",
            asset_namespace: "my_pack",
            dest_path: "item/custom_sword",
            src_path: "src/assets/custom_sword.png",
        };
        let outputs = tex.assets("my_pack");
        assert_eq!(outputs.len(), 1);
        assert_eq!(
            outputs[0].path,
            "assets/my_pack/textures/item/custom_sword.png"
        );
        match &outputs[0].content {
            AssetContent::CopyFrom(src) => assert_eq!(src, "src/assets/custom_sword.png"),
            _ => panic!("expected CopyFrom"),
        }
    }

    #[test]
    fn cross_namespace_override() {
        // Override a vanilla texture by using "minecraft" as asset_namespace.
        let tex = RawTexture {
            name: "custom_stone",
            asset_namespace: "minecraft",
            dest_path: "block/stone",
            src_path: "src/assets/my_stone.png",
        };
        let outputs = tex.assets("my_pack");
        assert_eq!(outputs[0].path, "assets/minecraft/textures/block/stone.png");
    }
}
