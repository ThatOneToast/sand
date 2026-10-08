# ADR 003: Owned programs and portable compiler protocol v1

Status: implementation contract for #396 (acceptance evidence pending).

## Boundary and ownership

Rust is a frontend. Inventory and factories belong to Rust collection, outside
the owned compiler. The supported backend receives owned State definitions,
functions, registrations, lifecycle contributions and tag memberships. It reads
no inventory, environment, filesystem or network and invokes no author callbacks.
Unsupported callback-bearing Rust features retain their existing collection path.
Static State descriptors adapt into owned data; arbitrary input is never leaked
or interned permanently. State allocation and initialization remain canonical.

The supported facade is `sand::advanced::compiler`. `Program` construction and
decoding feed identical validation. `Compiler::check` and `Compiler::compile`
take explicit target information and produce structured diagnostics; compilation
returns sorted relative-path/byte resources including shared pack metadata.
Publication and ZIP creation are adapter responsibilities.

## Envelope, modules and identities

The envelope contains `format: "sand.program"`, `format_version: 1`,
`target: {"minecraft": "26.2"}`, pack namespace/description, optional required
capabilities, and modules. A self-contained document embeds module definitions;
a manifest lists local module documents. Both resolve into the same Program.
Module documents contain an explicit module ID, State declarations, named
functions, ordered lifecycle contributions and typed tag memberships. Modules
cannot import modules. Embedded hosts supply all module contents explicitly.

State and function IDs are explicit namespaced resource locations. Field IDs are
names within their owning State. References distinguish internal and external
functions; internal references resolve before emission. External references have
an explicit context requirement but existence and external behavior cannot be
verified. Compiler-private resource identities are reserved and collision checked.
Module names, filenames, JSON field locations and origins never enter generated
identities. Top-level declaration permutations are equivalent; duplicate IDs are errors.

## Gameplay and ordering

V1 accepts player State, revision 1, integer score fields with zero defaults;
server/player function contexts; ordered score set/add, integer comparisons,
branches, player iteration and calls; load/tick hooks and function tags.
All functions are validated, including unreachable functions. Player-self access
requires player context; calls must satisfy callee requirements. Iteration changes
executor with `as @a`, without changing position. Branch conditions are evaluated
once: mutation in the selected body cannot cause the other body to execute.

Top-level declarations sort by semantic identity. State fields retain canonical
State declaration order. Bodies, execution edges and contributions
within an owner preserve authored order. Registration owners sort by identity,
matching Rust registration ordering. Canonical State setup precedes user load
work, and player initialization precedes user tick work. User tag membership
uses existing canonical ordering and deduplication after compiler lifecycle work.
Player-context entries ensure initialization before permitted State access even
when invoked directly. Repeated initialization preserves existing values.

Raw commands are an explicit capability and escape operation. Only line safety
and supported syntactic checks are promised; Sand cannot establish their State,
context or reference semantics. The acceptance program contains no raw commands.
Events, participants, other scopes, migrations, timers, scheduling, native
bindings and browser WebAssembly are rejected, not silently approximated.

## Versions, schemas and diagnostics

Protocol revision, State revision, compiler package identity and Minecraft target
are distinct. V1 requires an exact verified 26.x+ target using Sand's version
machinery; malformed, older and unverified targets fail. Capability discovery
reports supported operations, targets, limits and protocol/compiler identity.
Validation checks capabilities actually used even if requirements omit them.

Strict decoding rejects duplicate keys at every object depth before generic JSON
conversion, unknown fields/operations and invalid scalar types/ranges. Schemas
derive from Rust decoding models, with focused domain artifacts and offline
references plus a bundled schema. Regeneration must be checked for drift.

Diagnostics carry stable code, severity, message, module and JSON pointer, with
optional origin metadata. Hosts may attach DSL spans or editor node/port IDs.
Decode, resolution, semantic and lowering errors preserve these locations.

## Bounded work

V1 limits each UTF-8 document to 1 MiB and the complete program input to 8 MiB;
at most 64 modules per program; JSON nesting to 64 and operation nesting to 32;
10,000 declarations and 100,000 operations per program; 10,000 generated resources,
1 MiB per generated resource and 32 MiB total generated output per program.
Byte limits apply before parsing, depth/count limits during traversal, and output
limits before adding generated resources. Constructed Programs obey the same
semantic/count/depth/output limits. Limits produce machine-readable diagnostics.
Generated resource paths are limited to 768 UTF-8 bytes relative to the pack
root, with at most 255 bytes per segment including file extensions. Capability
discovery reports both bounds. Publication adapters must still account for the
length of their chosen destination root.

## Local adapter and publication

The CLI resolves a flat manifest under an explicit root (default: manifest
parent). It rejects absolute paths, `..`, empty path segments, duplicate module
paths, and symlinks in input paths. Stdin accepts self-contained documents only.
No implicit current-directory imports are permitted. Responses contain one
versioned JSON object on stdout; logs use stderr and failures exit nonzero.
Success reports paths, byte counts and diagnostics; file bodies require opt-in.

Compile and validate completely before publication. Validate managed paths and
ownership hashes; refuse unmanaged or externally modified generated-file
conflicts. Build a sibling staging directory retaining unrelated files, remove
only verified managed stale files, then replace the destination with rollback
on ordinary publication errors. Reject symlink destinations/descendants. This
is not a cross-process transaction or a promise of crash-atomic directory exchange.

## Acceptance example and evidence

Every frontend uses pack `demo`, description `Player counter`, State
`demo:counter`, field `value`, revision 1/default 0, server function `demo:tick`,
and player function `demo:increment`. The tick body iterates players and calls
increment; increment adds 1 to counter.value. Tick membership is `demo:tick`.
On first observation each player's absent value becomes 0, presence becomes 1,
then that player's value becomes 1. Subsequent ticks increment independently.
Reload provisions objectives without resetting scores. A joining player starts
at 1 on its first tick without changing existing players.

Compare complete output bytes across existing typed Rust authoring, constructed
Program, embedded and split JSON, Python and a minimal blueprint producer,
against a separately reviewed golden pack derived from canonical State behavior.
The blueprint execution path is tick → iterate players → increment; it rejects
ambiguous edges and cycles, preserves node/port origins and ignores editor layout.
Real 26.2 validation must distinguish automatic ticks from explicit invocations.
No merge until latest-head CI, independent review and runtime evidence pass.
