# Editor / LSP integration — maintainer notes

Audience: Sand maintainers evaluating what an editor integration (Zed,
IntelliJ, VS Code, …) could realistically do today, and what compiler work
would be required to do better. This is **not** user-facing documentation and
makes no promises about a shipped feature.

**Reality check up front: Sand does not have a language server today.** There
is no `sand-lsp` crate, no LSP server binary, and no editor extension in this
repository. Everything below either describes what already exists and could
be *reused* by an editor integration, or is explicitly marked future work.

## 1. Project discovery

An editor integration would find a Sand project the same way `sand-cli`
does: look for `sand.toml` in the current directory. There is currently
**no upward directory search** — every call site reads it from
`std::env::current_dir()` directly and fails if it's missing there:

- `sand-cli/src/build/mod.rs:26-31` (`sand build`)
- `sand-cli/src/run_cmd.rs:22-25` (`sand run`)
- `sand-cli/src/join_cmd.rs:10-12` (`sand join`)
- `sand-cli/src/add_cmd.rs:308-312` (`sand add`)

The schema (`sand-cli/src/config.rs:1-45`) is:

```rust
struct SandConfig {
    pack: PackConfig,                       // required
    resourcepack: Option<ResourcePackConfig>, // optional
}
struct PackConfig {
    namespace: PackNamespace,   // validated Minecraft namespace
    description: String,
    mc_version: String,         // "latest" or an explicit version string
    pack_format: Option<u32>,   // derived from mc_version if omitted
}
```

A Sand project is also an ordinary Cargo package/workspace member (it depends
on `sand` per ADR-001, or on `sand-core`/`sand-macros` directly pre-façade)
with a generated `src/bin/sand_export.rs` binary target (see
`sand-cli/src/scaffold.rs:170-185`, template
`sand-cli/src/templates/default/sand_export_rs.hbs`). An editor integration
that only needs "is this a Sand/Rust project" can check for **both**
`sand.toml` and `Cargo.toml` next to each other; `sand.toml` alone
distinguishes a Sand pack from an arbitrary Rust crate.

## 2. Build/check entry points available today

Nothing Sand-specific needs to be built to get *some* diagnostics — a plain
Rust toolchain is enough:

- **`cargo check -p <project>`** (or plain `cargo check` inside the project
  directory) — compiles the crate, including all `#[function]`/`#[component]`/
  `#[event]`/`#[item]` proc-macro expansions, and reports ordinary rustc
  diagnostics (type errors, unresolved names, and — importantly — proc-macro
  attribute-parsing errors emitted via `syn::Error::to_compile_error()`, see
  §4). This is the cheapest, fastest thing an editor integration could shell
  out to, and it's what `rust-analyzer` already does automatically for any
  Rust project, Sand or not. **No Sand-specific wiring is needed for this
  tier** — rust-analyzer already gives Sand authors macro-expansion diagnostics
  for free today.
- **`sand build`** — what `sand-cli/src/build/mod.rs:81-105` actually does:
  1. `cargo build --bin sand_export` (with `RUSTFLAGS=-Awarnings`) — a full
     compile, status inherited straight to the terminal, so **normal rustc
     diagnostics only**.
  2. Runs the resulting `sand_export` binary and captures its stdout (a JSON
     array of component records) and stderr.
  3. If the binary exits non-zero, `sand build` `bail!`s with the *entire*
     captured stderr text as one opaque error string (`sand-cli/src/build/mod.rs:100-104`).
  4. If it exits zero but produces malformed/unexpected JSON, `sand build`
     synthesizes a best-effort text hint (`sand-cli/src/build/mod.rs:107-125`).

So today, a `sand build` failure surfaces as: either a normal `cargo build`
diagnostic (span-accurate, already IDE-legible via rust-analyzer/rustc), or a
single formatted string from a `Result::Err(SandError)` / a Rust panic with no
attempt at structure beyond what `Display` produces. There is no
`--message-format=json`-equivalent for the second category.

## 3. Compiler diagnostic phases (where these errors actually come from)

The export pipeline lives under `sand-core/src/compiler/export/` (module doc:
`sand-core/src/compiler/export/mod.rs:1-9`):

