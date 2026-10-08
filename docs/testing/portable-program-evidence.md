# Portable program runtime evidence (#396)

The separately runnable harness starts the cached Minecraft Java 26.2 server,
loads the complete compiled counter pack, and joins real offline-mode players
with the existing minimal join client. Automatic ticks are frozen. A separate
audit datapack invokes three lifecycle/tick pairs explicitly while both players
are online; it does not change the counter pack.

The run on 2026-09-27 passed direct first-entry initialization (value 1 and
presence revision 1), two-player isolation (values 4 and 3 after the direct call
and three tick pairs), reload preservation (4 and 3), and a new player's isolated
first increment (1, with the earlier scores still 4 and 3).

This proves explicitly invoked generated lifecycle/tick behavior. The clients'
short play-phase connections do **not** establish sustained automatic-tick
behavior. Independent implementation review and latest-head CI remain merge
requirements; these local results do not claim either has occurred.

```sh
python3 scripts/mc_validation/run_portable_program_audit.py \
  --binary target/debug/sand \
  --jar ~/.sand/cache/26.2/server.jar \
  --evidence /tmp/portable-program-evidence.json
```

The machine observations are in [portable-program-evidence.json](portable-program-evidence.json).
