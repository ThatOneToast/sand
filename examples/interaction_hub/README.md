# Interaction hub

A shop sells a key, issues a written quest book, unlocks a block and grants a
personal reward. Gameplay uses Menu, Interactable, Purchase and canonical State.

Build with `sand build`. Execute these as a player at the desired positions:

- `function interaction_hub:place_shop`
- `function interaction_hub:place_gate`
- `function interaction_hub:place_chest`

The hitboxes are invisible. Place the gate hitbox at the block to remove and
place the chest hitbox where a player can click it. The reward is delivered by
the hitbox, not by intercepting native chest inventory clicks. Add ordinary
blocks or display entities for visuals. Hotbar slots 1, 2 and 3 must be empty
for the key, journal and reward respectively. Player State starts with 10 coins.

Run `interaction_hub:cleanup` as the owner to remove their hitboxes. Ownership
also ends after 6000 loaded ticks or owner loss. Another player's claims and
menu sessions remain independent. Menu close/escape expires without requiring
a client close callback. This example does not force a book screen open.