| Module | Phase |
|---|---|
| `pipeline.rs` | collection → aggregation → assembly driver (the "collect" phase; there is no separate `collect.rs` file — collection is inline in `pipeline.rs`) |
| `records.rs` | component → `ComponentRecord` boundary |
| `events.rs` | event graph lowering |
| `predicates.rs`, `armor.rs`, `schedules.rs`, `dialogs.rs`, `functions.rs` | per-kind export logic |
| `lifecycle.rs` | state lifecycle (load/tick) descriptor assembly |
| `diagnostics.rs` | final command-string validation phase — validates every collected `.mcfunction` line against the version-resolved command profile *before* any record is accepted (`sand-core/src/compiler/export/diagnostics.rs:1-38`) |
| `tags.rs` | function/item tag assembly |

The structured error type these phases return is
`sand_components::error::SandError`, re-exported as `ComponentExportError`
(`sand-core/src/component.rs:14`) and mirrored by `sand-core`'s own
`SandError` (`sand-core/src/error.rs:5-65`, converted via `From` at
`sand-core/src/error.rs:70-105`). Its two diagnostic-carrying variants
(`sand-components/src/error.rs:28-69`):

```rust
ComponentValidation {
    location: ResourceLocation, // e.g. "trail:grapple_core" — the GENERATED
                                 // resource's namespace:path, not a Rust path
    kind: String,                // "recipe", "advancement", "function", ...
    field: String,                // e.g. "commands[1].limit" or a JSON field path
    message: String,
},
VersionGating {
    location: String,
    kind: String,
    requested_version: String,
    is_fallback: bool,
    feature_name: String,        // e.g. "dialogs"
    fallback_note: String,
},
```

Both already carry a *generated-resource* location and a *field path* —
useful for saying "advancement `trail:obtain_core`, field `criteria`, said
X" — but neither carries any Rust source information. See §4.

## 4. Source-span availability — the honest split

Two categories of error exist, with very different span stories:

1. **Proc-macro attribute misuse** (e.g. malformed `#[event(...)]` attribute
   syntax, missing required arguments). `sand-macros` builds these with `syn`,
   which *does* carry real `proc_macro2::Span` positions (grep hits at, e.g.,
   `sand-macros/src/lib.rs:2542` capturing `lit.span()`), and reports them via
   `syn::Error::to_compile_error()` at all eight `#[proc_macro_attribute]`/
   `#[proc_macro]` entry points (`sand-macros/src/lib.rs:180,357,650,1547,1680,
   1749,1783,2438,2667,2835,3175`). These become ordinary `rustc` compile
   errors with accurate spans — **already fully IDE-mappable today**, via
   `cargo check`/rust-analyzer, no extra work needed.
2. **Runtime export/validation errors** — everything in §3
   (`ComponentValidation`, `VersionGating`), plus any `panic!`/`.unwrap()`/
   `.expect()` inside a `#[function]`/`#[component]`/`#[item]` body (e.g. a
   bad `ResourceLocation::new(...).unwrap()`). These are **only** discovered
   by *running* code — a `cargo test`, the `sand_export` binary, or a
   doctest — not by `cargo check` alone, since the invalid value is
   constructed and validated at runtime inside an ordinary function body, not
   during macro expansion. `ComponentExportError`/`SandError` carry **no**
   `proc_macro2::Span`, no file, and no line number — only the generated
   resource's `namespace:path` and a `field` string. A panic does get a
   real Rust `file:line` from the default panic hook, but that's a
   plain-text panic message, not a structured diagnostic, and nothing
   parses it back into an editor position today.

## 5. Mapping generated-resource errors back to Rust declarations

**Not implemented today.** A `ComponentValidation` error such as
`component `trail:grapple_core_recipe` (recipe): ... [field: pattern]` tells
you *which generated resource* (by its Minecraft `namespace:path`) and *which
JSON field* failed, but nothing in the error, and nothing in the
`ComponentFactory`/`inventory`-registration machinery that produced it,
records which Rust function/file/line built that value. There is no reverse
index from `ResourceLocation` back to a `(file, line)` of the `#[component]`/
`#[item]` function that returned it. Building one would require either:

- proc-macro-side: have `#[component]`/`#[item]`/`#[function]` capture
  `Span::call_site()` (or the source-file/line/column accessors
  `proc_macro::Span` exposes on stable) at expansion time and thread it
  through the `inventory`-registered descriptor so the exporter can attach it
  to any resulting error; or
