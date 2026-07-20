# 10. Event Composition

Trailforge's fifth event handler doesn't declare its own dispatch condition
from scratch — it **chains** off another event entirely:

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_sprint_while_exhausted}}
```

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:event_on_sprint_while_exhausted}}
```

`SandEventDispatch::chain::<PlayerSprintEvent>()` composes off
`PlayerSprintEvent`, one of Sand's built-in vanilla-derived events (sprint
detection itself needs a small amount of machinery — vanilla doesn't
directly expose "player started sprinting" as an advancement trigger, so
Sand's built-in event handles that detection once, centrally). Rather than
Trailforge re-deriving "is this player sprinting" from scratch, it composes
a *new, narrower* event — "sprinting **while also** exhausted" — by adding
one more `.when(...)` guard on top of an event Sand already provides.

## Why chain instead of writing a fresh tick check

Trailforge could instead add another guarded statement to `tick` (chapter
3) that checks sprint state and exhaustion together every tick. Chaining is
preferable here for two reasons:

1. **Reuse the correctness work already done.** `PlayerSprintEvent`'s own
   detection logic (whatever combination of movement-flag checks it takes
   to reliably notice a sprint start) lives in exactly one place. Chaining
   inherits that correctness instead of re-implementing sprint detection
   with a subtly different (and possibly buggier) condition.
2. **Named, single-purpose events compose better than a monolithic tick
   function.** `SprintingWhileExhaustedEvent` is a name you can find,
   test, and reason about independently — its own dispatch condition is
   exactly "sprint event fires AND exhausted" — versus one more line
   folded into `tick`'s growing list of unrelated `when(...)` blocks.

## Correlated vs. independent composition

`SandEventDispatch::chain::<E>()` is *correlated* composition: the new
event only fires on the same underlying occurrence as `E`, filtered by an
extra condition, and receives that occurrence's context. This is different
from two independent events that merely happen to both be true on the same
tick — chaining guarantees ordering and shared context with the event it
composes off, which matters when the base event carries information (like
which entity triggered it) your guard or handler needs to stay correlated
to.

## Inheriting a participant across the chain

"Shared context" from the previous section isn't only conditions — a
same-cycle child can also read a **participant** (a typed attacker/killer/
weapon reference, chapter on `sand::participant`) its parent already
captured, without capturing its own copy:

```rust,ignore
use sand::events::SandEventDispatch;
use sand::participant::{EntityParticipantRole, EventParticipantPlan};

impl SandEvent for ChildAfterDamage {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<EntityDamagePlayerEvent>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new()
            .inherit_entity::<EntityDamagePlayerEvent>(EntityParticipantRole::Attacker)
    }
}
```

`inherit_entity`/`inherit_item` generate zero extra setup or cleanup
commands — the parent's own capture already owns both, and a same-cycle
child only ever runs inside the parent's synchronous descendant call tree,
so the borrowed reference stays valid for the child's entire execution.
This works through an arbitrary-depth chain of plain `.after(...)`/
`chain::<...>()` edges (grandchild can inherit directly from the original
capturing ancestor), but is rejected at export — with a diagnostic naming
exactly which edge broke the chain — through `after_any`/`after_all`
fan-in, a bounded `.within(...)` window, or a second "inherit from an
event that itself only inherits" hop. See
`docs/testing/participant-role-evidence.md`'s propagation support matrix
for the complete edge/role table.

## When *not* to chain

Chaining only makes sense when the event you're composing off already
exists and already means what you need. `StaminaExhaustedEvent` (chapter 9)
isn't built by chaining off anything — "stamina reached zero" has no
existing Sand event to compose from, so it declares its own tick-scoped
dispatch condition directly. Reach for `chain::<E>()` when you're
*narrowing* an existing event's meaning; declare a fresh `SandEvent::dispatch()`
when you're detecting something genuinely new.
