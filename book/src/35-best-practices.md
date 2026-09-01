# 35. Best Practices

- **Keep `sand.build.rs` free of side effects beyond `SandBuild`
  construction.** It runs on every `sand build`/`sand run`; treat it like
  ordinary configuration code, not a place to shell out or write files
  directly. If you need to reference a hand-authored resource (a custom
  noise settings file, a hand-written `dimension_type`), use the typed
  `*Ref`/`Custom(ResourceLocation)` escape hatches to point at it — don't
  try to generate it from `sand.build.rs`.
- **Branch on `ctx.profile()` predicates, not string comparison.** Use
  `ctx.profile().is_dev()` rather than
  `ctx.profile().name() == "dev"` — the predicates exist so profile
  handling reads clearly and so `BuildProfile::Custom` names never need an
  exhaustive string match.
- **Always give `test`/`bench` profiles a fixed `Seed`.** Reproducibility
  is the entire value of those profiles — see
  [Testing And Benchmark Worlds](./32-testing-worlds.md).
- **Validate early, in your own tests too.** `SandBuild::validate()` is a
  plain function you can call directly in a `#[cfg(test)]` module, without
  going through `sand build` at all:

  ```rust,ignore
  use sand::build::{BuildContext, BuildProfile, SandBuild};

  #[test]
  fn dev_profile_world_is_valid() {
      let ctx = BuildContext::new(BuildProfile::Dev);
      // `build` is your project's own `sand.build.rs` build function.
      // assert!(build(&ctx).validate().is_ok());
  }
  ```
- **Don't reach for a custom profile name unless `dev`/`test`/`bench`/
  `release` genuinely don't fit.** A `staging` profile is a legitimate use
  of `BuildProfile::Custom`, but most projects only need the four
  well-known profiles.
- **Keep `ServerConfig` minimal.** Only set fields you actually need to
  differ from vanilla defaults — every field you set is one more thing to
  remember doesn't travel with the datapack (see
  [chapter 24](./23-world-vs-server.md)).
- **Remember the badge when writing your own docs.** If your project
  exposes any of its own build-time toggles that end up in `World` vs.
  `ServerConfig`, badge them the same way (🌍/🖥️) so downstream users of
  your datapack aren't surprised.
