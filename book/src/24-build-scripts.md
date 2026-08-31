# 24. Build Scripts (`sand.build.rs`)

`sand.build.rs` is Sand's typed, build-time world/server configuration
entry point — modeled on Rust's own `build.rs`. It's optional: a project
with no `sand.build.rs` behaves exactly as before this feature existed.

## Adding one

```console
$ sand add worldbuild
Adding sand.build.rs to my_pack...
  created sand.build.rs
  updated Cargo.toml
```

This scaffolds a starter `sand.build.rs` and wires it into `Cargo.toml` as
an ordinary `[[bin]]` target:

```toml
[[bin]]
name = "sand_build_world"
path = "sand.build.rs"
```

Sand compiles this the same way it compiles your `sand_export` binary —
through Cargo, with full access to your project's dependencies (including
`sand` itself). There is no separate build system to learn.

## The entry point

```rust,ignore
use sand::build::{BuildContext, SandBuild, World, Spawn};

fn build(ctx: &BuildContext) -> SandBuild {
    SandBuild::new().world(
        World::new().spawn(Spawn::at(0, 65, 0))
    )
}

fn main() {
    let profile = sand::build::BuildProfile::parse(
        &std::env::var("SAND_BUILD_PROFILE").unwrap_or_else(|_| "dev".to_string()),
    );
    let ctx = BuildContext::new(profile);
    sand::build::run_and_print("my_pack", ctx, build);
}
```

`build` receives a `BuildContext` carrying the resolved
[profile](./25-development-and-release-profiles.md) and target Minecraft
version, and returns a `SandBuild` — the top-level value combining a 🌍
[`World`](./26-worlds-dimensions-and-generators.md) and an optional 🖥️
[`ServerConfig`](./30-server-configuration.md).

## `SandBuild`

```rust
use sand::build::{SandBuild, World, ServerConfig};

let built = SandBuild::new()
    .world(World::new())
    .server(ServerConfig::new());

assert!(built.validate().is_ok());
```

`SandBuild::validate()` runs before anything is written to disk — see
[Generated World Resources](./32-generated-world-resources.md) for what
gets validated and how failures are reported.

## When it runs

`sand build` and `sand run` both compile and execute `sand_build_world`
after the ordinary component export step, passing the resolved profile
(`SAND_BUILD_PROFILE`) and Minecraft version (`SAND_EXPORT_MC_VERSION`) as
environment variables. Its output — validated world resources and an
optional server config — is written into `dist/` alongside your ordinary
datapack output.

```console
$ sand build --profile dev
Building my_pack (Minecraft 26.2, pack_format 107)...
Done! 6 component(s) written to dist/my_pack/ ...
Building sand.build.rs (profile: dev)...
  Done! 3 world resource(s) from sand.build.rs
```

## Conditional composition

Because `build` is ordinary Rust, branching on the profile is just an `if`:

```rust,ignore
fn build(ctx: &BuildContext) -> SandBuild {
    if ctx.profile().is_dev() {
        // fast flat world, verbose spawn platform, auto-reset
    } else if ctx.profile().is_release() {
        // full vanilla noise generation
    } else {
        // custom profile — e.g. "staging"
    }
}
```

See [Development Profiles](./25-development-and-release-profiles.md) for
the full `dev`/`test`/`bench`/`release` story.
