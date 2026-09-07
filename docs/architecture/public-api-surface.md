# Public API surface enforcement

Sand treats every reachable public façade item as an explicitly owned API
contract. `sand/build.rs` reconstructs the source and generated declaration
graph, assigns each identity to one scope from `sand/api-scopes.toml`, and
compares the deterministic aggregate report with the selected baseline.

Generated command and vanilla-registry providers embed their resolved
Minecraft version. They must agree on an exact reviewed profile from
`sand/api-surface-profiles.toml`; unknown or mixed generated data fails closed.
The normal profile targets Minecraft Java 26.2. Placeholder codegen remains an
internal compile-recovery mechanism with a separate baseline and cannot claim
real generated registry coverage.

An ordinary build fails for an uncontracted public item, conflicting ownership,
provider drift, missing useful Rustdoc, or a baseline mismatch. Input-dependent
macro expansion is covered by isolated compile fixtures using the same contract
guards. The generated report is the authoritative source for current counts;
this document intentionally does not duplicate them.
