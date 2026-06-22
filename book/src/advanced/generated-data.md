# Generated Data

Sand generates Minecraft registry-backed Rust types from the target Minecraft
version. The generated data is cached and reused by workspace builds.

Do not require network-heavy data regeneration in default CI unless a change
explicitly updates the generator or target data.
