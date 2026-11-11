//! Zed extension for the Sand datapack language.
//!
//! Highlighting comes from the tree-sitter grammar; everything semantic
//! (diagnostics, hover, completion, navigation, rename) comes from the Sand
//! language server, started as `sand lsp`.

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct SandExtension;

impl zed::Extension for SandExtension {
    fn new() -> Self {
        SandExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // `lsp.sand.binary` in Zed settings overrides discovery.
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let command = binary
            .as_ref()
            .and_then(|binary| binary.path.clone())
            .or_else(|| worktree.which("sand"))
            .ok_or_else(|| {
                "Sand: the `sand` command was not found on PATH. Install it with \
                 `cargo install --path sand-cli` from the Sand repository, or set \
                 `lsp.sand.binary.path` in your Zed settings."
                    .to_string()
            })?;
        let args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_else(|| vec!["lsp".into()]);
        Ok(zed::Command {
            command,
            args,
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(SandExtension);
