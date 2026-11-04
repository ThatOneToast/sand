# Interaction session boundary proof

The 2026-11-03 run of `scripts/validate_interactions.py` passed against a vanilla
26.2 server with two connected players. This extends the existing menu proof:

- Two nearby hitboxes owned by different players receive independent sessions.
  Cancelling the first owner removes only that object, invalidates its session,
  and leaves the second player's session able to execute its choice.
- A bound player transferring to the Nether loses the session. Returning to the
  original position does not make the captured old button valid again.
- Page navigation at the maximum token allocator value clears all choice tokens,
  leaves the allocator unchanged, rejects a stale button and rejects reopening.
  The proof never resets the allocator to recycle values.

The complete run also retains pagination, player isolation, authorization,
purchases, disconnect, reload and cleanup checks. Every new assertion is part of
the maintained vanilla CI interaction job.

Reproduce from the repository root with a local 26.2 server jar:

```sh
python3 scripts/validate_interactions.py --jar /path/to/server.jar --output /tmp/sand-interactions
```

[Machine-readable result](../../benchmarks/evidence/interaction-boundaries-2026-11-03/evidence.json)
and [compressed native run](../../benchmarks/evidence/interaction-boundaries-2026-11-03/native-run.log.gz)
record the successful run.

The harness seeds vanilla interaction records through the console and uses real
connected players for identity and session state. This proves native dispatch
and cleanup after a record exists; it does not measure physical mouse input,
click delivery, client rendering, or the 26.3 player protocol. Token exhaustion
is induced by setting the private allocator to its maximum solely in the
disposable test world.
