# Portable gameplay flow evidence

The Python `encounter_flows()` producer compiles through a prebuilt Sand binary
with Cargo absent from PATH. The resulting pack is loaded unchanged into
Minecraft 26.2. The retained `portable-flows-evidence.json` records the pack digest
and 21 successful assertions: global transitions and cadence with zero players,
guarded/fallback player transitions, enter-before-tick order, energy costs,
player isolation, state-specific hooks, and reload/cadence preservation.

Automatic ticks are frozen after startup; explicit generated lifecycle/flow
calls make each cycle observable. Two real passive clients provide player
entities. This proves server behavior, not client UI input or performance.

Reproduce with a fresh output directory:

```sh
cargo build -p sand-cli --bin sand
python3 scripts/validate_portable_flows.py --binary target/debug/sand \
  --jar "$HOME/.sand/cache/26.2/server.jar" --output target/portable-flow-audit
```

CI runs the audit on the latest verified target. The external acceptance tests
also compare the complete golden pack for Python, JSON, and reordered
modules' declarations with Cargo absent, and resolve the generated schemas
offline. Rust owned-model tests prove the portable compiler does not invoke
inventory factories.