- a naming convention plus a source-scanning pass correlating
  `ResourceLocation`s to the Rust item that declared them (fragile, easy to
  drift, not recommended as a first step).

Neither exists; this is future work, not a partially-built feature.

## 6. Minecraft version diagnostics

Version-gated rejections surface exactly like other validation errors: as a
`Result<_, SandError>` / `Result<_, String>` at **export time**
(`VersionCaps`-driven checks inside the `compiler/export/` phases, surfaced
through `resolve_export_caps` at `sand-core/src/version.rs:803-824`, itself
called from generated `__sand_export` hooks like
`examples/book_project/src/lib.rs:369-389`). This is late: a dialog that
requires a newer Minecraft version than `sand.toml`'s `mc_version` is only
rejected when you run `sand build` (or the export binary directly), not when
you write the offending code.

A real-time editor diagnostic ("this dialog needs 1.21.6+, your `sand.toml`
targets 1.21.4") would need version-capability validation to move *earlier*
— realistically into proc-macro expansion itself, where `#[component]`/
`#[event]` could read the target version (from `sand.toml`, or an
environment variable an LSP could set) and reject at `cargo check` time using
the same `syn::Error` span mechanism described in §4. This is a real compiler
change (moving `VersionCaps` gating from the runtime export pipeline into
`sand-macros`, or exposing it to `sand-macros` some other way), not just an
editor-side change. Not started.

## 7. Possible Zed / IntelliJ integration paths (realistic, incremental)

- **Zed**: Zed's extension API supports registering a custom LSP server per
  language. A minimal Sand extension could point Zed's existing Rust
  language support (already present) at the project, and additionally spawn
  a thin wrapper process that runs `cargo check --message-format=json` (or
  `cargo check -p <pkg> --message-format=json`) and translates rustc's JSON
  diagnostics into LSP `publish_diagnostics`. This gets you §4 category 1
  (proc-macro span errors) for free, and nothing from category 2 (export-time
  errors) — that's the honest ceiling of a "wrap cargo check" server.
- **IntelliJ (via the Rust plugin / a custom plugin)**: same ceiling — the
  existing Rust plugin already surfaces `cargo check` diagnostics; a
  Sand-specific IntelliJ plugin would only add value by understanding
  `sand.toml` (e.g. project templates, `sand build`/`sand run` run
  configurations) rather than by adding new diagnostics, until §5/§6 are
  addressed compiler-side.
- **Longer-term, real Sand diagnostics** (dialog/version errors, "which
  Rust function produced this failing resource") require the compiler
  changes in §5 and §6 first — moving validation earlier and threading spans
  through — before any editor server can show something better than "your
  crate failed to compile" / "the export binary printed this string".

## 8. What's implemented now vs. future work

**Implemented now:**
- `sand.toml` schema and single-directory (non-recursive) project discovery
  (`sand-cli/src/config.rs`, `sand-cli/src/build/mod.rs`).
- `cargo check`/`cargo build` already report accurate, span-mapped
  diagnostics for proc-macro attribute misuse via `syn::Error`
  (`sand-macros/src/lib.rs`) — usable by any editor today with zero new code,
  through rust-analyzer.
- A phased export pipeline (`sand-core/src/compiler/export/`) producing
  structured `ComponentExportError`/`SandError` values with a generated
  resource location, kind, field path, and message
  (`sand-components/src/error.rs`, `sand-core/src/error.rs`) — better than a
  bare string, but still runtime-only and Rust-source-location-free.
- `sand build`'s two-stage flow (`cargo build --bin sand_export` then run the
  binary) that an editor could shell out to today for a coarse pass/fail
  signal plus rustc diagnostics.

**Future work (not started):**
- No LSP server exists in this repository.
- No structured (JSON, span-carrying) diagnostic channel for export-time
  errors (`ComponentValidation`, `VersionGating`, panics) — today they are
  formatted `Display` strings on stderr.
- No mapping from a generated resource's `ResourceLocation` back to the Rust
  `(file, line)` of the `#[component]`/`#[item]`/`#[function]` that produced
  it.
- No early (macro-expansion-time) `VersionCaps` validation — version gating
  only happens at export/build time today.
- No editor extension (Zed, IntelliJ, VS Code, or otherwise) ships with this
  repository.
