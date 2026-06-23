# Coordinates And Rotation

`Vec3`, `Coord`, `BlockPos`, and `Rotation` distinguish absolute, relative `~`, and local `^` coordinates.

```rust
use sand_commands::{Coord, Vec3, tp_vec3, tp_with_rotation};
tp_vec3(Selector::self_(), Vec3::new(Coord::rel(), Coord::rel_n(1.0), Coord::rel()));
tp_with_rotation(Selector::self_(), Vec3::absolute(0.0, 64.0, 0.0), Rotation::absolute(90.0, 0.0));
```

Use local coordinates only when execution facing is deliberate. `Vec3::here()` is `~ ~ ~`; compose offsets with `Coord::rel_n`/`local_n`.
