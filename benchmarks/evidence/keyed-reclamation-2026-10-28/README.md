# Reclamation evidence

Final production, harness and full-gate revision: `78ed9e138694785a5ce993a71e9840a2f65c41ae`.
Baseline production pack: `8bfbfde322884e42612ae43ccc9b9dbcc1aeb65a`.
Both native runs use the final candidate harness and Java 25.0.3 / Minecraft 26.2.
The input pack ZIPs preserve the exact artifacts; JSON records retain pack,
harness/dependency/server identities, trace metrics and stage-separated scaling.
The final candidate has 200 passing assertions; baseline has 62. The compressed
full-validation log ends with `=== All checks passed ===`.

`independent-review.html` reviews implementation `330ce267` and identifies two
low-priority test gaps. `5874ac36` adds empty malformed-value and shared-hook
assertions; `78ed9e13` adds only canonical runtime baseline output. Final native
replay includes these assertions, actual old-score message checks, and baseline
cleanup measurements. No implementation defect was reported.

History is not final-revision proof. The preliminary integrated full gate was
intentionally interrupted before reclamation edits. The first focused attempt
exposed command deduplication of repeated UUID resolution; helper boundaries fix
that behavior and the unchanged lifecycle equality regression now passes. An
example build then rejected `Flag::set`; it was corrected to `enable`. A queued
native attempt had no built input pack and failed before server execution.
The exploratory dirty-tree run passed 194 assertions before final harness
strengthening; its recorded source revision is the prior base, not a claim that
base contains reclamation. All final conclusions use the clean runs instead.
