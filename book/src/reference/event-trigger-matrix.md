# Event Trigger Matrix

| Need | Trigger/system | Caveat |
|---|---|---|
| Active item use | `UsingItemTrigger` | trigger availability is vanilla-defined |
| Consume item | `ConsumeItemTrigger` | only consumption |
| Right-click entity | `PlayerInteractedWithEntityTrigger` | use tagged interaction entities |
| Entity summoned | `SummonedEntityTrigger` | player summon semantics only |
| Player hurt | `EntityHurtPlayer` | no exact amount in reward |
| Continuous state | tick component/system | author owns tick cost |
