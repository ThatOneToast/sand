# Installation

Sand is currently used from this workspace while the CLI is being prepared for
publication.

Requirements:

- Rust 1.93+ with edition 2024 support.
- Java 21+ for Minecraft data generation during builds.
- Network access on the first build so generated Minecraft data can be cached.

Create and build a project from the workspace:

```sh
cargo run -p sand -- new my_pack
cd my_pack
cargo run -p sand -- build
```

Build this book locally with:

```sh
mdbook build
mdbook serve --open
```
