# Empty-owner reclamation

This follow-up to indexed migration removes empty structural storage only during
explicit component detachment. It does not change `KeyedData::remove`: field
removal keeps component lifecycle and owner retention unchanged.

Cleanup hooks observe old scores, data and presence before deletion. Then declared
fields are removed, the detached component container is pruned if it is a compound
with zero children, and the shared components container is pruned under the same
rule. An owner with exactly its matching UUID remaining is removed. Unknown fields,
other components, malformed containers and authored empty-compound field values
prevent deletion. Archetype shared-claim guards still surround each component's
cleanup. Explicit State detach keeps its existing explicit ownership semantics.

Each deletion/pruning helper resolves the executor independently. This is required
because a hook can read another owner's keyed State and composition deduplicates
identical outer command lines. A helper keeps the UUID read and its mutation
together. No new observer, recurring scan, identity objective, or allocation index
is installed. Scratch child counts are removed on completed cleanup. The empty
indexed root and global version metadata are intentionally retained.

Offline players, unloaded entities and entities removed by external commands keep
persistent values. Absence is not death. No universal death cleanup or destructive
dormant-owner purge is provided. Permanent deletion of retained data is a separate
policy, not implicit empty-owner reclamation. Player detachment still publishes
suppression so initialization does not silently restore it.

Completed repeated detach does not invoke the cleanup hook again. Cleanup commands
are not transactional. Interruption before presence reset can replay a hook;
callers must not assume exactly-once hook side effects under command-chain
truncation. Repeated structural cleanup is safe, including an interruption between
component-container and owner deletion. Migration's readiness barrier applies to
the complete detach path, so pruning cannot race legacy migration.

Validation extends the maintained keyed-literal harness with actual exported
cleanup functions, independent fixed-UUID observations, hook prior-value reads,
nested owner access, two components, unknown and malformed fields, repeated detach,
field-only removal, missing executors and repeated attach/detach cycles. Historical
owner benchmarks retain separate seeding/migration/read/write stages and add
cleanup traces and storage checkpoints. Results and exact revisions are recorded
with the final evidence; no MSPT or universal entity-death guarantee is implied.

## Verified results

Clean revision `78ed9e13` passes full `scripts/check.sh`, 11 focused keyed tests,
21 State lifecycle tests and 17 harness tests. Final native replay passes 200
candidate and 62 baseline assertions, including zero-length malformed-container
retention and explicit shared-archetype hook suppression/final-hook execution.
[Evidence](../../benchmarks/evidence/keyed-reclamation-2026-10-28/summary.json)
retains exact artifacts and limits. The later evidence archive changes no Rust
or harness behavior. See `BENCHMARKS.md` for cleanup count/size interpretation.
