# Editor support for the Sand language

- [`tree-sitter-sand/`](tree-sitter-sand/grammar.js) — tree-sitter grammar used
  for highlighting, brackets, indentation, and outlines. Regenerate the parser
  after editing `grammar.js`:

  ```sh
  cd editors/tree-sitter-sand
  npx tree-sitter-cli@0.25.10 generate --abi 14
  ```

  The compiler's grammar is `sand-lang/src/sand.pest`; keep both in step.

- [`zed/`](zed/src/lib.rs) — the Zed extension: language configuration,
  queries, and a launcher for `sand lsp`. `sand lang zed` embeds these files
  and installs them with a pinned local copy of the grammar, so most users
  never touch this directory. Keep `zed/languages/sand/highlights.scm` identical
  to `tree-sitter-sand/queries/highlights.scm`.

Any other editor can run `sand lsp` as a stdio language server for `*.sand`.
See the [Sand language guide](../book/src/language.md#editor-setup-zed).
